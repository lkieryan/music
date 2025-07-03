use async_trait::async_trait;
use anyhow::Result;
use super::{UnifiedSong, SourceSearchResult};

/// 音乐源通用接口
#[async_trait]
pub trait MusicSource {
    /// 搜索歌曲
    async fn search_songs(&self, query: &str, page: u32, page_size: u32) -> Result<SourceSearchResult>;
    
    /// 获取歌曲播放链接
    async fn get_song_url(&self, song_id: &str) -> Result<String>;
    
    /// 获取歌词
    async fn get_lyrics(&self, song_id: &str) -> Result<Option<String>>;
    
    /// 获取歌手的歌曲列表
    async fn get_artist_songs(&self, artist_id: &str, page: u32) -> Result<Vec<UnifiedSong>>;
    
    /// 获取专辑歌曲列表
    async fn get_album_songs(&self, album_id: &str) -> Result<Vec<UnifiedSong>>;
    
    /// 获取歌单详情
    async fn get_playlist(&self, playlist_id: &str) -> Result<Playlist>;
    
    /// 获取热门搜索词
    async fn get_hot_search(&self) -> Result<Vec<String>>;
    
    /// 音源基本信息
    fn source_name(&self) -> &'static str;
    fn requires_auth(&self) -> bool;
    fn supports_download(&self) -> bool;
    fn supports_hq_audio(&self) -> bool;
}

/// 歌单数据结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub creator: String,
    pub song_count: u32,
    pub songs: Vec<UnifiedSong>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}