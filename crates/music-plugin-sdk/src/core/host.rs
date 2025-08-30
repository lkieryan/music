//! Host communication interface
//! 
//! This module provides interfaces for plugins to communicate with the host application.

use async_trait::async_trait;
use crate::types::base::PluginMetadata;
use crate::errors::Result;
use std::collections::HashMap;

/// Host interface provided to plugins
#[async_trait]
pub trait PluginHost: Send + Sync {
    /// Get host version
    fn host_version(&self) -> &str;
    
    /// Get available host capabilities
    fn host_capabilities(&self) -> &[String];
    
    /// Log message to host
    async fn log(&self, level: LogLevel, message: &str) -> Result<()>;
    
    /// Show notification to user
    async fn show_notification(&self, title: &str, message: &str, level: NotificationLevel) -> Result<()>;
    
    /// Request permission from host
    async fn request_permission(&self, permission: &str) -> Result<bool>;
    
    /// Store plugin data
    async fn store_data(&self, key: &str, value: &serde_json::Value) -> Result<()>;
    
    /// Retrieve plugin data
    async fn get_data(&self, key: &str) -> Result<Option<serde_json::Value>>;
    
    /// Delete plugin data
    async fn delete_data(&self, key: &str) -> Result<()>;
    
    /// Make HTTP request through host (for security/proxy)
    async fn http_request(&self, request: HttpRequest) -> Result<HttpResponse>;
    
    /// Get system information
    async fn get_system_info(&self) -> Result<SystemInfo>;
    
    /// Register event listener
    async fn register_event_listener(&self, event_type: &str, callback: Box<dyn PluginEventCallback>) -> Result<()>;
    
    /// Unregister event listener
    async fn unregister_event_listener(&self, event_type: &str) -> Result<()>;
}

/// Log levels
#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Notification levels
#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// HTTP request structure
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout: Option<std::time::Duration>,
}

/// HTTP response structure
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
    pub locale: String,
    pub timezone: String,
}

/// Plugin event callback trait
#[async_trait]
pub trait PluginEventCallback: Send + Sync {
    async fn on_event(&self, event: serde_json::Value) -> Result<()>;
}

/// Host configuration interface
pub trait HostConfig {
    /// Get configuration value
    fn get_config(&self, key: &str) -> Option<&serde_json::Value>;
    
    /// Get all configuration
    fn get_all_config(&self) -> &HashMap<String, serde_json::Value>;
    
    /// Check if configuration exists
    fn has_config(&self, key: &str) -> bool;
}

/// Plugin registry interface (for plugin discovery)
#[async_trait]
pub trait PluginRegistry: Send + Sync {
    /// Search for plugins
    async fn search_plugins(&self, query: &str) -> Result<Vec<PluginMetadata>>;
    
    /// Get plugin by ID
    async fn get_plugin(&self, id: &uuid::Uuid) -> Result<Option<PluginMetadata>>;
    
    /// List installed plugins
    async fn list_installed(&self) -> Result<Vec<PluginMetadata>>;
    
    /// Check if plugin is installed
    async fn is_installed(&self, id: &uuid::Uuid) -> Result<bool>;
    
    /// Install plugin
    async fn install_plugin(&self, source: &str) -> Result<PluginMetadata>;
    
    /// Uninstall plugin
    async fn uninstall_plugin(&self, id: &uuid::Uuid) -> Result<()>;
    
    /// Update plugin
    async fn update_plugin(&self, id: &uuid::Uuid) -> Result<PluginMetadata>;
    
    /// Check for plugin updates
    async fn check_updates(&self) -> Result<Vec<PluginUpdateInfo>>;
}

/// Plugin update information
#[derive(Debug, Clone)]
pub struct PluginUpdateInfo {
    pub plugin_id: uuid::Uuid,
    pub current_version: String,
    pub available_version: String,
    pub update_notes: Option<String>,
    pub is_security_update: bool,
}

/// Plugin loader interface
#[async_trait]
pub trait PluginLoader: Send + Sync {
    /// Load plugin from path
    async fn load_plugin(&self, path: &std::path::Path) -> Result<Box<dyn crate::traits::base::BasePlugin>>;
    
    /// Unload plugin
    async fn unload_plugin(&self, id: &uuid::Uuid) -> Result<()>;
    
    /// Reload plugin
    async fn reload_plugin(&self, id: &uuid::Uuid) -> Result<()>;
    
    /// Get loaded plugins
    fn get_loaded_plugins(&self) -> Vec<uuid::Uuid>;
    
    /// Check if plugin is loaded
    fn is_plugin_loaded(&self, id: &uuid::Uuid) -> bool;
}