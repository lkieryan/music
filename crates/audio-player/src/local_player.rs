use crate::{
    AudioError, AudioPlayer, AudioResult, AudioStatus, PlaybackState, 
    AudioQuality, PlaybackPosition, PositionTracker, MetadataExtractor
};
use async_trait::async_trait;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::Duration;
// (no symphonia imports needed here)
use types::songs::{Song, SongType};
use tracing::{debug, info, warn};

/// 本地文件播放器实现
/// 使用 rodio 播放本地音频文件，支持位置跟踪和 seek 功能
pub struct LocalPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Option<Sink>>>,
    current_song: Arc<Mutex<Option<Song>>>,
    volume: Arc<Mutex<f32>>,
    position_tracker: PositionTracker,
    current_file_path: Arc<Mutex<Option<String>>>,
}

impl LocalPlayer {
    /// 创建新的本地播放器实例
    pub fn new() -> AudioResult<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::PlaybackError(format!("Failed to create audio stream: {}", e)))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: Arc::new(Mutex::new(None)),
            current_song: Arc::new(Mutex::new(None)),
            volume: Arc::new(Mutex::new(1.0)),
            position_tracker: PositionTracker::new(),
            current_file_path: Arc::new(Mutex::new(None)),
        })
    }

    /// 从文件路径加载音频
    fn load_audio_file(&self, path: &str) -> AudioResult<Decoder<BufReader<File>>> {
        debug!("Loading audio file: {}", path);
        
        let file = File::open(path)
            .map_err(|e| AudioError::IoError(e))?;
        
        let buf_reader = BufReader::new(file);
        let decoder = Decoder::new(buf_reader)
            .map_err(|e| AudioError::UnsupportedFormat(format!("Failed to decode audio: {}", e)))?;
        
        Ok(decoder)
    }

    /// 从指定位置加载音频（用于 seek）
    fn load_audio_file_from_position(
        &self,
        path: &str,
        seek_position: Duration,
    ) -> AudioResult<rodio::source::SkipDuration<Decoder<BufReader<File>>>> {
        debug!("Loading audio file from position: {} at {:?}", path, seek_position);
        
        // 对于简单的 seek，我们重新创建 decoder
        // 实际项目中可能需要更复杂的实现来支持精确 seek
        let decoder = self.load_audio_file(path)?;
        
        // 通过时间偏移来实现粗略的 seek（rodio 层面，非精确）
        let seeked_decoder = decoder.skip_duration(seek_position);
        Ok(seeked_decoder)
    }

    /// 获取音频文件的元数据
    fn get_audio_metadata(&self, path: &str) -> Option<crate::metadata::AudioMetadata> {
        match MetadataExtractor::extract_from_file(path) {
            Ok(metadata) => Some(metadata),
            Err(e) => {
                warn!("Failed to extract metadata from {}: {}", path, e);
                None
            }
        }
    }

    /// 从解码器获取音频质量信息
    fn get_audio_quality_from_decoder(&self, decoder: &Decoder<BufReader<File>>) -> AudioQuality {
        AudioQuality {
            sample_rate: decoder.sample_rate(),
            channels: decoder.channels(),
            bits_per_sample: 16, // rodio 默认输出 16-bit
            codec: "pcm".to_string(), // rodio 输出 PCM
            bitrate: Some((decoder.sample_rate() * decoder.channels() as u32 * 16) / 1000),
        }
    }

    /// 启动位置更新任务
    fn start_position_update_task(&self) {
        // 这里可以启动一个后台任务来定期更新位置
        // 由于 rodio 的限制，这个实现相对简单
    }
}

#[async_trait(?Send)]
impl AudioPlayer for LocalPlayer {
    async fn play(&mut self, song: &Song) -> AudioResult<()> {
        info!("Playing local song: {:?}", song);

        // 检查是否是本地文件
        if song.song.type_ != SongType::LOCAL {
            return Err(AudioError::InvalidSource {
                message: format!("LocalPlayer only supports LOCAL files, got: {:?}", song.song.type_),
            });
        }

        let path = song.song.path.as_ref()
            .ok_or_else(|| AudioError::InvalidSource {
                message: "Local song missing file path".to_string(),
            })?;

        // 验证文件格式
        if !MetadataExtractor::is_supported_extension(path) {
            return Err(AudioError::UnsupportedFormat(
                format!("Unsupported file format: {}", path)
            ));
        }

        // 停止当前播放
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.take() {
                sink.stop();
            }
        }
        self.position_tracker.stop();

        // 获取音频元数据
        let metadata = self.get_audio_metadata(path);
        let duration = metadata.as_ref().and_then(|m| m.duration);

        // 加载新音频文件
        let decoder = self.load_audio_file(path)?;
        
        // 创建新的 sink
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackError(format!("Failed to create sink: {}", e)))?;

        // 设置音量
        let volume = *self.volume.lock().unwrap();
        sink.set_volume(volume);

        // 添加音频源并开始播放
        sink.append(decoder);

        // 启动位置跟踪
        self.position_tracker.start(duration);

        // 保存状态
        *self.sink.lock().unwrap() = Some(sink);
        *self.current_song.lock().unwrap() = Some(song.clone());
        *self.current_file_path.lock().unwrap() = Some(path.to_string());

        info!("Successfully started playing: {} (duration: {:?})", path, duration);
        Ok(())
    }

    async fn pause(&mut self) -> AudioResult<()> {
        debug!("Pausing playback");
        
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                sink.pause();
                self.position_tracker.pause();
                info!("Playback paused");
            } else {
                warn!("Cannot pause: no active playback");
            }
        }
        Ok(())
    }

    async fn resume(&mut self) -> AudioResult<()> {
        debug!("Resuming playback");
        
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                sink.play();
                self.position_tracker.resume();
                info!("Playback resumed");
            } else {
                warn!("Cannot resume: no active playback");
            }
        }
        Ok(())
    }

    async fn stop(&mut self) -> AudioResult<()> {
        debug!("Stopping playback");
        
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.take() {
                sink.stop();
                info!("Playback stopped");
            }
        }
        
        self.position_tracker.stop();
        *self.current_song.lock().unwrap() = None;
        *self.current_file_path.lock().unwrap() = None;
        Ok(())
    }

    async fn seek(&mut self, position: Duration) -> AudioResult<()> {
        debug!("Seeking to position: {:?}", position);
        
        let file_path = {
            let path_guard = self.current_file_path.lock().unwrap();
            path_guard.clone().ok_or_else(|| {
                AudioError::PlaybackError("No file currently loaded".to_string())
            })?
        };

        let was_playing = {
            let sink_guard = self.sink.lock().unwrap();
            sink_guard.as_ref().map_or(false, |sink| !sink.is_paused())
        };

        // 停止当前播放
        if let Ok(mut sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.take() {
                sink.stop();
            }
        }

        // 从指定位置重新加载文件
        let decoder = self.load_audio_file_from_position(&file_path, position)?;
        
        // 创建新的 sink
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::PlaybackError(format!("Failed to create sink for seek: {}", e)))?;

        // 设置音量
        let volume = *self.volume.lock().unwrap();
        sink.set_volume(volume);

        // 添加音频源
        sink.append(decoder);

        // 如果之前在播放，继续播放；否则暂停
        if !was_playing {
            sink.pause();
        }

        // 更新位置跟踪
        self.position_tracker.seek(position);
        
        // 保存新的 sink
        *self.sink.lock().unwrap() = Some(sink);

        info!("Successfully seeked to: {:?}", position);
        Ok(())
    }

    async fn set_volume(&mut self, volume: f32) -> AudioResult<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        debug!("Setting volume to: {}", clamped_volume);
        
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
                PlaybackState::Playing
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

        // 获取音质信息
        let quality = if let Some(file_path) = self.current_file_path.lock().unwrap().as_ref() {
            self.get_audio_metadata(file_path)
                .map(|metadata| metadata.quality)
        } else {
            None
        };

        Ok(AudioStatus {
            state,
            position,
            volume,
            quality,
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
        matches!(song_type, SongType::LOCAL)
    }

    fn name(&self) -> &'static str {
        "LocalPlayer"
    }
}

impl LocalPlayer {
    /// 获取当前播放进度 (0.0 - 1.0)
    pub fn get_progress(&self) -> f32 {
        self.position_tracker.get_progress()
    }

    /// 根据进度跳转 (0.0 - 1.0)
    pub async fn seek_to_progress(&mut self, progress: f32) -> AudioResult<()> {
        if let Some(target_position) = self.position_tracker.seek_to_progress(progress) {
            self.seek(target_position).await
        } else {
            Err(AudioError::PlaybackError("No duration information available for seek".to_string()))
        }
    }

    /// 获取当前音频文件的元数据
    pub fn get_current_metadata(&self) -> Option<crate::metadata::AudioMetadata> {
        let file_path = self.current_file_path.lock().unwrap();
        file_path.as_ref().and_then(|path| self.get_audio_metadata(path))
    }

    /// 检查当前播放的文件是否仍然存在
    pub fn is_current_file_available(&self) -> bool {
        if let Some(path) = self.current_file_path.lock().unwrap().as_ref() {
            std::path::Path::new(path).exists()
        } else {
            false
        }
    }
}

impl Default for LocalPlayer {
    fn default() -> Self {
        Self::new().expect("Failed to create LocalPlayer")
    }
}