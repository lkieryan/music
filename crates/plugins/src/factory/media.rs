//! Audio Plugin Factory
//!
//! Provides unified access to all media plugins (built-in and external)
//! through true polymorphism using MediaPlugin trait objects.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use types::settings::music::{MusicSourceSelection, MusicSourceMode};
use music_plugin_sdk::traits::media::MediaPlugin;

/// Audio plugin factory for true polymorphic access to media plugins
pub struct MediaPluginFactory {
    /// Direct storage of MediaPlugin trait objects - enables true polymorphism!
    /// Key insight: Store MediaPlugin directly, not the original Plugin
    media_plugins: HashMap<Uuid, Arc<Mutex<dyn MediaPlugin + Send + Sync>>>,
    
    /// Plugin enabled status
    enabled_plugins: HashMap<Uuid, bool>,
}

impl MediaPluginFactory {
    /// Create a new audio plugin factory
    pub fn new() -> Self {
        Self {
            media_plugins: HashMap::new(),
            enabled_plugins: HashMap::new(),
        }
    }
    
    /// Register media plugin directly to factory, avoiding subsequent filtering
    /// Called during plugin loading, only media plugins are registered here
    pub fn register_to_media_factory(
        &mut self, 
        plugin_id: Uuid, 
        media_plugin: Arc<tokio::sync::Mutex<dyn MediaPlugin + Send + Sync>>,
        enabled: bool
    ) {
        self.media_plugins.insert(plugin_id, media_plugin);
        self.enabled_plugins.insert(plugin_id, enabled);
    }
    
    /// Register external media plugin to factory (accepts Arc<Mutex<>> type)
    pub fn register_external_media_plugin_to_factory(
        &mut self, 
        plugin_id: Uuid, 
        media_plugin: Arc<tokio::sync::Mutex<dyn MediaPlugin + Send + Sync>>,
        enabled: bool
    ) {
        self.media_plugins.insert(plugin_id, media_plugin);
        self.enabled_plugins.insert(plugin_id, enabled);
    }
    
    /// Update media plugin status
    pub fn update_media_plugin_status(&mut self, plugin_id: Uuid, enabled: bool) {
        self.enabled_plugins.insert(plugin_id, enabled);
    }
    
    
    /// Get MediaPlugin by ID - THE MAGIC METHOD!
    /// Returns MediaPlugin trait object - no downcasting needed by caller!
    /// This enables true polymorphism: caller just calls plugin.search() without knowing the type
    pub fn get_media_plugin(&self, plugin_id: Uuid) -> Option<Arc<tokio::sync::Mutex<dyn MediaPlugin + Send + Sync>>> {
        // Check if plugin is enabled
        if !self.enabled_plugins.get(&plugin_id).copied().unwrap_or(false) {
            return None;
        }
        
        // Return MediaPlugin trait object directly - pure polymorphism!
        self.media_plugins.get(&plugin_id).cloned()
    }
    
    
    /// Get MediaPlugins by music source selection
    /// Returns list of MediaPlugin trait objects for true polymorphic access
    pub fn get_media_plugins_by_selection(
        &self, 
        selection: &MusicSourceSelection
    ) -> Vec<(Uuid, Arc<tokio::sync::Mutex<dyn MediaPlugin + Send + Sync>>)> {
        let target_ids: Vec<Uuid> = match &selection.mode {
            MusicSourceMode::All => {
                // Return all enabled plugins
                self.enabled_plugins.iter()
                    .filter(|(_, &enabled)| enabled)
                    .map(|(&id, _)| id)
                    .collect()
            },
            MusicSourceMode::Single | MusicSourceMode::Many => {
                // Convert string IDs to UUIDs
                selection.ids.iter()
                    .filter_map(|id_str| Uuid::parse_str(id_str).ok())
                    .collect()
            },
        };
        
        target_ids.into_iter()
            .filter_map(|id| self.get_media_plugin(id).map(|plugin| (id, plugin)))
            .collect()
    }
    
    /// Update plugin enabled status
    pub fn set_plugin_enabled(&mut self, plugin_id: Uuid, enabled: bool) {
        self.enabled_plugins.insert(plugin_id, enabled);
    }
    
    /// Get all registered plugin IDs
    pub fn get_plugin_ids(&self) -> Vec<Uuid> {
        self.media_plugins.keys().copied().collect()
    }
    
    /// Get enabled plugin IDs
    pub fn get_enabled_plugin_ids(&self) -> Vec<Uuid> {
        self.enabled_plugins.iter()
            .filter(|(_, &enabled)| enabled)
            .map(|(&id, _)| id)
            .collect()
    }
}

// Legacy compatibility - for any existing code
pub struct AudioProviderWrapper {
    _plugin_id: Uuid,
}

impl AudioProviderWrapper {
    pub fn id(&self) -> Uuid { self._plugin_id }
    pub async fn search(&self, _query: &str) -> Result<music_plugin_sdk::types::SearchResult, String> {
        Err("Legacy wrapper - use get_media_plugins_by_selection instead".to_string())
    }
    pub async fn get_stream_url(&self, _track_id: &str) -> Result<String, String> {
        Err("Legacy wrapper - use get_media_plugins_by_selection instead".to_string())
    }
}

