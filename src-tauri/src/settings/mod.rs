// use std::thread;

use macros::generate_command;
use ::settings::settings::SettingsConfig;
use serde_json::{json, Value};
use tauri::{async_runtime, App, AppHandle, Emitter, Manager, State};
use types::errors::error_helpers;
use std::io::Write;
use types::errors::Result;

use crate::{
    scanner::{start_scan, ScanTask},
};

const UI_KEYS: &[&str] = &[
    "prefs.queue_settings",
    "prefs.audio_settings",
    "prefs.gapless_skip",
    "prefs.volume_persist_mode",
    "prefs.spotify.enable",
    "prefs.spotify.username",
    "prefs.spotify.password",
    "prefs.themes.active_theme",
    "prefs.general.language",
    "prefs.general.minimize_to_tray",
    "prefs.general.launch_at_login",
    // music domain (platform selection, playback, effects)
    "prefs.music.source",
    "prefs.music.sources_order",
    "prefs.music.playback",
    "prefs.music.effects",
];

#[tracing::instrument(level = "debug", skip(app))]
pub fn handle_settings_changes(app: AppHandle) {
    async_runtime::spawn(async move {
        let pref_config: State<SettingsConfig> = app.state::<SettingsConfig>();
        let receiver = pref_config.get_receiver();
        for (key, value) in receiver {
            tracing::debug!("Received key: {} value: {}", key, value);
            if UI_KEYS.contains(&key.as_str()) {
                tracing::info!("Emitting settings-changed event");
                if let Err(e) = app.emit("settings-changed", (key.clone(), value.clone())) {
                    tracing::error!("Error emitting settings-changed event{}", e);
                } else {
                    tracing::info!("Emitted settings-changed event");
                }
            }

            // Mirror scan folders from prefs to flat scanner key (support both casing)
            if key == "prefs.general.scan_folders" || key == "prefs.general.scanFolders" {
                // scanner expects flat key `music_paths`
                if let Err(e) = pref_config.save_selective("music_paths".to_string(), Some(value.clone())) {
                    tracing::error!("Failed to mirror scan_folders to music_paths: {:?}", e);
                } else {
                    tracing::info!("Mirrored prefs.general.scan_folders -> music_paths");

                    let scan_task = app.state::<crate::scanner::ScanTask>();
                    if let Err(e) = scan_task.update_auto_scanner_config(&app) {
                        tracing::warn!("Failed to update AutoScanner config after path change: {:?}", e);
                    }

                    if let Err(e) = scan_task.trigger_auto_scan(None) {
                        tracing::warn!("Failed to trigger full scan after path change: {:?}", e);
                    } else {
                        tracing::info!("Triggered full scan after scan folder change");
                    }
                }
            }

            if key == "prefs.general.autoScanEnabled" {
                if let Some(enabled) = value.as_bool() {
                    if enabled {
                        tracing::info!("Auto scan enabled, starting AutoScanner");
                        app.state::<crate::scanner::ScanTask>().cancel_legacy_task();
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let scan_task = app_handle.state::<crate::scanner::ScanTask>();
                            if let Err(e) = scan_task.initialize_auto_scanner(&app_handle).await {
                                tracing::error!("Failed to start AutoScanner after enabling: {:?}", e);
                            }
                        });
                    } else {
                        tracing::info!("Auto scan disabled, stopping AutoScanner");
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let scan_task = app_handle.state::<crate::scanner::ScanTask>();
                            scan_task.stop_auto_scanner().await;
                        });
                    }
                }
            }

            if key == "prefs.general.scanMinDuration" {
                let _ = pref_config.save_selective("general.scan_min_duration".to_string(), Some(value.clone()));
                tracing::info!("Mirrored prefs.general.scanMinDuration -> general.scan_min_duration");
                let _ = app.state::<crate::scanner::ScanTask>().update_auto_scanner_config(&app);
            }
            if key == "prefs.general.scanFormats" {
                let _ = pref_config.save_selective("general.scan_formats".to_string(), Some(value.clone()));
                tracing::info!("Mirrored prefs.general.scanFormats -> general.scan_formats");
                let _ = app.state::<crate::scanner::ScanTask>().update_auto_scanner_config(&app);
            }

            // if key == "prefs.general.launch_at_login" { // unified key (bool)
            //     #[cfg(not(any(target_os = "android", target_os = "ios")))]
            //     {
            //         let manager: State<tauri_plugin_autostart::AutoLaunchManager> = app.state();

            //         let auto_start: Result<bool> = pref_config.load_selective("general.launch_at_login".into());
            //         tracing::info!("Setting autolaunch {:?}", auto_start);
            //         if let Ok(enabled) = auto_start {
            //             let res = if enabled { manager.enable() } else { manager.disable() };

            //             if let Err(e) = res {
            //                 tracing::error!("Error toggling autostart {:?}", e);
            //             }
            //         }
            //     }
            // }
        }
    });
}

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_settings_state(app: &mut App) -> Result<SettingsConfig> {
    let data_dir = app
        .path()
        .app_config_dir()
        .map_err(error_helpers::to_plugin_error)?;
    SettingsConfig::new(data_dir)
}

#[tracing::instrument(level = "debug", skip(app))]
pub fn initial(app: &mut App) {
    let pref_config: State<SettingsConfig> = app.state();
    if !pref_config.has_key("thumbnail_path") {
        let path = app.path().app_local_data_dir().unwrap().join("thumbnails");
        let _ = pref_config.save_selective("thumbnail_path".to_string(), Some(path));
    }

    if !pref_config.has_key("artwork_path") {
        let path = app.path().app_local_data_dir().unwrap().join("artwork");
        let _ = pref_config.save_selective("artwork_path".to_string(), Some(path));
    }

   if !pref_config.has_key("providers.instances") {
       use types::providers::{ProviderInstancePref, ProviderKind};
       let default_instances = vec![ProviderInstancePref {
           key: "spotify".to_string(),
           kind: ProviderKind::Spotify,
           enabled: true,
           cfg: None,
           secure_ref: None,
       }];
       let _ = pref_config.save_selective("providers.instances".to_string(), Some(default_instances));
   }


    // Single active UI language for renderer (BCP-47 like "en" | "zh-CN")
    if !pref_config.has_key("general.language") {
        let _ = pref_config.save_selective(
            "general.language".to_string(),
            Some("zh-CN".to_string()),
        );
    }

    // Mirror scanFolders/scan_folders -> music_paths at startup (so scanner can pick them)
    let startup_paths = pref_config
        .load_selective::<serde_json::Value>("general.scanFolders".into())
        .or_else(|_| pref_config.load_selective::<serde_json::Value>("general.scan_folders".into()));
    if let Ok(paths) = startup_paths {
        let _ = pref_config.save_selective("music_paths".to_string(), Some(paths));
        tracing::info!("Mirrored general.scanFolders -> music_paths at startup");
    }

    // Check if auto scan is enabled and start AutoScanner accordingly (camelCase first)
    let auto_scan_enabled = pref_config
        .load_selective::<bool>("general.autoScanEnabled".into())
        .or_else(|_| pref_config.load_selective::<bool>("general.auto_scan_enabled".into()))
        .unwrap_or(false);

    let scan_task: State<ScanTask> = app.state();
    
    if auto_scan_enabled {
        // Start AutoScanner if auto scan is enabled
        tracing::info!("Auto scan enabled at startup, initializing AutoScanner");
        let app_handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let scan_task = app_handle.state::<ScanTask>();
            if let Err(e) = scan_task.initialize_auto_scanner(&app_handle).await {
                tracing::error!("Failed to initialize AutoScanner at startup: {:?}", e);
            }
        });
    } else {
        // Fall back to legacy scanning if auto scan is disabled
        tracing::info!("Auto scan disabled, using legacy scan mode");
        
        // Spawn legacy scan task
        let scan_duration = pref_config.load_selective::<u64>("scan_interval".into());
        if let Ok(scan_duration) = scan_duration {
            scan_task.spawn_scan_task(app.handle().clone(), scan_duration.max(30));
        } else {
            tracing::warn!("Could not spawn scan task, no / invalid duration found");
        }

        // Run initial legacy scan
        let handle = app.handle().clone();
        tauri::async_runtime::spawn_blocking(move || {
            if let Err(e) = start_scan(handle, None) {
                tracing::error!("Failed to scan: {:?}", e);
            }
        });
    }
}

generate_command!(load_selective, SettingsConfig, Value, key: String);
generate_command!(save_selective, SettingsConfig, (), key: String, value: Option<Value>);
generate_command!(get_secure, SettingsConfig, Value, key: String);
generate_command!(set_secure, SettingsConfig, (), key: String, value: Option<Value>);
generate_command!(load_selective_array, SettingsConfig, Value, key: String);

#[tauri::command]
pub fn load_domain(config: State<'_, SettingsConfig>, domain: Option<String>) -> Result<Value> {
    let prefs_all = config.memcache.lock().unwrap().clone();
    let root = prefs_all.get("prefs").cloned().unwrap_or_else(|| json!({}));
    if let Some(dom) = domain {
        if dom.is_empty() { return Ok(root); }
        if let Some(v) = root.get(&dom) { return Ok(v.clone()); }
        Ok(json!({}))
    } else {
        Ok(root)
    }
}

#[tauri::command]
pub fn save_domain_partial(config: State<'_, SettingsConfig>, domain: Option<String>, patch: Value) -> Result<()> {
    if !patch.is_object() { return Err("patch must be an object".into()); }

    // Clone current prefs tree
    let mut all = { config.memcache.lock().unwrap().clone() };
    if !all.is_object() { all = json!({"prefs": {}}); }

    // Ensure prefs object exists
    let mut_root = all.as_object_mut().unwrap();
    let prefs_entry = mut_root.entry("prefs".to_string()).or_insert(json!({}));

    // Resolve target object: either prefs or prefs.<domain>
    let target_obj = if let Some(dom) = domain {
        if dom.is_empty() {
            prefs_entry
        } else {
            // Ensure nested domain object exists
            if !prefs_entry.is_object() { *prefs_entry = json!({}); }
            let obj = prefs_entry.as_object_mut().unwrap();
            obj.entry(dom).or_insert(json!({}))
        }
    } else {
        prefs_entry
    };

    // Merge patch into target object
    if !target_obj.is_object() { *target_obj = json!({}); }
    if let (Some(tobj), Some(pobj)) = (target_obj.as_object_mut(), patch.as_object()) {
        for (k, v) in pobj.iter() { tobj.insert(k.clone(), v.clone()); }
    }

    // Write back to memcache and file
    {
        let mut guard = config.memcache.lock().unwrap();
        *guard = all.clone();
    }
    let path = config.config_file.lock().unwrap().clone();
    let mut f = std::fs::File::create(path)?;
    f.write_all(&serde_json::to_vec(&all)?)?;
    f.flush()?;
    Ok(())
}
