//! Plugin manager for Tauri backend
//!
//! This module provides additional management functionality for plugins.

use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use plugins::system::manager::PluginManager;
use plugins::system::types::{PluginMetadata, PluginStatus, HealthStatus};
// use plugins::system::types::{PluginMetadata, PluginStatus, HealthStatus, PluginError};
// use tauri::State;
use types::errors::Result;

/// Plugin information structure for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID
    pub id: String,
    
    /// Plugin name
    pub name: String,
    
    /// Display name
    pub display_name: String,
    
    /// Description
    pub description: String,
    
    /// Version
    pub version: String,
    
    /// Author
    pub author: String,
    
    /// Plugin type
    pub plugin_type: String,
    
    /// Current status
    pub status: String,
    
    /// Health status
    pub health: String,
    
    /// Whether the plugin is enabled
    pub enabled: bool,
    
    /// Icon path (optional; file path or relative path)
    pub icon: Option<String>,
}

impl From<PluginMetadata> for PluginInfo {
    fn from(metadata: PluginMetadata) -> Self {
        Self {
            id: metadata.id.to_string(),
            name: metadata.name,
            display_name: metadata.display_name,
            description: metadata.description,
            version: metadata.version.to_string(),
            author: metadata.author,
            plugin_type: metadata.plugin_type.to_string(),
            status: "Unknown".to_string(), // Will be updated with actual status
            health: "Unknown".to_string(), // Will be updated with actual health
            enabled: true, // Will be updated with actual enabled status
            icon: None,
        }
    }
}

#[derive(Clone)]
pub struct PluginHandler {
    plugin_manager: Arc<PluginManager>,
}

impl PluginHandler {
    pub fn new(plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            plugin_manager,
        }
    }
    
    /// Get all plugins
    pub async fn get_plugins(&self) -> Result<Vec<PluginInfo>> {
        let plugins = self.plugin_manager.get_all_plugins().await
            .map_err(|e| format!("Failed to get plugins: {}", e))?;
        let mut plugin_infos = Vec::new();
        
        for plugin in plugins {
            // Extract metadata without holding the lock during async operations
            let (metadata, plugin_id) = {
                let plugin_guard = plugin.lock().unwrap();
                let metadata = plugin_guard.metadata();
                let plugin_id = plugin_guard.id();
                (metadata, plugin_id)
            };
            
            let mut plugin_info = PluginInfo::from(metadata);
            
            // Get plugin status from lifecycle manager to avoid async issues
            let status = self.plugin_manager.get_plugin_status(plugin_id).await
                .map_err(|e| format!("Failed to get plugin status: {}", e))?;
            plugin_info.status = match status {
                PluginStatus::Unloaded => "Unloaded".to_string(),
                PluginStatus::Loaded => "Loaded".to_string(),
                PluginStatus::Ready => "Ready".to_string(),
                PluginStatus::Running => "Running".to_string(),
                PluginStatus::Stopped => "Stopped".to_string(),
                PluginStatus::Error(_) => "Error".to_string(),
            };
            
            // Get plugin health from lifecycle manager to avoid async issues
            let health = self.plugin_manager.health_check_plugin(plugin_id).await
                .map_err(|e| format!("Failed to check plugin health: {}", e))?;
            plugin_info.health = match health {
                HealthStatus::Healthy => "Healthy".to_string(),
                HealthStatus::Unhealthy(_) => "Unhealthy".to_string(),
                HealthStatus::Maintenance => "Maintenance".to_string(),
            };
            
            // Get actual enabled status from state manager (DB)
            plugin_info.enabled = self.plugin_manager
                .get_plugin_enabled(plugin_id)
                .map_err(|e| format!("Failed to get plugin enabled state: {}", e))?;
            // Get icon path from DB state if available
            plugin_info.icon = self.plugin_manager
                .get_plugin_icon(plugin_id)
                .map_err(|e| format!("Failed to get plugin icon: {}", e))?;
            
            plugin_infos.push(plugin_info);
        }
        
        Ok(plugin_infos)
    }
    
    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: String) -> Result<PluginInfo> {
        let uuid = Uuid::parse_str(&plugin_id)
            .map_err(|_| "Invalid plugin ID format".to_string())?;
            
        let plugin = self.plugin_manager.get_plugin(uuid).await
            .map_err(|e| format!("Failed to get plugin: {}", e))?
            .ok_or("Plugin not found")?;
            
        // Extract metadata without holding the lock during async operations
        let (metadata, plugin_id) = {
            let plugin_guard = plugin.lock().unwrap();
            let metadata = plugin_guard.metadata();
            let plugin_id = plugin_guard.id();
            (metadata, plugin_id)
        };
        
        let mut plugin_info = PluginInfo::from(metadata);
        
        // Get plugin status from lifecycle manager to avoid async issues
        let status = self.plugin_manager.get_plugin_status(plugin_id).await
            .map_err(|e| format!("Failed to get plugin status: {}", e))?;
        plugin_info.status = match status {
            PluginStatus::Unloaded => "Unloaded".to_string(),
            PluginStatus::Loaded => "Loaded".to_string(),
            PluginStatus::Ready => "Ready".to_string(),
            PluginStatus::Running => "Running".to_string(),
            PluginStatus::Stopped => "Stopped".to_string(),
            PluginStatus::Error(_) => "Error".to_string(),
        };
        
        // Get plugin health from lifecycle manager to avoid async issues
        let health = self.plugin_manager.health_check_plugin(plugin_id).await
            .map_err(|e| format!("Failed to check plugin health: {}", e))?;
        plugin_info.health = match health {
            HealthStatus::Healthy => "Healthy".to_string(),
            HealthStatus::Unhealthy(_) => "Unhealthy".to_string(),
            HealthStatus::Maintenance => "Maintenance".to_string(),
        };

        // Get actual enabled status from state manager (DB)
        plugin_info.enabled = self.plugin_manager
            .get_plugin_enabled(plugin_id)
            .map_err(|e| format!("Failed to get plugin enabled state: {}", e))?;
        // Get icon path from DB state if available
        plugin_info.icon = self.plugin_manager
            .get_plugin_icon(plugin_id)
            .map_err(|e| format!("Failed to get plugin icon: {}", e))?;
        
        Ok(plugin_info)
    }
    
    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: String) -> Result<()> {
        let uuid = Uuid::parse_str(&plugin_id)
            .map_err(|_| "Invalid plugin ID format".to_string())?;
            
        self.plugin_manager.enable_plugin(uuid).await
            .map_err(|e| format!("Failed to enable plugin: {}", e).into())
    }
    
    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: String) -> Result<()> {
        let uuid = Uuid::parse_str(&plugin_id)
            .map_err(|_| "Invalid plugin ID format".to_string())?;
            
        self.plugin_manager.disable_plugin(uuid).await
            .map_err(|e| format!("Failed to disable plugin: {}", e).into())
    }
    
    /// Start a plugin (unified with enable: persists enabled=true and starts)
    pub async fn start_plugin(&self, plugin_id: String) -> Result<()> {
        let uuid = Uuid::parse_str(&plugin_id)
            .map_err(|_| "Invalid plugin ID format".to_string())?;
        // Delegate to enable to keep DB and runtime in sync
        self.plugin_manager.enable_plugin(uuid).await
            .map_err(|e| format!("Failed to start plugin: {}", e).into())
    }
    
    /// Stop a plugin (unified with disable: persists enabled=false and stops)
    pub async fn stop_plugin(&self, plugin_id: String) -> Result<()> {
        let uuid = Uuid::parse_str(&plugin_id)
            .map_err(|_| "Invalid plugin ID format".to_string())?;
        // Delegate to disable to keep DB and runtime in sync
        self.plugin_manager.disable_plugin(uuid).await
            .map_err(|e| format!("Failed to stop plugin: {}", e).into())
    }
    
    /// Load a plugin from file
    pub async fn load_plugin(&self, plugin_path: String) -> Result<()> {
        let path = std::path::Path::new(&plugin_path);
        
        self.plugin_manager.load_plugin_from_file(path).await
            .map_err(|e| format!("Failed to load plugin: {}", e).into())
    }
    
    /// Get the underlying plugin manager
    pub fn plugin_manager(&self) -> Arc<PluginManager> {
        Arc::clone(&self.plugin_manager)
    }
}
