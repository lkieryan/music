// use std::sync::Arc;
use tauri::{State, Emitter};
use serde::Deserialize;
use types::errors::Result;

use crate::plugins::manager::PluginHandler;

// #[tracing::instrument(level = "debug", skip(plugin_handler))]
#[tauri::command]
pub async fn get_plugins(
    plugin_handler: State<'_, PluginHandler>,
) -> Result<Vec<crate::plugins::manager::PluginInfo>> {
    plugin_handler.get_plugins().await
}

// #[tracing::instrument(level = "debug", skip(plugin_handler))]
// Note: keep signatures simple and consistent with other Tauri commands.
// Accept both snake_case and camelCase keys from the frontend for robustness.

#[tauri::command]
pub async fn get_plugin(
    plugin_handler: State<'_, PluginHandler>,
    plugin_id: Option<String>,
    pluginId: Option<String>,
) -> Result<crate::plugins::manager::PluginInfo> {
    let pid = plugin_id.or(pluginId).ok_or("missing plugin_id")?;
    plugin_handler.get_plugin(pid).await
}

// #[tracing::instrument(level = "debug", skip(plugin_handler))]
#[tauri::command]
pub async fn enable_plugin(
    app: tauri::AppHandle,
    plugin_handler: State<'_, PluginHandler>,
    plugin_id: Option<String>,
    pluginId: Option<String>,
) -> Result<()> {
    let pid = plugin_id.or(pluginId).ok_or("missing plugin_id")?;
    let res = plugin_handler.enable_plugin(pid.clone()).await;
    if res.is_ok() {
        let _ = app.emit("plugins-updated", pid.clone());
    }
    res
}

// #[tracing::instrument(level = "debug", skip(plugin_handler))]
#[tauri::command]
pub async fn disable_plugin(
    app: tauri::AppHandle,
    plugin_handler: State<'_, PluginHandler>,
    plugin_id: Option<String>,
    pluginId: Option<String>,
) -> Result<()> {
    let pid = plugin_id.or(pluginId).ok_or("missing plugin_id")?;
    let res = plugin_handler.disable_plugin(pid.clone()).await;
    if res.is_ok() {
        let _ = app.emit("plugins-updated", pid.clone());
    }
    res
}

// #[tracing::instrument(level = "debug", skip(plugin_handler))]
#[tauri::command]
pub async fn start_plugin(
    app: tauri::AppHandle,
    plugin_handler: State<'_, PluginHandler>,
    plugin_id: Option<String>,
    pluginId: Option<String>,
) -> Result<()> {
    let pid = plugin_id.or(pluginId).ok_or("missing plugin_id")?;
    let res = plugin_handler.start_plugin(pid.clone()).await;
    if res.is_ok() { let _ = app.emit("plugins-updated", pid.clone()); }
    res
}

// #[tracing::instrument(level = "debug", skip(plugin_handler))]
#[tauri::command]
pub async fn stop_plugin(
    app: tauri::AppHandle,
    plugin_handler: State<'_, PluginHandler>,
    plugin_id: Option<String>,
    pluginId: Option<String>,
) -> Result<()> {
    let pid = plugin_id.or(pluginId).ok_or("missing plugin_id")?;
    let res = plugin_handler.stop_plugin(pid.clone()).await;
    if res.is_ok() { let _ = app.emit("plugins-updated", pid.clone()); }
    res
}

// #[tracing::instrument(level = "debug", skip(plugin_handler))]
#[tauri::command]
pub async fn load_plugin(
    app: tauri::AppHandle,
    plugin_handler: State<'_, PluginHandler>,
    plugin_path: Option<String>,
    pluginPath: Option<String>,
) -> Result<()> {
    let pp = plugin_path.or(pluginPath).ok_or("missing plugin_path")?;
    let res = plugin_handler.load_plugin(pp).await;
    if res.is_ok() { let _ = app.emit("plugins-updated", serde_json::Value::Null); }
    res
}
