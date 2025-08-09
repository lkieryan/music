pub mod handler;

use tauri::{App, AppHandle, Emitter, Manager};

use settings::settings::SettingsConfig;
use serde_json::Value;
use types::providers::ProviderInstancePref;

// Initialize providers subsystem: create state, and bootstrap from settings
pub fn initialize_providers(app: &mut App) {
    let handler = handler::ProviderHandler::new(app.handle().clone());
    app.manage(handler.clone());
    let handle = app.handle().clone();
    bootstrap(handle);
}

pub fn bootstrap(handle: AppHandle) {
   tauri::async_runtime::spawn(async move {
       init_enabled_instances(&handle).await;
       if let Ok(statuses) = handle.state::<handler::ProviderHandler>().get_all_statuses().await {
           let _ = handle.emit("provider-status-update", statuses);
       }
   });
}

#[tracing::instrument(level = "debug", skip(handle))]
pub async fn init_enabled_instances(handle: &AppHandle) {
    let handler = handle.state::<handler::ProviderHandler>();
    // Build set of enabled keys from prefs
    let pref: tauri::State<'_, SettingsConfig> = handle.state();
    let instances = pref
        .load_selective::<Vec<ProviderInstancePref>>("providers.instances".into())
        .unwrap_or_else(|_| Vec::new());
    let enabled_keys: std::collections::HashSet<String> = instances
        .iter()
        .filter(|i| i.enabled)
        .map(|i| i.key.clone())
        .collect();

    // Remove any instance currently in registry but not enabled in prefs
    let existing = handler.list_keys().await;
    for k in existing {
        if !enabled_keys.contains(&k) {
            let removed = handler.remove_instance(&k).await;
            if removed {
                tracing::info!("Removed disabled/non-listed provider instance {}", k);
            }
        }
    }

    init_instances(handle, instances).await;
}


#[tracing::instrument(level = "debug", skip(handle, instances))]
pub async fn init_instances(handle: &AppHandle, instances: Vec<ProviderInstancePref>) {
    let handler = handle.state::<handler::ProviderHandler>();
    let pref: tauri::State<'_, SettingsConfig> = handle.state();

    for inst in instances.into_iter() {
        if !inst.enabled { continue; }
        // Merge cfg + secure
        let mut cfg = inst.cfg.unwrap_or_else(|| serde_json::json!({}));
        if let Some(sec_key) = inst.secure_ref.clone() {
            if let Ok(secret) = pref.get_secure::<Value>(sec_key) {
                cfg = merge_cfg(cfg, secret);
            }
        }
        if let Err(e) = handler
            .initialize(inst.kind.as_str().into(), Some(inst.key.clone()), Some(cfg))
            .await
        {
            tracing::error!("Failed to initialize provider instance {}: {:?}", inst.key, e);
        }
    }
}

fn merge_cfg(mut base: Value, secret: Value) -> Value {
    match (&mut base, secret) {
        (Value::Object(base_map), Value::Object(sec_map)) => {
            for (k, v) in sec_map { base_map.insert(k, v); }
            Value::Object(base_map.clone())
        }
        (base_val, sec_val) => {
            let mut obj = serde_json::Map::new();
            obj.insert("cfg".into(), base_val.clone());
            obj.insert("secret".into(), sec_val);
            Value::Object(obj)
        }
    }
}
