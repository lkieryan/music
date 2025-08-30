use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Plugin execution result type
pub type PluginResult<T> = Result<T, crate::errors::PluginError>;

/// Plugin metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin unique identifier
    pub id: Uuid,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin website
    pub website: Option<String>,
    /// Plugin icon URL
    pub icon: Option<String>,
    /// Supported capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Minimum SDK version required
    pub min_sdk_version: String,
    /// Plugin configuration schema
    pub config_schema: Option<serde_json::Value>,
}


/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is loaded but not started
    Loaded,
    /// Plugin is running
    Running,
    /// Plugin is paused
    Paused,
    /// Plugin has stopped
    Stopped,
    /// Plugin encountered an error
    Error(String),
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Configuration values
    pub values: HashMap<String, serde_json::Value>,
    /// Whether configuration is valid
    pub is_valid: bool,
    /// Configuration validation errors
    pub errors: Vec<String>,
}

/// Host context provided to plugins
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Plugin configuration
    pub config: PluginConfig,
    /// Host version
    pub host_version: String,
    /// Available host capabilities
    pub host_capabilities: Vec<String>,
    /// Plugin data directory
    pub data_dir: std::path::PathBuf,
    /// Plugin cache directory
    pub cache_dir: std::path::PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginCapability {
    /// Can search for music
    Search,
    /// Can play music
    Playback,
    /// Can download music
    Download,
    /// Can access user library
    Library,
    /// Can create playlists
    Playlist,
    /// Requires network access
    Network,
    /// Requires file system access
    FileSystem,
    /// Custom capability
    Custom(String),
}