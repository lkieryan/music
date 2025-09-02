use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Serialize, Deserialize};
use serde_json;
use std::{cmp::min, collections::HashMap, sync::Arc};
use types::{
    tracks::MediaContent,
    ui::player_details::{PlayerState, PlayerMode, VolumeMode},
    errors::Result,
};
use database::database::Database;

// No-op UI bridge hooks for backend-only usage
// These can be wired by the integrator if needed
fn set_position(_pos: f64) { /* noop */ }
fn set_playback_state(_state: PlayerState) { /* noop */ }

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub track_queue: Vec<String>,
    pub current_index: usize,
    pub data: HashMap<String, MediaContent>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PlayerDetails {
    pub current_time: f64,
    pub last_track: Option<String>,
    pub last_track_played_duration: f64,
    pub force_seek: f64,
    pub state: PlayerState,
    pub has_repeated: bool,
    #[serde(skip)]
    pub repeat: PlayerMode,
    old_volume: f64,
    volume: f64,
    #[serde(skip)]
    volume_mode: VolumeMode,
    volume_map: HashMap<String, f64>,
    clamp_map: HashMap<String, f64>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PlayerStoreData {
    pub queue: Queue,
    pub current_track: Option<MediaContent>,
    pub player_details: PlayerDetails,
    pub player_blacklist: Vec<String>,
    pub force_load_track: bool,
    // Shuffle bag for random playback: contains shuffled indices of the queue
    #[serde(skip)]
    pub shuffle_bag: Vec<usize>,
    #[serde(skip)]
    pub shuffle_index: usize,
}

#[derive(Debug)]
pub struct PlayerStore {
    pub data: PlayerStoreData,
    scrobble_time: f64,
    scrobbled: bool,
    is_mobile: bool,
    db: Option<Arc<Database>>,
}

impl PlayerStore {
    #[tracing::instrument(level = "debug")]
    pub fn new(db: Option<Arc<Database>>) -> Self {
        let mut player_store = Self {
            data: PlayerStoreData::default(),
            scrobble_time: 0f64,
            scrobbled: false,
            is_mobile: false, // Default to false for backend usage
            db,
        };

        // 自动从数据库加载状态
        if let Err(e) = player_store.load_from_db() {
            tracing::warn!("Failed to load player store from database: {:?}", e);
        }

        tracing::debug!("Created player store {:?}", player_store);
        player_store
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn load_from_db(&mut self) -> Result<()> {
        if let Some(db) = &self.db {
            let keys = vec!["player_state", "track_queue", "current_index", "queue_data"];
            let values = db.get_player_store_values(keys)?;
            
            if let Some(player_state_str) = values.get("player_state") {
                if let Ok(player_details) = serde_json::from_str::<PlayerDetails>(player_state_str) {
                    self.data.player_details = player_details;
                    // Reset current_time on load
                    self.data.player_details.current_time = 0f64;
                }
            }
            
            if let Some(track_queue_str) = values.get("track_queue") {
                if let Ok(track_queue) = serde_json::from_str::<Vec<String>>(track_queue_str) {
                    self.data.queue.track_queue = track_queue;
                }
            }
            
            if let Some(current_index_str) = values.get("current_index") {
                if let Ok(current_index) = serde_json::from_str::<usize>(current_index_str) {
                    self.data.queue.current_index = current_index;
                }
            }
            
            if let Some(queue_data_str) = values.get("queue_data") {
                if let Ok(queue_data) = serde_json::from_str::<HashMap<String, MediaContent>>(queue_data_str) {
                    self.data.queue.data = queue_data;
                }
            }
            
            // Update current track based on loaded data
            if let Some(track_id) = self.data.queue.track_queue.get(self.data.queue.current_index) {
                self.data.current_track = self.data.queue.data.get(track_id).cloned();
            }
            
            tracing::debug!("Loaded player store from database");
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn save_to_db(&self, keys: &[&str]) -> Result<()> {
        if let Some(db) = &self.db {
            let mut values = Vec::new();
            
            for &key in keys {
                match key {
                    "player_state" => {
                        let json = serde_json::to_string(&self.data.player_details)
                            .map_err(|e| types::errors::MusicError::String(format!("Failed to serialize player_details: {}", e)))?;
                        values.push(("player_state", json));
                    },
                    "track_queue" => {
                        let json = serde_json::to_string(&self.data.queue.track_queue)
                            .map_err(|e| types::errors::MusicError::String(format!("Failed to serialize track_queue: {}", e)))?;
                        values.push(("track_queue", json));
                    },
                    "current_index" => {
                        let json = serde_json::to_string(&self.data.queue.current_index)
                            .map_err(|e| types::errors::MusicError::String(format!("Failed to serialize current_index: {}", e)))?;
                        values.push(("current_index", json));
                    },
                    "queue_data" => {
                        let json = serde_json::to_string(&self.data.queue.data)
                            .map_err(|e| types::errors::MusicError::String(format!("Failed to serialize queue_data: {}", e)))?;
                        values.push(("queue_data", json));
                    },
                    _ => continue,
                }
            }
            
            let values_refs: Vec<(&str, &str)> = values.iter()
                .map(|(k, v)| (*k, v.as_str()))
                .collect();
            
            db.set_player_store_values(values_refs)?;
            tracing::debug!("Saved player store to database for keys: {:?}", keys);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_current_track(&self) -> Option<MediaContent> {
        self.data.current_track.clone()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_queue(&self) -> Queue {
        self.data.queue.clone()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_player_state(&self) -> PlayerState {
        self.data.player_details.state
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_queue_len(&self) -> usize {
        self.data.queue.track_queue.len()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_queue_index(&self) -> usize {
        self.data.queue.current_index
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_force_load(&self) -> bool {
        self.data.force_load_track
    }

    #[tracing::instrument(level = "debug", skip(self, has_repeated))]
    pub fn set_has_repeated(&mut self, has_repeated: bool) {
        self.data.player_details.has_repeated = has_repeated;
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_has_repeated(&self) -> bool {
        self.data.player_details.has_repeated
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_repeat(&self) -> PlayerMode {
        self.data.player_details.repeat
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_force_seek(&self) -> f64 {
        self.data.player_details.force_seek
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_current_time(&self) -> f64 {
        self.data.player_details.current_time
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_player_blacklist(&self) -> Vec<String> {
        self.data.player_blacklist.clone()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_current_track(&mut self, force: bool) {
        self.data.player_details.last_track_played_duration = self.data.player_details.current_time;
        self.data.player_details.last_track =
            self.get_current_track().map(|s| s.track._id.unwrap().clone());

        // TODO: Record play statistics for the last track if it played for more than 30 seconds
        self.data.player_details.current_time = 0f64;
        set_position(self.data.player_details.current_time);

        if self.data.queue.current_index >= self.data.queue.track_queue.len() {
            self.data.queue.current_index = 0;
        }
        let id = self
            .data
            .queue
            .track_queue
            .get(self.data.queue.current_index)
            .cloned()
            .unwrap_or_default();

        let track = self.data.queue.data.get(&id).cloned();

        if !force && track == self.data.current_track && self.data.player_blacklist.is_empty() {
            return;
        }

        tracing::debug!("Updating track in queue");
        self.data.current_track = track.clone();
        if self.data.current_track.is_none() {
            self.data.player_details.current_time = 0f64;
        }

        // Set metadata for new track (handled via callbacks in AudioPlayer)
        if let Some(ref current_track) = self.data.current_track {
            // title is Option<String>; clone and unwrap to avoid Display issue
            tracing::debug!(
                "Current track updated: {}",
                current_track.track.title.clone().unwrap_or_default()
            );
        }

        self.clear_blacklist();

        if force {
            self.data.force_load_track = !self.data.force_load_track;
        }

        self.scrobble_time = 0f64;
        self.scrobbled = false;

        let _ = self.save_to_db(&["current_index", "player_state"]);
    }



    #[tracing::instrument(level = "debug", skip(self, tracks))]
    pub fn add_to_queue(&mut self, tracks: Vec<MediaContent>) {
        self.add_to_queue_at_index(tracks, self.data.queue.track_queue.len());
        self.update_current_track(false);
    }

    #[tracing::instrument(level = "debug", skip(self, tracks, index))]
    fn add_to_queue_at_index(&mut self, tracks: Vec<MediaContent>, index: usize) {
        let mut index = index;
        for track in tracks {
            self.insert_track_at_index(track, index, false);
            index += 1;
        }

        let _ = self.save_to_db(&["queue_data", "track_queue"]);
    }

    #[tracing::instrument(level = "debug", skip(self, index))]
    pub fn remove_from_queue(&mut self, index: usize) {
        self.data.queue.track_queue.remove(index);
        if self.data.queue.current_index > index {
            self.data.queue.current_index -= 1;
        }

        if self.data.queue.current_index == index {
            self.update_current_track(false);
        }

        let _ = self.save_to_db(&["track_queue", "queue_data"]);
    }

    #[tracing::instrument(level = "debug", skip(self, track, index))]
    fn insert_track_at_index(&mut self, track: MediaContent, index: usize, dump: bool) {
        let track_id = track.track._id.clone().unwrap();
        // Update metadata in data map
        self.data.queue.data.insert(track_id.clone(), track);

        // Skip insertion if track already exists in queue (avoid duplicates)
        if self.data.queue.track_queue.contains(&track_id) {
            if dump {
                // Persist metadata changes if any
                let _ = self.save_to_db(&["queue_data"]);
            }
            return;
        }

        let insertion_index = min(self.data.queue.track_queue.len(), index);
        self.data.queue.track_queue.insert(insertion_index, track_id);

        if dump {
            let _ = self.save_to_db(&["queue_data", "track_queue"]);
        }
    }

    #[tracing::instrument(level = "debug", skip(self, track))]
    pub fn play_now(&mut self, track: MediaContent) {
        self.set_state(PlayerState::Playing);
        let track_id = track.track._id.clone().unwrap();

        // If track already exists in queue, jump to it instead of inserting duplicate
        if let Some(existing_index) = self
            .data
            .queue
            .track_queue
            .iter()
            .position(|id| id == &track_id)
        {
            self.data.queue.data.insert(track_id.clone(), track); // refresh metadata
            self.data.queue.current_index = existing_index;
            self.update_current_track(true);
            let _ = self.save_to_db(&["current_index", "queue_data"]);
            return;
        }

        // Otherwise insert after current and advance index
        self.insert_track_at_index(track, self.data.queue.current_index + 1, true);
        self.data.queue.current_index += 1;
        self.update_current_track(true);
    }

    #[tracing::instrument(level = "debug", skip(self, tracks))]
    pub fn play_now_multiple(&mut self, tracks: Vec<MediaContent>) {
        if tracks.is_empty() {
            return;
        }

        let first_track = tracks.first();
        if let Some(first_track) = first_track {
            self.play_now(first_track.clone())
        }

        if tracks.len() > 1 {
            self.add_to_queue_at_index(tracks[1..].to_vec(), self.data.queue.current_index + 1);
        }
    }

    #[tracing::instrument(level = "debug", skip(self, track))]
    pub fn play_next(&mut self, track: MediaContent) {
        self.insert_track_at_index(track, self.data.queue.current_index + 1, true);
    }

    #[tracing::instrument(level = "debug", skip(self, tracks))]
    pub fn play_next_multiple(&mut self, tracks: Vec<MediaContent>) {
        if tracks.is_empty() {
            return;
        }

        let first_track = tracks.first();
        if let Some(first_track) = first_track {
            self.play_next(first_track.clone())
        }

        if tracks.len() > 1 {
            self.add_to_queue_at_index(tracks[1..].to_vec(), self.data.queue.current_index + 1);
        }
    }

    #[tracing::instrument(level = "debug", skip(self, new_index))]
    pub fn change_index(&mut self, new_index: usize, force: bool) {
        self.data.queue.current_index = new_index;
        self.update_current_track(force);
    }

    #[tracing::instrument(level = "debug", skip(self, new_time))]
    pub fn update_time(&mut self, new_time: f64) {
        self.scrobble_time += 0f64.max(new_time - self.data.player_details.current_time);
        self.data.player_details.current_time = new_time;

        if self.scrobble_time > 20f64 && !self.scrobbled {
            if let Some(_current_track) = self.get_current_track() {
                self.scrobbled = true;
                // send_extension_event(ExtensionExtraEvent::Scrobble([current_track]));
            }
        }

        // Note: Position updates now handled via callbacks in AudioPlayer
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_time(&self) -> f64 {
        self.data.player_details.current_time
    }

    #[tracing::instrument(level = "debug", skip(self, new_time))]
    pub fn force_seek_percent(&mut self, new_time: f64) {
        let new_time_c = if let Some(current_track) = &self.data.current_track {
            current_track.track.duration.unwrap_or_default() * new_time
        } else {
            0f64
        };

        tracing::debug!(
            "Got seek {}, {:?}, {}",
            new_time,
            self.data.current_track.clone().map(|c| c.track.duration),
            new_time_c
        );
        self.data.player_details.force_seek = new_time_c;
        // send_extension_event(ExtensionExtraEvent::Seeked([new_time]))
    }

    #[tracing::instrument(level = "debug", skip(self, new_time))]
    pub fn force_seek(&mut self, new_time: f64) {
        self.data.player_details.force_seek = new_time;
        // send_extension_event(ExtensionExtraEvent::Seeked([new_time]))
    }

    #[tracing::instrument(level = "debug", skip(self, state))]
    pub fn set_state(&mut self, state: PlayerState) {
        tracing::debug!("Setting player state {:?}", state);
        self.data.player_details.state = state;
        let _ = self.save_to_db(&["player_state"]);

        set_playback_state(state);
        // send_extension_event(ExtensionExtraEvent::PlayerStateChanged([state]))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_track_key(&self) -> String {
        if let Some(current_track) = &self.data.current_track {
            return current_track
                .track
                .provider_extension
                .clone()
                .unwrap_or(current_track.track.type_.to_string());
        }
        "".to_string()
    }

    #[tracing::instrument(level = "debug", skip(self, volume))]
    pub fn set_volume(&mut self, volume: f64) {
        if let VolumeMode::PersistSeparate = self.data.player_details.volume_mode {
            let track_key = self.get_track_key();
            if !track_key.is_empty() {
                tracing::debug!("Setting volume for track: {}, {}", track_key, volume);
                self.data.player_details.volume_map.insert(track_key, volume);
            }
        }
        self.data.player_details.volume = volume;

        let _ = self.save_to_db(&["player_state"]);
        // send_extension_event(ExtensionExtraEvent::VolumeChanged([volume]))
    }

    pub fn toggle_mute(&mut self) {
        if self.data.player_details.volume > 0f64 {
            self.data.player_details.old_volume = self.data.player_details.volume;
            self.set_volume(0f64);
        } else if self.data.player_details.old_volume > 0f64 {
            self.set_volume(self.data.player_details.old_volume);
        } else {
            self.set_volume(50f64);
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_volume(&self) -> f64 {
        if self.is_mobile {
            return 100f64;
        }

        let mut clamp = 100f64;
        let mut volume = self.data.player_details.volume;
        let track_key = self.get_track_key();
        if !track_key.is_empty() {
            if let VolumeMode::PersistSeparate = self.data.player_details.volume_mode {
                if let Some(current_volume) = self.data.player_details.volume_map.get(&track_key) {
                    volume = *current_volume;
                }
            }

            if let VolumeMode::PersistClamp = self.data.player_details.volume_mode {
                if let Some(current_clamp) = self.data.player_details.clamp_map.get(&track_key) {
                    clamp = *current_clamp;
                }
            }
        }
        let maxv = (clamp).ln();
        let scale = maxv / 100f64;
        let volume = volume.clamp(0f64, 100f64);
        if volume > 0f64 {
            return volume.ln() / scale;
        }
        volume
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_raw_volume(&self) -> f64 {
        if let VolumeMode::PersistSeparate = self.data.player_details.volume_mode {
            let track_key = self.get_track_key();
            if !track_key.is_empty() {
                if let Some(volume) = self.data.player_details.volume_map.get(&track_key) {
                    return *volume;
                }
            }
        }
        self.data.player_details.volume
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_queue_tracks(&self) -> Vec<MediaContent> {
        self.data
            .queue
            .track_queue
            .iter()
            .map(|index| {
                self.data
                    .queue
                    .data
                    .get(index)
                    .cloned()
                    .expect("MediaContent does not exist in data")
            })
            .collect()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn next_track(&mut self) {
        self.data.queue.current_index += 1;
        if self.data.queue.current_index >= self.data.queue.track_queue.len() {
            self.data.queue.current_index = 0;
        }
        self.update_current_track(true);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn prev_track(&mut self) {
        if self.data.queue.current_index == 0 {
            self.data.queue.current_index = self.data.queue.track_queue.len() - 1;
        } else {
            self.data.queue.current_index -= 1;
        }
        self.update_current_track(false);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn toggle_player_mode(&mut self) {
        let new_mode = match self.data.player_details.repeat {
            PlayerMode::Sequential => PlayerMode::Single,
            PlayerMode::Single => PlayerMode::Shuffle,
            PlayerMode::Shuffle => PlayerMode::ListLoop,
            PlayerMode::ListLoop => PlayerMode::Sequential,
        };

        self.data.player_details.repeat = new_mode;
        
        // Initialize shuffle bag when switching to shuffle mode
        if new_mode == PlayerMode::Shuffle {
            self.rebuild_shuffle_bag();
        }
        
        let _ = self.save_to_db(&["player_state"]);
    }

    /// Explicitly set player mode from external callers (e.g., Tauri command)
    /// This ensures internal invariants and persistence are respected.
    #[tracing::instrument(level = "debug", skip(self, mode))]
    pub fn set_player_mode(&mut self, mode: PlayerMode) {
        self.data.player_details.repeat = mode;
        self.set_has_repeated(false);

        if mode == PlayerMode::Shuffle {
            self.rebuild_shuffle_bag();
        }

        let _ = self.save_to_db(&["player_state"]);
    }

    /// Rebuild shuffle bag with all queue indices except current
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn rebuild_shuffle_bag(&mut self) {
        let queue_len = self.data.queue.track_queue.len();
        if queue_len <= 1 {
            self.data.shuffle_bag.clear();
            self.data.shuffle_index = 0;
            return;
        }

        // Create indices excluding current index
        let mut indices: Vec<usize> = (0..queue_len)
            .filter(|&i| i != self.data.queue.current_index)
            .collect();
        
        // Shuffle the indices
        let mut rng = thread_rng();
        indices.shuffle(&mut rng);
        
        self.data.shuffle_bag = indices;
        self.data.shuffle_index = 0;
        
        tracing::debug!("Rebuilt shuffle bag with {} indices", self.data.shuffle_bag.len());
    }

    /// Get next index from shuffle bag, rebuild if exhausted
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_next_shuffle_index(&mut self) -> Option<usize> {
        if self.data.shuffle_bag.is_empty() || self.data.shuffle_index >= self.data.shuffle_bag.len() {
            self.rebuild_shuffle_bag();
        }
        
        if self.data.shuffle_bag.is_empty() {
            return None; // Only happens if queue has <= 1 track
        }
        
        let next_index = self.data.shuffle_bag[self.data.shuffle_index];
        self.data.shuffle_index += 1;
        Some(next_index)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn shuffle_queue(&mut self) {
        let binding = self.data.queue.track_queue.clone();
        let current_track = binding.get(self.data.queue.current_index).unwrap();
        let mut rng = thread_rng();
        self.data.queue.track_queue.shuffle(&mut rng);
        let new_index = self
            .data
            .queue
            .track_queue
            .iter()
            .position(|v| v == current_track)
            .unwrap();
        self.data.queue.current_index = new_index;

        let _ = self.save_to_db(&["current_index", "track_queue"]);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn clear_queue(&mut self) {
        self.data.queue.track_queue.clear();
        self.data.queue.current_index = 0;
        self.update_current_track(false);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn clear_queue_except_current(&mut self) {
        let current_track = self.get_current_track();

        let only_one_track = self.get_queue().track_queue.len() == 1;
        self.data.queue.track_queue.clear();
        self.data.queue.current_index = 0;

        if !only_one_track {
            if let Some(current_track) = current_track {
                self.add_to_queue(vec![current_track]);
            }
        }

        self.update_current_track(false);
        let _ = self.save_to_db(&["queue_data", "track_queue"]);
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub fn blacklist_player(&mut self, key: String) {
        if self.data.player_blacklist.contains(&key) {
            return;
        }
        self.data.player_blacklist.push(key);
        self.data.force_load_track = !self.data.force_load_track
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn clear_blacklist(&mut self) {
        self.data.player_blacklist.clear();
    }

    /// Set database for persistence
    pub fn set_database(&mut self, db: Arc<Database>) {
        self.db = Some(db);
        // Load state immediately when database is set
        if let Err(e) = self.load_from_db() {
            tracing::warn!("Failed to load state after setting database: {:?}", e);
        }
    }

    /// Static method to load state from database
    pub fn load_state_from_db(db: &Database) -> Option<PlayerStoreData> {
        let keys = vec!["player_state", "track_queue", "current_index", "queue_data"];
        
        match db.get_player_store_values(keys) {
            Ok(values) => {
                let mut data = PlayerStoreData::default();
                
                if let Some(player_state_str) = values.get("player_state") {
                    if let Ok(player_details) = serde_json::from_str::<PlayerDetails>(player_state_str) {
                        data.player_details = player_details;
                        data.player_details.current_time = 0f64; // Reset current_time on load
                    }
                }
                
                if let Some(track_queue_str) = values.get("track_queue") {
                    if let Ok(track_queue) = serde_json::from_str::<Vec<String>>(track_queue_str) {
                        data.queue.track_queue = track_queue;
                    }
                }
                
                if let Some(current_index_str) = values.get("current_index") {
                    if let Ok(current_index) = serde_json::from_str::<usize>(current_index_str) {
                        data.queue.current_index = current_index;
                    }
                }
                
                if let Some(queue_data_str) = values.get("queue_data") {
                    if let Ok(queue_data) = serde_json::from_str::<HashMap<String, MediaContent>>(queue_data_str) {
                        data.queue.data = queue_data;
                    }
                }
                
                // Update current track based on loaded data
                if let Some(track_id) = data.queue.track_queue.get(data.queue.current_index) {
                    data.current_track = data.queue.data.get(track_id).cloned();
                }
                
                tracing::debug!("Loaded player store state from database");
                Some(data)
            }
            Err(e) => {
                tracing::error!("Failed to load player store state from database: {:?}", e);
                None
            }
        }
    }
}
