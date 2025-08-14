use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::debug;

/// 播放位置跟踪器
/// 用于跟踪音频播放的当前位置和总时长
#[derive(Debug, Clone)]
pub struct PositionTracker {
    start_time: Arc<Mutex<Option<Instant>>>,
    total_duration: Arc<Mutex<Option<Duration>>>,
    seek_offset: Arc<Mutex<Duration>>,
    is_playing: Arc<Mutex<bool>>,
    elapsed_when_paused: Arc<Mutex<Duration>>,
}

impl PositionTracker {
    /// 创建新的位置跟踪器
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(Mutex::new(None)),
            total_duration: Arc::new(Mutex::new(None)),
            seek_offset: Arc::new(Mutex::new(Duration::ZERO)),
            is_playing: Arc::new(Mutex::new(false)),
            elapsed_when_paused: Arc::new(Mutex::new(Duration::ZERO)),
        }
    }

    /// 开始播放并记录开始时间
    pub fn start(&self, total_duration: Option<Duration>) {
        let mut start_time = self.start_time.lock().unwrap();
        let mut is_playing = self.is_playing.lock().unwrap();
        let mut duration = self.total_duration.lock().unwrap();
        let mut elapsed = self.elapsed_when_paused.lock().unwrap();

        *start_time = Some(Instant::now());
        *is_playing = true;
        *duration = total_duration;
        *elapsed = Duration::ZERO;

        debug!("Position tracker started with duration: {:?}", total_duration);
    }

    /// 暂停播放
    pub fn pause(&self) {
        let mut is_playing = self.is_playing.lock().unwrap();
        let mut elapsed = self.elapsed_when_paused.lock().unwrap();

        if *is_playing {
            *elapsed = self.get_current_position();
            *is_playing = false;
            debug!("Position tracker paused at: {:?}", *elapsed);
        }
    }

    /// 恢复播放
    pub fn resume(&self) {
        let mut start_time = self.start_time.lock().unwrap();
        let mut is_playing = self.is_playing.lock().unwrap();

        if !*is_playing {
            *start_time = Some(Instant::now());
            *is_playing = true;
            debug!("Position tracker resumed");
        }
    }

    /// 停止播放
    pub fn stop(&self) {
        let mut start_time = self.start_time.lock().unwrap();
        let mut is_playing = self.is_playing.lock().unwrap();
        let mut seek_offset = self.seek_offset.lock().unwrap();
        let mut elapsed = self.elapsed_when_paused.lock().unwrap();

        *start_time = None;
        *is_playing = false;
        *seek_offset = Duration::ZERO;
        *elapsed = Duration::ZERO;

        debug!("Position tracker stopped");
    }

    /// 跳转到指定位置
    pub fn seek(&self, position: Duration) {
        let mut seek_offset = self.seek_offset.lock().unwrap();
        let mut start_time = self.start_time.lock().unwrap();
        let mut elapsed = self.elapsed_when_paused.lock().unwrap();
        let is_playing = self.is_playing.lock().unwrap();

        *seek_offset = position;
        *elapsed = position;

        if *is_playing {
            *start_time = Some(Instant::now());
        }

        debug!("Position tracker seeked to: {:?}", position);
    }

    /// 获取当前播放位置
    pub fn get_current_position(&self) -> Duration {
        let start_time_guard = self.start_time.lock().unwrap();
        let seek_offset = self.seek_offset.lock().unwrap();
        let is_playing = self.is_playing.lock().unwrap();
        let elapsed_when_paused = self.elapsed_when_paused.lock().unwrap();

        if !*is_playing {
            return *elapsed_when_paused;
        }

        if let Some(start_time) = *start_time_guard {
            let elapsed = start_time.elapsed();
            *seek_offset + elapsed
        } else {
            *seek_offset
        }
    }

    /// 获取总时长
    pub fn get_total_duration(&self) -> Option<Duration> {
        *self.total_duration.lock().unwrap()
    }

    /// 设置总时长
    pub fn set_total_duration(&self, duration: Duration) {
        let mut total_duration = self.total_duration.lock().unwrap();
        *total_duration = Some(duration);
        debug!("Position tracker total duration set to: {:?}", duration);
    }

    /// 检查是否正在播放
    pub fn is_playing(&self) -> bool {
        *self.is_playing.lock().unwrap()
    }

    /// 获取播放进度 (0.0 - 1.0)
    pub fn get_progress(&self) -> f32 {
        let current = self.get_current_position();
        if let Some(total) = self.get_total_duration() {
            if total > Duration::ZERO {
                (current.as_secs_f32() / total.as_secs_f32()).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// 根据进度设置位置 (0.0 - 1.0)
    pub fn seek_to_progress(&self, progress: f32) -> Option<Duration> {
        if let Some(total) = self.get_total_duration() {
            let clamped_progress = progress.clamp(0.0, 1.0);
            let target_position = Duration::from_secs_f32(total.as_secs_f32() * clamped_progress);
            self.seek(target_position);
            Some(target_position)
        } else {
            None
        }
    }
}

impl Default for PositionTracker {
    fn default() -> Self {
        Self::new()
    }
}