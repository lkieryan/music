use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use types::songs::{Song, SongType};

// ==================================================================
//                            模块导出
// ==================================================================

pub mod local_player;
pub mod streaming_player;
pub mod manager;
pub mod position_tracker;
pub mod metadata;
pub mod queue;

// 可选模块
#[cfg(feature = "spotify")]
pub mod spotify_player;

// 重新导出主要类型和 trait
pub use local_player::LocalPlayer;
pub use streaming_player::{StreamingPlayer, DirectUrlResolver, BufferState};
pub use manager::{AudioManager, ConsoleEventListener};
pub use position_tracker::PositionTracker;
pub use metadata::{MetadataExtractor, AudioMetadata};
pub use queue::{PlayQueue, PlayMode, QueueItem};

// ==================================================================
//                            错误定义
// ==================================================================

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Playback error: {0}")]
    PlaybackError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Invalid source: {message}")]
    InvalidSource { message: String },
}

pub type AudioResult<T> = Result<T, AudioError>;

// ==================================================================
//                            音频播放状态
// ==================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQuality {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub codec: String,
    pub bitrate: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackPosition {
    pub current: Duration,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStatus {
    pub state: PlaybackState,
    pub position: Option<PlaybackPosition>,
    pub volume: f32,
    pub quality: Option<AudioQuality>,
    pub current_song: Option<Song>,
}

// ==================================================================
//                            音频播放器接口
// ==================================================================

/// 统一的音频播放器接口
/// 支持本地文件、流媒体URL和特殊平台（如Spotify）
#[async_trait(?Send)]
pub trait AudioPlayer {
    /// 播放指定歌曲
    async fn play(&mut self, song: &Song) -> AudioResult<()>;

    /// 暂停播放
    async fn pause(&mut self) -> AudioResult<()>;

    /// 恢复播放
    async fn resume(&mut self) -> AudioResult<()>;

    /// 停止播放
    async fn stop(&mut self) -> AudioResult<()>;

    /// 跳转到指定位置
    async fn seek(&mut self, position: Duration) -> AudioResult<()>;

    /// 设置音量 (0.0 - 1.0)
    async fn set_volume(&mut self, volume: f32) -> AudioResult<()>;

    /// 获取当前播放状态
    async fn get_status(&self) -> AudioResult<AudioStatus>;

    /// 获取当前播放位置
    async fn get_position(&self) -> AudioResult<Option<Duration>>;

    /// 检查是否支持指定的音源类型
    fn supports_source(&self, song_type: &SongType) -> bool;

    /// 获取播放器名称
    fn name(&self) -> &'static str;
}

// ==================================================================
//                            音频事件系统
// ==================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioEvent {
    /// 播放状态改变
    StateChanged(PlaybackState),
    
    /// 播放位置更新
    PositionChanged(Duration),
    
    /// 音量改变
    VolumeChanged(f32),
    
    /// 歌曲切换
    SongChanged(Option<Song>),
    
    /// 播放错误
    Error(String),
    
    /// 缓冲进度
    BufferProgress(f32),
}

/// 音频事件监听器
pub trait AudioEventListener: Send + Sync {
    fn on_event(&self, event: AudioEvent);
}

// ==================================================================
//                            流媒体解析器接口
// ==================================================================

/// 用于解析流媒体URL的接口
#[async_trait]
pub trait StreamResolver: Send + Sync {
    /// 解析歌曲获取可播放的URL
    async fn resolve_stream_url(&self, song: &Song) -> AudioResult<String>;
    
    /// 检查是否支持该歌曲类型
    fn supports(&self, song_type: &SongType) -> bool;
}

// ==================================================================
//                            音频解码器接口
// ==================================================================

/// 音频解码器接口，用于处理不同格式的音频
pub trait AudioDecoder: Send + Sync {
    /// 支持的音频格式
    fn supported_formats(&self) -> Vec<&'static str>;
    
    /// 检查是否支持指定的编解码器
    fn supports_codec(&self, codec: &str) -> bool;
}

// ==================================================================
//                            便捷函数
// ==================================================================

/// 创建一个包含所有默认播放器的音频管理器
pub fn create_default_audio_manager() -> AudioResult<AudioManager> {
    let mut manager = AudioManager::new();
    
    // 注册本地播放器
    let local_player = LocalPlayer::new()?;
    manager.register_player("local".to_string(), Box::new(local_player));
    
    // 注册流媒体播放器
    let mut streaming_player = StreamingPlayer::new()?;
    streaming_player.add_resolver(Box::new(DirectUrlResolver));
    manager.register_player("streaming".to_string(), Box::new(streaming_player));
    
    Ok(manager)
}