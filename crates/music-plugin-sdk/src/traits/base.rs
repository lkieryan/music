//! Base plugin trait
//! 
//! This module defines the base trait that all plugins must implement.

use async_trait::async_trait;
use crate::types::base::{
    PluginMetadata, PluginContext, PluginResult, PluginStatus, PluginConfig
};

/// Base plugin trait that all plugins must implement
#[async_trait]
pub trait BasePlugin: Send + Sync + std::fmt::Debug {

    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Get plugin unique identifier
    fn id(&self) -> uuid::Uuid {
        self.metadata().id
    }
    
    /// Initialize the plugin with host context
    async fn initialize(&mut self, context: &PluginContext) -> PluginResult<()>;
    
    /// Start the plugin
    async fn start(&mut self) -> PluginResult<()>;
    
    /// Stop the plugin
    async fn stop(&mut self) -> PluginResult<()>;
    
    /// Pause the plugin
    async fn pause(&mut self) -> PluginResult<()> {
        // Default implementation - plugins can override if needed
        Ok(())
    }
    
    /// Resume the plugin
    async fn resume(&mut self) -> PluginResult<()> {
        // Default implementation - plugins can override if needed
        Ok(())
    }
    
    /// Get current plugin status
    fn status(&self) -> PluginStatus;
    
    /// Configure the plugin
    async fn configure(&mut self, config: PluginConfig) -> PluginResult<()>;
    
    /// Get plugin configuration schema
    fn config_schema(&self) -> Option<serde_json::Value> {
        self.metadata().config_schema.clone()
    }
}