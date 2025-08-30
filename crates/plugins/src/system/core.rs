//! Core plugin traits and interfaces

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use std::fmt;

use crate::system::types::*;
use crate::PluginResult;

/// Plugin context providing access to system services
#[derive(Clone)]
pub struct PluginContext {
    /// Plugin host reference
    pub host: Arc<dyn PluginHost>,
    
    /// Plugin registry reference
    pub registry: Arc<dyn PluginRegistry>,
    
    /// Plugin settings
    pub settings: serde_json::Value,
}

impl fmt::Debug for PluginContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginContext")
            .field("settings", &self.settings)
            .finish()
    }
}

/// Plugin trait defining the core plugin interface
#[async_trait]
pub trait Plugin: Send + Sync + std::any::Any {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Get plugin ID
    fn id(&self) -> Uuid;
    
    /// Get plugin type
    fn plugin_type(&self) -> PluginType;
    
    /// Get plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability>;
    
    /// Initialize the plugin
    fn initialize(&mut self, context: &PluginContext) -> PluginResult<()>;
    
    /// Start the plugin
    fn start(&mut self) -> PluginResult<()>;
    
    /// Stop the plugin
    fn stop(&mut self) -> PluginResult<()>;
    
    /// Destroy the plugin and clean up resources
    fn destroy(&mut self) -> PluginResult<()>;
    
    /// Get plugin status
    fn status(&self) -> PluginResult<PluginStatus>;
    
    /// Handle plugin events
    async fn handle_event(&mut self, event: PluginEvent) -> PluginResult<Option<PluginResponse>>;
    
    /// Perform health check
    fn health_check(&self) -> PluginResult<HealthStatus>;
    
    /// Convert to Any trait object for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Convert to mutable Any trait object for downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Plugin host trait defining the host interface
#[async_trait]
pub trait PluginHost: Send + Sync {
    /// Get host information
    fn host_info(&self) -> HostInfo;
    
    /// Log a message from a plugin
    async fn log(&self, plugin_id: Uuid, level: LogLevel, message: &str);
    
    /// Emit an event from a plugin
    async fn emit_event(&self, plugin_id: Uuid, event: PluginEvent) -> PluginResult<()>;
    
    /// Request a service from the host
    async fn request_service(&self, plugin_id: Uuid, service: &str, data: serde_json::Value) -> PluginResult<serde_json::Value>;
    
    /// Get a setting value
    async fn get_setting(&self, plugin_id: Uuid, key: &str) -> PluginResult<Option<serde_json::Value>>;
    
    /// Set a setting value
    async fn set_setting(&self, plugin_id: Uuid, key: &str, value: serde_json::Value) -> PluginResult<()>;
}

/// Host information structure
#[derive(Debug, Clone)]
pub struct HostInfo {
    /// Host version
    pub version: String,
    
    /// Host platform
    pub platform: String,
    
    /// Available services
    pub services: Vec<String>,
}

/// Log level enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLevel {
    /// Debug level
    Debug,
    
    /// Info level
    Info,
    
    /// Warning level
    Warn,
    
    /// Error level
    Error,
}

/// Plugin registry trait for plugin management
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Register a plugin
    async fn register_plugin(&self, plugin: Box<dyn Plugin>) -> PluginResult<()>;
    
    /// Unregister a plugin
    async fn unregister_plugin(&self, plugin_id: Uuid) -> PluginResult<()>;
    
    /// Get a plugin by ID
    async fn get_plugin(&self, plugin_id: Uuid) -> PluginResult<Option<Arc<Mutex<dyn Plugin>>>>;
    
    /// Get all plugins
    async fn get_all_plugins(&self) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>>;
    
    /// Find plugins by type
    async fn find_plugins_by_type(&self, plugin_type: PluginType) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>>;
    
    /// Find plugins by capability
    async fn find_plugins_by_capability(&self, capability: PluginCapability) -> PluginResult<Vec<Arc<Mutex<dyn Plugin>>>>;
}