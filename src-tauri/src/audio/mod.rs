use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};
use types::errors::Result;
use audio_player::AudioPlayer;
use crate::playback::spotify::make_librespot_adapter;
use std::sync::Arc;
use database::database::Database;
use serde_json::json;

#[tracing::instrument(level = "debug", skip(app))]
pub fn build_audio_player(app: AppHandle) -> AudioPlayer {
    let db_state: State<'_, Arc<Database>> = app.state();
    let db = db_state.inner().clone();
    
    let cache_dir = app.path().app_cache_dir().expect("cache dir");
    
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let mut audio_player = AudioPlayer::new_mobile(cache_dir, db.clone(), app.clone());
    
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let mut audio_player = AudioPlayer::new_desktop(cache_dir, db.clone());
    
    if let Err(e) = audio_player.load_state(&db) {
        tracing::error!("Failed to load player state from database: {:?}", e);
    }
    if let Err(e) = audio_player.initialize_mpris() {
        tracing::error!("Failed to initialize MPRIS: {:?}", e);
    }
    
    #[cfg(any(target_os = "android", target_os = "ios"))]
    audio_player.set_mpris_app_handle(app.clone());
    
    // Sanitize initial playback state at startup to avoid stale "PLAYING" UI
    // If there's no current song or queue is empty, force STOPPED.
    // Otherwise, if state persisted as PLAYING, downgrade to PAUSED until actual playback starts.
    {
        let store_arc = audio_player.get_store();
        // Bind lock result to ensure its temporaries drop before store_arc
        let lock_res = store_arc.lock();
        if let Ok(mut store) = lock_res {
            let q_len = store.get_queue_len();
            let has_song = store.get_current_song().is_some();
            let state = store.get_player_state();
            use types::ui::player_details::PlayerState as Ps;
            if q_len == 0 || !has_song {
                if state != Ps::Stopped {
                    store.set_state(Ps::Stopped);
                }
            } else if state == Ps::Playing {
                store.set_state(Ps::Paused);
            }
        }
    }

    if let Some(_handle) = audio_player.start_mpris_event_listener() {
        tracing::info!("MPRIS event listener started");
    }
    
    let adapter = make_librespot_adapter(app.app_handle().clone());
    audio_player.register_spotify_adapter(adapter);
    
    let events_rx = audio_player.get_events_rx();
    let store_arc = audio_player.get_store();
    let app_for_thread = app.clone();
    thread::spawn(move || {
        use serde::Serialize;
        use serde_json::json;
        use types::ui::player_details::{PlayerEvents, PlayerState};

        #[derive(Serialize)]
        struct FrontendEnvelope<T: Serialize> {
            #[serde(rename = "type")] // keep the exact key name as front-end expects
            type_field: &'static str,
            data: T,
        }

        let rx = events_rx.lock().expect("lock events rx");
        while let Ok(ev) = rx.recv() {
            // Helper to emit a structured envelope with arbitrary JSON data
            let emit_json = |event_type: &'static str, data: serde_json::Value| {
                let payload = json!({
                    "type": event_type,
                    "data": data,
                });
                let _ = app_for_thread.emit("audio_event", payload);
            };

            match ev {
                PlayerEvents::Play => {
                    emit_json(
                        "PlaybackStateChanged",
                        json!({ "is_playing": true, "is_paused": false }),
                    );
                }
                PlayerEvents::Pause => {
                    emit_json(
                        "PlaybackStateChanged",
                        json!({ "is_playing": false, "is_paused": true }),
                    );
                }
                PlayerEvents::Loading => {
                    // Do NOT modify playback state on loading; avoid UI flicker.
                    // Optionally notify front-end about buffering if it wants to show an indicator.
                    emit_json("Buffering", json!({}));

                    // Also announce current song metadata if available
                    if let Ok(store) = store_arc.lock() {
                        if let Some(song) = store.get_current_song() {
                            emit_json("SongChanged", json!({ "song": song }));
                        }
                    }
                }
                PlayerEvents::Ended => {
                    // Track finished signal
                    emit_json("TrackFinished", json!({}));
                    // After store updates to next song (handled in core), announce new song
                    if let Ok(store) = store_arc.lock() {
                        if let Some(song) = store.get_current_song() {
                            emit_json("SongChanged", json!({ "song": song }));
                        }
                        // Reflect current playing state as well
                        let state = store.get_player_state();
                        let (is_playing, is_paused) = match state {
                            PlayerState::Playing => (true, false),
                            PlayerState::Paused => (false, true),
                            _ => (false, false),
                        };
                        emit_json(
                            "PlaybackStateChanged",
                            json!({ "is_playing": is_playing, "is_paused": is_paused }),
                        );
                        // Auto-play next track when store indicates Playing after Ended
                        if matches!(state, PlayerState::Playing) {
                            if let Some(mut song) = store.get_current_song() {
                                let app_clone = app_for_thread.clone();
                                tauri::async_runtime::spawn(async move {
                                    // Acquire AudioPlayer state
                                    let audio_state: State<'_, AudioPlayer> = app_clone.state();
                                    // Load the selected song and then play
                                    let _ = audio_state.audio_load(&mut song).await;
                                    let _ = audio_state.audio_play(None).await;
                                });
                            }
                        }
                    }
                }
                PlayerEvents::TimeUpdate(time) => {
                    // Convert seconds(f64) to Duration-like object { secs, nanos }
                    let secs = time.trunc() as i64;
                    let nanos = ((time - secs as f64) * 1_000_000_000f64).round() as i64;
                    emit_json(
                        "PositionChanged",
                        json!({ "position": { "secs": secs, "nanos": nanos } }),
                    );
                }
                PlayerEvents::Error(err) => {
                    emit_json("Error", json!({ "message": err.to_string() }));
                }
            }
        }
    });
    
    audio_player
}

// ---------- Commands (UI only sees these) ----------


#[tracing::instrument(level = "debug", skip(state, song))]
#[tauri::command]
pub async fn audio_play(app: AppHandle, state: State<'_, AudioPlayer>, song: Option<types::songs::Song>) -> Result<()> {
    let mut song_ref = song;
    let result = state.audio_play(song_ref.as_mut()).await;

    // Emit events after successful play
    if result.is_ok() {
        // If a song was explicitly provided, use it directly to avoid any race with store updates
        if let Some(provided_song) = song_ref {
            // emit SongChanged with the provided song
            let _ = app.emit(
                "audio_event",
                json!({ "type": "SongChanged", "data": { "song": provided_song } }),
            );
            // Optionally also notify queue changed since explicit play may update index
            let _ = app.emit(
                "audio_event",
                json!({ "type": "QueueChanged", "data": {} }),
            );
        } else {
            // Fallback: no song provided, emit current song from store
            if let Ok(store) = state.get_store().lock() {
                if let Some(song) = store.get_current_song() {
                    let _ = app.emit(
                        "audio_event",
                        json!({ "type": "SongChanged", "data": { "song": song } }),
                    );
                }
            }
        }
    }

    result
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn audio_pause(state: State<'_, AudioPlayer>) -> Result<()> {
    state.audio_pause().await
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn audio_stop(state: State<'_, AudioPlayer>) -> Result<()> {
    state.audio_stop().await
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn audio_seek(state: State<'_, AudioPlayer>, pos: f64) -> Result<()> {
    state.audio_seek(pos).await
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn audio_set_volume(app: AppHandle, state: State<'_, AudioPlayer>, volume: f32) -> Result<()> {
    state.audio_set_volume(volume).await?;
    // Emit VolumeChanged event
    let _ = app.emit(
        "audio_event",
        json!({
            "type": "VolumeChanged",
            "data": { "volume": volume }
        }),
    );
    Ok(())
}


#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn audio_get_volume(state: State<'_, AudioPlayer>) -> Result<f32> {
    state.audio_get_volume().await
}

// ---------- PlayerStore Commands ----------

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn get_current_song(state: State<'_, AudioPlayer>) -> Result<Option<types::songs::Song>> {
    let store_arc = state.get_store();
    let store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    // Compute current song from queue without mutating store to avoid side effects
    let q = store.get_queue();
    let song_opt = q
        .song_queue
        .get(q.current_index)
        .and_then(|id| q.data.get(id))
        .cloned()
        .or_else(|| store.get_current_song());
    Ok(song_opt)
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn get_queue(state: State<'_, AudioPlayer>) -> Result<audio_player::store::Queue> {
    let store_arc = state.get_store();
    let store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    Ok(store.get_queue())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn get_player_state(state: State<'_, AudioPlayer>) -> Result<types::ui::player_details::PlayerState> {
    let store_arc = state.get_store();
    let store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    Ok(store.get_player_state())
}

#[tracing::instrument(level = "debug", skip(state, songs))]
#[tauri::command]
pub fn add_to_queue(app: AppHandle, state: State<'_, AudioPlayer>, songs: Vec<types::songs::Song>) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.add_to_queue(songs);
    // Emit QueueChanged
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state, index))]
#[tauri::command]
pub fn remove_from_queue(app: AppHandle, state: State<'_, AudioPlayer>, index: usize) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.remove_from_queue(index);
    // Emit QueueChanged
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state, song))]
#[tauri::command]
pub fn play_now(app: AppHandle, state: State<'_, AudioPlayer>, song: types::songs::Song) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.play_now(song);
    // Emit QueueChanged (now playing changed implies queue index change)
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn shuffle_queue(app: AppHandle, state: State<'_, AudioPlayer>) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.shuffle_queue();
    // Emit QueueChanged
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn clear_queue(app: AppHandle, state: State<'_, AudioPlayer>) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.clear_queue();
    // Emit QueueChanged
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn toggle_player_mode(app: AppHandle, state: State<'_, AudioPlayer>) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.toggle_player_mode();
    // Emit PlayerModeChanged with current mode
    let current_mode = store.get_repeat();
    let _ = app.emit(
        "audio_event",
        json!({ "type": "PlayerModeChanged", "data": { "mode": current_mode } }),
    );
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn get_player_mode(state: State<'_, AudioPlayer>) -> Result<types::ui::player_details::PlayerMode> {
    let store_arc = state.get_store();
    let store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    Ok(store.get_repeat())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn set_player_mode(app: AppHandle, state: State<'_, AudioPlayer>, mode: types::ui::player_details::PlayerMode) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    // Use public API to ensure invariants and persistence
    store.set_player_mode(mode);
    
    // Emit PlayerModeChanged event
    let _ = app.emit(
        "audio_event",
        json!({ "type": "PlayerModeChanged", "data": { "mode": mode } }),
    );
    
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn next_song(app: AppHandle, state: State<'_, AudioPlayer>) -> Result<()> {
    // Delegate to core: updates index + load + play
    let song_opt = state.play_next().await?;

    // Emit events for UI
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    if let Some(song) = song_opt {
        let _ = app.emit(
            "audio_event",
            json!({ "type": "SongChanged", "data": { "song": song } }),
        );
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn prev_song(app: AppHandle, state: State<'_, AudioPlayer>) -> Result<()> {
    // Delegate to core: updates index + load + play
    let song_opt = state.play_prev().await?;

    // Emit events for UI
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    if let Some(song) = song_opt {
        let _ = app.emit(
            "audio_event",
            json!({ "type": "SongChanged", "data": { "song": song } }),
        );
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub fn change_index(app: AppHandle, state: State<'_, AudioPlayer>, new_index: usize, force: bool) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.change_index(new_index, force);
    // Emit QueueChanged (explicit index change)
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    Ok(())
}