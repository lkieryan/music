//! Spotify provider plugin (built-in)
use async_trait::async_trait;
use uuid::Uuid;
use semver::Version;

use crate::system::core::*;
use crate::system::types::*;
use crate::PluginResult;

#[derive(Debug)]
pub struct SpotifyPlugin {
    metadata: PluginMetadata,
    status: PluginStatus,
    context: Option<PluginContext>,
}

impl SpotifyPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            // Stable deterministic UUID for builtin
            id: Uuid::new_v5(&Uuid::NAMESPACE_OID, b"builtin:spotify"),
            name: "spotify".to_string(),
            display_name: "Spotify Music".to_string(),
            description: "Spotify music provider plugin".to_string(),
            version: Version::new(1, 0, 0),
            author: "Music Player Team".to_string(),
            homepage: Some("https://open.spotify.com".to_string()),
            repository: None,
            license: Some("MIT".to_string()),
            icon: None,
            keywords: vec!["spotify".into(), "music".into(), "audio".into()],
            plugin_type: PluginType::AudioProvider,
            capabilities: vec![
                PluginCapability::Search,
                PluginCapability::Playlists,
                PluginCapability::Streaming,
                PluginCapability::Authentication,
            ],
            dependencies: vec![],
            min_system_version: None,
            max_system_version: None,
        };

        Self { metadata, status: PluginStatus::Unloaded, context: None }
    }
}

#[async_trait]
impl Plugin for SpotifyPlugin {
    fn metadata(&self) -> PluginMetadata { self.metadata.clone() }
    fn id(&self) -> Uuid { self.metadata.id }
    fn plugin_type(&self) -> PluginType { self.metadata.plugin_type.clone() }
    fn capabilities(&self) -> Vec<PluginCapability> { self.metadata.capabilities.clone() }
    fn initialize(&mut self, context: &PluginContext) -> PluginResult<()> { self.context = Some(context.clone()); self.status = PluginStatus::Ready; Ok(()) }
    fn start(&mut self) -> PluginResult<()> { self.status = PluginStatus::Running; Ok(()) }
    fn stop(&mut self) -> PluginResult<()> { self.status = PluginStatus::Stopped; Ok(()) }
    fn destroy(&mut self) -> PluginResult<()> { self.status = PluginStatus::Unloaded; self.context = None; Ok(()) }
    fn status(&self) -> PluginResult<PluginStatus> { Ok(self.status.clone()) }
    async fn handle_event(&mut self, _event: PluginEvent) -> PluginResult<Option<PluginResponse>> { Ok(None) }
    fn health_check(&self) -> PluginResult<HealthStatus> { Ok(HealthStatus::Healthy) }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for SpotifyPlugin { fn default() -> Self { Self::new() } }

