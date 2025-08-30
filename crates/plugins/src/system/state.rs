//! Plugin state management
//!
//! This module provides functionality for managing plugin states in the database.
//! 
//! IMPORTANT: This module ONLY uses types::entities::PluginState for persistence.
//! No other types from the types crate should be used here.

use crate::system::types::{PluginMetadata, PluginType};
use crate::PluginResult;
use database::database::Database;
use uuid::Uuid;
use chrono;
use serde::{Serialize, Deserialize};

// RESTRICTED IMPORT: Only use PluginState for persistence
use types::entities::PluginState as DbPluginState;

/// Plugin state structure for internal plugin system use
/// 
/// This is a local definition to avoid broad dependency on crates/types.
/// Only used internally within the plugin system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub plugin_type: String,
    pub enabled: bool,
    pub installed: bool,
    pub builtin: bool,
    pub config: String,
    pub icon: Option<String>,
    pub manifest: Option<String>,
    pub installed_at: chrono::NaiveDateTime,
    pub last_updated: chrono::NaiveDateTime,
    pub last_used: Option<chrono::NaiveDateTime>,
}

/// Convert internal PluginState to database PluginState (for persistence only)
fn to_db_state(state: &PluginState) -> DbPluginState {
    DbPluginState {
        id: state.id.clone(),
        name: state.name.clone(),
        display_name: state.display_name.clone(),
        version: state.version.clone(),
        plugin_type: state.plugin_type.clone(),
        enabled: state.enabled,
        installed: state.installed,
        builtin: state.builtin,
        config: state.config.clone(),
        icon: state.icon.clone(),
        manifest: state.manifest.clone(),
        installed_at: state.installed_at,
        last_updated: state.last_updated,
        last_used: state.last_used,
    }
}

/// Convert database PluginState to internal PluginState 
fn from_db_state(state: &DbPluginState) -> PluginState {
    PluginState {
        id: state.id.clone(),
        name: state.name.clone(),
        display_name: state.display_name.clone(),
        version: state.version.clone(),
        plugin_type: state.plugin_type.clone(),
        enabled: state.enabled,
        installed: state.installed,
        builtin: state.builtin,
        config: state.config.clone(),
        icon: state.icon.clone(),
        manifest: state.manifest.clone(),
        installed_at: state.installed_at,
        last_updated: state.last_updated,
        last_used: state.last_used,
    }
}

/// Convert PluginMetadata to PluginState
pub fn metadata_to_state(metadata: &PluginMetadata, enabled: bool, config: &str) -> PluginState {
    PluginState {
        id: metadata.id.to_string(),
        name: metadata.name.clone(),
        display_name: metadata.display_name.clone(),
        version: metadata.version.to_string(),
        plugin_type: plugin_type_to_string(&metadata.plugin_type),
        enabled,
        installed: true,
        builtin: false, // Default value since PluginMetadata doesn't have this field
        config: config.to_string(),
        icon: metadata.icon.clone(),
        manifest: None, // This will be set when loading from manifest
        installed_at: chrono::Utc::now().naive_utc(),
        last_updated: chrono::Utc::now().naive_utc(),
        last_used: None,
    }
}

/// Convert PluginState to PluginMetadata
pub fn state_to_metadata(state: &PluginState) -> PluginMetadata {
    PluginMetadata {
        id: Uuid::parse_str(&state.id).unwrap_or_else(|_| Uuid::new_v4()),
        name: state.name.clone(),
        display_name: state.display_name.clone(),
        description: String::new(), // Default value since PluginState doesn't have this field
        version: state.version.parse().unwrap_or_else(|_| semver::Version::new(0, 1, 0)),
        author: String::new(), // Default value since PluginState doesn't have this field
        homepage: None, // Default value since PluginState doesn't have this field
        repository: None, // Default value since PluginState doesn't have this field
        license: None, // Default value since PluginState doesn't have this field
        icon: None, // Default value since PluginState doesn't have this field
        keywords: vec![], // Default value since PluginState doesn't have this field
        plugin_type: string_to_plugin_type(&state.plugin_type),
        capabilities: vec![], // These would be loaded from manifest
        dependencies: vec![], // These would be loaded from manifest
        min_system_version: None, // Default value since PluginState doesn't have this field
        max_system_version: None, // Default value since PluginState doesn't have this field
    }
}

/// Convert PluginType to string representation
fn plugin_type_to_string(plugin_type: &PluginType) -> String {
    match plugin_type {
        PluginType::AudioProvider => "AudioProvider".to_string(),
        PluginType::AudioProcessor => "AudioProcessor".to_string(),
        PluginType::Custom(name) => format!("Custom({})", name),
    }
}

/// Convert string representation to PluginType
fn string_to_plugin_type(plugin_type: &str) -> PluginType {
    match plugin_type {
        "AudioProvider" => PluginType::AudioProvider,
        "AudioProcessor" => PluginType::AudioProcessor,
        _ => {
            if plugin_type.starts_with("Custom(") && plugin_type.ends_with(")") {
                let name = &plugin_type[7..plugin_type.len() - 1];
                PluginType::Custom(name.to_string())
            } else {
                PluginType::Custom(plugin_type.to_string())
            }
        }
    }
}

/// Plugin state manager with database persistence
/// 
/// IMPORTANT: This only uses types::entities::PluginState for database operations.
/// All other plugin system operations use internal types.
#[derive(Debug)]
pub struct PluginStateManager {
    database: Database,
}

impl PluginStateManager {
    /// Create a new plugin state manager
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Get plugin state by ID
    pub fn get_plugin_state(&self, plugin_id: &str) -> PluginResult<Option<PluginState>> {
        let result = self.database.get_plugin_state(plugin_id)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })?;
        Ok(result.map(|state| from_db_state(&state)))
    }

    /// Get all plugin states
    pub fn get_all_plugin_states(&self) -> PluginResult<Vec<PluginState>> {
        let result = self.database.get_all_plugin_states()
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })?;
        Ok(result.into_iter().map(|state| from_db_state(&state)).collect())
    }

    /// Get plugin state by name
    pub fn get_plugin_state_by_name(&self, name: &str) -> PluginResult<Option<PluginState>> {
        let result = self.database
            .get_plugin_state_by_name(name)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })?;
        Ok(result.map(|state| from_db_state(&state)))
    }

    /// Get enabled plugin states
    pub fn get_enabled_plugin_states(&self) -> PluginResult<Vec<PluginState>> {
        let result = self.database.get_enabled_plugin_states()
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })?;
        Ok(result.into_iter().map(|state| from_db_state(&state)).collect())
    }

    /// Save plugin state
    pub fn save_plugin_state(&self, state: &PluginState) -> PluginResult<()> {
        let db_state = to_db_state(state);
        
        // Check if plugin state already exists
        match self.database.get_plugin_state(&state.id)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })? {
            Some(_) => {
                // Update existing plugin state
                self.database.update_plugin_state(&db_state)
                    .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })?;
            }
            None => {
                // Insert new plugin state
                self.database.insert_plugin_state(&db_state)
                    .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })?;
            }
        }
        Ok(())
    }

    /// Delete plugin state
    pub fn delete_plugin_state(&self, plugin_id: &str) -> PluginResult<()> {
        self.database.delete_plugin_state(plugin_id)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })
    }

    /// Update plugin state's primary key id from old to new
    pub fn update_plugin_state_id(&self, old_id: &str, new_id: &str) -> PluginResult<()> {
        self.database
            .update_plugin_state_id(old_id, new_id)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })
    }

    /// Enable plugin
    pub fn enable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        self.database.enable_plugin(plugin_id)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })
    }

    /// Disable plugin
    pub fn disable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        self.database.disable_plugin(plugin_id)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })
    }

    /// Update plugin last used timestamp
    pub fn update_plugin_last_used(&self, plugin_id: &str) -> PluginResult<()> {
        self.database.update_plugin_last_used(plugin_id)
            .map_err(|e| crate::system::types::PluginError::ExecutionFailed { reason: e.to_string() })
    }
}
