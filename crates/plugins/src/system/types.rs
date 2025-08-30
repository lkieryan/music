//! Plugin system core types and definitions

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashMap;
use uuid::Uuid;
use semver::Version;
use thiserror::Error;

/// Plugin type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// Audio provider plugin (Spotify, YouTube, etc.)
    AudioProvider,
    
    /// Audio processor plugin (effects, visualization, etc.)
    AudioProcessor,
    
    /// Other custom plugin types
    Custom(String),
}

impl fmt::Display for PluginType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginType::AudioProvider => write!(f, "AudioProvider"),
            PluginType::AudioProcessor => write!(f, "AudioProcessor"),
            PluginType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// Plugin capability enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCapability {
    /// Search functionality
    Search,
    
    /// Playlist management
    Playlists,
    
    /// Audio streaming
    Streaming,
    
    /// Authentication support
    Authentication,
    
    /// File system access
    FileSystem,
    
    /// Network access
    Network,
    
    /// UI customization
    UI,
    
    /// Background tasks
    BackgroundTasks,
    
    /// Data processing
    DataProcessing,
    
    /// Custom capabilities
    Custom(String),
}

/// Plugin metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique plugin identifier
    pub id: Uuid,
    
    /// Plugin name (internal identifier)
    pub name: String,
    
    /// Display name for UI
    pub display_name: String,
    
    /// Plugin description
    pub description: String,
    
    /// Plugin version
    pub version: Version,
    
    /// Plugin author
    pub author: String,
    
    /// Plugin homepage URL
    pub homepage: Option<String>,
    
    /// Source repository URL
    pub repository: Option<String>,
    
    /// License information
    pub license: Option<String>,
    
    /// Icon path or data
    pub icon: Option<String>,
    
    /// Keywords for search and categorization
    pub keywords: Vec<String>,
    
    /// Plugin type classification
    pub plugin_type: PluginType,
    
    /// Supported capabilities
    pub capabilities: Vec<PluginCapability>,
    
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    
    /// Minimum system version required
    pub min_system_version: Option<Version>,
    
    /// Maximum system version supported
    pub max_system_version: Option<Version>,
}

/// Plugin status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is not loaded
    Unloaded,
    
    /// Plugin is loaded but not initialized
    Loaded,
    
    /// Plugin is initialized and ready
    Ready,
    
    /// Plugin is running
    Running,
    
    /// Plugin is stopped
    Stopped,
    
    /// Plugin encountered an error
    Error(String),
}

/// Plugin event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// User action event
    UserAction {
        /// Action name
        action: String,
        
        /// Action parameters
        parameters: HashMap<String, serde_json::Value>,
    },
    
    /// System event
    SystemEvent {
        /// Event name
        event: String,
        
        /// Event data
        data: Option<serde_json::Value>,
    },
    
    /// Lifecycle event
    LifecycleEvent {
        /// Lifecycle event type
        event_type: LifecycleEventType,
    },
}

/// Lifecycle event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEventType {
    /// Plugin initialized
    Initialized,
    
    /// Plugin started
    Started,
    
    /// Plugin stopped
    Stopped,
    
    /// Plugin destroyed
    Destroyed,
}

/// Plugin response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginResponse {
    /// Success response with optional data
    Success {
        data: Option<serde_json::Value>,
    },
    
    /// Error response
    Error {
        message: String,
        details: Option<serde_json::Value>,
    },
    
    /// Async response (for long-running operations)
    Async {
        task_id: String,
        message: String,
    },
}

impl PluginResponse {
    /// Get the data from a success response
    pub fn data(&self) -> Option<&serde_json::Value> {
        match self {
            PluginResponse::Success { data } => data.as_ref(),
            _ => None,
        }
    }
    
    /// Check if this is a success response
    pub fn is_success(&self) -> bool {
        matches!(self, PluginResponse::Success { .. })
    }
    
    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        matches!(self, PluginResponse::Error { .. })
    }
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Plugin is healthy
    Healthy,
    
    /// Plugin has issues
    Unhealthy(String),
    
    /// Plugin is in maintenance mode
    Maintenance,
}

/// Plugin error types
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin loading failed
    #[error("Plugin load failed: {reason}")]
    LoadFailed { reason: String },
    
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {reason}")]
    InitializationFailed { reason: String },
    
    /// Plugin execution failed
    #[error("Plugin execution failed: {reason}")]
    ExecutionFailed { reason: String },
    
    /// Plugin not found
    #[error("Plugin not found: {id}")]
    NotFound { id: Uuid },
    
    /// Plugin already exists
    #[error("Plugin already exists: {id}")]
    AlreadyExists { id: Uuid },
    
    /// Invalid plugin manifest
    #[error("Invalid plugin manifest: {reason}")]
    InvalidManifest { reason: String },
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Version compatibility error
    #[error("Version compatibility error: {reason}")]
    VersionMismatch { reason: String },
    
    /// Security violation
    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Other errors
    #[error("Other error: {reason}")]
    Other { reason: String },
}