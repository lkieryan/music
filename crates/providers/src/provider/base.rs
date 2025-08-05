use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use types::errors::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
    pub token: Option<String>,
    pub is_first: bool,
    pub is_valid: bool,
}

impl Pagination {
    pub fn new_limit(limit: u32, offset: u32) -> Self {
        Self { limit, offset, is_first: true, is_valid: true, token: None }
    }
    pub fn new_token(token: Option<String>) -> Self {
        Self { token, is_first: true, is_valid: true, ..Default::default() }
    }
    pub fn next_page(&self) -> Self {
        Self { limit: self.limit, offset: self.offset + self.limit.max(1), token: self.token.clone(), is_first: false, is_valid: true }
    }
    pub fn next_page_wtoken(&self, token: Option<String>) -> Self {
        Self { limit: self.limit, offset: self.offset + self.limit.max(1), token, is_first: false, is_valid: true }
    }
    pub fn invalidate(&mut self) { self.is_valid = false; }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProviderCapability {
    Search,
    Playlists,
    Lyrics,
    StreamUrl,
    UrlMatch,
    Suggestions,
    OAuth2,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigKey {
    pub key: String,
    pub required: bool,
    pub secret: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderMetadata {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub capabilities: Vec<ProviderCapability>,
    pub config_keys: Vec<ConfigKey>,
    pub docs_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderStatus {
    pub key: String,
    pub name: String,
    pub user_name: Option<String>,
    pub logged_in: bool,
    pub bg_color: Option<String>,
    pub account_id: Option<String>,
    // Expose provider capabilities to the UI for filtering/selecting providers
    pub capabilities: Vec<ProviderCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageMetrics {
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub avg_latency_ms: Option<f64>,
    pub rate_limited: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryablePlaylist { pub id: String, pub name: String }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryableArtist { pub id: String, pub name: String }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryableAlbum { pub id: String, pub name: String }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Song { pub id: String, pub title: String, pub artist: String, pub duration_ms: Option<u32>, pub provider_extension: Option<String> }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResult { pub songs: Vec<Song> }

#[async_trait]
pub trait BaseProvider: std::fmt::Debug + Send + Sync {
    fn metadata(&self) -> ProviderMetadata;
    fn key(&self) -> String;
    fn capabilities(&self) -> Vec<ProviderCapability> { self.metadata().capabilities }
    /// Check if this provider supports a given capability (from metadata)
    fn supports(&self, cap: &ProviderCapability) -> bool { self.capabilities().contains(cap) }

    async fn initialize(&self) -> Result<()> { Ok(()) }
    async fn get_status(&self) -> Result<ProviderStatus> {
        Ok(ProviderStatus {
            key: self.key(),
            name: self.metadata().display_name.clone(),
            capabilities: self.capabilities(),
            ..Default::default()
        })
    }

    // auth
    async fn login(&self, _account_id: String) -> Result<String> { Err("Unsupported".into()) }
    async fn signout(&self, _account_id: String) -> Result<()> { Err("Unsupported".into()) }
    async fn authorize(&self, _code: String) -> Result<()> { Err("Unsupported".into()) }

    // core domain
    async fn search(&self, _term: String) -> Result<SearchResult> { Err("Unsupported".into()) }
    async fn fetch_user_playlists(&self, _pagination: Pagination) -> Result<(Vec<QueryablePlaylist>, Pagination)> { Err("Unsupported".into()) }
    async fn get_playlist_content(&self, _playlist: QueryablePlaylist, _pagination: Pagination) -> Result<(Vec<Song>, Pagination)> { Err("Unsupported".into()) }
    async fn get_playback_url(&self, _song: Song, _player: String) -> Result<String> { Err("Unsupported".into()) }

    async fn match_url(&self, _url: String) -> Result<bool> { Err("Unsupported".into()) }
    async fn playlist_from_url(&self, _url: String) -> Result<QueryablePlaylist> { Err("Unsupported".into()) }
    async fn song_from_url(&self, _url: String) -> Result<Song> { Err("Unsupported".into()) }

    async fn get_suggestions(&self) -> Result<Vec<Song>> { Err("Unsupported".into()) }
    async fn get_album_content(&self, _album: QueryableAlbum, _pagination: Pagination) -> Result<(Vec<Song>, Pagination)> { Err("Unsupported".into()) }
    async fn get_artist_content(&self, _artist: QueryableArtist, _pagination: Pagination) -> Result<(Vec<Song>, Pagination)> { Err("Unsupported".into()) }

    async fn get_lyrics(&self, _song: Song) -> Result<String> { Err("Unsupported".into()) }
}
