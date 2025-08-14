use crate::audio::{PlayerHandle, PlayerState};
use audio_player::{PlayMode, QueueItem};
use std::time::Duration;
use types::songs::Song;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaySongRequest {
    pub song: Song,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeekRequest {
    pub position_seconds: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeRequest {
    pub volume: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayModeRequest {
    pub mode: PlayMode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddToQueueRequest {
    pub song: Song,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveFromQueueRequest {
    pub index: usize,
}

// ============================================================================
//                              播放控制命令
// ============================================================================

#[tauri::command]
pub async fn play_song(
    request: PlaySongRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到播放歌曲命令: {:?}", request.song.song.title);
    
    player_service.play_song(request.song).await
}

#[tauri::command]
pub async fn pause_playback(
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到暂停播放命令");
    
    player_service.pause().await
}

#[tauri::command]
pub async fn resume_playback(
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到恢复播放命令");
    
    player_service.resume().await
}

#[tauri::command]
pub async fn stop_playback(
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到停止播放命令");
    
    player_service.stop().await
}

#[tauri::command]
pub async fn seek_to_position(
    request: SeekRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到跳转命令: {} 秒", request.position_seconds);
    
    let position = Duration::from_secs_f64(request.position_seconds);
    player_service.seek(position).await
}

#[tauri::command]
pub async fn set_volume(
    request: VolumeRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到设置音量命令: {}", request.volume);
    
    player_service.set_volume(request.volume).await
}

#[tauri::command]
pub async fn next_track(
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到播放下一首命令");
    
    player_service.next().await
}

#[tauri::command]
pub async fn previous_track(
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到播放上一首命令");
    
    player_service.previous().await
}

// ============================================================================
//                              播放模式和队列命令
// ============================================================================

#[tauri::command]
pub async fn set_play_mode(
    request: PlayModeRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到设置播放模式命令: {:?}", request.mode);
    
    player_service.set_play_mode(request.mode).await
}

#[tauri::command]
pub async fn add_to_queue(
    request: AddToQueueRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<String, String> {
    tracing::info!("收到添加到队列命令: {:?}", request.song.song.title);
    
    player_service.add_to_queue(request.song).await
}

#[tauri::command]
pub async fn remove_from_queue(
    request: RemoveFromQueueRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<QueueItem, String> {
    tracing::info!("收到从队列移除命令: 索引 {}", request.index);
    
    player_service.remove_from_queue(request.index).await
}

#[tauri::command]
pub async fn get_queue(
    player_service: State<'_, PlayerHandle>,
) -> Result<Vec<QueueItem>, String> {
    tracing::info!("收到获取播放队列命令");
    
    player_service.get_queue().await
}

// ============================================================================
//                              状态查询命令
// ============================================================================

#[tauri::command]
pub async fn get_player_status(
    player_service: State<'_, PlayerHandle>,
) -> Result<PlayerState, String> {
    player_service.get_status().await
}

// ============================================================================
//                              实用工具命令
// ============================================================================

#[tauri::command]
pub async fn toggle_playback(
    player_service: State<'_, PlayerHandle>,
) -> Result<bool, String> {
    tracing::info!("收到切换播放/暂停命令");
    
    let status = player_service.get_status().await?;
    
    if status.is_playing {
        player_service.pause().await?;
        Ok(false)
    } else {
        player_service.resume().await?;
        Ok(true)
    }
}

#[tauri::command]
pub async fn get_current_song(
    player_service: State<'_, PlayerHandle>,
) -> Result<Option<Song>, String> {
    player_service.get_current_song().await
}

#[tauri::command]
pub async fn clear_queue(
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到清空队列命令");
    player_service.clear_queue().await
}

// ============================================================================
//                              批量操作命令
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AddSongsToQueueRequest {
    pub songs: Vec<Song>,
}

#[tauri::command]
pub async fn add_songs_to_queue(
    request: AddSongsToQueueRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<Vec<String>, String> {
    tracing::info!("收到批量添加到队列命令: {} 首歌曲", request.songs.len());
    
    let mut ids = Vec::new();
    
    for song in request.songs {
        let id = player_service
            .add_to_queue(song)
            .await
            .map_err(|e| format!("添加歌曲到队列失败: {}", e))?;
        ids.push(id);
    }
    
    Ok(ids)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayPlaylistRequest {
    pub songs: Vec<Song>,
    pub start_index: Option<usize>,
}

#[tauri::command]
pub async fn play_playlist(
    request: PlayPlaylistRequest,
    player_service: State<'_, PlayerHandle>,
) -> Result<(), String> {
    tracing::info!("收到播放播放列表命令: {} 首歌曲", request.songs.len());
    
    if request.songs.is_empty() {
        return Err("播放列表为空".to_string());
    }
    
    // 先清空当前队列
    // TODO: 实现清空队列
    
    // 添加所有歌曲到队列
    for song in &request.songs {
        player_service
            .add_to_queue(song.clone())
            .await
            .map_err(|e| format!("添加歌曲到队列失败: {}", e))?;
    }
    
    // 播放指定索引的歌曲，如果未指定则播放第一首
    let start_index = request.start_index.unwrap_or(0);
    if let Some(song) = request.songs.get(start_index) {
        player_service
            .play_song(song.clone())
            .await
            .map_err(|e| format!("播放失败: {}", e))?;
    }
    
    Ok(())
}