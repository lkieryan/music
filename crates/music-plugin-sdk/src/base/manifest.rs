//! Plugin manifest handling
//! 
//! This module provides structures and utilities for plugin manifest files
//! that define plugin metadata and capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::base::{PluginCapability, PluginMetadata};
use crate::errors::{PluginError, Result};
use uuid::Uuid;

/// Plugin manifest structure (plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin unique identifier
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version (semver)
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin license
    pub license: Option<String>,
    /// Plugin website
    pub website: Option<String>,
    /// Plugin repository
    pub repository: Option<String>,
    /// Plugin icon path (relative to plugin root)
    pub icon: Option<String>,
    /// Plugin entry point
    pub entry: String,
    /// Minimum SDK version required
    pub min_sdk_version: String,
    /// Maximum SDK version supported
    pub max_sdk_version: Option<String>,
    /// Plugin capabilities
    pub capabilities: Vec<PluginCapability>,
    /// Plugin dependencies
    pub dependencies: Option<HashMap<String, String>>,
    /// Plugin configuration schema
    pub config_schema: Option<serde_json::Value>,
    /// Supported platforms
    pub platforms: Vec<String>,
    /// Plugin keywords for discovery
    pub keywords: Vec<String>,
    /// Plugin category
    pub category: PluginCategory,
}

/// Plugin categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCategory {
    /// Music streaming provider
    #[serde(rename = "music-provider")]
    MusicProvider,
    /// Audio effects
    #[serde(rename = "audio-effects")]
    AudioEffects,
    /// Visualization
    #[serde(rename = "visualization")]
    Visualization,
    /// User interface
    #[serde(rename = "ui")]
    UserInterface,
    /// Utility
    #[serde(rename = "utility")]
    Utility,
    /// Integration
    #[serde(rename = "integration")]
    Integration,
    /// Custom category
    #[serde(rename = "custom")]
    Custom(String),
}

impl PluginManifest {
    /// Load manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| PluginError::SerializationError(format!("Invalid manifest JSON: {}", e)))
    }
    
    /// Load manifest from file
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PluginError::FileSystemError(format!("Cannot read manifest file: {}", e)))?;
        Self::from_json(&content)
    }
    
    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PluginError::SerializationError(format!("Cannot serialize manifest: {}", e)))
    }
    
    /// Save manifest to file
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)
            .map_err(|e| PluginError::FileSystemError(format!("Cannot write manifest file: {}", e)))
    }
    
    /// Validate manifest
    pub fn validate(&self) -> Result<()> {
        // Validate ID is a valid UUID
        Uuid::parse_str(&self.id)
            .map_err(|_| PluginError::InvalidInput("Plugin ID must be a valid UUID".to_string()))?;
        
        // Validate version is semver
        if !is_valid_semver(&self.version) {
            return Err(PluginError::InvalidInput(
                "Plugin version must be valid semver".to_string()
            ));
        }
        
        // Validate min_sdk_version is semver
        if !is_valid_semver(&self.min_sdk_version) {
            return Err(PluginError::InvalidInput(
                "Minimum SDK version must be valid semver".to_string()
            ));
        }
        
        // Validate max_sdk_version if present
        if let Some(max_version) = &self.max_sdk_version {
            if !is_valid_semver(max_version) {
                return Err(PluginError::InvalidInput(
                    "Maximum SDK version must be valid semver".to_string()
                ));
            }
        }
        
        // Validate required fields are not empty
        if self.name.trim().is_empty() {
            return Err(PluginError::InvalidInput("Plugin name cannot be empty".to_string()));
        }
        
        if self.description.trim().is_empty() {
            return Err(PluginError::InvalidInput("Plugin description cannot be empty".to_string()));
        }
        
        if self.author.trim().is_empty() {
            return Err(PluginError::InvalidInput("Plugin author cannot be empty".to_string()));
        }
        
        if self.entry.trim().is_empty() {
            return Err(PluginError::InvalidInput("Plugin entry point cannot be empty".to_string()));
        }
        
        Ok(())
    }
    
    /// Convert to PluginMetadata
    pub fn to_metadata(&self) -> Result<PluginMetadata> {
        self.validate()?;
        
        Ok(PluginMetadata {
            id: Uuid::parse_str(&self.id).unwrap(), // Safe after validation
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            website: self.website.clone(),
            icon: self.icon.clone(),
            capabilities: self.capabilities.clone(),
            min_sdk_version: self.min_sdk_version.clone(),
            config_schema: self.config_schema.clone(),
        })
    }
    
    /// Check if plugin is compatible with SDK version
    pub fn is_compatible_with_sdk(&self, sdk_version: &str) -> bool {
        // Basic semver comparison - in real implementation you'd use a proper semver library
        compare_versions(sdk_version, &self.min_sdk_version) >= 0
            && self.max_sdk_version
                .as_ref()
                .map_or(true, |max_ver| compare_versions(sdk_version, max_ver) <= 0)
    }
    
    /// Check if plugin supports platform
    pub fn supports_platform(&self, platform: &str) -> bool {
        self.platforms.is_empty() || self.platforms.contains(&platform.to_string())
    }
}

/// Basic semver validation (simplified)
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3 && parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Basic version comparison (simplified)
fn compare_versions(v1: &str, v2: &str) -> i32 {
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.').map(|s| s.parse().unwrap_or(0)).collect()
    };
    
    let ver1 = parse_version(v1);
    let ver2 = parse_version(v2);
    
    for i in 0..3 {
        let a = ver1.get(i).unwrap_or(&0);
        let b = ver2.get(i).unwrap_or(&0);
        
        if a > b {
            return 1;
        } else if a < b {
            return -1;
        }
    }
    0
}

/// Plugin manifest builder
pub struct PluginManifestBuilder {
    manifest: PluginManifest,
}

impl PluginManifestBuilder {
    /// Create new manifest builder
    pub fn new(id: &str, name: &str, version: &str) -> Self {
        Self {
            manifest: PluginManifest {
                id: id.to_string(),
                name: name.to_string(),
                version: version.to_string(),
                description: String::new(),
                author: String::new(),
                license: None,
                website: None,
                repository: None,
                icon: None,
                entry: "main.wasm".to_string(),
                min_sdk_version: "0.1.0".to_string(),
                max_sdk_version: None,
                capabilities: vec![],
                dependencies: None,
                config_schema: None,
                platforms: vec!["windows".to_string(), "macos".to_string(), "linux".to_string()],
                keywords: vec![],
                category: PluginCategory::MusicProvider,
            },
        }
    }
    
    /// Set description
    pub fn description(mut self, description: &str) -> Self {
        self.manifest.description = description.to_string();
        self
    }
    
    /// Set author
    pub fn author(mut self, author: &str) -> Self {
        self.manifest.author = author.to_string();
        self
    }
    
    /// Set entry point
    pub fn entry(mut self, entry: &str) -> Self {
        self.manifest.entry = entry.to_string();
        self
    }
    
    /// Add capability
    pub fn capability(mut self, capability: PluginCapability) -> Self {
        self.manifest.capabilities.push(capability);
        self
    }
    
    /// Set category
    pub fn category(mut self, category: PluginCategory) -> Self {
        self.manifest.category = category;
        self
    }
    
    /// Build the manifest
    pub fn build(self) -> PluginManifest {
        self.manifest
    }
}