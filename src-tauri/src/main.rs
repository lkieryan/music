#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod audio;
mod sources;
mod database;
mod commands;
mod utils;
mod state;

use tauri::{Manager, SystemTray, SystemTrayMenu, SystemTrayMenuItem, CustomMenuItem};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    log::info!("Starting MoeKoe Music Rust...");
    
    // 创建系统托盘
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("play_pause".to_string(), "播放/暂停"))
        .add_item(CustomMenuItem::new("previous".to_string(), "上一首"))
        .add_item(CustomMenuItem::new("next".to_string(), "下一首"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("show".to_string(), "显示主界面"))
        .add_item(CustomMenuItem::new("quit".to_string(), "退出"));
    
    let system_tray = SystemTray::new().with_menu(tray_menu);
    
    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            use tauri::SystemTrayEvent;
            
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "play_pause" => {
                            let _ = app.emit_all("tray_play_pause", ());
                        }
                        "previous" => {
                            let _ = app.emit_all("tray_previous", ());
                        }
                        "next" => {
                            let _ = app.emit_all("tray_next", ());
                        }
                        "show" => {
                            if let Some(window) = app.get_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }
                SystemTrayEvent::LeftClick { .. } => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::play_song,
            commands::pause_playback,
            commands::resume_playback,
            commands::stop_playback,
            commands::previous_song,
            commands::next_song,
            commands::set_volume,
            commands::seek_to_position,
            commands::get_playback_status,
        ])
        .setup(|app| {
            // 初始化数据库和应用状态
            tauri::async_runtime::spawn(async move {
                match database::init_database().await {
                    Ok(db_pool) => {
                        let app_state = state::AppState::new(db_pool);
                        // TODO: 将 app_state 设置到 Tauri 的状态管理中
                        log::info!("Database and state initialized successfully");
                    }
                    Err(e) => {
                        log::error!("Failed to initialize database: {}", e);
                    }
                }
            });
            
            log::info!("Application setup completed");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
        
    Ok(())
}