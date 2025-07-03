use async_trait::async_trait;
use anyhow::Result;
use super::{MusicSource, UnifiedSong, SourceSearchResult};
use super::traits::Playlist;

/// B站音乐源
pub struct BilibiliSource {
    client: reqwest::Client,
    base_url: String,
}

impl BilibiliSource {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.bilibili.com".to_string(),
        }
    }
}

#[async_trait]
impl MusicSource for BilibiliSource {
    async fn search_songs(&self, query: &str, page: u32, page_size: u32) -> Result<SourceSearchResult> {
        // TODO: 实现B站音乐搜索API调用
        Ok(SourceSearchResult {
            source_name: "bilibili".to_string(),
            total: 0,
            songs: vec![],
        })
    }

    async fn get_song_url(&self, song_id: &str) -> Result<String> {
        // TODO: 实现获取音频播放链接
        Ok(String::new())
    }

    async fn get_lyrics(&self, song_id: &str) -> Result<Option<String>> {
        // TODO: B站一般没有歌词
        Ok(None)
    }

    async fn get_artist_songs(&self, artist_id: &str, page: u32) -> Result<Vec<UnifiedSong>> {
        // TODO: 实现获取UP主的音乐视频
        Ok(vec![])
    }

    async fn get_album_songs(&self, album_id: &str) -> Result<Vec<UnifiedSong>> {
        // TODO: 实现获取专辑/合集
        Ok(vec![])
    }

    async fn get_playlist(&self, playlist_id: &str) -> Result<Playlist> {
        // TODO: 实现获取播放列表
        Ok(Playlist {
            id: playlist_id.to_string(),
            name: String::new(),
            description: None,
            cover_url: None,
            creator: String::new(),
            song_count: 0,
            songs: vec![],
            created_at: None,
        })
    }

    async fn get_hot_search(&self) -> Result<Vec<String>> {
        // TODO: 实现获取热门搜索词
        Ok(vec![])
    }

    fn source_name(&self) -> &'static str {
        "哔哩哔哩"
    }

    fn requires_auth(&self) -> bool {
        false
    }

    fn supports_download(&self) -> bool {
        false // B站一般不支持直接下载
    }

    fn supports_hq_audio(&self) -> bool {
        false // B站音质相对较低
    }
}