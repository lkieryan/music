//! Music Player Plugin System
//!
//! This crate provides a unified plugin system for the music player application.
//! It includes:
//! - Plugin system core (loading, management, lifecycle)
//! - Built-in plugins (local, netease, youtube, bilibili)
//! - External plugin support (WASM, dynamic libraries)
//! - Security sandboxing
//! - Plugin manifest handling

// Core plugin system modules
pub mod system;

// Built-in plugins
pub mod internal;

// External plugin support
pub mod external;

// Plugin factory system
pub mod factory;

// Re-export core types for easier usage
pub use system::core::*;
pub use system::types::*;
pub use system::state::PluginStateManager;
pub use system::manager::PluginManager;
pub use factory::MediaPluginFactory;

/// Plugin system version
pub const PLUGIN_SYSTEM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, crate::system::types::PluginError>;