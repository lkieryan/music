//! Plugin management module for Tauri backend
//!
//! This module provides the Tauri commands for plugin management
//! and acts as a bridge between the frontend and the plugin system.

use tauri::async_runtime;
use tauri::AppHandle;
use tauri::Manager;
use tauri::State;

pub mod handler;
pub mod manager;

// Re-export the handler functions for easier access
pub use handler::*;


// pub fn get_plugin_state(app: AppHandle) -> Result<PluginHandler> {

//     let plugin_manager = Arc::new(PluginManager::new(app.state::<Database>().inner().clone(), plugins_root));

//     let plugin_handler = plugins::manager::PluginHandler::new(plugin_manager.clone());

// }