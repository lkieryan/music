use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliSearchResponse {
    pub result: Option<Vec<BilibiliSearchVideo>>,
    #[serde(rename = "numPages")]
    pub num_pages: Option<u32>,
    #[serde(rename = "numResults")]
    pub num_results: Option<u32>,
    pub page: Option<u32>,
    pub pagesize: Option<u32>,
    pub seid: Option<String>,
    pub rqt_type: Option<String>,
    pub suggest_keyword: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliSearchVideo {
    pub aid: u64,
    pub bvid: String,
    pub title: String,
    pub description: String,
    pub pic: String,
    pub author: String,
    pub mid: u64,
    pub duration: String,
    pub pubdate: u64,
    pub play: u64,
    pub video_review: u64,
    pub favorites: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliVideoDetails {
    pub aid: u64,
    pub bvid: String,
    pub cid: u64,
    pub title: String,
    pub desc: String,
    pub pic: String,
    pub owner: BilibiliOwner,
    pub stat: BilibiliVideoStat,
    pub duration: u64,
    pub pubdate: u64,
    pub pages: Option<Vec<BilibiliMultipageVideo>>,
    pub subtitle: Option<BilibiliSubtitleInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliOwner {
    pub mid: u64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliVideoStat {
    pub view: u64,
    pub danmaku: u64,
    pub reply: u64,
    pub favorite: u64,
    pub coin: u64,
    pub share: u64,
    pub like: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliMultipageVideo {
    pub cid: u64,
    pub page: u32,
    pub from: String,
    pub part: String,
    pub duration: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliUserInfo {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub sign: String,
    pub level: u32,
    pub vip: BilibiliVip,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliVip {
    #[serde(rename = "type")]
    pub vip_type: u32,
    pub status: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliPlaylist {
    pub id: u64,
    pub fid: u64,
    pub mid: u64,
    pub attr: u32,
    pub title: String,
    pub fav_state: u32,
    pub media_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliFavoriteListContents {
    pub info: BilibiliPlaylistInfo,
    pub medias: Option<Vec<BilibiliMediaItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliPlaylistInfo {
    pub id: u64,
    pub fid: u64,
    pub mid: u64,
    pub attr: u32,
    pub title: String,
    pub cover: String,
    pub intro: String,
    pub media_count: u64,
    pub upper: BilibiliOwner,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliMediaItem {
    pub id: u64,
    #[serde(rename = "type")]
    pub media_type: u32,
    pub title: String,
    pub cover: String,
    pub intro: String,
    pub page: u32,
    pub duration: u32,
    pub upper: BilibiliOwner,
    pub attr: u32,
    pub cnt_info: BilibiliCountInfo,
    pub link: String,
    pub ctime: u64,
    pub pubtime: u64,
    pub fav_time: u64,
    pub bv_id: String,
    pub bvid: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliCountInfo {
    pub collect: u32,
    pub play: u32,
    pub danmaku: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliAudioStreamResponse {
    pub dash: Option<BilibiliDash>,
    /// Progressive playlist entries (MP4/FLV); present when fnval requests non-DASH
    pub durl: Option<Vec<BilibiliDurlItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliDash {
    pub audio: Option<Vec<BilibiliAudioTrack>>,
    pub dolby: Option<BilibiliDolbyAudio>,
    pub flac: Option<BilibiliFlacAudio>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliAudioTrack {
    pub id: u32,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliDolbyAudio {
    pub audio: Option<Vec<BilibiliAudioTrack>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliFlacAudio {
    pub audio: BilibiliAudioTrack,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliDurlItem {
    pub url: String,
    #[serde(default)]
    pub backup_url: Option<Vec<String>>,
    #[serde(default)]
    pub length: Option<u64>,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliSubtitleInfo {
    pub allow_submit: bool,
    pub list: Vec<BilibiliSubtitle>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliSubtitle {
    pub id: u64,
    pub lan: String,
    pub lan_doc: String,
    pub is_lock: bool,
    pub subtitle_url: String,
    pub author: Option<BilibiliSubtitleAuthor>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliSubtitleAuthor {
    pub mid: u64,
    pub name: String,
    pub face: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliSubtitleContent {
    pub body: Vec<BilibiliSubtitleLine>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BilibiliSubtitleLine {
    pub from: u64,
    pub to: u64,
    pub content: String,
}

// ===== 扫码登录相关类型定义 =====

/// 二维码生成响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QrGenerateResponse {
    /// 二维码内容 (登录页面 url)
    pub url: String,
    /// 扫码登录秘钥
    pub qrcode_key: String,
}

/// 扫码状态轮询响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QrPollResponse {
    /// 游戏分站跨域登录 url
    pub url: String,
    /// 刷新 token
    pub refresh_token: String,
    /// 登录时间
    pub timestamp: u64,
    /// 状态码
    /// 0：扫码登录成功
    /// 86038：二维码已失效
    /// 86090：二维码已扫码未确认
    /// 86101：未扫码
    pub code: i32,
    /// 状态信息
    pub message: String,
}

/// 扫码状态枚举
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum QrStatus {
    /// 未扫码
    NotScanned,
    /// 已扫描未确认
    ScannedNotConfirmed,
    /// 登录成功
    Success,
    /// 二维码失效
    Expired,
    /// 未知状态
    Unknown,
}

impl From<i32> for QrStatus {
    fn from(code: i32) -> Self {
        match code {
            0 => QrStatus::Success,
            86038 => QrStatus::Expired,
            86090 => QrStatus::ScannedNotConfirmed,
            86101 => QrStatus::NotScanned,
            _ => QrStatus::Unknown,
        }
    }
}

/// 登录成功后的 Cookie 信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginCookieInfo {
    /// 用户 ID
    pub dede_user_id: String,
    /// 用户 ID MD5
    pub dede_user_id_ck_md5: String,
    /// 会话数据
    pub sessdata: String,
    /// CSRF token
    pub bili_jct: String,
    /// 会话 ID
    pub sid: String,
}

/// 登录会话信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginSession {
    /// 登录成功后的 Cookie 信息
    pub cookies: LoginCookieInfo,
    /// 刷新 token
    pub refresh_token: String,
    /// 登录时间戳
    pub timestamp: u64,
    /// 用户信息 (可选)
    pub user_info: Option<BilibiliUserInfo>,
}
