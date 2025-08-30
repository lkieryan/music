//! Plugin security management

use std::collections::{HashSet, HashMap};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::system::core::*;
use crate::system::types::*;
use crate::PluginResult;

/// Security manager for plugin sandboxing
#[derive(Debug)]
pub struct SecurityManager {
    /// Allowed capabilities
    allowed_capabilities: HashSet<PluginCapability>,
    
    /// Restricted paths
    restricted_paths: HashSet<String>,
    
    /// Allowed file system paths for each plugin
    plugin_fs_permissions: HashMap<Uuid, FsPermissions>,
    
    /// Allowed network hosts for each plugin
    plugin_network_permissions: HashMap<Uuid, NetworkPermissions>,
    
    /// Plugin-specific capability restrictions
    plugin_capability_restrictions: HashMap<Uuid, HashSet<PluginCapability>>,
    
    /// Global file system restrictions
    global_fs_restrictions: FsRestrictions,
    
    /// Global network restrictions
    global_network_restrictions: NetworkRestrictions,
}

/// File system restrictions
#[derive(Debug, Clone)]
pub struct FsRestrictions {
    /// Paths that are completely forbidden
    pub forbidden_paths: HashSet<PathBuf>,
    
    /// Paths that require special permission
    pub restricted_paths: HashSet<PathBuf>,
    
    /// Maximum file size allowed (in bytes)
    pub max_file_size: Option<u64>,
    
    /// Allowed file extensions
    pub allowed_extensions: HashSet<String>,
}

/// Network restrictions
#[derive(Debug, Clone)]
pub struct NetworkRestrictions {
    /// Hosts that are completely forbidden
    pub forbidden_hosts: HashSet<String>,
    
    /// Hosts that require special permission
    pub restricted_hosts: HashSet<String>,
    
    /// Maximum request size (in bytes)
    pub max_request_size: Option<u64>,
    
    /// Maximum response size (in bytes)
    pub max_response_size: Option<u64>,
    
    /// Rate limiting (requests per second)
    pub rate_limit: Option<u32>,
}

/// File system permissions
#[derive(Debug, Clone)]
pub struct FsPermissions {
    /// Allowed read paths
    allowed_read_paths: HashSet<PathBuf>,
    
    /// Allowed write paths
    allowed_write_paths: HashSet<PathBuf>,
    
    /// Allowed execute paths
    allowed_execute_paths: HashSet<PathBuf>,
}

/// Network permissions
#[derive(Debug, Clone)]
pub struct NetworkPermissions {
    /// Allowed hosts
    allowed_hosts: HashSet<String>,
    
    /// Allowed ports
    allowed_ports: HashSet<u16>,
    
    /// Allowed protocols
    allowed_protocols: HashSet<String>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            allowed_capabilities: HashSet::new(),
            restricted_paths: HashSet::new(),
            plugin_fs_permissions: HashMap::new(),
            plugin_network_permissions: HashMap::new(),
            plugin_capability_restrictions: HashMap::new(),
            global_fs_restrictions: FsRestrictions::default(),
            global_network_restrictions: NetworkRestrictions::default(),
        }
    }
    
    /// Add an allowed capability
    pub fn add_allowed_capability(&mut self, capability: PluginCapability) {
        self.allowed_capabilities.insert(capability);
    }
    
    /// Remove an allowed capability
    pub fn remove_allowed_capability(&mut self, capability: &PluginCapability) {
        self.allowed_capabilities.remove(capability);
    }
    
    /// Check if a capability is allowed
    pub fn is_capability_allowed(&self, capability: &PluginCapability) -> bool {
        self.allowed_capabilities.contains(capability)
    }
    
    /// Add a restricted path
    pub fn add_restricted_path(&mut self, path: String) {
        self.restricted_paths.insert(path);
    }
    
    /// Remove a restricted path
    pub fn remove_restricted_path(&mut self, path: &str) {
        self.restricted_paths.remove(path);
    }
    
    /// Check if a path is restricted
    pub fn is_path_restricted(&self, path: &str) -> bool {
        self.restricted_paths.contains(path)
    }
    
    /// Set file system permissions for a plugin
    pub fn set_plugin_fs_permissions(&mut self, plugin_id: Uuid, permissions: FsPermissions) {
        self.plugin_fs_permissions.insert(plugin_id, permissions);
    }
    
    /// Set network permissions for a plugin
    pub fn set_plugin_network_permissions(&mut self, plugin_id: Uuid, permissions: NetworkPermissions) {
        self.plugin_network_permissions.insert(plugin_id, permissions);
    }
    
    /// Add plugin-specific capability restriction
    pub fn add_plugin_capability_restriction(&mut self, plugin_id: Uuid, capability: PluginCapability) {
        self.plugin_capability_restrictions
            .entry(plugin_id)
            .or_insert_with(HashSet::new)
            .insert(capability);
    }
    
    /// Remove plugin-specific capability restriction
    pub fn remove_plugin_capability_restriction(&mut self, plugin_id: Uuid, capability: &PluginCapability) {
        if let Some(restrictions) = self.plugin_capability_restrictions.get_mut(&plugin_id) {
            restrictions.remove(capability);
        }
    }
    
    /// Check if a plugin has a specific capability
    pub fn is_plugin_capability_allowed(&self, plugin_id: Uuid, capability: &PluginCapability) -> bool {
        // First check global restrictions
        if !self.is_capability_allowed(capability) {
            return false;
        }
        
        // Then check plugin-specific restrictions
        if let Some(restrictions) = self.plugin_capability_restrictions.get(&plugin_id) {
            if restrictions.contains(capability) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if a plugin can access a file path
    pub fn is_plugin_fs_access_allowed(&self, plugin_id: Uuid, path: &Path, access_type: FsAccessType) -> bool {
        // Check if path is globally restricted
        if self.is_path_restricted(path.to_str().unwrap_or("")) {
            return false;
        }
        
        // Check plugin-specific permissions
        if let Some(permissions) = self.plugin_fs_permissions.get(&plugin_id) {
            match access_type {
                FsAccessType::Read => {
                    permissions.allowed_read_paths.iter().any(|allowed_path| {
                        path.starts_with(allowed_path)
                    })
                },
                FsAccessType::Write => {
                    permissions.allowed_write_paths.iter().any(|allowed_path| {
                        path.starts_with(allowed_path)
                    })
                },
                FsAccessType::Execute => {
                    permissions.allowed_execute_paths.iter().any(|allowed_path| {
                        path.starts_with(allowed_path)
                    })
                },
            }
        } else {
            // If no specific permissions are set, deny access
            false
        }
    }
    
    /// Check if a plugin can access a network host
    pub fn is_plugin_network_access_allowed(&self, plugin_id: Uuid, host: &str, port: u16, protocol: &str) -> bool {
        if let Some(permissions) = self.plugin_network_permissions.get(&plugin_id) {
            // Check if host is allowed
            if !permissions.allowed_hosts.is_empty() && 
               !permissions.allowed_hosts.contains(host) && 
               !permissions.allowed_hosts.contains("*") {
                return false;
            }
            
            // Check if port is allowed
            if !permissions.allowed_ports.is_empty() && 
               !permissions.allowed_ports.contains(&port) {
                return false;
            }
            
            // Check if protocol is allowed
            if !permissions.allowed_protocols.is_empty() && 
               !permissions.allowed_protocols.contains(protocol) {
                return false;
            }
            
            true
        } else {
            // If no specific permissions are set, deny access
            false
        }
    }
    
    /// Validate plugin permissions
    pub fn validate_plugin_permissions(&self, plugin: &dyn Plugin) -> PluginResult<()> {
        let plugin_id = plugin.id();
        let capabilities = plugin.capabilities();
        
        // Check if all plugin capabilities are allowed
        for capability in &capabilities {
            if !self.is_plugin_capability_allowed(plugin_id, capability) {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Plugin {} does not have permission for capability: {:?}", plugin_id, capability)
                });
            }
        }
        
        Ok(())
    }
    
    /// Create default file system permissions
    pub fn create_default_fs_permissions() -> FsPermissions {
        FsPermissions {
            allowed_read_paths: HashSet::new(),
            allowed_write_paths: HashSet::new(),
            allowed_execute_paths: HashSet::new(),
        }
    }
    
    /// Create default network permissions
    pub fn create_default_network_permissions() -> NetworkPermissions {
        NetworkPermissions {
            allowed_hosts: HashSet::new(),
            allowed_ports: HashSet::new(),
            allowed_protocols: HashSet::new(),
        }
    }
    
    /// Create restricted file system permissions for a plugin
    pub fn create_restricted_fs_permissions(&self, plugin: &dyn Plugin) -> FsPermissions {
        let mut permissions = Self::create_default_fs_permissions();
        
        // Based on plugin capabilities, set appropriate file system permissions
        for capability in plugin.capabilities() {
            match capability {
                PluginCapability::FileSystem => {
                    // Allow read/write access to plugin-specific directories
                    let plugin_dir = PathBuf::from(format!("./plugins/data/{}", plugin.id()));
                    permissions.allowed_read_paths.insert(plugin_dir.clone());
                    permissions.allowed_write_paths.insert(plugin_dir);
                },
                PluginCapability::Streaming => {
                    // Allow read access to temporary directories for streaming
                    permissions.allowed_read_paths.insert(PathBuf::from("./temp"));
                },
                _ => {
                    // Other capabilities don't require file system access
                }
            }
        }
        
        permissions
    }
    
    /// Create restricted network permissions for a plugin
    pub fn create_restricted_network_permissions(&self, plugin: &dyn Plugin) -> NetworkPermissions {
        let mut permissions = Self::create_default_network_permissions();
        
        // Based on plugin capabilities, set appropriate network permissions
        for capability in plugin.capabilities() {
            match capability {
                PluginCapability::Network => {
                    // Allow access to common music service hosts
                    permissions.allowed_hosts.insert("*".to_string()); // Allow all hosts for network capability
                    permissions.allowed_ports.insert(80);
                    permissions.allowed_ports.insert(443);
                    permissions.allowed_protocols.insert("http".to_string());
                    permissions.allowed_protocols.insert("https".to_string());
                },
                PluginCapability::Streaming => {
                    // Allow access to streaming services
                    permissions.allowed_hosts.insert("*.spotify.com".to_string());
                    permissions.allowed_hosts.insert("*.youtube.com".to_string());
                    permissions.allowed_hosts.insert("*.bilibili.com".to_string());
                    permissions.allowed_hosts.insert("*.163.com".to_string()); // Netease
                    permissions.allowed_ports.insert(80);
                    permissions.allowed_ports.insert(443);
                    permissions.allowed_protocols.insert("http".to_string());
                    permissions.allowed_protocols.insert("https".to_string());
                },
                PluginCapability::Authentication => {
                    // Allow access to authentication services
                    permissions.allowed_hosts.insert("accounts.spotify.com".to_string());
                    permissions.allowed_hosts.insert("oauth2.googleapis.com".to_string());
                    permissions.allowed_ports.insert(443);
                    permissions.allowed_protocols.insert("https".to_string());
                },
                _ => {
                    // Other capabilities don't require network access
                }
            }
        }
        
        permissions
    }
    
    /// Set global file system restrictions
    pub fn set_global_fs_restrictions(&mut self, restrictions: FsRestrictions) {
        self.global_fs_restrictions = restrictions;
    }
    
    /// Set global network restrictions
    pub fn set_global_network_restrictions(&mut self, restrictions: NetworkRestrictions) {
        self.global_network_restrictions = restrictions;
    }
    
    /// Check global file system restrictions
    pub fn check_global_fs_restrictions(&self, path: &Path, _access_type: FsAccessType) -> PluginResult<()> {
        // Check forbidden paths
        for forbidden_path in &self.global_fs_restrictions.forbidden_paths {
            if path.starts_with(forbidden_path) {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Access to path {:?} is forbidden", path)
                });
            }
        }
        
        // Check restricted paths
        for restricted_path in &self.global_fs_restrictions.restricted_paths {
            if path.starts_with(restricted_path) {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Access to path {:?} requires special permission", path)
                });
            }
        }
        
        Ok(())
    }
    
    /// Check global network restrictions
    pub fn check_global_network_restrictions(&self, host: &str) -> PluginResult<()> {
        // Check forbidden hosts
        for forbidden_host in &self.global_network_restrictions.forbidden_hosts {
            if host == forbidden_host || (forbidden_host.starts_with("*.") && host.ends_with(&forbidden_host[1..])) {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Access to host {} is forbidden", host)
                });
            }
        }
        
        // Check restricted hosts
        for restricted_host in &self.global_network_restrictions.restricted_hosts {
            if host == restricted_host || (restricted_host.starts_with("*.") && host.ends_with(&restricted_host[1..])) {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Access to host {} requires special permission", host)
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate file size against restrictions
    pub fn validate_file_size(&self, size: u64) -> PluginResult<()> {
        if let Some(max_size) = self.global_fs_restrictions.max_file_size {
            if size > max_size {
                return Err(PluginError::SecurityViolation {
                    reason: format!("File size {} exceeds maximum allowed size {}", size, max_size)
                });
            }
        }
        Ok(())
    }
    
    /// Validate file extension against restrictions
    pub fn validate_file_extension(&self, path: &Path) -> PluginResult<()> {
        if !self.global_fs_restrictions.allowed_extensions.is_empty() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if !self.global_fs_restrictions.allowed_extensions.contains(extension) {
                    return Err(PluginError::SecurityViolation {
                        reason: format!("File extension '{}' is not allowed", extension)
                    });
                }
            } else {
                return Err(PluginError::SecurityViolation {
                    reason: "File has no extension".to_string()
                });
            }
        }
        Ok(())
    }
    
    /// Validate network request size
    pub fn validate_request_size(&self, size: u64) -> PluginResult<()> {
        if let Some(max_size) = self.global_network_restrictions.max_request_size {
            if size > max_size {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Request size {} exceeds maximum allowed size {}", size, max_size)
                });
            }
        }
        Ok(())
    }
    
    /// Validate network response size
    pub fn validate_response_size(&self, size: u64) -> PluginResult<()> {
        if let Some(max_size) = self.global_network_restrictions.max_response_size {
            if size > max_size {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Response size {} exceeds maximum allowed size {}", size, max_size)
                });
            }
        }
        Ok(())
    }
}

/// File system access types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsAccessType {
    /// Read access
    Read,
    
    /// Write access
    Write,
    
    /// Execute access
    Execute,
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FsRestrictions {
    fn default() -> Self {
        Self {
            forbidden_paths: HashSet::new(),
            restricted_paths: HashSet::new(),
            max_file_size: None,
            allowed_extensions: HashSet::new(),
        }
    }
}

impl Default for NetworkRestrictions {
    fn default() -> Self {
        Self {
            forbidden_hosts: HashSet::new(),
            restricted_hosts: HashSet::new(),
            max_request_size: None,
            max_response_size: None,
            rate_limit: None,
        }
    }
}