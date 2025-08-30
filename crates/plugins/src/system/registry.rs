//! Plugin registry for managing plugins

// use async_trait::async_trait;  // Not currently used
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::system::core::*;
use crate::system::types::*;
use crate::PluginResult;

/// Plugin trait information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PluginTrait {
    /// AudioProvider trait for music/audio functionality
    AudioProvider,
    /// AuthProvider trait for authentication
    AuthProvider,
    /// Custom trait with name
    Custom(String),
}

/// Plugin registry implementation
pub struct PluginRegistry {
    /// Registered plugins
    plugins: Arc<RwLock<HashMap<Uuid, Arc<Mutex<dyn Plugin>>>>>,
    
    /// Plugin traits mapping - which plugins implement which traits
    plugin_traits: Arc<RwLock<HashMap<Uuid, Vec<PluginTrait>>>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_traits: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a plugin
    pub async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> PluginResult<()> {
        let plugin_id = plugin.id();
        
        // Determine which traits this plugin implements
        let traits = self.determine_plugin_traits(&plugin);
        
        // We need to convert the Box<dyn Plugin> to a type that implements Plugin
        // Let's create a wrapper that implements Plugin
        let plugin_wrapper = PluginWrapper::new(plugin);
        let plugin_mutex = Arc::new(Mutex::new(plugin_wrapper));
        
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin_id, plugin_mutex);
        drop(plugins);
        
        // Store the trait information
        let mut plugin_traits = self.plugin_traits.write().await;
        plugin_traits.insert(plugin_id, traits);
        
        Ok(())
    }
    
    /// Unregister a plugin
    pub async fn unregister_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        let mut plugins = self.plugins.write().await;
        plugins.remove(&plugin_id);
        drop(plugins);
        
        // Remove trait information
        let mut plugin_traits = self.plugin_traits.write().await;
        plugin_traits.remove(&plugin_id);
        
        Ok(())
    }
    
    /// Get a plugin by ID
    pub async fn get_plugin(&self, plugin_id: Uuid) -> PluginResult<Option<Arc<Mutex<dyn Plugin>>>> {
        let plugins = self.plugins.read().await;
        Ok(plugins.get(&plugin_id).cloned())
    }
    
    /// Get all plugins
    pub async fn get_all_plugins(&self) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>> {
        let plugins = self.plugins.read().await;
        Ok(plugins.values().cloned().collect())
    }
    
    /// Find plugins by type
    pub async fn find_plugins_by_type(&self, plugin_type: PluginType) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>> {
        let plugins = self.plugins.read().await;
        let mut candidates = Vec::new();
        
        // Collect candidate plugin IDs first
        for (plugin_id, plugin) in plugins.iter() {
            candidates.push((*plugin_id, plugin.clone()));
        }
        
        // Release the read lock
        drop(plugins);
        
        // Now check each candidate
        let mut result = Vec::new();
        for (_plugin_id, plugin) in candidates {
            let plugin_guard = plugin.lock().unwrap();
            if plugin_guard.plugin_type() == plugin_type {
                drop(plugin_guard); // Release the lock before cloning
                result.push(plugin.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Find plugins by capability
    pub async fn find_plugins_by_capability(&self, capability: PluginCapability) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>> {
        let plugins = self.plugins.read().await;
        let mut candidates = Vec::new();
        
        // Collect candidate plugins first
        for (plugin_id, plugin) in plugins.iter() {
            candidates.push((*plugin_id, plugin.clone()));
        }
        
        // Release the read lock
        drop(plugins);
        
        // Now check each candidate
        let mut result = Vec::new();
        for (_plugin_id, plugin) in candidates {
            let plugin_guard = plugin.lock().unwrap();
            if plugin_guard.capabilities().contains(&capability) {
                drop(plugin_guard); // Release the lock before cloning
                result.push(plugin.clone());
            }
        }
        
        Ok(result)
    }
    
    /// Get plugins that implement a specific trait
    pub async fn find_plugins_by_trait(&self, trait_type: PluginTrait) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>> {
        let plugin_traits = self.plugin_traits.read().await;
        let mut result = Vec::new();
        
        // Find plugins that implement the requested trait
        for (plugin_id, traits) in plugin_traits.iter() {
            if traits.contains(&trait_type) {
                if let Ok(Some(plugin)) = self.get_plugin(*plugin_id).await {
                    result.push(plugin);
                }
            }
        }
        
        Ok(result)
    }
    
    /// Check if a plugin implements a specific trait
    pub async fn plugin_implements_trait(&self, plugin_id: Uuid, trait_type: &PluginTrait) -> PluginResult<bool> {
        let plugin_traits = self.plugin_traits.read().await;
        
        if let Some(traits) = plugin_traits.get(&plugin_id) {
            Ok(traits.contains(trait_type))
        } else {
            Ok(false)
        }
    }
    
    /// Get all traits implemented by a plugin
    pub async fn get_plugin_traits(&self, plugin_id: Uuid) -> PluginResult<Vec<PluginTrait>> {
        let plugin_traits = self.plugin_traits.read().await;
        
        if let Some(traits) = plugin_traits.get(&plugin_id) {
            Ok(traits.clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Determine which traits a plugin implements
    fn determine_plugin_traits(&self, plugin: &Box<dyn Plugin>) -> Vec<PluginTrait> {
        let mut traits = Vec::new();
        
        // Check if plugin implements AudioProvider trait
        if plugin.as_any().downcast_ref::<crate::internal::bilibili::BilibiliPlugin>().is_some() {
            traits.push(PluginTrait::AudioProvider);
        }
        
        // Check if plugin implements AuthProvider trait
        if plugin.as_any().downcast_ref::<crate::internal::bilibili::BilibiliPlugin>().is_some() {
            // BilibiliPlugin also implements AuthProvider
            traits.push(PluginTrait::AuthProvider);
        }
        
        // Note: We would add similar checks for other plugin types
        // when they implement the respective traits
        
        traits
    }
}

#[async_trait::async_trait]
impl crate::system::core::PluginRegistry for PluginRegistry {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> PluginResult<()> {
        self.register_plugin(plugin).await
    }
    
    /// Unregister a plugin
    async fn unregister_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        self.unregister_plugin(plugin_id).await
    }
    
    /// Get a plugin by ID
    async fn get_plugin(&self, plugin_id: Uuid) -> PluginResult<Option<Arc<Mutex<dyn Plugin>>>> {
        self.get_plugin(plugin_id).await
    }
    
    /// Get all plugins
    async fn get_all_plugins(&self) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>> {
        self.get_all_plugins().await
    }
    
    /// Find plugins by type
    async fn find_plugins_by_type(&self, plugin_type: PluginType) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>> {
        self.find_plugins_by_type(plugin_type).await
    }
    
    /// Find plugins by capability
    async fn find_plugins_by_capability(&self, capability: PluginCapability) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>> {
        self.find_plugins_by_capability(capability).await
    }
}

impl fmt::Debug for PluginRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginRegistry")
            .finish()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper to convert Box<dyn Plugin> to a type that implements Plugin
struct PluginWrapper {
    plugin: Box<dyn Plugin>,
}

impl PluginWrapper {
    fn new(plugin: Box<dyn Plugin>) -> Self {
        Self { plugin }
    }
}

#[async_trait::async_trait]
impl Plugin for PluginWrapper {
    fn metadata(&self) -> PluginMetadata {
        self.plugin.metadata()
    }
    
    fn id(&self) -> Uuid {
        self.plugin.id()
    }
    
    fn plugin_type(&self) -> PluginType {
        self.plugin.plugin_type()
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        self.plugin.capabilities()
    }
    
    fn initialize(&mut self, context: &PluginContext) -> PluginResult<()> {
        self.plugin.initialize(context)
    }
    
    fn start(&mut self) -> PluginResult<()> {
        self.plugin.start()
    }
    
    fn stop(&mut self) -> PluginResult<()> {
        self.plugin.stop()
    }
    
    fn destroy(&mut self) -> PluginResult<()> {
        self.plugin.destroy()
    }
    
    fn status(&self) -> PluginResult<PluginStatus> {
        self.plugin.status()
    }
    
    async fn handle_event(&mut self, event: PluginEvent) -> PluginResult<Option<PluginResponse>> {
        self.plugin.handle_event(event).await
    }
    
    fn health_check(&self) -> PluginResult<HealthStatus> {
        self.plugin.health_check()
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self.plugin.as_any()
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self.plugin.as_any_mut()
    }
}