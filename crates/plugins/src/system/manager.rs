//! Plugin manager for coordinating plugin operations

use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use uuid::Uuid;

use crate::system::core::*;
use crate::system::types::*;
use crate::system::registry::PluginRegistry;
use crate::system::loader::PluginLoader;
use crate::system::host::PluginHost;
use crate::system::security::{SecurityManager, FsRestrictions, NetworkRestrictions};
use crate::system::lifecycle::LifecycleManager;
use crate::system::state::PluginStateManager;
use crate::system::state::metadata_to_state;
use crate::system::sandbox::{SandboxManager, ProcessIsolation, ResourceLimits};
use crate::system::secure_host::SecurePluginHost;
use crate::factory::MediaPluginFactory;
use crate::PluginResult;
use include_dir::{include_dir, Dir};
use music_plugin_sdk::traits::media::MediaPlugin;
use music_plugin_sdk::traits::BasePlugin;
use async_trait::async_trait;
// use async_trait::async_trait; // 未使用，移除


static BUILTIN_ICONS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets/builtin-icons");

/// Plugin manager for coordinating all plugin operations
pub struct PluginManager {
    /// Plugin host
    host: Arc<dyn crate::system::core::PluginHost>,
    /// Secure plugin host for sandboxed operations
    _secure_host: Arc<dyn crate::system::core::PluginHost>,
    /// Plugin registry
    registry: Arc<PluginRegistry>,
    /// Plugin loader
    loader: Arc<PluginLoader>,
    /// Lifecycle manager
    lifecycle: Arc<LifecycleManager>,
    /// Security manager
    security: Arc<Mutex<SecurityManager>>,
    /// Sandbox manager
    sandbox_manager: Arc<Mutex<SandboxManager>>,
    /// Plugin state manager
    state_manager: Arc<PluginStateManager>,
    /// Audio plugin factory
    audio_factory: Arc<Mutex<MediaPluginFactory>>,
    /// Root directory for plugin installation
    plugin_root: PathBuf,
}

// Manual Debug implementation to avoid issues with trait objects
impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("host", &"PluginHost")
            .field("secure_host", &"SecurePluginHost")
            .field("registry", &self.registry)
            .field("loader", &self.loader)
            .field("lifecycle", &self.lifecycle)
            .field("security", &self.security)
            .field("sandbox_manager", &self.sandbox_manager)
            .field("state_manager", &self.state_manager)
            .finish()
    }
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(database: database::database::Database, plugin_root: PathBuf) -> Self {
        // Create core components
        let security = Arc::new(Mutex::new(SecurityManager::new()));
        
        // Set up global security restrictions
        {
            let mut security_manager = security.lock().unwrap();
            
            // Set up global file system restrictions
            let mut fs_restrictions = FsRestrictions::default();
            // Forbid access to system directories
            fs_restrictions.forbidden_paths.insert(PathBuf::from("/etc"));
            fs_restrictions.forbidden_paths.insert(PathBuf::from("/usr"));
            fs_restrictions.forbidden_paths.insert(PathBuf::from("/bin"));
            fs_restrictions.forbidden_paths.insert(PathBuf::from("/sbin"));
            fs_restrictions.forbidden_paths.insert(PathBuf::from("/root"));
            // On Windows
            fs_restrictions.forbidden_paths.insert(PathBuf::from("C:\\Windows"));
            fs_restrictions.forbidden_paths.insert(PathBuf::from("C:\\Program Files"));
            
            // Set up global network restrictions
            let mut network_restrictions = NetworkRestrictions::default();
            // Forbid access to internal network hosts
            network_restrictions.forbidden_hosts.insert("localhost".to_string());
            network_restrictions.forbidden_hosts.insert("127.0.0.1".to_string());
            network_restrictions.forbidden_hosts.insert("192.168.0.0/16".to_string());
            network_restrictions.forbidden_hosts.insert("10.0.0.0/8".to_string());
            network_restrictions.forbidden_hosts.insert("172.16.0.0/12".to_string());
            
            security_manager.set_global_fs_restrictions(fs_restrictions);
            security_manager.set_global_network_restrictions(network_restrictions);

            // Allow core capabilities required by built-in audio providers
            use crate::system::types::PluginCapability;
            security_manager.add_allowed_capability(PluginCapability::Search);
            security_manager.add_allowed_capability(PluginCapability::Playlists);
            security_manager.add_allowed_capability(PluginCapability::Streaming);
            security_manager.add_allowed_capability(PluginCapability::Authentication);
        }
        
        let sandbox_manager = Arc::new(Mutex::new(SandboxManager::new(
            Arc::clone(&security),
            Path::new("./sandboxes").to_path_buf()
        )));
        
        let registry = Arc::new(PluginRegistry::new());
        
        // Create hosts
        let host: Arc<dyn crate::system::core::PluginHost> = Arc::new(PluginHost::new());
        let secure_host: Arc<dyn crate::system::core::PluginHost> = Arc::new(SecurePluginHost::new(
            Arc::clone(&security),
            Arc::new(Mutex::new(HashMap::new()))
        ));
        
        // Create lifecycle manager
        let lifecycle = Arc::new(LifecycleManager::new(
            Arc::clone(&registry),
            Arc::clone(&security)  // 克隆Arc而不是移动
        ));
        
        // Create plugin loader
        let loader = Arc::new(PluginLoader::new(Arc::clone(&registry)));
        
        let state_manager = Arc::new(PluginStateManager::new(database));
        
        // Create audio plugin factory
        let audio_factory = Arc::new(Mutex::new(MediaPluginFactory::new()));
        
        // Ensure plugin root exists
        std::fs::create_dir_all(&plugin_root).ok();

        Self {
            host,
            _secure_host: secure_host,
            registry,
            loader,
            lifecycle,
            security,
            sandbox_manager,
            state_manager,
            audio_factory,
            plugin_root,
        }
    }
    
    /// Initialize the plugin manager
    pub async fn initialize(&self) -> PluginResult<()> {
        // Load plugin states from database
        let _plugin_states = self.state_manager.get_all_plugin_states()?;
        
        // Load all plugins (built-in and external)
        self.load_all_plugins().await?;

        // Seed database records for any loaded plugins missing state rows
        {
            let plugins = self.registry.get_all_plugins().await?;
            for plugin_mutex in plugins {
                let (metadata, plugin_id) = {
                    let plugin_guard = plugin_mutex.lock().unwrap();
                    (plugin_guard.metadata(), plugin_guard.id())
                };

                let plugin_id_str = plugin_id.to_string();

                // Prefer existing record by id
                if self.state_manager.get_plugin_state(&plugin_id_str)?.is_none() {
                    // Fallback: try find by name to avoid duplicates if ID changed historically
                    if let Some(existing) = self.state_manager.get_plugin_state_by_name(&metadata.name)? {
                        if existing.id != plugin_id_str {
                            // Migrate primary key to current deterministic id
                            let _ = self.state_manager.update_plugin_state_id(&existing.id, &plugin_id_str);
                        }
                    } else {
                        // No record found by id or name: insert new
                        let state = metadata_to_state(&metadata, true, "{}");
                        let _ = self.state_manager.save_plugin_state(&state);
                    }
                }

                // Ensure minimal install layout and icon for builtin plugins
                let _ = self.ensure_install_layout(&metadata);
            }
        }
        
        // Initialize all loaded plugins
        let context = PluginContext {
            host: Arc::clone(&self.host),
            registry: Arc::clone(&self.registry) as Arc<dyn crate::system::core::PluginRegistry>,
            settings: serde_json::Value::Object(serde_json::Map::new()),
        };
        
        self.lifecycle.initialize_all_plugins(context).await?;
        
        // Initialize audio plugin factory - no need to iterate!
        // Media plugins are already registered to factory during loading        
        Ok(())
    }

    /// Ensure minimal install layout <app_data_dir>/plugins/<plugin-id>/assets/icons/icon.png
    fn ensure_install_layout(&self, metadata: &PluginMetadata) -> PluginResult<()> {
        let install_dir = self.plugin_root.join(metadata.id.to_string());
        let icons_dir = install_dir.join("assets").join("icons");
        std::fs::create_dir_all(&icons_dir)
            .map_err(|e| PluginError::ExecutionFailed { reason: format!("Failed to create icons dir: {}", e) })?;

        // Resolve an embedded builtin icon by name or id
        let by_name = format!("{}.png", metadata.name);
        let by_id = format!("{}.png", metadata.id);
        let icon_entry = BUILTIN_ICONS.get_file(&by_name).or_else(|| BUILTIN_ICONS.get_file(&by_id));

        if let Some(file) = icon_entry {
            let target = icons_dir.join("icon.png");
            // Write only if not exists to avoid overwriting user-provided icons
            if !target.exists() {
                std::fs::write(&target, file.contents())
                    .map_err(|e| PluginError::ExecutionFailed { reason: format!("Failed to write builtin icon: {}", e) })?;
            }

            // Persist icon path to DB if missing
            if let Some(mut st) = self.state_manager.get_plugin_state(&metadata.id.to_string())? {
                if st.icon.is_none() {
                    st.icon = Some(target.to_string_lossy().to_string());
                    st.last_updated = chrono::Utc::now().naive_utc();
                    let _ = self.state_manager.save_plugin_state(&st);
                }
            }
        }

        // Ensure a minimal manifest.json exists
        let manifest_path = install_dir.join("manifest.json");
        if !manifest_path.exists() {
            let icon_rel = "assets/icons/icon.png";
            let caps: Vec<String> = metadata
                .capabilities
                .iter()
                .map(|c| format!("{:?}", c))
                .collect();
            let manifest = serde_json::json!({
                "id": metadata.id.to_string(),
                "name": metadata.name,
                "display_name": metadata.display_name,
                "version": metadata.version.to_string(),
                "type": metadata.plugin_type.to_string(),
                "capabilities": caps,
                "entry": serde_json::Value::Null,
                "icon": icon_rel,
            });
            std::fs::create_dir_all(&install_dir)
                .map_err(|e| PluginError::ExecutionFailed { reason: format!("Failed to create install dir: {}", e) })?;
            std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest).unwrap())
                .map_err(|e| PluginError::ExecutionFailed { reason: format!("Failed to write manifest.json: {}", e) })?;
        }

        Ok(())
    }
    
    /// Start all enabled plugins
    pub async fn start_plugins(&self) -> PluginResult<()> {
        // Start only plugins marked enabled in DB
        let enabled_states = self.state_manager.get_enabled_plugin_states()?;
        let mut ids = Vec::new();
        for st in enabled_states {
            if let Ok(uuid) = Uuid::parse_str(&st.id) {
                ids.push(uuid);
            }
        }
        for id in ids {
            // Start individually; ignore errors per plugin to continue others
            let _ = self.lifecycle.start_plugin(id).await;
        }
        Ok(())
    }
    
    /// Stop all running plugins
    pub async fn stop_plugins(&self) -> PluginResult<()> {
        self.lifecycle.stop_all_plugins().await
    }
    
    /// Destroy all plugins
    pub async fn destroy_plugins(&self) -> PluginResult<()> {
        self.lifecycle.destroy_all_plugins().await
    }
    
    /// Built-in media plugin loader - automatically registers to media factory
    async fn load_builtin_media_plugin<T>(&self, plugin: T) -> PluginResult<()> 
    where 
        T: Plugin + MediaPlugin + Clone + Send + Sync + 'static 
    {
        let plugin_id = <T as crate::system::core::Plugin>::id(&plugin);
        
        // 1. Register to system plugin manager
        let plugin_box: Box<dyn crate::system::core::Plugin> = Box::new(plugin.clone());
        self.registry.register_plugin(plugin_box).await?;
        
        // 2. Get plugin status
        let enabled = self.get_plugin_enabled(plugin_id)?;
        
        // 3. Directly register to media factory! No need for subsequent iteration
        {
            let mut audio_factory = self.audio_factory.lock().unwrap();
            let media_plugin: Arc<tokio::sync::Mutex<dyn MediaPlugin + Send + Sync>> = 
                Arc::new(tokio::sync::Mutex::new(plugin));
            audio_factory.register_to_media_factory(plugin_id, media_plugin, enabled);
        }
        
        Ok(())
    }
    /// Load all plugins from default directories
    pub async fn load_all_plugins(&self) -> PluginResult<()> {
        // Load built-in media plugins - directly register to media factory
        self.load_builtin_media_plugin(crate::internal::BilibiliPlugin::new()).await?;
        
        // TODO: Uncomment other built-in media plugins
        // self.load_builtin_media_plugin(crate::internal::YouTubePlugin::new()).await?;
        // self.load_builtin_media_plugin(crate::internal::SpotifyPlugin::new()).await?;
        
        // Load external media plugins
        self.load_external_media_plugins().await?;
        
        // Load other type plugins (non-media plugins)
        self.load_other_plugins().await?;
        
        Ok(())
    }
    
    /// Load external media plugins - automatically register to media factory
    async fn load_external_media_plugins(&self) -> PluginResult<()> {
        // WASM media plugins
        let wasm_plugins_dir = std::path::Path::new("./plugins/wasm");
        if wasm_plugins_dir.exists() {
            if let Err(e) = self.load_external_media_plugins_from_directory(wasm_plugins_dir).await {
                eprintln!("Warning: Failed to load WASM media plugins: {}", e);
            }
        }
        
        // Dynamic library media plugins
        let dynamic_plugins_dir = std::path::Path::new("./plugins/dynamic");
        if dynamic_plugins_dir.exists() {
            if let Err(e) = self.load_external_media_plugins_from_directory(dynamic_plugins_dir).await {
                eprintln!("Warning: Failed to load dynamic library media plugins: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Load external media plugins from directory
    async fn load_external_media_plugins_from_directory(&self, dir_path: &std::path::Path) -> PluginResult<()> {
        // Iterate through plugin files in directory
        let entries = std::fs::read_dir(dir_path)
            .map_err(|e| PluginError::LoadFailed { reason: format!("Failed to read plugin directory: {}", e) })?;
            
        for entry in entries {
            let entry = entry.map_err(|e| PluginError::LoadFailed { reason: e.to_string() })?;
            let path = entry.path();
            
            // Only process plugin files
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                match extension {
                    "wasm" => {
                        if let Ok(plugin) = self.load_wasm_media_plugin(&path).await {
                            self.register_external_media_plugin(plugin).await?;
                        }
                    },
                    "dll" | "so" | "dylib" => {
                        if let Ok(plugin) = self.load_dynamic_media_plugin(&path).await {
                            self.register_external_media_plugin(plugin).await?;
                        }
                    },
                    _ => continue,
                }
            }
        }
        
        Ok(())
    }
    
    /// Register external media plugin to factory
    async fn register_external_media_plugin(&self, plugin: Box<dyn MediaPlugin + Send + Sync>) -> PluginResult<()> {
        // Get basic information through MediaPlugin
        let plugin_metadata = plugin.metadata();
        let plugin_id = plugin_metadata.id;
        
        // Get plugin status
        let enabled = self.get_plugin_enabled(plugin_id).unwrap_or(true);
        
        // Create an external plugin wrapper
        #[derive(Debug)]
        struct ExternalMediaPluginWrapper {
            inner: Box<dyn MediaPlugin + Send + Sync>,
        }
        
        #[async_trait]
        impl MediaPlugin for ExternalMediaPluginWrapper {
            async fn search(&self, query: &music_plugin_sdk::types::SearchQuery) -> music_plugin_sdk::types::base::PluginResult<music_plugin_sdk::types::SearchResult> {
                self.inner.search(query).await
            }
            
            async fn get_track(&self, track_id: &str) -> music_plugin_sdk::types::base::PluginResult<music_plugin_sdk::types::Track> {
                self.inner.get_track(track_id).await
            }
            
            async fn get_media_stream(&self, track_id: &str, req: &music_plugin_sdk::types::StreamRequest) -> music_plugin_sdk::types::base::PluginResult<music_plugin_sdk::types::StreamSource> {
                self.inner.get_media_stream(track_id, req).await
            }
            
            async fn get_album(&self, album_id: &str) -> music_plugin_sdk::types::base::PluginResult<music_plugin_sdk::types::Album> {
                self.inner.get_album(album_id).await
            }
            
            async fn get_artist(&self, artist_id: &str) -> music_plugin_sdk::types::base::PluginResult<music_plugin_sdk::types::Artist> {
                self.inner.get_artist(artist_id).await
            }
            
            async fn get_playlist(&self, playlist_id: &str) -> music_plugin_sdk::types::base::PluginResult<music_plugin_sdk::types::Playlist> {
                self.inner.get_playlist(playlist_id).await
            }
            
            async fn is_track_available(&self, track_id: &str) -> music_plugin_sdk::types::base::PluginResult<bool> {
                self.inner.is_track_available(track_id).await
            }
        }
        
        #[async_trait]
        impl BasePlugin for ExternalMediaPluginWrapper {
            fn metadata(&self) -> music_plugin_sdk::types::base::PluginMetadata {
                self.inner.metadata()
            }
            
            async fn initialize(&mut self, _context: &music_plugin_sdk::types::base::PluginContext) -> music_plugin_sdk::types::base::PluginResult<()> {
                // We can't call initialize on the inner plugin since it's behind a Box
                // External plugins should handle initialization themselves
                Ok(())
            }
            
            async fn start(&mut self) -> music_plugin_sdk::types::base::PluginResult<()> {
                Ok(())
            }
            
            async fn stop(&mut self) -> music_plugin_sdk::types::base::PluginResult<()> {
                Ok(())
            }
            
            fn status(&self) -> music_plugin_sdk::types::base::PluginStatus {
                music_plugin_sdk::types::base::PluginStatus::Running
            }
            
            async fn configure(&mut self, _config: music_plugin_sdk::types::base::PluginConfig) -> music_plugin_sdk::types::base::PluginResult<()> {
                Ok(())
            }
        }
        
        let wrapper = ExternalMediaPluginWrapper { inner: plugin };
        let arc_plugin = Arc::new(tokio::sync::Mutex::new(wrapper));
        
        // Directly register to media factory
        {
            let mut audio_factory = self.audio_factory.lock().unwrap();
            audio_factory.register_external_media_plugin_to_factory(
                plugin_id, 
                arc_plugin, 
                enabled
            );
        }
        
        println!("External media plugin loaded: {} ({})", plugin_metadata.name, plugin_id);
        Ok(())
    }
    
    /// Load WASM media plugin
    async fn load_wasm_media_plugin(&self, _path: &std::path::Path) -> PluginResult<Box<dyn MediaPlugin + Send + Sync>> {
        // TODO: Implement WASM plugin loading logic
        Err(PluginError::ExecutionFailed { reason: "WASM media plugin loading not implemented yet".to_string() })
    }
    
    /// Load dynamic library media plugin
    async fn load_dynamic_media_plugin(&self, _path: &std::path::Path) -> PluginResult<Box<dyn MediaPlugin + Send + Sync>> {
        // TODO: Implement dynamic library plugin loading logic
        Err(PluginError::ExecutionFailed { reason: "Dynamic library media plugin loading not implemented yet".to_string() })
    }
    
    /// Load other type plugins (theme, tool, etc. non-media plugins)
    async fn load_other_plugins(&self) -> PluginResult<()> {
        // Load non-media plugins here, not registered to media factory
        // For example: theme plugins, tool plugins, etc.
        println!("Other type plugins loading completed");
        Ok(())
    }
    
    /// Load a plugin from file
    pub async fn load_plugin_from_file(&self, plugin_path: &Path) -> PluginResult<()> {
        self.loader.load_plugin_from_file(plugin_path).await
    }
    
    /// Load plugins from directory
    pub async fn load_plugins_from_directory(&self, dir_path: &Path) -> PluginResult<()> {
        self.loader.load_plugins_from_directory(dir_path).await
    }
    
    /// Perform health check on all plugins
    pub async fn health_check_all_plugins(&self) -> PluginResult<Vec<(Uuid, HealthStatus)>> {
        self.lifecycle.health_check_all_plugins().await
    }
    
    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: Uuid) -> PluginResult<Option<Arc<std::sync::Mutex<dyn Plugin>>>> {
        self.registry.get_plugin(plugin_id).await
    }
    
    /// Get all plugins
    pub async fn get_all_plugins(&self) -> PluginResult<Vec<Arc<std::sync::Mutex<dyn Plugin>>>> {
        self.registry.get_all_plugins().await
    }
    
    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        // Ensure state exists (upsert) and dedupe by name if needed
        let pid = plugin_id.to_string();
        if self.state_manager.get_plugin_state(&pid)?.is_none() {
            if let Some(plugin) = self.registry.get_plugin(plugin_id).await? {
                let (metadata, _) = {
                    let p = plugin.lock().unwrap();
                    (p.metadata(), p.id())
                };
                if let Some(existing) = self.state_manager.get_plugin_state_by_name(&metadata.name)? {
                    if existing.id != pid {
                        let _ = self.state_manager.update_plugin_state_id(&existing.id, &pid);
                    }
                } else {
                    let state = metadata_to_state(&metadata, true, "{}");
                    let _ = self.state_manager.save_plugin_state(&state);
                }
            }
        }
        // Update DB and start runtime
        self.state_manager.enable_plugin(&pid)?;
        let _ = self.lifecycle.start_plugin(plugin_id).await;
        Ok(())
    }
    
    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        // Ensure state exists (upsert with enabled=false) and dedupe by name
        let pid = plugin_id.to_string();
        if self.state_manager.get_plugin_state(&pid)?.is_none() {
            if let Some(plugin) = self.registry.get_plugin(plugin_id).await? {
                let (metadata, _) = {
                    let p = plugin.lock().unwrap();
                    (p.metadata(), p.id())
                };
                if let Some(existing) = self.state_manager.get_plugin_state_by_name(&metadata.name)? {
                    if existing.id != pid {
                        let _ = self.state_manager.update_plugin_state_id(&existing.id, &pid);
                    }
                } else {
                    let mut state = metadata_to_state(&metadata, false, "{}");
                    state.enabled = false;
                    let _ = self.state_manager.save_plugin_state(&state);
                }
            }
        }
        // Update DB and stop runtime
        self.state_manager.disable_plugin(&pid)?;
        let _ = self.lifecycle.stop_plugin(plugin_id).await;
        Ok(())
    }
    
    /// Get plugin status
    pub async fn get_plugin_status(&self, plugin_id: Uuid) -> PluginResult<PluginStatus> {
        self.lifecycle.get_plugin_status(plugin_id).await
    }

    /// Get whether a plugin is enabled according to the database
    pub fn get_plugin_enabled(&self, plugin_id: Uuid) -> PluginResult<bool> {
        let enabled = self
            .state_manager
            .get_plugin_state(&plugin_id.to_string())?
            .map(|st| st.enabled)
            .unwrap_or(true);
        Ok(enabled)
    }

    /// Get plugin icon path from the database, if any
    pub fn get_plugin_icon(&self, plugin_id: Uuid) -> PluginResult<Option<String>> {
        let icon = self
            .state_manager
            .get_plugin_state(&plugin_id.to_string())?
            .and_then(|st| st.icon);
        Ok(icon)
    }
    
    /// Perform health check on a plugin
    pub async fn health_check_plugin(&self, plugin_id: Uuid) -> PluginResult<HealthStatus> {
        self.lifecycle.health_check_plugin(plugin_id).await
    }
    
    /// Set plugin capability permission
    pub fn set_plugin_capability_permission(&self, capability: PluginCapability, allowed: bool) {
        let mut security = self.security.lock().unwrap();
        if allowed {
            security.add_allowed_capability(capability);
        } else {
            security.remove_allowed_capability(&capability);
        }
    }
    
    /// Set plugin-specific capability restriction
    pub fn set_plugin_capability_restriction(&self, plugin_id: Uuid, capability: PluginCapability, restricted: bool) {
        let mut security = self.security.lock().unwrap();
        if restricted {
            security.add_plugin_capability_restriction(plugin_id, capability);
        } else {
            security.remove_plugin_capability_restriction(plugin_id, &capability);
        }
    }
    
    /// Create sandbox for a plugin
    pub fn create_sandbox(&self, plugin: &dyn Plugin) -> PluginResult<()> {
        let mut sandbox_manager = self.sandbox_manager.lock().unwrap();
        sandbox_manager.create_sandbox(plugin)?;
        Ok(())
    }
    
    /// Create sandbox for a plugin with specific settings
    pub fn create_sandbox_with_settings(
        &self, 
        plugin: &dyn Plugin, 
        process_isolation: ProcessIsolation, 
        resource_limits: ResourceLimits
    ) -> PluginResult<()> {
        let mut sandbox_manager = self.sandbox_manager.lock().unwrap();
        let sandbox = sandbox_manager.create_sandbox(plugin)?;
        
        // Configure sandbox settings
        let mut sandbox = sandbox.lock().unwrap();
        sandbox.set_process_isolation(process_isolation);
        sandbox.set_resource_limits(resource_limits);
        
        Ok(())
    }
    
    /// Set global file system restrictions
    pub fn set_global_fs_restrictions(&self, restrictions: FsRestrictions) {
        let mut security = self.security.lock().unwrap();
        security.set_global_fs_restrictions(restrictions);
    }
    
    /// Set global network restrictions
    pub fn set_global_network_restrictions(&self, restrictions: NetworkRestrictions) {
        let mut security = self.security.lock().unwrap();
        security.set_global_network_restrictions(restrictions);
    }
    
    /// Get security manager for advanced configuration
    pub fn security_manager(&self) -> Arc<Mutex<SecurityManager>> {
        Arc::clone(&self.security)
    }
    
    /// Get sandbox manager for advanced configuration
    pub fn sandbox_manager(&self) -> Arc<Mutex<SandboxManager>> {
        Arc::clone(&self.sandbox_manager)
    }
    
    /// Start a plugin
    pub async fn start_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        self.lifecycle.start_plugin(plugin_id).await
    }
    
    /// Stop a plugin
    pub async fn stop_plugin(&self, plugin_id: Uuid) -> PluginResult<()> {
        self.lifecycle.stop_plugin(plugin_id).await
    }
    
    /// Get plugin status synchronously
    pub fn get_plugin_status_sync(&self, _plugin_id: Uuid) -> PluginResult<PluginStatus> {
        // This is a placeholder implementation
        // In a real implementation, we would need to handle this differently
        Err(PluginError::Other { reason: "Synchronous status check not implemented".to_string() })
    }
    
    /// Health check plugin synchronously
    pub fn health_check_plugin_sync(&self, _plugin_id: Uuid) -> PluginResult<HealthStatus> {
        // This is a placeholder implementation
        // In a real implementation, we would need to handle this differently
        Err(PluginError::Other { reason: "Synchronous health check not implemented".to_string() })
    }
    
    
    /// Get plugin instances by IDs
    pub async fn get_plugins_by_ids(
        &self,
        plugin_ids: &[Uuid],
    ) -> PluginResult<Vec<(Uuid, Arc<std::sync::Mutex<dyn Plugin>>)>> {
        let mut result = Vec::new();
        
        for &plugin_id in plugin_ids {
            if let Some(plugin) = self.get_plugin(plugin_id).await? {
                // Check if plugin is enabled
                if self.get_plugin_enabled(plugin_id)? {
                    result.push((plugin_id, plugin));
                }
            }
        }
        
        Ok(result)
    }
    
    /// Get all enabled plugins
    pub async fn get_all_enabled_plugins(&self) -> PluginResult<Vec<(Uuid, Arc<std::sync::Mutex<dyn Plugin>>)>> {
        let mut result = Vec::new();
        
        // Get all enabled plugins from database
        let enabled_states = self.state_manager.get_enabled_plugin_states()?;
        for state in enabled_states {
            if let Ok(uuid) = Uuid::parse_str(&state.id) {
                if let Some(plugin) = self.get_plugin(uuid).await? {
                    result.push((uuid, plugin));
                }
            }
        }
        
        Ok(result)
    }
    
    /// Check if a plugin supports a specific capability
    pub async fn plugin_supports_capability(&self, plugin_id: Uuid, capability: PluginCapability) -> PluginResult<bool> {
        if let Some(plugin_mutex) = self.registry.get_plugin(plugin_id).await? {
            let plugin_guard = plugin_mutex.lock().unwrap();
            let capabilities = plugin_guard.capabilities();
            Ok(capabilities.contains(&capability))
        } else {
            Ok(false)
        }
    }

    
    /// Get all enabled plugins that support a specific capability
    pub async fn get_enabled_plugins_by_capability(&self, capability: PluginCapability) -> PluginResult<Vec<(Uuid, Arc<std::sync::Mutex<dyn Plugin>>)>> {
        let all_plugins = self.registry.find_plugins_by_capability(capability).await?;
        let mut result = Vec::new();
        
        for plugin_mutex in all_plugins {
            // Get the plugin ID from the plugin instance
            let plugin_id = {
                let guard = plugin_mutex.lock().unwrap();
                guard.id()
            };
            
            if self.get_plugin_enabled(plugin_id)? {
                result.push((plugin_id, plugin_mutex));
            }
        }
        
        Ok(result)
    }
    
    /// Get plugin traits and capabilities combined
    pub async fn get_plugin_features(&self, plugin_id: Uuid) -> PluginResult<PluginFeatures> {
        let traits = self.registry.get_plugin_traits(plugin_id).await?;
        
        let capabilities = if let Some(plugin_mutex) = self.get_plugin(plugin_id).await? {
            let plugin_guard = plugin_mutex.lock().unwrap();
            plugin_guard.capabilities()
        } else {
            Vec::new()
        };
        
        Ok(PluginFeatures {
            traits,
            capabilities,
        })
    }


    /// Safely get plugin instance
    pub async fn get_plugin_instance_safe(
        &self,
        plugin_id: Uuid,
    ) -> PluginResult<Option<Arc<std::sync::Mutex<dyn Plugin>>>> {
        // Check if plugin exists
        let plugin = self.get_plugin(plugin_id).await?;
        
        // Check if plugin is enabled
        if let Some(plugin_mutex) = plugin {
            let enabled = self.get_plugin_enabled(plugin_id)?;
            if enabled {
                Ok(Some(plugin_mutex))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get audio plugin factory
    pub fn audio_factory(&self) -> Arc<Mutex<MediaPluginFactory>> {
        Arc::clone(&self.audio_factory)
    }
    
    /// Get audio providers by selection (for Tauri compatibility)
    pub async fn get_audio_providers_by_selection(
        &self,
        selection: &types::settings::music::MusicSourceSelection,
    ) -> PluginResult<Vec<(uuid::Uuid, std::sync::Arc<tokio::sync::Mutex<dyn music_plugin_sdk::traits::media::MediaPlugin + Send + Sync>>)>> {
        let factory = self.audio_factory.lock().unwrap();
        Ok(factory.get_media_plugins_by_selection(selection))
    }
}

/// Plugin features information
#[derive(Debug, Clone)]
pub struct PluginFeatures {
    /// Traits implemented by the plugin
    pub traits: Vec<crate::system::registry::PluginTrait>,
    /// Capabilities supported by the plugin
    pub capabilities: Vec<PluginCapability>,
}
