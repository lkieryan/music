//! Plugin lifecycle management

use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

use crate::system::core::*;
use crate::system::types::*;
use crate::system::registry::PluginRegistry;
use crate::system::security::SecurityManager;
use crate::PluginResult;

/// Plugin lifecycle manager
pub struct LifecycleManager {
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    
    /// Security manager
    security: Arc<Mutex<SecurityManager>>,
}

// Manual Debug implementation to avoid issues with trait objects
impl std::fmt::Debug for LifecycleManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LifecycleManager")
            .field("registry", &self.registry)
            .field("security", &"Mutex<SecurityManager>")
            .finish()
    }
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(registry: Arc<PluginRegistry>, security: Arc<Mutex<SecurityManager>>) -> Self {
        Self {
            registry,
            security,
        }
    }
    
    /// Start a plugin
    pub async fn start_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        // Get the plugin mutex
        let plugin_mutex = self.registry.get_plugin(plugin_id).await?
            .ok_or(PluginError::NotFound { id: plugin_id })?;
        
        // Validate plugin permissions and start plugin in separate scopes
        {
            let plugin = plugin_mutex.lock().unwrap();
            let security = self.security.lock().unwrap();
            security.validate_plugin_permissions(&*plugin)?;
        }
        
        {
            let mut plugin = plugin_mutex.lock().unwrap();
            plugin.start()?;
        }
        
        Ok(())
    }
    
    /// Stop a plugin
    pub async fn stop_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        // Get the plugin mutex
        let plugin_mutex = self.registry.get_plugin(plugin_id).await?
            .ok_or(PluginError::NotFound { id: plugin_id })?;
        
        // Call the sync method on the locked plugin
        {
            let mut plugin = plugin_mutex.lock().unwrap();
            plugin.stop()?;
        }
        
        Ok(())
    }
    
    /// Initialize a plugin
    pub async fn initialize_plugin(&self, plugin_id: Uuid, context: PluginContext) -> PluginResult<()> {
        // Get the plugin mutex
        let plugin_mutex = self.registry.get_plugin(plugin_id).await?
            .ok_or(PluginError::NotFound { id: plugin_id })?;
        
        // Validate plugin permissions and initialize in separate scopes
        {
            let plugin = plugin_mutex.lock().unwrap();
            let security = self.security.lock().unwrap();
            security.validate_plugin_permissions(&*plugin)?;
        }
        
        {
            let mut plugin = plugin_mutex.lock().unwrap();
            plugin.initialize(&context)?;
        }
        
        Ok(())
    }
    
    /// Destroy a plugin
    pub async fn destroy_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        // Get the plugin mutex
        let plugin_mutex = self.registry.get_plugin(plugin_id).await?
            .ok_or(PluginError::NotFound { id: plugin_id })?;
        
        // Call the sync method on the locked plugin
        {
            let mut plugin = plugin_mutex.lock().unwrap();
            plugin.destroy()?;
        }
        
        Ok(())
    }
    
    /// Get plugin status
    pub async fn get_plugin_status(&self, plugin_id: Uuid) -> PluginResult<PluginStatus> {
        // Get the plugin mutex
        let plugin_mutex = self.registry.get_plugin(plugin_id).await?
            .ok_or(PluginError::NotFound { id: plugin_id })?;
        
        // Call the sync method on the locked plugin
        let status = {
            let plugin = plugin_mutex.lock().unwrap();
            plugin.status()?
        };
        
        Ok(status)
    }
    
    /// Perform health check on a plugin
    pub async fn health_check_plugin(&self, plugin_id: Uuid) -> PluginResult<HealthStatus> {
        // Get the plugin mutex
        let plugin_mutex = self.registry.get_plugin(plugin_id).await?
            .ok_or(PluginError::NotFound { id: plugin_id })?;
        
        // Call the sync method on the locked plugin
        let health_status = {
            let plugin = plugin_mutex.lock().unwrap();
            plugin.health_check()?
        };
        
        Ok(health_status)
    }
    
    /// Perform health check on all plugins
    pub async fn health_check_all_plugins(&self) -> PluginResult<Vec<(Uuid, HealthStatus)>> {
        let plugins = self.registry.get_all_plugins().await?;
        let mut results = Vec::new();
        
        for plugin_mutex in plugins {
            // Call the sync method on the locked plugin
            let (plugin_id, health_status) = {
                let plugin = plugin_mutex.lock().unwrap();
                let plugin_id = plugin.id();
                let health_status = plugin.health_check()?;
                (plugin_id, health_status)
            };
            
            results.push((plugin_id, health_status));
        }
        
        Ok(results)
    }
    
    /// Start all plugins
    pub async fn start_all_plugins(&self) -> PluginResult<()> {
        let plugins = self.registry.get_all_plugins().await?;
        let mut plugin_ids = Vec::new();
        
        // Collect plugin IDs and validate permissions first
        for plugin_mutex in plugins {
            let plugin_id = {
                let plugin = plugin_mutex.lock().unwrap();
                plugin.id()
            };
            
            // Validate plugin permissions before starting
            {
                let plugin = plugin_mutex.lock().unwrap();
                let security = self.security.lock().unwrap();
                security.validate_plugin_permissions(&*plugin)?;
            }
            
            plugin_ids.push(plugin_id);
        }
        
        // Start each plugin
        for plugin_id in plugin_ids {
            self.start_plugin(plugin_id).await?;
        }
        
        Ok(())
    }
    
    /// Stop all plugins
    pub async fn stop_all_plugins(&self) -> PluginResult<()> {
        let plugins = self.registry.get_all_plugins().await?;
        let mut plugin_ids = Vec::new();
        
        // Collect plugin IDs first
        for plugin_mutex in plugins {
            let plugin_id = {
                let plugin = plugin_mutex.lock().unwrap();
                plugin.id()
            };
            plugin_ids.push(plugin_id);
        }
        
        // Stop each plugin
        for plugin_id in plugin_ids {
            self.stop_plugin(plugin_id).await?;
        }
        Ok(())
    }
    
    /// Destroy all plugins
    pub async fn destroy_all_plugins(&self) -> PluginResult<()> {
        let plugins = self.registry.get_all_plugins().await?;
        let mut plugin_ids = Vec::new();
        
        // Collect plugin IDs first
        for plugin_mutex in plugins {
            let plugin_id = {
                let plugin = plugin_mutex.lock().unwrap();
                plugin.id()
            };
            plugin_ids.push(plugin_id);
        }
        
        // Destroy each plugin
        for plugin_id in plugin_ids {
            self.destroy_plugin(plugin_id).await?;
        }
        Ok(())
    }
    
    /// Initialize all plugins
    pub async fn initialize_all_plugins(&self, context: PluginContext) -> PluginResult<()> {
        let plugins = self.registry.get_all_plugins().await?;
        let mut plugin_ids = Vec::new();
        
        // Collect plugin IDs and validate permissions first
        for plugin_mutex in plugins {
            let plugin_id = {
                let plugin = plugin_mutex.lock().unwrap();
                plugin.id()
            };
            
            // Validate plugin permissions before initialization
            {
                let plugin = plugin_mutex.lock().unwrap();
                let security = self.security.lock().unwrap();
                security.validate_plugin_permissions(&*plugin)?;
            }
            
            plugin_ids.push(plugin_id);
        }
        
        // Initialize each plugin
        for plugin_id in plugin_ids {
            self.initialize_plugin(plugin_id, context.clone()).await?;
        }
        Ok(())
    }
}