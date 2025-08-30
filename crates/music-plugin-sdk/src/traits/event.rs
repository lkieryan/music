//! Plugin trait definitions
//! 
//! This module defines the core traits that plugins must implement
//! to integrate with the music player.

use async_trait::async_trait;
use crate::types::base::{PluginResult, PluginConfig};
use std::collections::HashMap;

/// Core plugin trait that all plugins must implement



/// Plugin event handler trait
#[async_trait]
pub trait PluginEventHandler: Send + Sync {
    /// Handle plugin events
    async fn handle_event(&self, event: PluginEvent) -> PluginResult<()>;
}

/// Plugin events
#[derive(Debug, Clone)]
pub enum PluginEvent {
    /// Plugin was loaded
    Loaded(uuid::Uuid),
    /// Plugin was started
    Started(uuid::Uuid),
    /// Plugin was stopped
    Stopped(uuid::Uuid),
    /// Plugin encountered an error
    Error(uuid::Uuid, String),
    /// Plugin configuration changed
    ConfigChanged(uuid::Uuid, PluginConfig),
    /// Custom event
    Custom(uuid::Uuid, String, serde_json::Value),
}
