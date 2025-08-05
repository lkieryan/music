use super::base::*;
use async_trait::async_trait;
use regex::Regex;
use types::errors::Result;

#[derive(Debug)]
pub struct SpotifyProvider {
    key: String,
}

impl SpotifyProvider {
    pub fn from_config(key: String, _cfg: serde_json::Value) -> Result<Self> { Ok(Self { key }) }
}

#[async_trait]
impl BaseProvider for SpotifyProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "spotify".into(),
            display_name: "Spotify".into(),
            description: "Spotify provider".into(),
            capabilities: vec![
                ProviderCapability::Search,
                ProviderCapability::Playlists,
                ProviderCapability::StreamUrl,
                ProviderCapability::UrlMatch,
            ],
            ..Default::default()
        }
    }
    fn key(&self) -> String { self.key.clone() }

    // 示例：返回几条模拟搜索结果
    async fn search(&self, term: String) -> Result<SearchResult> {
        let mut songs = vec![];
        for i in 1..=3 {
            songs.push(Song {
                id: format!("spfake{:02}", i),
                title: format!("{} - Spotify Track {}", term, i),
                artist: "Spotify Artist".into(),
                duration_ms: Some(200_000 + i * 1000),
                provider_extension: Some(self.key()),
            });
        }
        Ok(SearchResult { songs })
    }

    // 示例：根据 id 拼接一个可用的播放链接（演示用）
    async fn get_playback_url(&self, song: Song, _player: String) -> Result<String> {
        let looks_like_track_id = Regex::new(r"^[A-Za-z0-9]{8,}$").expect("valid regex");
        if looks_like_track_id.is_match(&song.id) {
            Ok(format!("https://open.spotify.com/track/{}", song.id))
        } else {
            // 回退到搜索页（演示）
            Ok("https://open.spotify.com/search".to_string())
        }
    }

    // 示例：匹配常见的 spotify 链接形式
    async fn match_url(&self, url: String) -> Result<bool> {
        let re = Regex::new(
            r#"^(https?://open\.spotify\.com/(track|playlist|embed)/[A-Za-z0-9]+|spotify:(track|playlist):[A-Za-z0-9]+)"#
        ).expect("valid regex");
        Ok(re.is_match(&url))
    }
}
