//! Plugin sandboxing implementation

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::system::core::*;
use crate::system::types::*;
use crate::system::security::{SecurityManager, FsAccessType, FsRestrictions, NetworkRestrictions};
use crate::PluginResult;

/// Plugin sandbox for isolating plugin execution
#[derive(Debug)]
pub struct PluginSandbox {
    /// Plugin ID
    plugin_id: Uuid,
    
    /// Security manager reference
    security_manager: Arc<Mutex<SecurityManager>>,
    
    /// Sandbox root directory
    sandbox_root: PathBuf,
    
    /// Virtual file system mappings
    vfs_mappings: HashMap<String, PathBuf>,
    
    /// Process isolation settings
    process_isolation: ProcessIsolation,
    
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Process isolation settings
#[derive(Debug, Clone)]
pub struct ProcessIsolation {
    /// Whether to run in a separate process
    pub separate_process: bool,
    
    /// User ID to run as (if supported by platform)
    pub user_id: Option<u32>,
    
    /// Group ID to run as (if supported by platform)
    pub group_id: Option<u32>,
}

/// Resource limits for the sandbox
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum memory usage (in bytes)
    pub max_memory: Option<u64>,
    
    /// Maximum CPU time (in seconds)
    pub max_cpu_time: Option<u64>,
    
    /// Maximum number of file descriptors
    pub max_file_descriptors: Option<u32>,
    
    /// Maximum number of network connections
    pub max_network_connections: Option<u32>,
}

impl PluginSandbox {
    /// Create a new plugin sandbox
    pub fn new(plugin_id: Uuid, security_manager: Arc<Mutex<SecurityManager>>, sandbox_root: PathBuf) -> Self {
        Self {
            plugin_id,
            security_manager,
            sandbox_root,
            vfs_mappings: HashMap::new(),
            process_isolation: ProcessIsolation::default(),
            resource_limits: ResourceLimits::default(),
        }
    }
    
    /// Initialize the sandbox for a plugin
    pub fn initialize(&mut self, plugin: &dyn Plugin) -> PluginResult<()> {
        // Create sandbox directory
        std::fs::create_dir_all(&self.sandbox_root)
            .map_err(|e| PluginError::ExecutionFailed {
                reason: format!("Failed to create sandbox directory: {}", e)
            })?;
        
        // Set up virtual file system mappings
        self.setup_vfs_mappings(plugin)?;
        
        // Apply security restrictions
        self.apply_security_restrictions(plugin)?;
        
        Ok(())
    }
    
    /// Set up virtual file system mappings
    fn setup_vfs_mappings(&mut self, _plugin: &dyn Plugin) -> PluginResult<()> {
        // let security_manager = self.security_manager.lock().unwrap();
        
        // Map plugin-specific data directory
        let plugin_data_dir = self.sandbox_root.join("data");
        self.vfs_mappings.insert("/data".to_string(), plugin_data_dir);
        
        // Map temporary directory
        let temp_dir = self.sandbox_root.join("temp");
        self.vfs_mappings.insert("/tmp".to_string(), temp_dir);
        
        // Map configuration directory
        let config_dir = self.sandbox_root.join("config");
        self.vfs_mappings.insert("/config".to_string(), config_dir);
        
        // Create the actual directories
        for (_, real_path) in &self.vfs_mappings {
            std::fs::create_dir_all(real_path)
                .map_err(|e| PluginError::ExecutionFailed {
                    reason: format!("Failed to create sandbox directory {:?}: {}", real_path, e)
                })?;
        }
        
        Ok(())
    }
    
    /// Set process isolation settings
    pub fn set_process_isolation(&mut self, isolation: ProcessIsolation) {
        self.process_isolation = isolation;
    }
    
    /// Set resource limits
    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
    }
    
    /// Apply security restrictions to the sandbox
    fn apply_security_restrictions(&self, plugin: &dyn Plugin) -> PluginResult<()> {
        let mut security_manager = self.security_manager.lock().unwrap();
        
        // Set up file system permissions
        let fs_permissions = security_manager.create_restricted_fs_permissions(plugin);
        security_manager.set_plugin_fs_permissions(self.plugin_id, fs_permissions);
        
        // Set up network permissions
        let network_permissions = security_manager.create_restricted_network_permissions(plugin);
        security_manager.set_plugin_network_permissions(self.plugin_id, network_permissions);
        
        // Apply global restrictions
        security_manager.set_global_fs_restrictions(FsRestrictions::default());
        security_manager.set_global_network_restrictions(NetworkRestrictions::default());
        
        Ok(())
    }
    
    /// Translate a virtual path to a real path within the sandbox
    pub fn translate_path(&self, virtual_path: &str) -> Option<PathBuf> {
        // Handle absolute paths in the virtual file system
        if virtual_path.starts_with('/') {
            for (vfs_path, real_path) in &self.vfs_mappings {
                if virtual_path.starts_with(vfs_path) {
                    let relative_part = &virtual_path[vfs_path.len()..];
                    return Some(real_path.join(relative_part.trim_start_matches('/')));
                }
            }
        }
        
        // Handle relative paths
        Some(self.sandbox_root.join(virtual_path))
    }
    
    /// Check if file system access is allowed
    pub fn is_fs_access_allowed(&self, path: &Path, access_type: FsAccessType) -> bool {
        let security_manager = self.security_manager.lock().unwrap();
        
        // Check global restrictions first
        if security_manager.check_global_fs_restrictions(path, access_type.clone()).is_err() {
            return false;
        }
        
        security_manager.is_plugin_fs_access_allowed(self.plugin_id, path, access_type)
    }
    
    /// Check if network access is allowed
    pub fn is_network_access_allowed(&self, host: &str, port: u64, protocol: &str) -> bool {
        let security_manager = self.security_manager.lock().unwrap();
        
        // Check global restrictions first
        if security_manager.check_global_network_restrictions(host).is_err() {
            return false;
        }
        
        security_manager.is_plugin_network_access_allowed(
            self.plugin_id, 
            host, 
            port as u16, 
            protocol
        )
    }
    
    /// Validate file operation
    pub fn validate_file_operation(&self, path: &Path, size: u64) -> PluginResult<()> {
        let security_manager = self.security_manager.lock().unwrap();
        
        // Check global restrictions
        security_manager.check_global_fs_restrictions(path, FsAccessType::Read)?;
        security_manager.validate_file_size(size)?;
        security_manager.validate_file_extension(path)?;
        
        Ok(())
    }
    
    /// Validate network operation
    pub fn validate_network_operation(&self, host: &str, request_size: u64, response_size: u64) -> PluginResult<()> {
        let security_manager = self.security_manager.lock().unwrap();
        
        // Check global restrictions
        security_manager.check_global_network_restrictions(host)?;
        security_manager.validate_request_size(request_size)?;
        security_manager.validate_response_size(response_size)?;
        
        Ok(())
    }
    
    /// Get the sandbox root directory
    pub fn sandbox_root(&self) -> &Path {
        &self.sandbox_root
    }
    
    /// Clean up the sandbox
    pub fn cleanup(&self) -> PluginResult<()> {
        // In a real implementation, we might want to clean up the sandbox directory
        // For now, we'll just log that cleanup was requested
        println!("Sandbox cleanup requested for plugin {}", self.plugin_id);
        Ok(())
    }
}

/// Sandbox manager for managing multiple plugin sandboxes
#[derive(Debug)]
pub struct SandboxManager {
    /// Security manager reference
    security_manager: Arc<Mutex<SecurityManager>>,
    
    /// Sandboxes for each plugin
    sandboxes: HashMap<Uuid, Arc<Mutex<PluginSandbox>>>,
    
    /// Base directory for all sandboxes
    sandboxes_root: PathBuf,
}

impl SandboxManager {
    /// Create a new sandbox manager
    pub fn new(security_manager: Arc<Mutex<SecurityManager>>, sandboxes_root: PathBuf) -> Self {
        Self {
            security_manager,
            sandboxes: HashMap::new(),
            sandboxes_root,
        }
    }
    
    /// Create a sandbox for a plugin
    pub fn create_sandbox(&mut self, plugin: &dyn Plugin) -> PluginResult<Arc<Mutex<PluginSandbox>>> {
        let plugin_id = plugin.id();
        
        // Create sandbox directory path
        let sandbox_root = self.sandboxes_root.join(plugin_id.to_string());
        
        // Create the sandbox
        let mut sandbox = PluginSandbox::new(
            plugin_id,
            Arc::clone(&self.security_manager),
            sandbox_root
        );
        
        // Initialize the sandbox
        sandbox.initialize(plugin)?;
        
        // Store the sandbox
        let sandbox_arc = Arc::new(Mutex::new(sandbox));
        self.sandboxes.insert(plugin_id, Arc::clone(&sandbox_arc));
        
        Ok(sandbox_arc)
    }
    
    /// Get a sandbox for a plugin
    pub fn get_sandbox(&self, plugin_id: Uuid) -> Option<Arc<Mutex<PluginSandbox>>> {
        self.sandboxes.get(&plugin_id).cloned()
    }
    
    /// Remove a sandbox for a plugin
    pub fn remove_sandbox(&mut self, plugin_id: Uuid) -> PluginResult<()> {
        if let Some(sandbox) = self.sandboxes.remove(&plugin_id) {
            let sandbox = sandbox.lock().unwrap();
            sandbox.cleanup()?;
        }
        Ok(())
    }
    
    /// Clean up all sandboxes
    pub fn cleanup_all(&mut self) -> PluginResult<()> {
        let plugin_ids: Vec<Uuid> = self.sandboxes.keys().cloned().collect();
        for plugin_id in plugin_ids {
            self.remove_sandbox(plugin_id)?;
        }
        Ok(())
    }
}

impl Default for ProcessIsolation {
    fn default() -> Self {
        Self {
            separate_process: false,
            user_id: None,
            group_id: None,
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: None,
            max_cpu_time: None,
            max_file_descriptors: None,
            max_network_connections: None,
        }
    }
}