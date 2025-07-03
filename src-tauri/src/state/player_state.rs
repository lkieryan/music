use crate::sources::UnifiedSong;
use std::time::Duration;

/// 播放器状态
#[derive(Debug, Clone)]
pub struct PlayerState {
    pub is_playing: bool,
    pub current_song: Option<UnifiedSong>,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,
    pub playlist: Vec<UnifiedSong>,
    pub current_index: usize,
}

/// 重复播放模式
#[derive(Debug, Clone, PartialEq)]
pub enum RepeatMode {
    None,    // 不重复
    One,     // 单曲循环
    All,     // 列表循环
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            current_song: None,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            volume: 1.0,
            shuffle: false,
            repeat_mode: RepeatMode::None,
            playlist: Vec::new(),
            current_index: 0,
        }
    }

    pub fn set_current_song(&mut self, song: UnifiedSong) {
        self.current_song = Some(song);
        self.position = Duration::ZERO;
    }

    pub fn set_playlist(&mut self, playlist: Vec<UnifiedSong>, start_index: usize) {
        self.playlist = playlist;
        self.current_index = start_index.min(self.playlist.len().saturating_sub(1));
        
        if let Some(song) = self.playlist.get(self.current_index) {
            self.current_song = Some(song.clone());
        }
    }

    pub fn next_song(&mut self) -> Option<&UnifiedSong> {
        if self.playlist.is_empty() {
            return None;
        }

        match self.repeat_mode {
            RepeatMode::One => {
                // 单曲循环，不改变索引
                self.playlist.get(self.current_index)
            }
            RepeatMode::All => {
                // 列表循环
                self.current_index = (self.current_index + 1) % self.playlist.len();
                self.playlist.get(self.current_index)
            }
            RepeatMode::None => {
                // 不重复，到最后一首就停止
                if self.current_index < self.playlist.len() - 1 {
                    self.current_index += 1;
                    self.playlist.get(self.current_index)
                } else {
                    None
                }
            }
        }
    }

    pub fn previous_song(&mut self) -> Option<&UnifiedSong> {
        if self.playlist.is_empty() {
            return None;
        }

        if self.current_index > 0 {
            self.current_index -= 1;
        } else if self.repeat_mode == RepeatMode::All {
            // 列表循环模式下，从最后一首开始
            self.current_index = self.playlist.len() - 1;
        }

        self.playlist.get(self.current_index)
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        // TODO: 实现播放列表随机化逻辑
    }

    pub fn set_repeat_mode(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
    }

    pub fn get_current_song(&self) -> Option<&UnifiedSong> {
        self.current_song.as_ref()
    }
}