use async_trait::async_trait;
use semver::Version;
use uuid::Uuid;
use reqwest::Client;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex as StdMutex};

use crate::system::core::*;
use crate::system::types::*;
use crate::PluginResult;
use music_plugin_sdk::traits::BasePlugin;


/// 字幕缓存条目，包含内容和过期时间
#[derive(Debug, Clone)]
pub struct SubtitleCacheEntry {
    /// 字幕内容
    pub content: serde_json::Value,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub accessed_at: Instant,
    /// 过期时间（从创建开始算起）
    pub ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct BilibiliPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
    context: Option<PluginContext>,
    pub http: Client,
    // Use Arc for shared state to enable Clone
    pub wbi_salt_cache: Arc<RwLock<Option<String>>>,
    pub session_data: Option<String>,
    pub subtitle_cache: Arc<RwLock<std::collections::HashMap<String, SubtitleCacheEntry>>>,
    /// 缓存最大条目数
    pub max_cache_entries: usize,
    /// 缓存条目默认过期时间（24小时）
    pub default_cache_ttl: Duration,
}

impl BilibiliPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            id: Uuid::new_v5(&Uuid::NAMESPACE_OID, b"builtin:bilibili"),
            name: "bilibili".to_string(),
            display_name: "Bilibili Audio".to_string(),
            description: "Bilibili Audio provider plugin".to_string(),
            version: Version::new(1, 0, 0),
            author: "Music Player Team".to_string(),
            homepage: Some("https://www.bilibili.com/audio".to_string()),
            repository: None,
            license: Some("MIT".to_string()),
            icon: None,
            keywords: vec!["bilibili".into(), "audio".into(), "music".into(), "video".into()],
            plugin_type: PluginType::AudioProvider,
            capabilities: vec![PluginCapability::Search, PluginCapability::Playlists, PluginCapability::Streaming],
            dependencies: vec![],
            min_system_version: None,
            max_system_version: None,
        };
        // Build HTTP client with sensible timeouts to avoid hangs
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            metadata,
            status: PluginStatus::Unloaded,
            context: None,
            http,
            wbi_salt_cache: Arc::new(RwLock::new(None)),
            session_data: None,
            subtitle_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_cache_entries: 100, // 最多缓存100个字幕
            default_cache_ttl: Duration::from_secs(24 * 60 * 60), // 24小时过期
        }
    }


    /// 清理过期的字幕缓存条目
    pub async fn cleanup_expired_subtitle_cache(&self) {
        let mut cache = self.subtitle_cache.write().await;
        let now = Instant::now();
        
        cache.retain(|_, entry| {
            entry.created_at + entry.ttl > now
        });
    }

    /// 清理最久未访问的字幕缓存条目（当缓存超过最大限制时）
    pub async fn cleanup_oldest_subtitle_cache(&self) {
        let mut cache = self.subtitle_cache.write().await;
        
        if cache.len() > self.max_cache_entries {
            // 收集所有键并按访问时间排序
            let mut urls: Vec<_> = cache.iter()
                .map(|(url, entry)| (url.clone(), entry.accessed_at))
                .collect();
            urls.sort_by_key(|(_, accessed_at)| *accessed_at);
            
            // 移除最旧的 (len - max_cache_entries) 个条目
            let to_remove = urls.len() - self.max_cache_entries;
            for (url, _) in urls.iter().take(to_remove) {
                cache.remove(url);
            }
        }
    }

    /// 获取字幕缓存统计信息
    pub async fn get_subtitle_cache_stats(&self) -> (usize, usize) {
        let cache = self.subtitle_cache.read().await;
        let now = Instant::now();
        let expired_count = cache.values()
            .filter(|entry| entry.created_at + entry.ttl <= now)
            .count();
        (cache.len(), expired_count)
    }
}

#[async_trait]
impl Plugin for BilibiliPlugin {
    fn metadata(&self) -> PluginMetadata { self.metadata.clone() }
    fn id(&self) -> Uuid { self.metadata.id }
    fn plugin_type(&self) -> PluginType { self.metadata.plugin_type.clone() }
    fn capabilities(&self) -> Vec<PluginCapability> { self.metadata.capabilities.clone() }
    fn initialize(&mut self, context: &PluginContext) -> PluginResult<()> { self.context = Some(context.clone()); self.status = PluginStatus::Ready; Ok(()) }
    fn start(&mut self) -> PluginResult<()> { self.status = PluginStatus::Running; Ok(()) }
    fn stop(&mut self) -> PluginResult<()> { self.status = PluginStatus::Stopped; Ok(()) }
    fn destroy(&mut self) -> PluginResult<()> { self.status = PluginStatus::Unloaded; self.context = None; Ok(()) }
    fn status(&self) -> PluginResult<PluginStatus> { Ok(self.status.clone()) }
    async fn handle_event(&mut self, event: PluginEvent) -> PluginResult<Option<PluginResponse>> {
        match event {
            _ => Ok(None)
        }
    }
    fn health_check(&self) -> PluginResult<HealthStatus> { Ok(HealthStatus::Healthy) }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for BilibiliPlugin { fn default() -> Self { Self::new() } }

// MediaPlugin trait implementation is in audio.rs with full business logic

// Implement SDK Plugin trait for AudioProvider
#[async_trait]
impl BasePlugin for BilibiliPlugin {
    fn metadata(&self) -> music_plugin_sdk::types::base::PluginMetadata {
        music_plugin_sdk::types::base::PluginMetadata {
            id: self.metadata.id,
            name: self.metadata.name.clone(),
            version: self.metadata.version.to_string(),
            description: self.metadata.description.clone(),
            author: self.metadata.author.clone(),
            website: self.metadata.homepage.clone(),
            icon: self.metadata.icon.clone(),
            capabilities: vec![
                music_plugin_sdk::types::base::PluginCapability::Search,
                music_plugin_sdk::types::base::PluginCapability::Playback,
                music_plugin_sdk::types::base::PluginCapability::Network
            ],
            min_sdk_version: "1.0.0".to_string(),
            config_schema: None,
        }
    }

    async fn initialize(&mut self, _context: &music_plugin_sdk::types::base::PluginContext) -> music_plugin_sdk::types::base::PluginResult<()> {
        self.status = PluginStatus::Ready;
        Ok(())
    }

    async fn start(&mut self) -> music_plugin_sdk::types::base::PluginResult<()> {
        self.status = PluginStatus::Running;
        Ok(())
    }

    async fn stop(&mut self) -> music_plugin_sdk::types::base::PluginResult<()> {
        self.status = PluginStatus::Stopped;
        Ok(())
    }

    fn status(&self) -> music_plugin_sdk::types::base::PluginStatus {
        match self.status {
            PluginStatus::Unloaded => music_plugin_sdk::types::base::PluginStatus::Loaded,
            PluginStatus::Ready => music_plugin_sdk::types::base::PluginStatus::Loaded,
            PluginStatus::Running => music_plugin_sdk::types::base::PluginStatus::Running,
            PluginStatus::Stopped => music_plugin_sdk::types::base::PluginStatus::Stopped,
            _ => music_plugin_sdk::types::base::PluginStatus::Error("Plugin error".to_string()),
        }
    }

    async fn configure(&mut self, _config: music_plugin_sdk::types::base::PluginConfig) -> music_plugin_sdk::types::base::PluginResult<()> {
        // Handle configuration if needed
        Ok(())
    }
}
