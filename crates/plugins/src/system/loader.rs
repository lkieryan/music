//! Plugin loader for loading plugins from various sources

use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::system::core::*;
use crate::system::types::*;
use crate::system::registry::PluginRegistry;
use crate::system::external::{WasmPluginLoader, DynamicPluginLoader};
use crate::PluginResult;

/// Plugin loader implementation
pub struct PluginLoader {
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    /// WASM plugin loader
    wasm_loader: WasmPluginLoader,
    /// Dynamic library plugin loader
    dynamic_loader: DynamicPluginLoader,
}

// Manual Debug implementation to avoid issues with trait objects
impl std::fmt::Debug for PluginLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginLoader")
            .field("registry", &self.registry)
            .field("wasm_loader", &self.wasm_loader)
            .field("dynamic_loader", &self.dynamic_loader)
            .finish()
    }
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        let wasm_loader = WasmPluginLoader::new("./plugins/wasm".to_string());
        let dynamic_loader = DynamicPluginLoader::new("./plugins/dynamic".to_string());
        
        Self {
            registry,
            wasm_loader,
            dynamic_loader,
        }
    }
    
    /// Load a plugin from a file
    pub async fn load_plugin_from_file(&self, plugin_path: &Path) -> PluginResult<()> {
        // Determine plugin type by file extension
        let extension = plugin_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let plugin: Box<dyn Plugin> = match extension.as_str() {
            "wasm" => {
                // Validate WASM plugin
                if !self.wasm_loader.validate_plugin(plugin_path)? {
                    return Err(PluginError::InvalidManifest {
                        reason: "Invalid WASM plugin file".to_string()
                    });
                }
                
                // Load WASM plugin
                self.wasm_loader.load_plugin(plugin_path).await?
            },
            "dll" | "so" | "dylib" => {
                // Validate dynamic library plugin
                if !self.dynamic_loader.validate_plugin(plugin_path)? {
                    return Err(PluginError::InvalidManifest {
                        reason: "Invalid dynamic library plugin file".to_string()
                    });
                }
                
                // Load dynamic library plugin
                self.dynamic_loader.load_plugin(plugin_path).await?
            },
            _ => {
                return Err(PluginError::LoadFailed {
                    reason: format!("Unsupported plugin file type: {}", extension)
                });
            }
        };
        
        // Register the loaded plugin
        self.registry.register_plugin(plugin).await?;
        
        Ok(())
    }
    
    /// Load a built-in plugin
    pub async fn load_builtin_plugin(&self, plugin_name: &str) -> PluginResult<()> {
        let plugin: Box<dyn Plugin> = match plugin_name.to_lowercase().as_str() {
            "spotify" => {
                Box::new(crate::internal::SpotifyPlugin::new())
            },
            "bilibili" => {
                Box::new(crate::internal::BilibiliPlugin::new())
            },
            "youtube" => {
                Box::new(crate::internal::YoutubePlugin::new())
            },
            _ => {
                return Err(PluginError::LoadFailed {
                    reason: format!("Unknown built-in plugin: {}", plugin_name)
                });
            }
        };
        
        // Register the built-in plugin
        self.registry.register_plugin(plugin).await?;
        
        Ok(())
    }
    
    /// Load all plugins from a directory
    pub async fn load_plugins_from_directory(&self, dir_path: &Path) -> PluginResult<()> {
        // Check if directory exists
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(PluginError::LoadFailed {
                reason: format!("Plugin directory does not exist: {:?}", dir_path)
            });
        }
        
        // Read directory entries
        let entries = fs::read_dir(dir_path)
            .map_err(|e| PluginError::LoadFailed {
                reason: format!("Failed to read plugin directory: {}", e)
            })?;
        
        // Process each entry
        for entry in entries {
            let entry = entry
                .map_err(|e| PluginError::LoadFailed {
                    reason: format!("Failed to read directory entry: {}", e)
                })?;
            
            let path = entry.path();
            
            // Skip directories
            if path.is_dir() {
                continue;
            }
            
            // Skip files without extensions
            if path.extension().is_none() {
                continue;
            }
            
            // Try to load the plugin file
            match self.load_plugin_from_file(&path).await {
                Ok(_) => {
                    // Plugin loaded successfully
                    // In a real implementation, we might want to log this
                }
                Err(e) => {
                    // Failed to load plugin, log the error and continue with other plugins
                    // In a real implementation, we would use a proper logging system
                    eprintln!("Failed to load plugin from {:?}: {}", path, e);
                }
            }
        }
        
        Ok(())
    }
}
