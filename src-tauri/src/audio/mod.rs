pub mod player_service;
pub mod player_commands;
pub mod player_events;
pub mod player_handle;

pub use player_service::{PlayerService, PlayerState};
pub use player_handle::PlayerHandle;
pub use player_commands::*;
pub use player_events::*;

use tauri::{AppHandle, Emitter};
use tracing::{debug, error};

/// 设置播放器事件处理
pub fn setup_player_events(app_handle: AppHandle, player_handle: PlayerHandle) {
    debug!("设置播放器事件处理");
    
    let mut event_receiver = player_handle.subscribe_events();
    let app_handle_clone = app_handle.clone();
    
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = event_receiver.recv().await {
            debug!("收到播放器事件: {:?}", event);
            
            // 将播放器事件转发到前端
            if let Err(e) = app_handle_clone.emit("player-event", &event) {
                error!("发送播放器事件到前端失败: {}", e);
            }
        }
    });
}