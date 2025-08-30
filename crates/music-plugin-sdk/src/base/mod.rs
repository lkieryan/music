//! Base module for plugin SDK
//!
//! This module contains base functionality for plugins.

pub mod manifest;

// Re-export all base types and structures
pub use manifest::{
    PluginManifest, 
    PluginManifestBuilder, 
    PluginCategory
};