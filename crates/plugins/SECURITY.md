# Plugin Security System

This document describes the enhanced security mechanisms implemented in the music player plugin system.

## Overview

The plugin security system provides multiple layers of protection to ensure that plugins can only access resources they are authorized to use. The system includes:

1. Capability-based permissions
2. File system access controls
3. Network access controls
4. Sandboxing
5. Resource limits
6. Global restrictions

## Security Components

### SecurityManager

The `SecurityManager` is the core component that handles all security-related operations:

- **Capability Management**: Controls which plugin capabilities are allowed globally or per-plugin
- **File System Permissions**: Manages read/write/execute permissions for specific paths
- **Network Permissions**: Controls which hosts, ports, and protocols plugins can access
- **Global Restrictions**: Sets system-wide restrictions on file system and network access

### PluginSandbox

The `PluginSandbox` provides isolation for each plugin:

- **Virtual File System**: Maps plugin paths to real file system locations
- **Process Isolation**: Can run plugins in separate processes with different user permissions
- **Resource Limits**: Controls memory, CPU, file descriptors, and network connections

### SecurePluginHost

The `SecurePluginHost` acts as a secure intermediary between plugins and system services:

- **Access Control**: Validates all plugin requests for file system and network access
- **Resource Monitoring**: Tracks resource usage and enforces limits
- **Service Proxy**: Provides controlled access to system services

## Security Features

### Capability-Based Permissions

Plugins declare their required capabilities in their metadata. The security system ensures that:

1. Only allowed capabilities can be used globally
2. Individual plugins can have specific capability restrictions
3. Plugins are validated before initialization and startup

### File System Access Controls

The system provides fine-grained control over file system access:

- **Path Restrictions**: Global forbidden and restricted paths
- **Plugin-Specific Permissions**: Each plugin can only access specific directories
- **File Size Limits**: Maximum file sizes can be enforced
- **File Extension Filtering**: Only specific file types may be allowed

### Network Access Controls

Network access is controlled through:

- **Host Restrictions**: Forbidden and restricted hosts
- **Port Controls**: Only specific ports may be accessed
- **Protocol Filtering**: Only allowed protocols (HTTP, HTTPS, etc.)
- **Request/Response Size Limits**: Controls data transfer sizes

### Sandboxing

Each plugin runs in its own sandbox with:

- **Virtual File System**: Plugins see a virtualized file system
- **Process Isolation**: Optional separate process execution
- **User/Group Isolation**: Optional execution under different user accounts

### Resource Limits

Plugins are subject to resource limits:

- **Memory Usage**: Maximum memory allocation
- **CPU Time**: Maximum CPU time allocation
- **File Descriptors**: Maximum open file count
- **Network Connections**: Maximum concurrent connections

### Global Restrictions

System-wide restrictions protect critical system resources:

- **File System**: Forbidden system directories (/etc, /usr, C:\Windows, etc.)
- **Network**: Forbidden internal network hosts (localhost, 127.0.0.1, etc.)

## Usage Examples

### Setting Global Restrictions

```rust
let mut security_manager = SecurityManager::new();

// Set up global file system restrictions
let mut fs_restrictions = FsRestrictions::default();
fs_restrictions.forbidden_paths.insert(PathBuf::from("/etc"));
fs_restrictions.forbidden_paths.insert(PathBuf::from("C:\\Windows"));

// Set up global network restrictions
let mut network_restrictions = NetworkRestrictions::default();
network_restrictions.forbidden_hosts.insert("localhost".to_string());
network_restrictions.forbidden_hosts.insert("127.0.0.1".to_string());

security_manager.set_global_fs_restrictions(fs_restrictions);
security_manager.set_global_network_restrictions(network_restrictions);
```

### Creating a Sandboxed Plugin

```rust
let security_manager = Arc::new(Mutex::new(SecurityManager::new()));
let mut sandbox_manager = SandboxManager::new(
    Arc::clone(&security_manager),
    Path::new("./sandboxes").to_path_buf()
);

let plugin_id = Uuid::new_v4();
let plugin = MyPlugin::new(plugin_id);

// Create sandbox with specific settings
let sandbox = sandbox_manager.create_sandbox(&plugin)?;
let mut sandbox = sandbox.lock().unwrap();

// Configure process isolation
let isolation = ProcessIsolation {
    separate_process: true,
    user_id: Some(1000),
    group_id: Some(1000),
};

// Configure resource limits
let limits = ResourceLimits {
    max_memory: Some(100 * 1024 * 1024), // 100MB
    max_cpu_time: Some(60), // 60 seconds
    max_file_descriptors: Some(100),
    max_network_connections: Some(10),
};

sandbox.set_process_isolation(isolation);
sandbox.set_resource_limits(limits);
```

### Secure Plugin Host Usage

```rust
let security_manager = Arc::new(Mutex::new(SecurityManager::new()));
let sandboxes = Arc::new(Mutex::new(HashMap::new()));

let secure_host = SecurePluginHost::new(
    Arc::clone(&security_manager),
    Arc::clone(&sandboxes)
);

// Plugins can request services through the secure host
// All requests are validated against security policies
```

## Security Best Practices

1. **Principle of Least Privilege**: Only grant plugins the minimum capabilities they need
2. **Regular Audits**: Periodically review plugin permissions and capabilities
3. **Global Restrictions**: Always set appropriate global restrictions
4. **Resource Limits**: Set reasonable resource limits to prevent abuse
5. **Monitoring**: Monitor plugin resource usage and behavior
6. **Updates**: Keep the security system updated with the latest protections

## Future Enhancements

Planned future enhancements to the security system include:

- **Dynamic Permission Requests**: Plugins can request additional permissions at runtime
- **Machine Learning-Based Anomaly Detection**: Detect unusual plugin behavior
- **Enhanced Sandboxing**: Integration with OS-level sandboxing features
- **Audit Logging**: Comprehensive security event logging
- **Certificate-Based Plugin Signing**: Ensure plugin authenticity