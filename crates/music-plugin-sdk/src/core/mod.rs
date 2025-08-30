//! Core module for plugin SDK
//!
//! This module contains core functionality for plugins.

pub mod host;

// Re-export all core traits
pub use host::{
    PluginHost, 
    PluginRegistry, 
    PluginLoader, 
    PluginEventCallback, 
    HostConfig
};