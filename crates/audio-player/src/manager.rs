use crate::{AudioError, AudioEvent, AudioEventListener, AudioPlayer, AudioResult, AudioStatus, PlaybackState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use types::songs::{Song, SongType};
use tracing::{debug, error, info, warn};

/// 音频播放管理器
/// 负责管理多个播放器实例，根据音源类型路由到正确的播放器
pub struct AudioManager {
    players: HashMap<String, Box<dyn AudioPlayer>>,
    current_player: Arc<Mutex<Option<String>>>,
    current_song: Arc<Mutex<Option<Song>>>,
    event_sender: broadcast::Sender<AudioEvent>,
    _event_receiver: broadcast::Receiver<AudioEvent>,
}

impl AudioManager {
    /// 创建新的音频管理器
    pub fn new() -> Self {
        let (event_sender, event_receiver) = broadcast::channel(100);
        
        Self {
            players: HashMap::new(),
            current_player: Arc::new(Mutex::new(None)),
            current_song: Arc::new(Mutex::new(None)),
            event_sender,
            _event_receiver: event_receiver,
        }
    }

    /// 注册播放器
    pub fn register_player(&mut self, name: String, player: Box<dyn AudioPlayer>) {
        info!("Registering audio player: {}", name);
        self.players.insert(name, player);
    }

    /// 获取事件发送器的克隆，用于监听音频事件
    pub fn get_event_sender(&self) -> broadcast::Sender<AudioEvent> {
        self.event_sender.clone()
    }

    /// 订阅音频事件
    pub fn subscribe_events(&self) -> broadcast::Receiver<AudioEvent> {
        self.event_sender.subscribe()
    }

    /// 发送音频事件
    fn emit_event(&self, event: AudioEvent) {
        if let Err(e) = self.event_sender.send(event) {
            warn!("Failed to send audio event: {}", e);
        }
    }

    /// 根据歌曲类型找到合适的播放器
    fn find_suitable_player(&self, song: &Song) -> AudioResult<String> {
        for (name, player) in &self.players {
            if player.supports_source(&song.song.type_) {
                debug!("Found suitable player '{}' for song type: {:?}", name, song.song.type_);
                return Ok(name.clone());
            }
        }

        Err(AudioError::InvalidSource {
            message: format!("No player supports song type: {:?}", song.song.type_),
        })
    }

    /// 获取当前活跃的播放器
    fn get_current_player(&mut self) -> AudioResult<&mut Box<dyn AudioPlayer>> {
        let current_player_name = self.current_player.lock().unwrap().clone()
            .ok_or_else(|| AudioError::PlaybackError("No active player".to_string()))?;

        self.players.get_mut(&current_player_name)
            .ok_or_else(|| AudioError::PlaybackError(format!("Player '{}' not found", current_player_name)))
    }

    /// 播放指定歌曲
    pub async fn play(&mut self, song: Song) -> AudioResult<()> {
        info!("AudioManager playing song: {:?}", song);

        // 找到合适的播放器
        let player_name = self.find_suitable_player(&song)?;

        // 如果当前播放器不同，先停止当前播放
        let current_player_name = self.current_player.lock().unwrap().clone();
        if let Some(current_name) = &current_player_name {
            if current_name != &player_name {
                if let Some(current_player) = self.players.get_mut(current_name) {
                    if let Err(e) = current_player.stop().await {
                        warn!("Failed to stop current player: {}", e);
                    }
                }
            }
        }

        // 获取目标播放器并播放
        let player = self.players.get_mut(&player_name)
            .ok_or_else(|| AudioError::PlaybackError(format!("Player '{}' not found", player_name)))?;

        match player.play(&song).await {
            Ok(()) => {
                // 更新当前播放器和歌曲
                *self.current_player.lock().unwrap() = Some(player_name.clone());
                *self.current_song.lock().unwrap() = Some(song.clone());

                // 发送事件
                self.emit_event(AudioEvent::SongChanged(Some(song)));
                self.emit_event(AudioEvent::StateChanged(PlaybackState::Playing));

                info!("Successfully started playing with player: {}", player_name);
                Ok(())
            }
            Err(e) => {
                error!("Failed to play song with player '{}': {}", player_name, e);
                self.emit_event(AudioEvent::Error(e.to_string()));
                Err(e)
            }
        }
    }

    /// 暂停播放
    pub async fn pause(&mut self) -> AudioResult<()> {
        debug!("AudioManager pausing playback");
        
        let player = self.get_current_player()?;
        match player.pause().await {
            Ok(()) => {
                self.emit_event(AudioEvent::StateChanged(PlaybackState::Paused));
                Ok(())
            }
            Err(e) => {
                self.emit_event(AudioEvent::Error(e.to_string()));
                Err(e)
            }
        }
    }

    /// 恢复播放
    pub async fn resume(&mut self) -> AudioResult<()> {
        debug!("AudioManager resuming playback");
        
        let player = self.get_current_player()?;
        match player.resume().await {
            Ok(()) => {
                self.emit_event(AudioEvent::StateChanged(PlaybackState::Playing));
                Ok(())
            }
            Err(e) => {
                self.emit_event(AudioEvent::Error(e.to_string()));
                Err(e)
            }
        }
    }

    /// 停止播放
    pub async fn stop(&mut self) -> AudioResult<()> {
        debug!("AudioManager stopping playback");
        
        let player = self.get_current_player()?;
        match player.stop().await {
            Ok(()) => {
                *self.current_song.lock().unwrap() = None;
                self.emit_event(AudioEvent::StateChanged(PlaybackState::Stopped));
                self.emit_event(AudioEvent::SongChanged(None));
                Ok(())
            }
            Err(e) => {
                self.emit_event(AudioEvent::Error(e.to_string()));
                Err(e)
            }
        }
    }

    /// 跳转到指定位置
    pub async fn seek(&mut self, position: Duration) -> AudioResult<()> {
        debug!("AudioManager seeking to: {:?}", position);
        
        let player = self.get_current_player()?;
        match player.seek(position).await {
            Ok(()) => {
                self.emit_event(AudioEvent::PositionChanged(position));
                Ok(())
            }
            Err(e) => {
                self.emit_event(AudioEvent::Error(e.to_string()));
                Err(e)
            }
        }
    }

    /// 设置音量
    pub async fn set_volume(&mut self, volume: f32) -> AudioResult<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        debug!("AudioManager setting volume to: {}", clamped_volume);
        
        let player = self.get_current_player()?;
        match player.set_volume(clamped_volume).await {
            Ok(()) => {
                self.emit_event(AudioEvent::VolumeChanged(clamped_volume));
                Ok(())
            }
            Err(e) => {
                self.emit_event(AudioEvent::Error(e.to_string()));
                Err(e)
            }
        }
    }

    /// 获取当前播放状态
    pub async fn get_status(&self) -> AudioResult<AudioStatus> {
        let current_player_name = self.current_player.lock().unwrap().clone();
        
        if let Some(player_name) = current_player_name {
            if let Some(player) = self.players.get(&player_name) {
                return player.get_status().await;
            }
        }

        // 如果没有活跃播放器，返回默认状态
        Ok(AudioStatus {
            state: PlaybackState::Stopped,
            position: None,
            volume: 1.0,
            quality: None,
            current_song: None,
        })
    }

    /// 获取当前播放位置
    pub async fn get_position(&self) -> AudioResult<Option<Duration>> {
        let current_player_name = self.current_player.lock().unwrap().clone();
        
        if let Some(player_name) = current_player_name {
            if let Some(player) = self.players.get(&player_name) {
                return player.get_position().await;
            }
        }

        Ok(None)
    }

    /// 获取当前歌曲
    pub fn get_current_song(&self) -> Option<Song> {
        self.current_song.lock().unwrap().clone()
    }

    /// 获取所有注册的播放器名称
    pub fn get_registered_players(&self) -> Vec<String> {
        self.players.keys().cloned().collect()
    }

    /// 检查是否支持指定的歌曲类型
    pub fn supports_song_type(&self, song_type: &SongType) -> bool {
        self.players.values().any(|player| player.supports_source(song_type))
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}

// ==================================================================
//                            事件监听器实现示例
// ==================================================================

/// 简单的控制台日志事件监听器
pub struct ConsoleEventListener;

impl AudioEventListener for ConsoleEventListener {
    fn on_event(&self, event: AudioEvent) {
        match event {
            AudioEvent::StateChanged(state) => {
                info!("🎵 Playback state changed: {:?}", state);
            }
            AudioEvent::PositionChanged(pos) => {
                debug!("⏱️ Position: {:?}", pos);
            }
            AudioEvent::VolumeChanged(vol) => {
                info!("🔊 Volume changed: {:.2}", vol);
            }
            AudioEvent::SongChanged(song) => {
                if let Some(song) = song {
                    info!("🎶 Now playing: {:?}", song);
                } else {
                    info!("⏹️ Playback stopped");
                }
            }
            AudioEvent::Error(err) => {
                error!("❌ Audio error: {}", err);
            }
            AudioEvent::BufferProgress(progress) => {
                debug!("📶 Buffer progress: {:.1}%", progress * 100.0);
            }
        }
    }
}