use crate::sources::{UnifiedSong, LocalSong};
use std::collections::HashMap;

/// 音乐库状态
#[derive(Debug)]
pub struct LibraryState {
    pub local_songs: Vec<LocalSong>,
    pub is_scanning: bool,
    pub scan_progress: f32,
    pub last_scan_time: Option<chrono::DateTime<chrono::Utc>>,
    pub favorites: Vec<UnifiedSong>,
    pub recent_played: Vec<UnifiedSong>,
    pub playlists: HashMap<String, Playlist>,
}

/// 播放列表
#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub songs: Vec<UnifiedSong>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl LibraryState {
    pub fn new() -> Self {
        Self {
            local_songs: Vec::new(),
            is_scanning: false,
            scan_progress: 0.0,
            last_scan_time: None,
            favorites: Vec::new(),
            recent_played: Vec::new(),
            playlists: HashMap::new(),
        }
    }

    pub fn update_local_songs(&mut self, songs: Vec<LocalSong>) {
        self.local_songs = songs;
        self.last_scan_time = Some(chrono::Utc::now());
    }

    pub fn start_scanning(&mut self) {
        self.is_scanning = true;
        self.scan_progress = 0.0;
    }

    pub fn update_scan_progress(&mut self, progress: f32) {
        self.scan_progress = progress.clamp(0.0, 1.0);
    }

    pub fn finish_scanning(&mut self) {
        self.is_scanning = false;
        self.scan_progress = 1.0;
    }

    pub fn add_to_favorites(&mut self, song: UnifiedSong) {
        if !self.favorites.iter().any(|s| s.id == song.id) {
            self.favorites.push(song);
        }
    }

    pub fn remove_from_favorites(&mut self, song_id: &str) {
        self.favorites.retain(|s| s.id != song_id);
    }

    pub fn is_favorite(&self, song_id: &str) -> bool {
        self.favorites.iter().any(|s| s.id == song_id)
    }

    pub fn add_to_recent(&mut self, song: UnifiedSong) {
        // 移除重复项
        self.recent_played.retain(|s| s.id != song.id);
        
        // 添加到开头
        self.recent_played.insert(0, song);
        
        // 限制最近播放数量
        if self.recent_played.len() > 50 {
            self.recent_played.truncate(50);
        }
    }

    pub fn create_playlist(&mut self, name: String, description: Option<String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        
        let playlist = Playlist {
            id: id.clone(),
            name,
            description,
            songs: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        
        self.playlists.insert(id.clone(), playlist);
        id
    }

    pub fn add_to_playlist(&mut self, playlist_id: &str, song: UnifiedSong) -> bool {
        if let Some(playlist) = self.playlists.get_mut(playlist_id) {
            if !playlist.songs.iter().any(|s| s.id == song.id) {
                playlist.songs.push(song);
                playlist.updated_at = chrono::Utc::now();
                return true;
            }
        }
        false
    }

    pub fn remove_from_playlist(&mut self, playlist_id: &str, song_id: &str) -> bool {
        if let Some(playlist) = self.playlists.get_mut(playlist_id) {
            let original_len = playlist.songs.len();
            playlist.songs.retain(|s| s.id != song_id);
            
            if playlist.songs.len() != original_len {
                playlist.updated_at = chrono::Utc::now();
                return true;
            }
        }
        false
    }

    pub fn get_playlist(&self, playlist_id: &str) -> Option<&Playlist> {
        self.playlists.get(playlist_id)
    }

    pub fn delete_playlist(&mut self, playlist_id: &str) -> bool {
        self.playlists.remove(playlist_id).is_some()
    }
}