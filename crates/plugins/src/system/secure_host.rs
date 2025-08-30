//! Secure plugin host implementation with access control

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::path::Path;
use uuid::Uuid;

use crate::system::core::*;
use crate::system::types::*;
use crate::system::registry::PluginRegistry;
use crate::system::security::{SecurityManager, FsAccessType};
use crate::system::sandbox::PluginSandbox;
use crate::PluginResult;

/// Secure plugin host implementation
#[derive(Debug)]
pub struct SecurePluginHost {
    /// Host information
    info: HostInfo,
    
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    
    /// Security manager
    security_manager: Arc<Mutex<SecurityManager>>,
    
    /// Plugin sandboxes
    sandboxes: Arc<Mutex<std::collections::HashMap<Uuid, Arc<Mutex<PluginSandbox>>>>>,
    
    /// Resource usage tracking
    resource_usage: Arc<Mutex<std::collections::HashMap<Uuid, ResourceUsage>>>,
}

/// Resource usage tracking
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Memory usage (in bytes)
    memory_usage: u64,
    
    /// CPU time (in seconds)
    cpu_time: u64,
    
    /// Network bytes sent
    network_sent: u64,
    
    /// Network bytes received
    network_received: u64,
}

impl SecurePluginHost {
    /// Create a new secure plugin host
    pub fn new(
        security_manager: Arc<Mutex<SecurityManager>>,
        sandboxes: Arc<Mutex<std::collections::HashMap<Uuid, Arc<Mutex<PluginSandbox>>>>>,
    ) -> Self {
        let info = HostInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            services: vec![
                "logging".to_string(),
                "settings".to_string(),
                "database".to_string(),
            ],
        };
        
        let registry = Arc::new(PluginRegistry::new());
        
        Self {
            info,
            registry,
            security_manager,
            sandboxes,
            resource_usage: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    /// Check if file system access is allowed for a plugin
    fn check_fs_access(&self, plugin_id: Uuid, path: &Path, access_type: FsAccessType) -> PluginResult<()> {
        // First check with the security manager
        let security_manager = self.security_manager.lock().unwrap();
        if !security_manager.is_plugin_fs_access_allowed(plugin_id, path, access_type.clone()) {
            return Err(PluginError::SecurityViolation {
                reason: format!("Plugin {} does not have permission for {:?} access to {:?}", 
                               plugin_id, access_type, path)
            });
        }
        
        // Then check with the sandbox if it exists
        let sandboxes = self.sandboxes.lock().unwrap();
        if let Some(sandbox) = sandboxes.get(&plugin_id) {
            let sandbox = sandbox.lock().unwrap();
            if !sandbox.is_fs_access_allowed(path, access_type.clone()) {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Sandbox denied {:?} access to {:?} for plugin {}", 
                                   access_type, path, plugin_id)
                });
            }
        }
        
        Ok(())
    }
    
    /// Check if network access is allowed for a plugin
    fn check_network_access(&self, plugin_id: Uuid, host: &str, port: u64, protocol: &str) -> PluginResult<()> {
        // First check with the security manager
        let security_manager = self.security_manager.lock().unwrap();
        if !security_manager.is_plugin_network_access_allowed(plugin_id, host, port as u16, protocol) {
            return Err(PluginError::SecurityViolation {
                reason: format!("Plugin {} does not have permission for {}://{}:{} access", 
                               plugin_id, protocol, host, port)
            });
        }
        
        // Then check with the sandbox if it exists
        let sandboxes = self.sandboxes.lock().unwrap();
        if let Some(sandbox) = sandboxes.get(&plugin_id) {
            let sandbox = sandbox.lock().unwrap();
            if !sandbox.is_network_access_allowed(host, port, protocol) {
                return Err(PluginError::SecurityViolation {
                    reason: format!("Sandbox denied {}://{}:{} access for plugin {}", 
                                   protocol, host, port, plugin_id)
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate file operation for a plugin
    fn validate_file_operation(&self, plugin_id: Uuid, path: &Path, size: u64) -> PluginResult<()> {
        let sandboxes = self.sandboxes.lock().unwrap();
        if let Some(sandbox) = sandboxes.get(&plugin_id) {
            let sandbox = sandbox.lock().unwrap();
            sandbox.validate_file_operation(path, size)?;
        }
        Ok(())
    }
    
    /// Validate network operation for a plugin
    fn validate_network_operation(&self, plugin_id: Uuid, host: &str, request_size: u64, response_size: u64) -> PluginResult<()> {
        let sandboxes = self.sandboxes.lock().unwrap();
        if let Some(sandbox) = sandboxes.get(&plugin_id) {
            let sandbox = sandbox.lock().unwrap();
            sandbox.validate_network_operation(host, request_size, response_size)?;
        }
        Ok(())
    }
    
    /// Update resource usage for a plugin
    fn update_resource_usage(&self, plugin_id: Uuid, usage: ResourceUsage) {
        let mut resource_usage = self.resource_usage.lock().unwrap();
        resource_usage.insert(plugin_id, usage);
    }
    
    /// Check resource limits for a plugin
    fn check_resource_limits(&self, plugin_id: Uuid) -> PluginResult<()> {
        let _security_manager = self.security_manager.lock().unwrap();
        let sandboxes = self.sandboxes.lock().unwrap();
        
        if let Some(sandbox) = sandboxes.get(&plugin_id) {
            let sandbox = sandbox.lock().unwrap();
            let resource_limits = &sandbox.resource_limits;
            
            let resource_usage = self.resource_usage.lock().unwrap();
            if let Some(usage) = resource_usage.get(&plugin_id) {
                // Check memory limit
                if let Some(max_memory) = resource_limits.max_memory {
                    if usage.memory_usage > max_memory {
                        return Err(PluginError::SecurityViolation {
                            reason: format!("Plugin {} exceeded memory limit: {} > {}", 
                                           plugin_id, usage.memory_usage, max_memory)
                        });
                    }
                }
                
                // Check CPU time limit
                if let Some(max_cpu_time) = resource_limits.max_cpu_time {
                    if usage.cpu_time > max_cpu_time {
                        return Err(PluginError::SecurityViolation {
                            reason: format!("Plugin {} exceeded CPU time limit: {} > {}", 
                                           plugin_id, usage.cpu_time, max_cpu_time)
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl PluginHost for SecurePluginHost {
    fn host_info(&self) -> HostInfo {
        self.info.clone()
    }
    
    async fn log(&self, plugin_id: Uuid, level: LogLevel, message: &str) {
        // Logging is always allowed
        println!("[Plugin {}] {:?}: {}", plugin_id, level, message);
    }
    
    async fn emit_event(&self, plugin_id: Uuid, event: PluginEvent) -> PluginResult<()> {
        // Event emission is always allowed
        println!("Plugin {} emitted event: {:?}", plugin_id, event);
        Ok(())
    }
    
    async fn request_service(&self, plugin_id: Uuid, service: &str, data: serde_json::Value) -> PluginResult<serde_json::Value> {
        // Check if the plugin has permission to request this service
        match service {
            "filesystem" => {
                // Extract path from data and check access
                if let Some(path_str) = data.get("path").and_then(|v| v.as_str()) {
                    let path = Path::new(path_str);
                    let access_type = if data.get("write").and_then(|v| v.as_bool()).unwrap_or(false) {
                        FsAccessType::Write
                    } else {
                        FsAccessType::Read
                    };
                    
                    // Check file size if provided
                    let file_size = data.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                    self.validate_file_operation(plugin_id, path, file_size)?;
                    self.check_fs_access(plugin_id, path, access_type)?;
                }
            },
            "network" => {
                // Extract network details from data and check access
                if let Some(host) = data.get("host").and_then(|v| v.as_str()) {
                    let port = data.get("port").and_then(|v| v.as_u64()).unwrap_or(80);
                    let protocol = data.get("protocol").and_then(|v| v.as_str()).unwrap_or("http");
                    
                    // Check request/response sizes if provided
                    let request_size = data.get("request_size").and_then(|v| v.as_u64()).unwrap_or(0);
                    let response_size = data.get("response_size").and_then(|v| v.as_u64()).unwrap_or(0);
                    self.validate_network_operation(plugin_id, host, request_size, response_size)?;
                    self.check_network_access(plugin_id, host, port, protocol)?;
                }
            },
            "database" => {
                // Database access might have specific permissions
                // For now, we'll allow it but in a real implementation we might want to check
            },
            "settings" => {
                // Settings access is generally allowed
            },
            _ => {
                // Unknown services are denied
                return Err(PluginError::SecurityViolation {
                    reason: format!("Plugin {} requested unknown service: {}", plugin_id, service)
                });
            }
        }
        
        // Check resource limits
        self.check_resource_limits(plugin_id)?;
        
        // In a real implementation, we would actually provide the service
        Ok(serde_json::Value::Null)
    }
    
    async fn get_setting(&self, plugin_id: Uuid, key: &str) -> PluginResult<Option<serde_json::Value>> {
        // Settings access is generally allowed
        println!("Plugin {} requested setting: {}", plugin_id, key);
        Ok(None)
    }
    
    async fn set_setting(&self, plugin_id: Uuid, key: &str, value: serde_json::Value) -> PluginResult<()> {
        // Settings modification is generally allowed
        println!("Plugin {} set setting {} to {:?}", plugin_id, key, value);
        Ok(())
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_usage: 0,
            cpu_time: 0,
            network_sent: 0,
            network_received: 0,
        }
    }
}