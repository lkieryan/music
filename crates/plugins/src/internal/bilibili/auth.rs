use async_trait::async_trait;
use std::collections::HashMap;

use music_plugin_sdk::{
    traits::MediaAuthPlugin,
    types::media::*,
    errors::PluginError
};
use chrono::Utc;
use super::plugin::BilibiliPlugin;
use super::types::*;

impl BilibiliPlugin {
    /// 生成二维码
    async fn generate_qrcode_internal(&self) -> PluginResult<QrGenerateResponse> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
        
        let req = self.http.get(url)
            .header("Referer", "https://www.bilibili.com")
            .header("User-Agent", concat!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
                "AppleWebKit/537.36 (KHTML, like Gecko) ",
                "Chrome/116.0.0.0 Safari/537.36 Edg/116.0.1938.54"
            ));
        
        let text = req.send().await
            .map_err(|e| PluginError::Internal(format!("Failed to generate qrcode: {}", e)))?
            .text().await
            .map_err(|e| PluginError::Internal(format!("Failed to read response: {}", e)))?;
        
        let v: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse response: {}", e)))?;
        
        if v["code"].as_i64() != Some(0) {
            return Err(PluginError::Internal(format!("generate_qrcode failed: {}", v["message"])));
        }
        
        let data = &v["data"];
        let url = data["url"].as_str()
            .ok_or_else(|| PluginError::Internal("url not found in response".to_string()))?;
        let qrcode_key = data["qrcode_key"].as_str()
            .ok_or_else(|| PluginError::Internal("qrcode_key not found in response".to_string()))?;
        
        Ok(QrGenerateResponse {
            url: url.to_string(),
            qrcode_key: qrcode_key.to_string(),
        })
    }

    /// 轮询扫码状态
    async fn poll_qrcode_status(&self, qrcode_key: &str) -> PluginResult<(QrPollResponse, Option<LoginCookieInfo>)> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
        
        let mut params = std::collections::BTreeMap::new();
        params.insert("qrcode_key".to_string(), qrcode_key.to_string());
        
        let req = self.http.get(url)
            .header("Referer", "https://www.bilibili.com")
            .header("User-Agent", concat!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) ",
                "AppleWebKit/537.36 (KHTML, like Gecko) ",
                "Chrome/116.0.0.0 Safari/537.36 Edg/116.0.1938.54"
            ))
            .query(&params);
        
        let resp = req.send().await
            .map_err(|e| PluginError::Internal(format!("Failed to poll qrcode status: {}", e)))?;
        
        // 尝试从响应头中提取 cookie 信息
        let cookie_info = self.extract_cookies_from_response(&resp).await.ok();
        
        let text = resp.text().await
            .map_err(|e| PluginError::Internal(format!("Failed to read response: {}", e)))?;
        let v: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse response: {}", e)))?;
        
        if v["code"].as_i64() != Some(0) {
            return Err(PluginError::Internal(format!("poll_qrcode_status failed: {}", v["message"])));
        }
        
        let data = &v["data"];
        let poll_response = QrPollResponse {
            url: data["url"].as_str().unwrap_or_default().to_string(),
            refresh_token: data["refresh_token"].as_str().unwrap_or_default().to_string(),
            timestamp: data["timestamp"].as_u64().unwrap_or(0),
            code: data["code"].as_i64().unwrap_or(-1) as i32,
            message: data["message"].as_str().unwrap_or_default().to_string(),
        };
        
        Ok((poll_response, cookie_info))
    }

    /// 从响应头中提取 Cookie 信息
    async fn extract_cookies_from_response(&self, resp: &reqwest::Response) -> PluginResult<LoginCookieInfo> {
        let headers = resp.headers();
        
        let cookie_header = headers
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join("; ");
        
        let mut dede_user_id = String::new();
        let mut dede_user_id_ck_md5 = String::new();
        let mut sessdata = String::new();
        let mut bili_jct = String::new();
        let mut sid = String::new();
        
        for cookie in cookie_header.split("; ") {
            if let Some((key, value)) = cookie.split_once('=') {
                match key {
                    "DedeUserID" => dede_user_id = value.to_string(),
                    "DedeUserID__ckMd5" => dede_user_id_ck_md5 = value.to_string(),
                    "SESSDATA" => sessdata = value.to_string(),
                    "bili_jct" => bili_jct = value.to_string(),
                    "sid" => sid = value.to_string(),
                    _ => {}
                }
            }
        }
        
        if dede_user_id.is_empty() || sessdata.is_empty() {
            return Err(PluginError::Internal("Failed to extract required cookies from response".to_string()));
        }
        
        Ok(LoginCookieInfo {
            dede_user_id,
            dede_user_id_ck_md5,
            sessdata,
            bili_jct,
            sid,
        })
    }

    /// 获取用户信息
    async fn get_user_info_internal(&self) -> PluginResult<BilibiliUserInfo> {
        let response = super::wbi::wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/space/myinfo",
            std::collections::BTreeMap::new(),
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get user info failed: {}", e)))?;

        let user_info: BilibiliUserInfo = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse user info: {}", e)))?;

        Ok(user_info)
    }
}

// 扩展 BilibiliPlugin 以支持认证相关操作
impl BilibiliPlugin {
    /// 设置会话数据
    pub fn set_session_data(&mut self, session_data: String) {
        self.session_data = Some(session_data);
    }
    
    /// 获取会话数据
    pub fn get_session_data(&self) -> Option<&str> {
        self.session_data.as_deref()
    }
    
}

#[async_trait]
impl MediaAuthPlugin for BilibiliPlugin {
    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        vec![
            AuthMethod::QrCode,
            // TODO: 添加其他认证方式
            // AuthMethod::Phone,
            // AuthMethod::Password,
        ]
    }

    fn is_authenticated(&self) -> bool {
        self.session_data.is_some()
    }

    fn get_user_info(&self) -> Option<AuthUserInfo> {
        // 如果已认证，可以返回缓存的用户信息
        // 实际实现可能需要异步获取最新信息
        if self.session_data.is_some() {
            Some(AuthUserInfo {
                user_id: "unknown".to_string(), // 实际应该从session中获取
                display_name: None,
                avatar_url: None,
                metadata: HashMap::new(),
            })
        } else {
            None
        }
    }

    async fn logout(&mut self) -> PluginResult<()> {
        // 清除会话数据
        self.session_data = None;
        Ok(())
    }

    async fn refresh_auth(&mut self) -> PluginResult<()> {
        // B站可能需要特殊的刷新逻辑
        Err(PluginError::NotSupported("Auth refresh not supported for Bilibili".to_string()))
    }

    // QR Code Authentication
    async fn generate_qrcode(&mut self) -> PluginResult<QrCodeResponse> {
        let qr_response = self.generate_qrcode_internal().await?;
        
        Ok(QrCodeResponse {
            content: qr_response.url,
            image_url: None, // 可以在这里生成二维码图片的URL
            qrcode_key: qr_response.qrcode_key,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(180)), // 180秒过期
        })
    }

    async fn check_qrcode_status(&self, qrcode_key: &str) -> PluginResult<QrCodeStatus> {
        let (poll_response, cookie_info) = self.poll_qrcode_status(qrcode_key).await?;
        
        let status: QrStatus = poll_response.code.into();
        
        match status {
            QrStatus::Success => {
                // 登录成功
                if let Some(cookies) = cookie_info {
                    Ok(QrCodeStatus {
                        status: QrCodeState::Success,
                        user_info: Some(AuthUserInfo {
                            user_id: cookies.dede_user_id,
                            display_name: None, // 可以通过其他API获取用户名
                            avatar_url: None,
                            metadata: HashMap::new(),
                        }),
                        session_token: Some(cookies.sessdata.clone()),
                        error_message: None,
                    })
                } else {
                    Ok(QrCodeStatus {
                        status: QrCodeState::Failed,
                        user_info: None,
                        session_token: None,
                        error_message: Some("Login succeeded but no cookies received".to_string()),
                    })
                }
            },
            QrStatus::NotScanned => {
                // 未扫描，继续轮询
                Ok(QrCodeStatus {
                    status: QrCodeState::WaitingForScan,
                    user_info: None,
                    session_token: None,
                    error_message: None,
                })
            },
            QrStatus::ScannedNotConfirmed => {
                // 已扫描未确认，继续轮询
                Ok(QrCodeStatus {
                    status: QrCodeState::WaitingForConfirmation,
                    user_info: None,
                    session_token: None,
                    error_message: None,
                })
            },
            QrStatus::Expired => {
                // 二维码已失效
                Ok(QrCodeStatus {
                    status: QrCodeState::Expired,
                    user_info: None,
                    session_token: None,
                    error_message: Some("二维码已失效，请重新获取".to_string()),
                })
            },
            QrStatus::Unknown => {
                // 未知状态
                Ok(QrCodeStatus {
                    status: QrCodeState::Failed,
                    user_info: None,
                    session_token: None,
                    error_message: Some(format!("未知状态: {}", poll_response.message)),
                })
            },
        }
    }

    // SMS Authentication - 目前B站不支持
    async fn send_sms_code(&mut self, _phone: &str, _country_code: Option<&str>) -> PluginResult<SmsResponse> {
        Err(PluginError::NotSupported("SMS authentication not supported for Bilibili".to_string()))
    }

    async fn verify_sms_code(&mut self, _phone: &str, _code: &str) -> PluginResult<AuthResult> {
        Err(PluginError::NotSupported("SMS authentication not supported for Bilibili".to_string()))
    }

    // Password Authentication - 目前B站不支持
    async fn login_with_password(&mut self, _username: &str, _password: &str) -> PluginResult<AuthResult> {
        Err(PluginError::NotSupported("Password authentication not supported for Bilibili".to_string()))
    }

    async fn submit_verification(&mut self, _session_id: &str, _data: HashMap<String, String>) -> PluginResult<AuthResult> {
        Err(PluginError::NotSupported("Additional verification not supported for Bilibili".to_string()))
    }
}