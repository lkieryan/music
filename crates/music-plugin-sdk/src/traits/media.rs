//! Media provider trait
//! 
//! This module defines the trait for media provider plugins.

use async_trait::async_trait;
use crate::traits::base::BasePlugin;
use crate::types::base::{PluginResult, PluginContext, PluginConfig, PluginStatus};
use crate::types::media::{
    SearchQuery, SearchResult, Track, Album, Artist, Playlist, PageInput, SearchType,
    AuthMethod, AuthUserInfo, QrCodeResponse, QrCodeStatus, SmsResponse, AuthResult,
    AudioQuality, StreamRequest, StreamSource, StreamProtocol
};
use std::collections::HashMap;

#[async_trait]
pub trait MediaPlugin: BasePlugin {
    /// Search for music content
    async fn search(&self, query: &SearchQuery) -> PluginResult<SearchResult>;

    /// Get track details by ID
    async fn get_track(&self, track_id: &str) -> PluginResult<Track>;
    
    /// Get track media stream with format/quality hints.
    async fn get_media_stream(&self, track_id: &str, req: &StreamRequest) -> PluginResult<StreamSource>;

    /// Get album details by ID
    async fn get_album(&self, album_id: &str) -> PluginResult<Album>;
    
    /// Get artist details by ID
    async fn get_artist(&self, artist_id: &str) -> PluginResult<Artist>;
    
    /// Get playlist details by ID
    async fn get_playlist(&self, playlist_id: &str) -> PluginResult<Playlist>;
    
    /// Check if track is available for streaming
    async fn is_track_available(&self, track_id: &str) -> PluginResult<bool>;
    
    /// Get user's library (if authenticated)
    async fn get_user_library(&self) -> PluginResult<Vec<Track>> {
        Err(crate::errors::PluginError::NotSupported(
            "User library access not supported".to_string()
        ))
    }
    
    /// Get user's playlists (if authenticated)
    async fn get_user_playlists(&self) -> PluginResult<Vec<Playlist>> {
        Err(crate::errors::PluginError::NotSupported(
            "User playlists access not supported".to_string()
        ))
    }

}

#[async_trait]
pub trait MediaAuthPlugin: MediaPlugin {
    /// Get supported authentication methods
    fn supported_auth_methods(&self) -> Vec<AuthMethod> {
        vec![]
    }
    
    /// Check if user is authenticated
    fn is_authenticated(&self) -> bool {
        false
    }
    
    /// Get authenticated user info
    fn get_user_info(&self) -> Option<AuthUserInfo> {
        None
    }
    
    /// Logout user
    async fn logout(&mut self) -> PluginResult<()> {
        Err(crate::errors::PluginError::NotSupported(
            "Logout not supported".to_string()
        ))
    }
    
    /// Refresh authentication token if needed
    async fn refresh_auth(&mut self) -> PluginResult<()> {
        Err(crate::errors::PluginError::NotSupported(
            "Auth refresh not supported".to_string()
        ))
    }
    
    // QR Code Authentication   
    /// Generate QR code for login
    async fn generate_qrcode(&mut self) -> PluginResult<QrCodeResponse> {
        Err(crate::errors::PluginError::NotSupported(
            "QR code authentication not supported".to_string()
        ))
    }
    
    /// Check QR code login status
    async fn check_qrcode_status(&self, qrcode_key: &str) -> PluginResult<QrCodeStatus> {
        Err(crate::errors::PluginError::NotSupported(
            "QR code authentication not supported".to_string()
        ))
    }
    
    // SMS Authentication
    /// Send verification code to phone number
    async fn send_sms_code(&mut self, phone: &str, country_code: Option<&str>) -> PluginResult<SmsResponse> {
        Err(crate::errors::PluginError::NotSupported(
            "SMS authentication not supported".to_string()
        ))
    }
    
    /// Verify SMS code
    async fn verify_sms_code(&mut self, phone: &str, code: &str) -> PluginResult<AuthResult> {
        Err(crate::errors::PluginError::NotSupported(
            "SMS authentication not supported".to_string()
        ))
    }
    
    // Password Authentication
    /// Login with username and password
    async fn login_with_password(&mut self, username: &str, password: &str) -> PluginResult<AuthResult> {
        Err(crate::errors::PluginError::NotSupported(
            "Password authentication not supported".to_string()
        ))
    }
    
    /// Submit additional verification if required (e.g., captcha, 2FA)
    async fn submit_verification(&mut self, session_id: &str, data: HashMap<String, String>) -> PluginResult<AuthResult> {
        Err(crate::errors::PluginError::NotSupported(
            "Additional verification not supported".to_string()
        ))
    }
}

/// Download capability trait
#[async_trait]
pub trait MediaDownloadPlugin: MediaPlugin {
    /// Download track
    async fn download_track(&self, track_id: &str, output_path: &std::path::Path) -> PluginResult<()>;
    
    /// Get download progress for a track
    async fn get_download_progress(&self, track_id: &str) -> PluginResult<f32>;
    
    /// Cancel download
    async fn cancel_download(&self, track_id: &str) -> PluginResult<()>;
    
    /// Check if track can be downloaded
    async fn can_download(&self, track_id: &str) -> PluginResult<bool>;
}
