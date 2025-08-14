use crate::{AudioError, AudioResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use types::songs::Song;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// 播放模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PlayMode {
    /// 顺序播放
    Sequential,
    /// 单曲循环
    RepeatOne,
    /// 全部循环
    RepeatAll,
    /// 随机播放
    Shuffle,
}

impl Default for PlayMode {
    fn default() -> Self {
        Self::Sequential
    }
}

/// 播放队列项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
    pub id: String,
    pub song: Song,
    pub added_at: std::time::SystemTime,
    pub played_count: u32,
}

impl QueueItem {
    pub fn new(song: Song) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            song,
            added_at: std::time::SystemTime::now(),
            played_count: 0,
        }
    }
}

/// 播放队列管理器
/// 负责管理播放列表、播放模式、历史记录等
pub struct PlayQueue {
    /// 当前播放队列
    queue: Arc<Mutex<VecDeque<QueueItem>>>,
    /// 当前播放位置
    current_index: Arc<Mutex<Option<usize>>>,
    /// 播放模式
    play_mode: Arc<Mutex<PlayMode>>,
    /// 播放历史（最近播放的歌曲）
    history: Arc<Mutex<VecDeque<QueueItem>>>,
    /// 历史记录最大长度
    max_history_size: usize,
    /// 随机播放的历史索引
    shuffle_history: Arc<Mutex<Vec<usize>>>,
    /// 随机播放的当前位置
    shuffle_position: Arc<Mutex<usize>>,
}

impl PlayQueue {
    /// 创建新的播放队列
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            current_index: Arc::new(Mutex::new(None)),
            play_mode: Arc::new(Mutex::new(PlayMode::default())),
            history: Arc::new(Mutex::new(VecDeque::new())),
            max_history_size: 100,
            shuffle_history: Arc::new(Mutex::new(Vec::new())),
            shuffle_position: Arc::new(Mutex::new(0)),
        }
    }

    /// 添加歌曲到队列末尾
    pub fn add_song(&self, song: Song) -> AudioResult<String> {
        let item = QueueItem::new(song);
        let item_id = item.id.clone();
        
        self.queue.lock().unwrap().push_back(item);
        
        // 如果是第一首歌曲，设置为当前播放
        if self.current_index.lock().unwrap().is_none() {
            *self.current_index.lock().unwrap() = Some(0);
        }
        
        info!("Added song to queue: {}", item_id);
        Ok(item_id)
    }

    /// 添加多首歌曲到队列
    pub fn add_songs(&self, songs: Vec<Song>) -> AudioResult<Vec<String>> {
        let mut ids = Vec::new();
        
        for song in songs {
            let id = self.add_song(song)?;
            ids.push(id);
        }
        
        Ok(ids)
    }

    /// 在指定位置插入歌曲
    pub fn insert_song(&self, index: usize, song: Song) -> AudioResult<String> {
        let item = QueueItem::new(song);
        let item_id = item.id.clone();
        
        let mut queue = self.queue.lock().unwrap();
        if index > queue.len() {
            return Err(AudioError::InvalidSource {
                message: format!("Index {} out of bounds for queue size {}", index, queue.len()),
            });
        }
        
        queue.insert(index, item);
        
        // 调整当前播放索引
        let mut current_index = self.current_index.lock().unwrap();
        if let Some(current) = *current_index {
            if index <= current {
                *current_index = Some(current + 1);
            }
        } else if index == 0 {
            *current_index = Some(0);
        }
        
        info!("Inserted song at index {}: {}", index, item_id);
        Ok(item_id)
    }

    /// 从队列中移除歌曲
    pub fn remove_song(&self, index: usize) -> AudioResult<QueueItem> {
        let mut queue = self.queue.lock().unwrap();
        
        if index >= queue.len() {
            return Err(AudioError::InvalidSource {
                message: format!("Index {} out of bounds for queue size {}", index, queue.len()),
            });
        }
        
        let removed_item = queue.remove(index).unwrap();
        
        // 调整当前播放索引
        let mut current_index = self.current_index.lock().unwrap();
        if let Some(current) = *current_index {
            if index == current {
                // 移除的是当前播放的歌曲
                if queue.is_empty() {
                    *current_index = None;
                } else if current >= queue.len() {
                    *current_index = Some(queue.len().saturating_sub(1));
                }
            } else if index < current {
                *current_index = Some(current - 1);
            }
        }
        
        info!("Removed song at index {}: {}", index, removed_item.id);
        Ok(removed_item)
    }

    /// 清空队列
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
        *self.current_index.lock().unwrap() = None;
        *self.shuffle_position.lock().unwrap() = 0;
        self.shuffle_history.lock().unwrap().clear();
        
        info!("Cleared play queue");
    }

    /// 获取当前播放的歌曲
    pub fn current_song(&self) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        let current_index_opt = *self.current_index.lock().unwrap();
        let current_index = match current_index_opt {
            Some(i) => i,
            None => return None,
        };

        queue.get(current_index).map(|item| item.song.clone())
    }

    /// 获取当前播放的队列项
    pub fn current_item(&self) -> Option<QueueItem> {
        let queue = self.queue.lock().unwrap();
        let current_index_opt = *self.current_index.lock().unwrap();
        let current_index = match current_index_opt {
            Some(i) => i,
            None => return None,
        };

        queue.get(current_index).cloned()
    }

    /// 跳到下一首歌曲
    pub fn next(&self) -> Option<Song> {
        let play_mode = *self.play_mode.lock().unwrap();
        
        match play_mode {
            PlayMode::Sequential => self.next_sequential(),
            PlayMode::RepeatOne => self.current_song(), // 单曲循环返回当前歌曲
            PlayMode::RepeatAll => self.next_repeat_all(),
            PlayMode::Shuffle => self.next_shuffle(),
        }
    }

    /// 跳到上一首歌曲
    pub fn previous(&self) -> Option<Song> {
        let play_mode = *self.play_mode.lock().unwrap();
        
        match play_mode {
            PlayMode::Sequential => self.previous_sequential(),
            PlayMode::RepeatOne => self.current_song(), // 单曲循环返回当前歌曲
            PlayMode::RepeatAll => self.previous_repeat_all(),
            PlayMode::Shuffle => self.previous_shuffle(),
        }
    }

    /// 跳转到指定索引的歌曲
    pub fn jump_to(&self, index: usize) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        
        if index >= queue.len() {
            warn!("Cannot jump to index {} (queue size: {})", index, queue.len());
            return None;
        }
        
        *self.current_index.lock().unwrap() = Some(index);
        let song = queue.get(index).map(|item| item.song.clone());
        
        if let Some(ref s) = song {
            info!("Jumped to song at index {}: {:?}", index, s);
        }
        
        song
    }

    /// 设置播放模式
    pub fn set_play_mode(&self, mode: PlayMode) {
        *self.play_mode.lock().unwrap() = mode;
        
        // 如果切换到随机模式，重新生成随机历史
        if mode == PlayMode::Shuffle {
            self.regenerate_shuffle_history();
        }
        
        info!("Set play mode to: {:?}", mode);
    }

    /// 获取当前播放模式
    pub fn get_play_mode(&self) -> PlayMode {
        *self.play_mode.lock().unwrap()
    }

    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// 获取当前播放索引
    pub fn current_index(&self) -> Option<usize> {
        *self.current_index.lock().unwrap()
    }

    /// 获取队列的快照
    pub fn get_queue_snapshot(&self) -> Vec<QueueItem> {
        self.queue.lock().unwrap().iter().cloned().collect()
    }

    /// 移动歌曲到新位置
    pub fn move_song(&self, from_index: usize, to_index: usize) -> AudioResult<()> {
        let mut queue = self.queue.lock().unwrap();
        
        if from_index >= queue.len() || to_index >= queue.len() {
            return Err(AudioError::InvalidSource {
                message: "Index out of bounds".to_string(),
            });
        }
        
        let item = queue.remove(from_index).unwrap();
        queue.insert(to_index, item);
        
        // 调整当前播放索引
        let mut current_index = self.current_index.lock().unwrap();
        if let Some(current) = *current_index {
            if from_index == current {
                *current_index = Some(to_index);
            } else if from_index < current && to_index >= current {
                *current_index = Some(current - 1);
            } else if from_index > current && to_index <= current {
                *current_index = Some(current + 1);
            }
        }
        
        info!("Moved song from index {} to {}", from_index, to_index);
        Ok(())
    }

    /// 添加到播放历史
    pub fn add_to_history(&self, item: QueueItem) {
        let mut history = self.history.lock().unwrap();
        
        // 避免重复添加相同的歌曲
        if let Some(last) = history.back() {
            if last.song.song._id == item.song.song._id {
                return;
            }
        }
        
        history.push_back(item);
        
        // 限制历史记录大小
        while history.len() > self.max_history_size {
            history.pop_front();
        }
    }

    /// 获取播放历史
    pub fn get_history(&self) -> Vec<QueueItem> {
        self.history.lock().unwrap().iter().cloned().collect()
    }

    // 私有辅助方法

    fn next_sequential(&self) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        let mut current_index = self.current_index.lock().unwrap();
        
        if let Some(current) = *current_index {
            if current + 1 < queue.len() {
                *current_index = Some(current + 1);
                return queue.get(current + 1).map(|item| item.song.clone());
            }
        }
        
        None
    }

    fn next_repeat_all(&self) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        let mut current_index = self.current_index.lock().unwrap();
        
        if queue.is_empty() {
            return None;
        }
        
        if let Some(current) = *current_index {
            let next = (current + 1) % queue.len();
            *current_index = Some(next);
            return queue.get(next).map(|item| item.song.clone());
        }
        
        None
    }

    fn previous_sequential(&self) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        let mut current_index = self.current_index.lock().unwrap();
        
        if let Some(current) = *current_index {
            if current > 0 {
                *current_index = Some(current - 1);
                return queue.get(current - 1).map(|item| item.song.clone());
            }
        }
        
        None
    }

    fn previous_repeat_all(&self) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        let mut current_index = self.current_index.lock().unwrap();
        
        if queue.is_empty() {
            return None;
        }
        
        if let Some(current) = *current_index {
            let prev = if current == 0 { queue.len() - 1 } else { current - 1 };
            *current_index = Some(prev);
            return queue.get(prev).map(|item| item.song.clone());
        }
        
        None
    }

    fn next_shuffle(&self) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        let mut shuffle_position = self.shuffle_position.lock().unwrap();
        let shuffle_history = self.shuffle_history.lock().unwrap();
        
        if shuffle_history.is_empty() {
            return None;
        }
        
        if *shuffle_position + 1 < shuffle_history.len() {
            *shuffle_position += 1;
        } else {
            // 重新生成随机序列
            drop(shuffle_history);
            drop(shuffle_position);
            self.regenerate_shuffle_history();
            
            let shuffle_position = self.shuffle_position.lock().unwrap();
            let shuffle_history = self.shuffle_history.lock().unwrap();
            if shuffle_history.is_empty() {
                return None;
            }
        }
        
        let shuffle_position = *self.shuffle_position.lock().unwrap();
        let shuffle_history = self.shuffle_history.lock().unwrap();
        
        if let Some(&index) = shuffle_history.get(shuffle_position) {
            *self.current_index.lock().unwrap() = Some(index);
            return queue.get(index).map(|item| item.song.clone());
        }
        
        None
    }

    fn previous_shuffle(&self) -> Option<Song> {
        let queue = self.queue.lock().unwrap();
        let mut shuffle_position = self.shuffle_position.lock().unwrap();
        let shuffle_history = self.shuffle_history.lock().unwrap();
        
        if shuffle_history.is_empty() || *shuffle_position == 0 {
            return None;
        }
        
        *shuffle_position -= 1;
        
        if let Some(&index) = shuffle_history.get(*shuffle_position) {
            *self.current_index.lock().unwrap() = Some(index);
            return queue.get(index).map(|item| item.song.clone());
        }
        
        None
    }

    fn regenerate_shuffle_history(&self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        
        let queue = self.queue.lock().unwrap();
        let mut indices: Vec<usize> = (0..queue.len()).collect();
        indices.shuffle(&mut thread_rng());
        
        *self.shuffle_history.lock().unwrap() = indices;
        *self.shuffle_position.lock().unwrap() = 0;
        
        debug!("Regenerated shuffle history for {} songs", queue.len());
    }
}

impl Default for PlayQueue {
    fn default() -> Self {
        Self::new()
    }
}