use crate::{
    AudioError, AudioPlayer, AudioResult, AudioStatus, PlaybackState, 
    StreamResolver, PositionTracker, AudioQuality, PlaybackPosition
};
use async_trait::async_trait;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::RwLock;
use types::songs::{Song, SongType};
use tracing::{debug, error, info, warn};

/// 流媒体缓冲状态
#[derive(Debug, Clone, PartialEq)]
pub enum BufferState {
    /// 未开始缓冲
    Empty,
    /// 正在缓冲，包含进度百分比 (0.0 - 1.0)
    Buffering(f32),
    /// 缓冲完成
    Ready,
    /// 缓冲失败
    Failed(String),
}

/// 流媒体播放器实现
/// 处理在线音源，如网易云、QQ音乐、YouTube等，支持缓冲管理
pub struct StreamingPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Option<Sink>>>,
    current_song: Arc<Mutex<Option<Song>>>,
    volume: Arc<Mutex<f32>>,
    http_client: reqwest::Client,
    resolvers: Vec<Box<dyn StreamResolver>>,
    position_tracker: PositionTracker,
    buffer_state: Arc<RwLock<BufferState>>,
    cached_audio_data: Arc<Mutex<Option<Vec<u8>>>>,
    current_stream_url: Arc<Mutex<Option<String>>>,
    retry_count: Arc<Mutex<u32>>,
    max_retries: u32,
}

impl StreamingPlayer {
    /// 创建新的流媒体播放器实例
    pub fn new() -> AudioResult<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::PlaybackError(format!("Failed to create audio stream: {}", e)))?;

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .map_err(|e| AudioError::NetworkError(e))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: Arc::new(Mutex::new(None)),
            current_song: Arc::new(Mutex::new(None)),
            volume: Arc::new(Mutex::new(1.0)),
            http_client,
            resolvers: Vec::new(),
            position_tracker: PositionTracker::new(),
            buffer_state: Arc::new(RwLock::new(BufferState::Empty)),
            cached_audio_data: Arc::new(Mutex::new(None)),
            current_stream_url: Arc::new(Mutex::new(None)),
            retry_count: Arc::new(Mutex::new(0)),
            max_retries: 3,
        })
    }

    /// 添加流媒体解析器
    pub fn add_resolver(&mut self, resolver: Box<dyn StreamResolver>) {
        self.resolvers.push(resolver);
    }

    /// 从URL获取音频流，支持重试和缓冲状态更新
    async fn fetch_audio_stream(&self, url: &str) -> AudioResult<Vec<u8>> {
        debug!("Fetching audio stream from: {}", url);
        
        // 设置缓冲状态为开始
        *self.buffer_state.write().await = BufferState::Buffering(0.0);
        *self.retry_count.lock().unwrap() = 0;
        
        let mut last_error = None;
        
        // 重试逻辑
        for attempt in 0..=self.max_retries {
            match self.fetch_with_progress(url).await {
                Ok(data) => {
                    *self.buffer_state.write().await = BufferState::Ready;
                    info!("Successfully fetched {} bytes from {}", data.len(), url);
                    return Ok(data);
                }
                Err(e) => {
                    warn!("Attempt {} failed for {}: {}", attempt + 1, url, e);
                    last_error = Some(e);
                    
                    if attempt < self.max_retries {
                        // 指数退避
                        let delay = Duration::from_millis(1000 * (2_u64.pow(attempt as u32)));
                        tokio::time::sleep(delay).await;
                        *self.retry_count.lock().unwrap() = attempt + 1;
                    }
                }
            }
        }
        
        let error_msg = format!("Failed to fetch audio after {} attempts", self.max_retries + 1);
        *self.buffer_state.write().await = BufferState::Failed(error_msg.clone());

        // If we have a last reqwest error, return it; otherwise return a descriptive playback error
        Err(last_error.unwrap_or_else(|| AudioError::PlaybackError("Request timeout".to_string())))
    }
    
    /// 带进度更新的下载方法
    async fn fetch_with_progress(&self, url: &str) -> AudioResult<Vec<u8>> {
        let response = self.http_client
            .get(url)
            .send()
            .await
            .map_err(|e| AudioError::NetworkError(e))?;

        if !response.status().is_success() {
            return Err(AudioError::PlaybackError(format!("HTTP {}", response.status().as_u16())));
        }
        
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded = 0u64;
        let mut buffer = Vec::new();
        
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| AudioError::NetworkError(e))?;
            buffer.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            
            // 更新缓冲进度
            if total_size > 0 {
                let progress = (downloaded as f32 / total_size as f32).min(1.0);
                *self.buffer_state.write().await = BufferState::Buffering(progress);
            }
        }
        
        Ok(buffer)
    }

    /// 解析歌曲获取流媒体URL
    async fn resolve_stream_url(&self, song: &Song) -> AudioResult<String> {
        // 如果歌曲已经有直接的播放URL
        if let Some(playback_url) = &song.song.playback_url {
            if !playback_url.is_empty() {
                return Ok(playback_url.clone());
            }
        }

        // 使用注册的解析器
        for resolver in &self.resolvers {
            if resolver.supports(&song.song.type_) {
                match resolver.resolve_stream_url(song).await {
                    Ok(url) => {
                        debug!("Resolved stream URL: {}", url);
                        return Ok(url);
                    }
                    Err(e) => {
                        warn!("Resolver failed: {}", e);
                        continue;
                    }
                }
            }
        }

        Err(AudioError::InvalidSource {
            message: format!("No resolver found for song type: {:?}", song.song.type_),
        })
    }

    /// 获取缓冲状态
    pub async fn get_buffer_state(&self) -> BufferState {
        self.buffer_state.read().await.clone()
    }
    
    /// 获取重试次数
    pub fn get_retry_count(&self) -> u32 {
        *self.retry_count.lock().unwrap()
    }
    
    /// 设置最大重试次数
    pub fn set_max_retries(&mut self, max_retries: u32) {
        self.max_retries = max_retries;
    }
    
    /// 清除缓存的音频数据
    pub fn clear_cache(&self) {
        *self.cached_audio_data.lock().unwrap() = None;
        *self.current_stream_url.lock().unwrap() = None;
    }
    
    /// 预加载音频数据（用于减少播放延迟）
    pub async fn preload(&mut self, song: &Song) -> AudioResult<()> {
        info!("Preloading streaming song: {:?}", song);
        
        if !self.supports_source(&song.song.type_) {
            return Err(AudioError::InvalidSource {
                message: format!("StreamingPlayer doesn't support: {:?}", song.song.type_),
            });
        }
        
        // 解析流媒体URL
        let stream_url = self.resolve_stream_url(song).await?;
        
        // 下载音频数据
        let audio_data = self.fetch_audio_stream(&stream_url).await?;
        
        // 缓存数据
        *self.cached_audio_data.lock().unwrap() = Some(audio_data);
        *self.current_stream_url.lock().unwrap() = Some(stream_url);
        
        info!("Successfully preloaded streaming audio");
        Ok(())
    }
}

#[async_trait(?Send)]
impl AudioPlayer for StreamingPlayer {
    async fn play(&mut self, song: &Song) -> AudioResult<()> {
        info!("Playing streaming song: {:?}", song);

        // 检查是否支持该音源类型
        if !self.supports_source(&song.song.type_) {
            return Err(AudioError::InvalidSource {
                message: format!("StreamingPlayer doesn't support: {:?}", song.song.type_),
            });
        }

        // 停止当前播放
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.take() {
                sink.stop();
            }
        }
        self.position_tracker.stop();

        // 检查是否有预缓存的数据
        let audio_data = {
            let cached_data = self.cached_audio_data.lock().unwrap();
            let cached_url = self.current_stream_url.lock().unwrap();
            
            if let (Some(data), Some(_url)) = (cached_data.as_ref(), cached_url.as_ref()) {
                // 使用缓存的数据
                info!("Using preloaded audio data");
                data.clone()
            } else {
                // 实时下载
                drop(cached_data);
                drop(cached_url);
                
                let stream_url = self.resolve_stream_url(song).await?;
                self.fetch_audio_stream(&stream_url).await?
            }
        };
        
        // 创建音频解码器
        let cursor = Cursor::new(audio_data);
        let decoder = Decoder::new(cursor)
            .map_err(|e| AudioError::UnsupportedFormat(format!("Failed to decode stream: {}", e)))?;

        // 创建新的 sink
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackError(format!("Failed to create sink: {}", e)))?;

        // 设置音量
        let volume = *self.volume.lock().unwrap();
        sink.set_volume(volume);

        // 添加音频源并开始播放
        sink.append(decoder);

        // 启动位置跟踪（流媒体时长通常未知，使用估算）
        self.position_tracker.start(None);

        // 保存 sink 和当前歌曲
        *self.sink.lock().unwrap() = Some(sink);
        *self.current_song.lock().unwrap() = Some(song.clone());

        info!("Successfully started streaming");
        Ok(())
    }

    async fn pause(&mut self) -> AudioResult<()> {
        debug!("Pausing streaming playback");
        
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                sink.pause();
                self.position_tracker.pause();
                info!("Streaming playback paused");
            } else {
                warn!("Cannot pause: no active streaming playback");
            }
        }
        Ok(())
    }

    async fn resume(&mut self) -> AudioResult<()> {
        debug!("Resuming streaming playback");
        
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                sink.play();
                self.position_tracker.resume();
                info!("Streaming playback resumed");
            } else {
                warn!("Cannot resume: no active streaming playback");
            }
        }
        Ok(())
    }

    async fn stop(&mut self) -> AudioResult<()> {
        debug!("Stopping streaming playback");
        
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.take() {
                sink.stop();
                info!("Streaming playback stopped");
            }
        }
        
        self.position_tracker.stop();
        *self.current_song.lock().unwrap() = None;
        self.clear_cache();
        Ok(())
    }

    async fn seek(&mut self, position: Duration) -> AudioResult<()> {
        debug!("Seeking streaming to position: {:?}", position);
        
        // 流媒体的 seek 更复杂，需要重新请求特定位置的数据
        // 这里是简化实现，实际项目中可能需要支持Range请求
        warn!("Seek for streaming audio requires re-downloading from position");
        
        // 更新位置跟踪器
        self.position_tracker.seek(position);
        
        // 简化实现：目前不支持流媒体的精确seek
        Err(AudioError::NotImplemented("Streaming seek operation requires server-side support".to_string()))
    }

    async fn set_volume(&mut self, volume: f32) -> AudioResult<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        debug!("Setting streaming volume to: {}", clamped_volume);
        
        *self.volume.lock().unwrap() = clamped_volume;
        
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                sink.set_volume(clamped_volume);
            }
        }
        
        Ok(())
    }

    async fn get_status(&self) -> AudioResult<AudioStatus> {
        let sink_guard = self.sink.lock().unwrap();
        let current_song = self.current_song.lock().unwrap().clone();
        let volume = *self.volume.lock().unwrap();

        let state = if let Some(sink) = sink_guard.as_ref() {
            if sink.is_paused() {
                PlaybackState::Paused
            } else if sink.empty() {
                PlaybackState::Stopped
            } else {
                // 检查缓冲状态
                match *self.buffer_state.try_read().unwrap_or_else(|_| {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(self.buffer_state.read())
                    })
                }) {
                    BufferState::Buffering(_) => PlaybackState::Buffering,
                    _ => PlaybackState::Playing,
                }
            }
        } else {
            PlaybackState::Stopped
        };

        // 从位置跟踪器获取位置信息
        let position = if state == PlaybackState::Playing || state == PlaybackState::Paused {
            let current = self.position_tracker.get_current_position();
            let duration = self.position_tracker.get_total_duration()
                .unwrap_or(Duration::from_secs(0));
            
            Some(PlaybackPosition {
                current,
                duration,
            })
        } else {
            None
        };

        Ok(AudioStatus {
            state,
            position,
            volume,
            quality: None, // 流媒体质量信息通常在播放时动态确定
            current_song,
        })
    }

    async fn get_position(&self) -> AudioResult<Option<Duration>> {
        if self.position_tracker.is_playing() || 
           matches!(self.sink.lock().unwrap().as_ref().map(|s| s.is_paused()), Some(true)) {
            Ok(Some(self.position_tracker.get_current_position()))
        } else {
            Ok(None)
        }
    }

    fn supports_source(&self, song_type: &SongType) -> bool {
        matches!(song_type, 
            SongType::URL | 
            SongType::DASH | 
            SongType::HLS
            // 可以添加其他支持的流媒体类型
        )
    }

    fn name(&self) -> &'static str {
        "StreamingPlayer"
    }
}

impl Default for StreamingPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create StreamingPlayer")
    }
}

// ==================================================================
//                            示例流媒体解析器
// ==================================================================

/// 通用URL解析器 - 直接使用提供的URL
pub struct DirectUrlResolver;

#[async_trait]
impl StreamResolver for DirectUrlResolver {
    async fn resolve_stream_url(&self, song: &Song) -> AudioResult<String> {
        if let Some(url) = &song.song.url {
            if !url.is_empty() {
                return Ok(url.clone());
            }
        }
        
        if let Some(playback_url) = &song.song.playback_url {
            if !playback_url.is_empty() {
                return Ok(playback_url.clone());
            }
        }

        Err(AudioError::InvalidSource {
            message: "No valid URL found in song".to_string(),
        })
    }

    fn supports(&self, song_type: &SongType) -> bool {
        matches!(song_type, SongType::URL)
    }
}