use std::sync::{Arc, Mutex};
use std::time::Duration;
use audio_player::{
    AudioManager, AudioStatus, PlayQueue, PlayMode, QueueItem, 
    LocalPlayer, StreamingPlayer, DirectUrlResolver, AudioResult, AudioError
};
use types::songs::Song;
use database::database::Database;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use super::player_events::{PlayerEvent, PlayerEventEmitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub current_song: Option<Song>,
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: f32,
    pub position: Option<Duration>,
    pub duration: Option<Duration>,
    pub play_mode: PlayMode,
    pub queue_length: usize,
    pub current_index: Option<usize>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current_song: None,
            is_playing: false,
            is_paused: false,
            volume: 1.0,
            position: None,
            duration: None,
            play_mode: PlayMode::Sequential,
            queue_length: 0,
            current_index: None,
        }
    }
}

pub struct PlayerService {
    audio_manager: Arc<Mutex<AudioManager>>,
    play_queue: Arc<Mutex<PlayQueue>>,
    database: Arc<Database>,
    event_emitter: PlayerEventEmitter,
    current_state: Arc<Mutex<PlayerState>>,
}

impl PlayerService {
    pub fn new(database: Arc<Database>) -> AudioResult<Self> {
        info!("初始化播放器服务");

        // 创建音频管理器
        let mut audio_manager = AudioManager::new();
        
        // 注册本地播放器
        let local_player = LocalPlayer::new()?;
        audio_manager.register_player("local".to_string(), Box::new(local_player));
        
        // 注册流媒体播放器
        let mut streaming_player = StreamingPlayer::new()?;
        streaming_player.add_resolver(Box::new(DirectUrlResolver));
        audio_manager.register_player("streaming".to_string(), Box::new(streaming_player));

        let audio_manager = Arc::new(Mutex::new(audio_manager));
        let play_queue = Arc::new(Mutex::new(PlayQueue::new()));
        let event_emitter = PlayerEventEmitter::new();
        let current_state = Arc::new(Mutex::new(PlayerState::default()));

        let service = Self {
            audio_manager,
            play_queue,
            database,
            event_emitter,
            current_state,
        };

        // 从数据库恢复播放器状态
        if let Err(e) = service.restore_player_state() {
            warn!("无法恢复播放器状态: {}", e);
        }

        info!("播放器服务初始化完成");
        Ok(service)
    }

    /// 播放指定歌曲
    pub async fn play_song(&self, song: Song) -> AudioResult<()> {
        info!("播放歌曲: {:?}", song.song.title);

        let mut audio_manager = self.audio_manager.lock().unwrap();
        
        match audio_manager.play(song.clone()).await {
            Ok(()) => {
                // 更新播放状态
                self.update_current_state(|state| {
                    state.current_song = Some(song.clone());
                    state.is_playing = true;
                    state.is_paused = false;
                });

                // 保存播放历史
                self.save_play_history(&song).await?;
                
                // 发送事件
                self.event_emitter.emit_song_changed(Some(song)).await;
                self.event_emitter.emit_playback_state_changed(true, false).await;

                Ok(())
            }
            Err(e) => {
                error!("播放失败: {}", e);
                self.event_emitter.emit_error(format!("播放失败: {}", e)).await;
                Err(e)
            }
        }
    }

    /// 暂停播放
    pub async fn pause(&self) -> AudioResult<()> {
        debug!("暂停播放");

        let mut audio_manager = self.audio_manager.lock().unwrap();
        
        match audio_manager.pause().await {
            Ok(()) => {
                self.update_current_state(|state| {
                    state.is_playing = false;
                    state.is_paused = true;
                });

                self.event_emitter.emit_playback_state_changed(false, true).await;
                Ok(())
            }
            Err(e) => {
                error!("暂停失败: {}", e);
                Err(e)
            }
        }
    }

    /// 恢复播放
    pub async fn resume(&self) -> AudioResult<()> {
        debug!("恢复播放");

        let mut audio_manager = self.audio_manager.lock().unwrap();
        
        match audio_manager.resume().await {
            Ok(()) => {
                self.update_current_state(|state| {
                    state.is_playing = true;
                    state.is_paused = false;
                });

                self.event_emitter.emit_playback_state_changed(true, false).await;
                Ok(())
            }
            Err(e) => {
                error!("恢复播放失败: {}", e);
                Err(e)
            }
        }
    }

    /// 停止播放
    pub async fn stop(&self) -> AudioResult<()> {
        debug!("停止播放");

        let mut audio_manager = self.audio_manager.lock().unwrap();
        
        match audio_manager.stop().await {
            Ok(()) => {
                self.update_current_state(|state| {
                    state.current_song = None;
                    state.is_playing = false;
                    state.is_paused = false;
                    state.position = None;
                    state.duration = None;
                });

                self.event_emitter.emit_song_changed(None).await;
                self.event_emitter.emit_playback_state_changed(false, false).await;
                Ok(())
            }
            Err(e) => {
                error!("停止播放失败: {}", e);
                Err(e)
            }
        }
    }

    /// 跳转到指定位置
    pub async fn seek(&self, position: Duration) -> AudioResult<()> {
        debug!("跳转到位置: {:?}", position);

        let mut audio_manager = self.audio_manager.lock().unwrap();
        
        match audio_manager.seek(position).await {
            Ok(()) => {
                self.update_current_state(|state| {
                    state.position = Some(position);
                });

                self.event_emitter.emit_position_changed(position).await;
                Ok(())
            }
            Err(e) => {
                error!("跳转失败: {}", e);
                Err(e)
            }
        }
    }

    /// 设置音量
    pub async fn set_volume(&self, volume: f32) -> AudioResult<()> {
        debug!("设置音量: {}", volume);

        let clamped_volume = volume.clamp(0.0, 1.0);
        let mut audio_manager = self.audio_manager.lock().unwrap();
        
        match audio_manager.set_volume(clamped_volume).await {
            Ok(()) => {
                self.update_current_state(|state| {
                    state.volume = clamped_volume;
                });

                self.event_emitter.emit_volume_changed(clamped_volume).await;
                self.save_player_setting("volume", &clamped_volume.to_string()).await?;
                Ok(())
            }
            Err(e) => {
                error!("设置音量失败: {}", e);
                Err(e)
            }
        }
    }

    /// 下一首
    pub async fn next(&self) -> AudioResult<()> {
        debug!("播放下一首");

        let next_song = {
            let play_queue = self.play_queue.lock().unwrap();
            play_queue.next()
        };

        if let Some(song) = next_song {
            self.play_song(song).await?;
        } else {
            warn!("队列中没有下一首歌曲");
        }

        Ok(())
    }

    /// 上一首
    pub async fn previous(&self) -> AudioResult<()> {
        debug!("播放上一首");

        let prev_song = {
            let play_queue = self.play_queue.lock().unwrap();
            play_queue.previous()
        };

        if let Some(song) = prev_song {
            self.play_song(song).await?;
        } else {
            warn!("队列中没有上一首歌曲");
        }

        Ok(())
    }

    /// 设置播放模式
    pub async fn set_play_mode(&self, mode: PlayMode) -> AudioResult<()> {
        debug!("设置播放模式: {:?}", mode);

        {
            let play_queue = self.play_queue.lock().unwrap();
            play_queue.set_play_mode(mode);
        }

        self.update_current_state(|state| {
            state.play_mode = mode;
        });

        self.save_player_setting("play_mode", &format!("{:?}", mode)).await?;
        self.event_emitter.emit_play_mode_changed(mode).await;

        Ok(())
    }

    /// 添加歌曲到队列
    pub async fn add_to_queue(&self, song: Song) -> AudioResult<String> {
        debug!("添加歌曲到队列: {:?}", song.song.title);

        let item_id = {
            let play_queue = self.play_queue.lock().unwrap();
            play_queue.add_song(song.clone())?
        };

        self.update_queue_info();
        self.save_queue_to_database().await?;
        self.event_emitter.emit_queue_changed().await;

        Ok(item_id)
    }

    /// 从队列移除歌曲
    pub async fn remove_from_queue(&self, index: usize) -> AudioResult<QueueItem> {
        debug!("从队列移除歌曲，索引: {}", index);

        let removed_item = {
            let play_queue = self.play_queue.lock().unwrap();
            play_queue.remove_song(index)?
        };

        self.update_queue_info();
        self.save_queue_to_database().await?;
        self.event_emitter.emit_queue_changed().await;

        Ok(removed_item)
    }

    /// 清空播放队列
    pub async fn clear_queue(&self) -> AudioResult<()> {
        debug!("清空播放队列");

        // 清空队列
        {
            let play_queue = self.play_queue.lock().unwrap();
            play_queue.clear();
        }

        // 停止播放（忽略错误，保证清理流程继续）
        {
            let mut audio_manager = self.audio_manager.lock().unwrap();
            if let Err(e) = audio_manager.stop().await {
                warn!("清空队列时停止播放失败: {}", e);
            }
        }

        // 重置状态
        self.update_current_state(|state| {
            state.current_song = None;
            state.is_playing = false;
            state.is_paused = false;
            state.position = None;
            state.duration = None;
            state.queue_length = 0;
            state.current_index = None;
        });

        // 持久化并发送事件
        self.save_queue_to_database().await?;
        self.event_emitter.emit_queue_changed().await;
        self.event_emitter.emit_song_changed(None).await;
        self.event_emitter.emit_playback_state_changed(false, false).await;

        Ok(())
    }

    /// 获取当前播放状态
    pub async fn get_status(&self) -> AudioResult<PlayerState> {
        let audio_status = {
            let audio_manager = self.audio_manager.lock().unwrap();
            audio_manager.get_status().await?
        };

        // 更新状态中的位置信息
        if let Some(position_info) = audio_status.position {
            self.update_current_state(|state| {
                state.position = Some(position_info.current);
                state.duration = Some(position_info.duration);
            });
        }

        let current_state = self.current_state.lock().unwrap().clone();
        Ok(current_state)
    }

    /// 获取播放队列
    pub fn get_queue(&self) -> Vec<QueueItem> {
        let play_queue = self.play_queue.lock().unwrap();
        play_queue.get_queue_snapshot()
    }

    /// 获取事件接收器
    pub fn subscribe_events(&self) -> broadcast::Receiver<PlayerEvent> {
        self.event_emitter.subscribe()
    }

    // 私有辅助方法

    fn update_current_state<F>(&self, updater: F) 
    where 
        F: FnOnce(&mut PlayerState)
    {
        let mut state = self.current_state.lock().unwrap();
        updater(&mut state);
    }

    fn update_queue_info(&self) {
        let play_queue = self.play_queue.lock().unwrap();
        let queue_length = play_queue.len();
        let current_index = play_queue.current_index();
        
        self.update_current_state(|state| {
            state.queue_length = queue_length;
            state.current_index = current_index;
        });
    }

    async fn save_play_history(&self, song: &Song) -> AudioResult<()> {
        // TODO: 实现保存播放历史到数据库
        debug!("保存播放历史: {:?}", song.song.title);
        Ok(())
    }

    async fn save_player_setting(&self, key: &str, value: &str) -> AudioResult<()> {
        // TODO: 实现保存播放器设置到数据库
        debug!("保存播放器设置: {} = {}", key, value);
        Ok(())
    }

    async fn save_queue_to_database(&self) -> AudioResult<()> {
        // TODO: 实现保存播放队列到数据库
        debug!("保存播放队列到数据库");
        Ok(())
    }

    fn restore_player_state(&self) -> AudioResult<()> {
        // TODO: 从数据库恢复播放器状态
        debug!("从数据库恢复播放器状态");
        Ok(())
    }
}