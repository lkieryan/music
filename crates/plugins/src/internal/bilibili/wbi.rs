use anyhow::{bail, Result};
use regex::Regex;
use reqwest::header::{COOKIE, REFERER, USER_AGENT};
use serde_json::Value as Json;
use std::collections::BTreeMap;
use tokio::sync::RwLock;


/// Sign parameters using WBI (pure function).
/// Input and output are both BTreeMap<String, String>; no external state is modified.
pub fn sign_wbi(mut params: BTreeMap<String, String>, salt: &str, ts: i64) -> BTreeMap<String, String> {
    // Inject timestamp parameter
    params.insert("wts".to_string(), ts.to_string());

    // Build uq: keys sorted by BTreeMap, URL-encode values
    let uq: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    // Compute w_rid
    let w_rid = format!("{:x}", md5::compute(uq + salt));
    params.insert("w_rid".to_string(), w_rid);
    params
}

/* removed: buvid SPI/activation not needed for minimal flow */

/// Fetch navigation API to obtain two wbi_img URLs and compute the salt.
pub async fn fetch_wbi_salt(http: &reqwest::Client, sessdata: Option<&str>) -> Result<String> {
    // Use nav API
    let url = "https://api.bilibili.com/x/web-interface/nav";
    let mut req = http.get(url)
        .header(REFERER, "https://www.bilibili.com")
        .header(USER_AGENT, concat!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
            "AppleWebKit/537.36 (KHTML, like Gecko) ",
            "Chrome/116.0.0.0 Safari/537.36 Edg/116.0.1938.54"
        ));
    if let Some(token) = sessdata { req = req.header(COOKIE, format!("SESSDATA={}", token)); }

    let text = req.send().await?.text().await?;
    let v: Json = serde_json::from_str(&text)?;
    let Some(imgurl) = v["data"]["wbi_img"]["img_url"].as_str() else {
        bail!("fetch_wbi_salt: wbi_img/img_url invalid");
    };
    let Some(suburl) = v["data"]["wbi_img"]["sub_url"].as_str() else {
        bail!("fetch_wbi_salt: wbi_img/sub_url invalid");
    };
    Ok(wbi_salt_compute(imgurl, suburl))
}

fn wbi_parse_ae(imgurl: &str, suburl: &str) -> Option<String> {
    let Ok(re) = Regex::new(r"https://i0\.hdslb\.com/bfs/wbi/(\w+)\.png") else {
        return None;
    };
    let img = re.captures(imgurl)?.get(1)?.as_str();
    let sub = re.captures(suburl)?.get(1)?.as_str();
    Some(img.to_owned() + sub)
}

/// Compute 32-char salt based on two image URLs and wbi_oe.json.
fn wbi_salt_compute(imgurl: &str, suburl: &str) -> String {
    let ae: String = wbi_parse_ae(imgurl, suburl).unwrap_or_else(|| {
        imgurl[imgurl.len() - 36..imgurl.len() - 4].to_owned()
            + &suburl[suburl.len() - 36..suburl.len() - 4]
    });
    // Read wbi_oe.json
    let oe_json: Json = serde_json::from_str(include_str!("../../../../../other/bilibili-api-rs/bilibili-api-rs/src/wbi_oe.json")).expect("wbi_oe.json invalid");
    let oe: Vec<i64> = oe_json.as_array().expect("wbi_oe not array")
        .iter().map(|v| v.as_i64().expect("wbi_oe[i] not i64")).collect();
    let le: String = oe
        .iter()
        .filter_map(|x| usize::try_from(*x).ok())
        .filter(|x| *x < ae.len())
        .fold(String::new(), |acc, x| acc + &ae[x..=x]);
    le[..32].into()
}

/// Simple check if an API path requires WBI signing.
pub fn should_sign(path: &str) -> bool {
    path.contains("/wbi/")
}

/// Ensure salt exists: read from cache or fetch and write into cache.
pub async fn ensure_salt(
    http: &reqwest::Client,
    sessdata: Option<&str>,
    cache: &RwLock<Option<String>>,
)
-> Result<String> {
    if let Some(s) = cache.read().await.clone() { return Ok(s); }
    let s = fetch_wbi_salt(http, sessdata).await?;
    let mut w = cache.write().await;
    *w = Some(s.clone());
    Ok(s)
}

/// Single-function requester: auto-detects WBI need, signs if required, and performs the request.
pub async fn wbi_request(
    http: &reqwest::Client,
    method: reqwest::Method,
    base_url: &str,
    path: &str,
    mut params: BTreeMap<String, String>,
    sessdata: Option<&str>,
    salt_cache: &RwLock<Option<String>>,
) -> Result<Json> {
    // If signing is required, ensure salt and sign parameters
    if should_sign(path) {
        let salt = ensure_salt(http, sessdata, salt_cache).await?;
        let ts = chrono::Local::now().timestamp();
        params = sign_wbi(params, &salt, ts);
    }

    let url = format!("{}{}", base_url, path);
    
    let mut req = http.request(method, &url)
        .header(REFERER, "https://www.bilibili.com")
        .header(USER_AGENT, concat!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
            "AppleWebKit/537.36 (KHTML, like Gecko) ",
            "Chrome/116.0.0.0 Safari/537.36 Edg/116.0.1938.54"
        ))
        .query(&params);
    
    // Build full Cookie header
    let mut cookie_parts = Vec::new();
    
    // Add basic buvid identifiers
    cookie_parts.push("buvid3=00000000-0000-0000-0000-000000000000infoc".to_string());
    cookie_parts.push("buvid4=00000000-0000-0000-0000-000000000000".to_string());
    cookie_parts.push("buvid_fp=00000000000000000000000000000000".to_string());
    cookie_parts.push("_uuid=00000000-0000-0000-0000-000000000000".to_string());
    
    // Include SESSDATA if present
    if let Some(token) = sessdata { 
        cookie_parts.push(format!("SESSDATA={}", token));
    }
    
    // Add other necessary cookies
    cookie_parts.push("ac_time_value=0".to_string());
    cookie_parts.push("bili_jct=".to_string());
    cookie_parts.push("DedeUserID=0".to_string());
    
    let cookie_string = cookie_parts.join("; ");
    req = req.header(COOKIE, cookie_string);

    let text = req.send().await?.text().await?;
    
    // Prefer to parse as {code,data,message}
    if let Ok(v) = serde_json::from_str::<Json>(&text) {
        if v["code"].as_i64() == Some(0) {
            return Ok(v["data"].clone());
        }
        return Ok(v);
    }
    
    bail!("invalid json response")
}
