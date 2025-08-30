//! Plugin traits module
//! 
//! This module contains all the trait definitions for the plugin SDK.

pub mod base;
pub mod media;
pub mod event;

// Re-export all traits
pub use base::BasePlugin;
pub use media::{MediaPlugin, MediaAuthPlugin, MediaDownloadPlugin};
pub use event::{PluginEventHandler, PluginEvent};