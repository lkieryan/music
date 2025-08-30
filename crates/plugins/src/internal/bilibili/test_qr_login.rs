//! B站扫码登录功能测试

use super::*;
use music_plugin_sdk::types::{AuthMethod};
use music_plugin_sdk::types::media::QrCodeState;
use music_plugin_sdk::traits::MediaAuthPlugin;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_qrcode() {
        let mut plugin = BilibiliPlugin::new();
        
        // 测试生成二维码
        let result = plugin.generate_qrcode().await;
        
        match result {
            Ok(qr_response) => {
                println!("QR Code URL: {}", qr_response.content);
                println!("QR Code Key: {}", qr_response.qrcode_key);
                assert!(!qr_response.content.is_empty());
                assert!(!qr_response.qrcode_key.is_empty());
            },
            Err(e) => {
                panic!("Failed to generate QR code: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_qr_login_flow() {
        let mut plugin = BilibiliPlugin::new();
        
        // 1. 生成二维码
        let qr_response = plugin.generate_qrcode().await.unwrap();
        let qrcode_key = qr_response.qrcode_key;
        
        println!("QR Code Key: {}", qrcode_key);
        
        // 2. 检查登录状态（这会失败，因为没有人扫码）
        let status = plugin.check_qrcode_status(&qrcode_key).await.unwrap();
        
        match status.status {
            QrCodeState::WaitingForScan => {
                println!("Login pending as expected");
            },
            QrCodeState::Failed => {
                if let Some(reason) = status.error_message {
                    println!("Login failed: {}", reason);
                }
            },
            _ => {
                panic!("Unexpected status: {:?}", status.status);
            }
        }
    }

    #[tokio::test]
    async fn test_supported_auth_methods() {
        let plugin = BilibiliPlugin::new();
        
        // 检查支持的认证方法
        let methods = plugin.supported_auth_methods();
        assert!(methods.contains(&AuthMethod::QrCode));
        println!("Supported auth methods: {:?}", methods);
    }

    #[tokio::test]
    async fn test_user_info() {
        let mut plugin = BilibiliPlugin::new();
        
        // 初始状态下应该没有用户信息
        assert!(plugin.get_user_info().is_none());
        
        // 设置一个假的会话令牌
        plugin.set_session_data("fake_sessdata".to_string());
        
        // 现在应该有用户信息（虽然是模拟的）
        let user_info = plugin.get_user_info();
        assert!(user_info.is_some());
        
        if let Some(info) = user_info {
            println!("User ID: {}", info.user_id);
            assert!(!info.user_id.is_empty());
        }
    }

    #[tokio::test]
    async fn test_logout() {
        let mut plugin = BilibiliPlugin::new();
        
        // 设置一个假的会话令牌
        plugin.set_session_data("fake_sessdata".to_string());
        
        // 检查是否已认证
        assert!(plugin.is_authenticated());
        
        // 登出
        let result = plugin.logout().await;
        assert!(result.is_ok());
        
        // 检查是否已登出
        assert!(!plugin.is_authenticated());
    }
}