//! Builder and validator utilities
//! 
//! This module provides builder patterns and validation utilities for plugin development.

use crate::types::base::{PluginConfig, PluginCapability};
use crate::errors::{PluginError, Result};
use std::collections::HashMap;

/// Plugin builder utility
pub struct PluginBuilder<T> {
    plugin: T,
}

impl<T> PluginBuilder<T> {
    /// Create new plugin builder
    pub fn new(plugin: T) -> Self {
        Self { plugin }
    }
    
    /// Build the plugin
    pub fn build(self) -> T {
        self.plugin
    }
}

/// Configuration validator
pub struct ConfigValidator {
    schema: serde_json::Value,
}

impl ConfigValidator {
    /// Create new validator with JSON schema
    pub fn new(schema: serde_json::Value) -> Self {
        Self { schema }
    }
    
    /// Validate configuration against schema
    pub fn validate(&self, config: &PluginConfig) -> Result<()> {
        // Basic validation - in real implementation use jsonschema crate
        if config.values.is_empty() && !self.schema.is_null() {
            return Err(PluginError::ConfigurationError(
                "Configuration is required but empty".to_string()
            ));
        }
        Ok(())
    }
    
    /// Get default configuration from schema
    pub fn get_defaults(&self) -> HashMap<String, serde_json::Value> {
        // Extract defaults from JSON schema
        let mut defaults = HashMap::new();
        
        if let Some(properties) = self.schema.get("properties").and_then(|p| p.as_object()) {
            for (key, value) in properties {
                if let Some(default) = value.get("default") {
                    defaults.insert(key.clone(), default.clone());
                }
            }
        }
        
        defaults
    }
}