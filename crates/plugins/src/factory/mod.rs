//! Plugin factory system for managing plugin instances
//!
//! This module provides a factory-based approach to plugin management,
//! allowing direct access to plugin instances by ID without complex routing.

pub mod media;

pub use media::MediaPluginFactory;