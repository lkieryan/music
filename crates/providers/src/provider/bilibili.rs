use super::base::*;
use async_trait::async_trait;
use regex::Regex;
use types::errors::Result;

#[derive(Debug)]
pub struct BilibiliProvider {
    key: String,
}

impl BilibiliProvider {
    pub fn from_config(key: String, _cfg: serde_json::Value) -> Result<Self> { Ok(Self { key }) }
}

#[async_trait]
impl BaseProvider for BilibiliProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "bilibili".into(),
            display_name: "Bilibili".into(),
            description: "Bilibili provider".into(),
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

    // 简化示例：用关键词生成伪数据，真实实现应调用 B 站 API 或解析搜索页
    async fn search(&self, term: String) -> Result<SearchResult> {
        // 这里返回 3 条模拟结果，标题/作者中包含 term
        let mut songs = vec![];
        for i in 1..=3 {
            songs.push(Song {
                id: format!("bvfake{:02}", i),
                title: format!("{} - 视频{}", term, i),
                artist: "UP主".into(),
                duration_ms: Some(180_000 + i * 1000),
                provider_extension: Some(self.key()),
            });
        }
        Ok(SearchResult { songs })
    }

    // 简化示例：将传入的 Song.id 拼接成一个伪播放地址
    async fn get_playback_url(&self, song: Song, _player: String) -> Result<String> {
        Ok(format!("https://www.bilibili.com/video/{}", song.id))
    }

    // 简化的 URL 匹配：匹配 bilibili 视频链接
    async fn match_url(&self, url: String) -> Result<bool> {
        let re = Regex::new(r"https?://(www\.)?bilibili\.com/video/[A-Za-z0-9]+")
            .expect("valid regex");
        Ok(re.is_match(&url))
    }
}
