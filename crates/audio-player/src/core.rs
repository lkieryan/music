use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use crossbeam_channel::{unbounded, Receiver};
use tokio::sync::oneshot;
use types::errors::Result;
use types::songs::{SongType, Song};
use types::ui::player_details::{PlayerEvents, PlayerState, PlayerMode};
use database::database::Database;
use crate::players::base::{BasePlayer, PlayerEventsSender};
use crate::players::librespot::{LibrespotAdapter, LibrespotPlayer};
use crate::players::rodio::RodioPlayer;
use crate::store::PlayerStore;
use crate::events::{apply_event_basic, apply_event_with_hooks, EventHooks};

use ::mpris;

/// A minimal, backend-only audio player core used by Tauri.
/// It manages a small set of BasePlayer implementations without any UI deps.
pub struct AudioPlayer {
    // Managed players
    players: std::sync::Mutex<Vec<Box<dyn BasePlayer + Send + Sync>>>,
    // Index of active player in `players`
    active: AtomicUsize,
    // Outgoing events for UI bridge
    pub(crate) events_tx: crossbeam_channel::Sender<PlayerEvents>,
    events_rx: Arc<Mutex<Receiver<PlayerEvents>>>,
    // Player state and queue management
    store: Arc<Mutex<PlayerStore>>,
    // Cache dir (reserved for future use)
    _cache_dir: PathBuf,
    // MPRIS integration
    pub(crate) mpris_holder: Option<::mpris::MprisHolder>,
}

impl AudioPlayer {
    /// Base initializer shared by desktop and mobile constructors
    /// NOTE: This is an internal helper.
    fn new_base(cache_dir: PathBuf) -> Self {
        let (tx, rx) = unbounded::<PlayerEvents>();
        
        // Initialize player store (without database initially)
        let store = Arc::new(Mutex::new(PlayerStore::new(None)));
        
        // Initialize players
        let players = Self::initialize_players(store.clone(), tx.clone(), cache_dir.clone());
        
        Self {
            players: std::sync::Mutex::new(players),
            active: AtomicUsize::new(0),
            events_tx: tx,
            events_rx: Arc::new(Mutex::new(rx)),
            store,
            _cache_dir: cache_dir,
            mpris_holder: None,
        }
    }

    /// Acquire players mutex guard with unified error mapping
    /// comments: Provide unified Mutex lock and error mapping
    fn players_guard(&self) -> Result<std::sync::MutexGuard<'_, Vec<Box<dyn BasePlayer + Send + Sync>>>> {
        self.players
            .lock()
            .map_err(|_| types::errors::MusicError::from("players lock poisoned"))
    }
    

    // Removed set_mpris_callbacks; external callback integration has been dropped

    /// Create AudioPlayer with database for persistence (desktop)
    pub fn new_desktop(cache_dir: PathBuf, db: Arc<Database>) -> Self {
      let player = Self::new_base(cache_dir);
      
      // Set database for persistence
      if let Ok(mut store) = player.store.lock() {
          store.set_database(db);
      }
      
      player
  }

  /// Create AudioPlayer with database and mobile support
  #[cfg(any(target_os = "android", target_os = "ios"))]
  pub fn new_mobile(cache_dir: PathBuf, db: Arc<Database>, _app_handle: tauri::AppHandle) -> Self {
      let player = Self::new_base(cache_dir);
      
      // Set database for persistence
      if let Ok(mut store) = player.store.lock() {
          store.set_database(db);
      }
      
      player
  }
  /// Initialize and configure all players
  fn initialize_players(
      store: Arc<Mutex<PlayerStore>>,
      events_tx: crossbeam_channel::Sender<PlayerEvents>,
      cache_dir: PathBuf
  ) -> Vec<Box<dyn BasePlayer + Send + Sync>> {
      let state_setter = Self::create_player_event_handler(store, events_tx);
      
      let mut players: Vec<Box<dyn BasePlayer + Send + Sync>> = Vec::new();
      
      // Initialize Rodio player (for local files, URLs, HLS, DASH)
      let mut rodio = RodioPlayer::new(cache_dir.clone());
      rodio.add_listeners(state_setter.clone());
      players.push(Box::new(rodio));
      
      // Initialize Librespot player (for Spotify)
      let mut librespot = LibrespotPlayer::new();
      librespot.add_listeners(state_setter.clone());
      players.push(Box::new(librespot));
      
      // Initialize each player
      for p in players.iter() { 
          p.initialize(); 
      }
      
      players
  }

  /// Create event handler for player events
  fn create_player_event_handler(
      store: Arc<Mutex<PlayerStore>>,
      events_tx: crossbeam_channel::Sender<PlayerEvents>
  ) -> PlayerEventsSender {
      Arc::new(move |player_key: String, ev: PlayerEvents| {
          // Handle player events and update store
          if let Ok(mut player_store) = store.lock() {
              if let PlayerEvents::Error(err) = &ev {
                  // Preserve original error handling semantics
                  Self::handle_player_error(&mut player_store, &player_key, err);
              } else {
                  // Delegate to centralized basic event application
                  apply_event_basic(&mut player_store, &ev);
              }
          }
          
          // Also send event to UI bridge
          let _ = events_tx.send(ev);
      })
  }

  /// Handle player error events
  fn handle_player_error(player_store: &mut PlayerStore, player_key: &str, err: &types::errors::MusicError) {
      tracing::error!("Player {} error: {:?}", player_key, err);
      // Only blacklist if it's a valid player key format
      if player_key.starts_with("player_") {
          player_store.blacklist_player(player_key.to_string());
      }
      player_store.set_state(PlayerState::Stopped);
  }

  fn get_player(&self, song: &mut Song) -> Result<usize> {
      let blacklist = if let Ok(store) = self.store.lock() {
          store.get_player_blacklist()
      } else {
          Vec::new()
      };
      
      tracing::debug!("Getting players for song {:?}", song.song.title);
      // First attempt: find player that can handle the song
      let player_index = {
          let players = self.players_guard()?;
          players.iter().position(|p| {
              let player_key = p.key();
              tracing::debug!(
                  "Checking player capabilities {}, type: {:?}, url: {:?}",
                  player_key,
                  song.song.type_,
                  song.song.playback_url
              );
              let res = !blacklist.contains(&player_key)
                  && p.provides().contains(&song.song.type_)
                  && p.can_play(song);
              tracing::debug!("Player {} can handle song: {}", player_key, res);
              res
          })
      };
      
      if let Some(player_idx) = player_index {
          tracing::info!("Found player {}", player_idx);
          return Ok(player_idx);
      }
      
      // TODO: Second attempt with playback URL fetching (Extension support)
      // This would require provider store integration, skipping for now
      tracing::warn!("No suitable player found for song type: {:?}", song.song.type_);
      
      Err(types::errors::MusicError::String("Player not found".into()))
  }

  fn find_player_by_type(&self, ty: SongType) -> Option<usize> {
      self.players.lock().ok().and_then(|players| players.iter().position(|p| p.provides().contains(&ty)))
  }

  /// Expose event receiver for Tauri bridge thread
  pub fn get_events_rx(&self) -> Arc<Mutex<Receiver<PlayerEvents>>> { 
      self.events_rx.clone() 
  }

  /// Get access to the player store
  pub fn get_store(&self) -> Arc<Mutex<PlayerStore>> { 
      self.store.clone() 
  }

  /// Load player state from database and update internal store.
  /// Intended to be called during initialization.
  pub fn load_state(&self, db: &Database) -> Result<()> {
      if let Some(data) = PlayerStore::load_state_from_db(db) {
          if let Ok(mut store) = self.store.lock() {
              store.data = data;
              tracing::info!("Loaded player state from database");
          }
      }
      Ok(())
  }

  /// Register Spotify adapter callbacks (internal use only)
  pub fn register_spotify_adapter(&self, adapter: LibrespotAdapter) {
      // Broadcast to all players; only LibrespotPlayer will accept
      if let Ok(mut players) = self.players.lock() {
          for p in players.iter_mut() {
              p.configure("spotify.adapter", &adapter);
          }
      } else {
          tracing::error!("players lock poisoned while registering spotify adapter");
      }
  }

  pub async fn audio_load(&self, song: &mut Song) -> Result<()> {
      let idx = self.get_player(song)?;
      self.active.store(idx, Ordering::SeqCst);
      
      // Get the actual player key from the player itself
      let player_key = {
          let players = self.players_guard()?;
          players[idx].key()
      };
      let store_clone = self.store.clone();
      let events_tx_clone = self.events_tx.clone();
      
      // Use the playback_url or path from the song
      let src = song.song.playback_url.clone().or(song.song.path.clone());
      if src.is_none() {
          return Err(types::errors::MusicError::String("No playback URL or path available".into()));
      }
      
      let state_setter: PlayerEventsSender = Arc::new(move |_player_key: String, ev: PlayerEvents| {
          let actual_player_key = player_key.clone();
          
          // Handle player events and update store
          if let Ok(mut player_store) = store_clone.lock() {
              if let PlayerEvents::Error(err) = &ev {
                  tracing::error!("Player {} error: {:?}", actual_player_key, err);
                  player_store.blacklist_player(actual_player_key);
                  player_store.set_state(PlayerState::Stopped);
              } else {
                  let hooks = EventHooks::default();
                  apply_event_with_hooks(&mut player_store, &ev, &hooks);
              }
          }
          
          let _ = events_tx_clone.send(ev);
      });
      
      tracing::debug!("Loading song with player {}: {:?}", idx, song.song.title);
      
      let (tx, rx) = oneshot::channel::<()>();
      {
          let mut players = self.players_guard()?;
          players[idx].add_listeners(state_setter);
          players[idx].load(src.unwrap(), true, tx);
      }
      let _ = rx.await;
      
      // Notify MPRIS of metadata change for the loaded song
      self.notify_mpris_metadata(song);
      
      Ok(())
  }

  /// Play the current or provided song, loading media when necessary.
  ///
  /// Behavior:
  /// - When `song` is `Some(&mut Song)`: if it is different from the current song (by `_id`),
  ///   this function updates the store (queue/index and current song), loads the provided song
  ///   into the active backend player via `audio_load()`, and then issues `play()`.
  ///   If it is the same as the current song, it will skip reloading and only `play()`.
  /// - When `song` is `None`: this function implements a first-resume heuristic for app startup.
  ///   If the store indicates `current_time == 0.0` and there is a `current_song` (i.e. the app
  ///   persisted state but no player has loaded media yet), it will load that song once before `play()`.
  ///   Otherwise, it assumes the player is already loaded and only issues `play()`.
  ///
  /// Concurrency & locking:
  /// - Store access uses short-lived `Mutex` locks. The function avoids holding a lock across `await` points.
  /// - Song loading (I/O and backend initialization) happens outside of any store lock.
  ///
  /// Side effects:
  /// - May update the store's current song and queue index via `store.play_now()` when a new song is provided.
  /// - May perform `audio_load()` before playing.
  /// - On successful `play()`, notifies MPRIS state as `Playing`.
  ///
  /// Errors:
  /// - Propagates errors from store locking, loading (`audio_load()`), and backend `play()`.
  ///
  /// Notes:
  /// - The startup heuristic (`current_time == 0.0`) is used to detect the "restoring from persisted state
  ///   but media not yet loaded" scenario on first app launch.
  pub async fn audio_play(&self, song: Option<&mut Song>) -> Result<()> { 
      // Decide whether we need to load something before play
      enum LoadAction<'a> {
          None,
          Provided(&'a mut Song),
          Current(Song),
      }

      let mut action = LoadAction::None;

      match song {
          Some(s) => {
              // Compare provided song id with current song id
              let provided_id = s.song._id.clone();
              let is_same_as_current = {
                  let store = self
                      .store
                      .lock()
                      .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
                  let current = store.get_current_song();
                  match (current.and_then(|s| s.song._id), provided_id.clone()) {
                      (Some(cur_id), Some(prov_id)) => cur_id == prov_id,
                      _ => false,
                  }
              };

              if !is_same_as_current {
                  // Update store with the new song without holding the lock across await
                  {
                      let mut store = self
                          .store
                          .lock()
                          .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
                      store.play_now(s.clone());
                  }
                  action = LoadAction::Provided(s);
              }
          }
          None => {
              // First-resume heuristic: if app just started (current_time == 0.0)
              // and there is a current song, load it before play
              let mut current_song_opt: Option<Song> = None;
              {
                  let store = self
                      .store
                      .lock()
                      .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
                  if store.get_current_time() == 0.0 {
                      current_song_opt = store.get_current_song();
                  }
              }
              if let Some(s) = current_song_opt {
                  action = LoadAction::Current(s);
              }
          }
      }

      // Execute load if required
      match action {
          LoadAction::None => {}
          LoadAction::Provided(s) => {
              self.audio_load(s).await?;
          }
          LoadAction::Current(mut s) => {
              self.audio_load(&mut s).await?;
          }
      }

      // Play the currently loaded song
      let idx = self.active.load(Ordering::SeqCst);
      let result = {
          let players = self.players_guard()?;
          players[idx].play()
      };
      if result.is_ok() {
          self.notify_mpris_state(PlayerState::Playing);
      }
      result
  }

  /// Advance to next song in queue: update index in store, load and play.
  pub async fn play_next(&self) -> Result<Option<Song>> {
      // Move index and fetch song snapshot without holding lock across await
      let mut song_opt = None;
      {
          let mut store = self
              .store
              .lock()
              .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
          store.next_song();
          song_opt = store.get_current_song();
      }
      if let Some(mut song) = song_opt.clone() {
          // Ensure the selected song is actually loaded and then play
          self.audio_load(&mut song).await?;
          self.audio_play(None).await?;
      }
      Ok(song_opt)
  }

  /// Go back to previous song in queue: update index in store, load and play.
  pub async fn play_prev(&self) -> Result<Option<Song>> {
      let mut song_opt = None;
      {
          let mut store = self
              .store
              .lock()
              .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
          store.prev_song();
          song_opt = store.get_current_song();
      }
      if let Some(mut song) = song_opt.clone() {
          self.audio_load(&mut song).await?;
          self.audio_play(None).await?;
      }
      Ok(song_opt)
  }

  pub async fn audio_pause(&self) -> Result<()> { 
      let idx = self.active.load(Ordering::SeqCst);
      let result = {
          let players = self.players_guard()?;
          players[idx].pause()
      };
      if result.is_ok() {
          self.notify_mpris_state(PlayerState::Paused);
      }
      result
  }

  pub async fn audio_stop(&self) -> Result<()> { 
      let idx = self.active.load(Ordering::SeqCst);
      let result = {
          let mut players = self.players_guard()?;
          players[idx].stop()
      };
      if result.is_ok() {
          self.notify_mpris_state(PlayerState::Stopped);
      }
      result
  }

  pub async fn audio_seek(&self, pos: f64) -> Result<()> { 
      let idx = self.active.load(Ordering::SeqCst);
      let result = {
          let players = self.players_guard()?;
          players[idx].seek(pos)
      };
      if result.is_ok() {
          self.notify_mpris_position(pos);
      }
      result
  }

  pub async fn audio_set_volume(&self, volume: f32) -> Result<()> { 
      // Update and persist volume in store (DB)
      //    Frontend passes 0.0 - 1.0; Store expects 0 - 100 raw scale
      {
          let mut store = self
              .store
              .lock()
              .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
          let raw = (volume as f64 * 100.0).clamp(0.0, 100.0);
          store.set_volume(raw);
      }

      // Propagate to active backend player
      let idx = self.active.load(Ordering::SeqCst);
      let players = self.players_guard()?;
      players[idx].set_volume(volume as f64)
  }

  pub async fn audio_get_volume(&self) -> Result<f32> { 
      // Read persisted raw volume (0-100) from Store and convert to 0.0-1.0
      let raw = {
          let store = self
              .store
              .lock()
              .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
          store.get_raw_volume()
      };
      Ok((raw / 100.0) as f32)
  }
}