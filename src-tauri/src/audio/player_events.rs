use audio_player::PlayMode;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;
use types::songs::Song;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PlayerEvent {
    /// 歌曲改变
    SongChanged { song: Option<Song> },
    /// 播放状态改变
    PlaybackStateChanged { 
        is_playing: bool, 
        is_paused: bool 
    },
    /// 播放位置更新
    PositionChanged { position: Duration },
    /// 音量改变
    VolumeChanged { volume: f32 },
    /// 播放模式改变
    PlayModeChanged { mode: PlayMode },
    /// 播放队列改变
    QueueChanged,
    /// 播放错误
    Error { message: String },
    /// 缓冲进度更新
    BufferProgress { progress: f32 },
}

pub struct PlayerEventEmitter {
    sender: broadcast::Sender<PlayerEvent>,
}

impl PlayerEventEmitter {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PlayerEvent> {
        self.sender.subscribe()
    }

    pub async fn emit_song_changed(&self, song: Option<Song>) {
        let event = PlayerEvent::SongChanged { song };
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit song changed event: {}", e);
        }
    }

    pub async fn emit_playback_state_changed(&self, is_playing: bool, is_paused: bool) {
        let event = PlayerEvent::PlaybackStateChanged { is_playing, is_paused };
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit playback state changed event: {}", e);
        }
    }

    pub async fn emit_position_changed(&self, position: Duration) {
        let event = PlayerEvent::PositionChanged { position };
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit position changed event: {}", e);
        }
    }

    pub async fn emit_volume_changed(&self, volume: f32) {
        let event = PlayerEvent::VolumeChanged { volume };
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit volume changed event: {}", e);
        }
    }

    pub async fn emit_play_mode_changed(&self, mode: PlayMode) {
        let event = PlayerEvent::PlayModeChanged { mode };
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit play mode changed event: {}", e);
        }
    }

    pub async fn emit_queue_changed(&self) {
        let event = PlayerEvent::QueueChanged;
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit queue changed event: {}", e);
        }
    }

    pub async fn emit_error(&self, message: String) {
        let event = PlayerEvent::Error { message };
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit error event: {}", e);
        }
    }

    pub async fn emit_buffer_progress(&self, progress: f32) {
        let event = PlayerEvent::BufferProgress { progress };
        if let Err(e) = self.sender.send(event) {
            tracing::warn!("Failed to emit buffer progress event: {}", e);
        }
    }
}