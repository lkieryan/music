use std::sync::Arc;
use tokio::sync::RwLock;
use sqlx::SqlitePool;
use crate::sources::SourceManager;
use crate::audio::AudioPlayer;
use super::{PlayerState, LibraryState};

/// 全局应用状态
pub struct AppState {
    pub db: SqlitePool,
    pub source_manager: Arc<RwLock<SourceManager>>,
    pub audio_player: Arc<RwLock<AudioPlayer>>,
    pub player_state: Arc<RwLock<PlayerState>>,
    pub library_state: Arc<RwLock<LibraryState>>,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            source_manager: Arc::new(RwLock::new(SourceManager::new())),
            audio_player: Arc::new(RwLock::new(AudioPlayer::new())),
            player_state: Arc::new(RwLock::new(PlayerState::new())),
            library_state: Arc::new(RwLock::new(LibraryState::new())),
        }
    }
}