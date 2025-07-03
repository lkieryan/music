pub mod traits;
pub mod local;
pub mod kugou;
pub mod netease;
pub mod qq_music;
pub mod bilibili;

use std::collections::HashMap;
use anyhow::Result;

pub use traits::*;
pub use local::*;

/// 音源管理器
pub struct SourceManager {
    sources: HashMap<String, Box<dyn MusicSource + Send + Sync>>,
    local_source: LocalMusicSource,
    default_source: String,
}

impl SourceManager {
    pub fn new() -> Self {
        let mut sources: HashMap<String, Box<dyn MusicSource + Send + Sync>> = HashMap::new();
        
        // TODO: 注册各个音源
        // sources.insert("kugou".into(), Box::new(kugou::KugouSource::new()));
        // sources.insert("netease".into(), Box::new(netease::NeteaseSource::new()));
        // sources.insert("qq".into(), Box::new(qq_music::QQMusicSource::new()));
        
        Self {
            sources,
            local_source: LocalMusicSource::new(),
            default_source: "local".into(),
        }
    }
    
    pub async fn unified_search(&self, query: &str) -> Result<UnifiedSearchResult> {
        // TODO: 同时搜索多个音源并聚合结果
        Ok(UnifiedSearchResult {
            total: 0,
            songs: vec![],
            source_results: HashMap::new(),
        })
    }
    
    pub fn get_source(&self, name: &str) -> Option<&Box<dyn MusicSource + Send + Sync>> {
        self.sources.get(name)
    }
    
    pub fn get_local_source(&self) -> &LocalMusicSource {
        &self.local_source
    }
}

/// 统一搜索结果
#[derive(Debug)]
pub struct UnifiedSearchResult {
    pub total: u32,
    pub songs: Vec<UnifiedSong>,
    pub source_results: HashMap<String, SourceSearchResult>,
}

/// 单个音源的搜索结果
#[derive(Debug)]
pub struct SourceSearchResult {
    pub source_name: String,
    pub total: u32,
    pub songs: Vec<UnifiedSong>,
}

/// 统一歌曲数据结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnifiedSong {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: std::time::Duration,
    pub source: MusicSourceType,
    pub source_id: String,
    pub cover_url: Option<String>,
    pub play_url: Option<String>,
    pub local_path: Option<std::path::PathBuf>,
    pub metadata: SongMetadata,
}

/// 音乐来源类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MusicSourceType {
    Local,
    Kugou,
    Netease,
    QQMusic,
    Bilibili,
}

/// 歌曲元数据
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SongMetadata {
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub codec: Option<String>,
    pub file_size: Option<u64>,
    pub lyrics: Option<String>,
}