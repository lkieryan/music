//! Error types for plugin development

use thiserror::Error;

/// Main plugin error type
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin initialization failed
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    /// Plugin configuration error
    #[error("Plugin configuration error: {0}")]
    ConfigurationError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Feature not supported
    #[error("Feature not supported: {0}")]
    NotSupported(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    /// File system error
    #[error("File system error: {0}")]
    FileSystemError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Plugin dependency error
    #[error("Plugin dependency error: {0}")]
    DependencyError(String),
    
    /// Plugin version incompatibility
    #[error("Plugin version incompatibility: {0}")]
    VersionIncompatibility(String),
    
    /// Plugin security violation
    #[error("Plugin security violation: {0}")]
    SecurityViolation(String),
    
    /// Plugin timeout
    #[error("Plugin timeout: {0}")]
    Timeout(String),
    
    /// Internal plugin error
    #[error("Internal plugin error: {0}")]
    Internal(String),
    
    /// Custom error with context
    #[error("Plugin error: {message}")]
    Custom {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl PluginError {
    /// Create a custom error with source
    pub fn custom_with_source<E>(message: impl Into<String>, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Custom {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
    
    /// Create a simple custom error
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
            source: None,
        }
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            PluginError::NetworkError(_)
                | PluginError::RateLimitExceeded(_)
                | PluginError::Timeout(_)
        )
    }
    
    /// Check if error requires user action
    pub fn requires_user_action(&self) -> bool {
        matches!(
            self,
            PluginError::AuthenticationError(_)
                | PluginError::AuthorizationError(_)
                | PluginError::ConfigurationError(_)
        )
    }
}

/// Convert from common error types
impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::FileSystemError(err.to_string())
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::SerializationError(err.to_string())
    }
}

impl From<url::ParseError> for PluginError {
    fn from(err: url::ParseError) -> Self {
        PluginError::InvalidInput(format!("Invalid URL: {}", err))
    }
}

impl From<anyhow::Error> for PluginError {
    fn from(err: anyhow::Error) -> Self {
        PluginError::Internal(err.to_string())
    }
}

/// Result type alias for plugin operations
pub type Result<T> = std::result::Result<T, PluginError>;