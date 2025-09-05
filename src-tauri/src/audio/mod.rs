use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Emitter, Manager, State};
use types::errors::Result;
use audio_player::AudioPlayer;
use crate::playback::spotify::make_librespot_adapter;
use database::database::Database;
use serde_json::json;
use crate::plugins::manager::PluginHandler;
use music_plugin_sdk::types::media::{ StreamRequest, StreamFormatPreference, QualityPreference };

#[tracing::instrument(level = "debug", skip(app))]
pub fn build_audio_player(app: AppHandle) -> AudioPlayer {
    let db_state: State<'_, Database> = app.state();
    let db = db_state.inner().clone();
    
    let cache_dir = app.path().app_cache_dir().expect("cache dir");
    
    #[cfg(any(target_os = "android", target_os = "ios"))]
    let mut audio_player = AudioPlayer::new_mobile(cache_dir, Arc::new(db.clone()), app.clone());
    
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let mut audio_player = AudioPlayer::new_desktop(cache_dir, Arc::new(db.clone()));
    
    if let Err(e) = audio_player.load_state(&db) {
        tracing::error!("Failed to load player state from database: {:?}", e);
    }
    if let Err(e) = audio_player.initialize_mpris() {
        tracing::error!("Failed to initialize MPRIS: {:?}", e);
    }
    
    #[cfg(any(target_os = "android", target_os = "ios"))]
    audio_player.set_mpris_app_handle(app.clone());
    
    // Sanitize initial playback state at startup to avoid stale "PLAYING" UI
    // If there's no current track or queue is empty, force STOPPED.
    // Otherwise, if state persisted as PLAYING, downgrade to PAUSED until actual playback starts.
    {
        let store_arc = audio_player.get_store();
        // Bind lock result to ensure its temporaries drop before store_arc
        let lock_res = store_arc.lock();
        if let Ok(mut store) = lock_res {
            let q_len = store.get_queue_len();
            let has_track = store.get_current_track().is_some();
            let state = store.get_player_state();
            use types::ui::player_details::PlayerState as Ps;
            if q_len == 0 || !has_track {
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

    // 注入流媒体URL解析器
    let plugin_handler: State<'_, PluginHandler> = app.state();
    let resolver = {
        let plugin_handler = plugin_handler.inner().clone();
        let app_for_headers = app.clone();
        Arc::new(move |track: &types::tracks::MediaContent| {
            // Clone captured handles per-call to avoid moving from the environment (Fn vs FnOnce)
            let plugin_handler = plugin_handler.clone();
            let app_handle = app_for_headers.clone();
            let track = track.clone();
            Box::pin(async move {
                tracing::debug!("Resolving stream URL for track: {:?}", track.track.title);
                
                // 获取插件管理器
                let plugin_manager = plugin_handler.plugin_manager();
                
                // 使用现有的方法获取音频提供者
                let selection = types::settings::music::MusicSourceSelection::default();
                let audio_providers = plugin_manager
                    .get_audio_providers_by_selection(&selection)
                    .await
                    .map_err(|e| types::errors::MusicError::String(format!("Failed to get audio providers: {}", e)))?;
                
                if audio_providers.is_empty() {
                    return Err(types::errors::MusicError::String("No audio providers found".into()));
                }
                
                // 尝试从提供者获取流媒体URL
                for (provider_id, provider_plugin) in audio_providers {
                    tracing::debug!("Trying provider: {}", provider_id);
                    
                    let track_id = track.track._id.as_ref()
                        .ok_or_else(|| types::errors::MusicError::String("No track ID found".into()))?;
                    
                    // 获取流媒体描述（格式/质量由默认 StreamRequest 指示）
                    let stream_result = {
                        let plugin_guard = provider_plugin.lock().await;
                        let req = StreamRequest {
                            format: StreamFormatPreference::Auto,
                            quality: QualityPreference::Qn(16),
                            extra: None,
                        };
                        plugin_guard.get_media_stream(track_id, &req).await
                    };
                    
                    match stream_result {
                        Ok(stream) => {
                            let stream_url = stream.url.clone();
                            // store headers for audio player prefetch
                            if let Some(headers) = stream.headers.clone() {
                                let audio_state: State<'_, AudioPlayer> = app_handle.state();
                                audio_state.set_url_headers(stream_url.clone(), headers.into_iter().collect());
                            }
                            tracing::info!("Successfully resolved stream URL from provider {}: {}", provider_id, stream_url);
                            return Ok(stream_url);
                        }
                        Err(e) => {
                            tracing::warn!("Provider {} failed to resolve stream URL: {}", provider_id, e);
                            continue;
                        }
                    }
                }
                
                Err(types::errors::MusicError::String("No provider could resolve stream URL".into()))
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>>
        })
    };
    
    audio_player.set_stream_url_resolver(resolver);
    
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

                    // Also announce current track metadata if available
                    if let Ok(store) = store_arc.lock() {
                        if let Some(track) = store.get_current_track() {
                            emit_json("TrackChanged", json!({ "track": track }));
                        }
                    }
                }
                PlayerEvents::Ended => {
                    // Track finished signal
                    emit_json("TrackFinished", json!({}));
                    
                    // 异步更新播放统计和存储（放入阻塞线程池，避免占用 async runtime）
                    if let Ok(store) = store_arc.lock() {
                        if let Some(track) = store.get_current_track() {
                            let db_state: State<'_, Database> = app_for_thread.state();
                            let db = db_state.inner().clone();
                            let track_for_storage = track.clone();
                            
                            // 在阻塞线程池中执行同步 Diesel 写操作，内部用 block_on 调用现有 async API
                            tauri::async_runtime::spawn_blocking(move || {
                                if let Some(track_id) = &track_for_storage.track._id {
                                    // 增加播放次数
                                    if let Err(e) = tauri::async_runtime::block_on(db.increment_play_count(track_id)) {
                                        tracing::warn!("Failed to increment play count for {}: {}", track_id, e);
                                    }

                                    // 如果是在线歌曲且首次播放，存储基本信息（不包含播放URL）
                                    if track_for_storage.track.provider_extension.is_some() {
                                        let mut track_for_db = track_for_storage.clone();
                                        // 清除临时的播放URL，只存储基本元数据
                                        track_for_db.track.playback_url = None;

                                        // 使用 upsert 避免重复插入
                                        if let Err(e) = tauri::async_runtime::block_on(db.upsert_track(&track_for_db)) {
                                            tracing::warn!("Failed to store track metadata for {}: {}", track_id, e);
                                        } else {
                                            tracing::debug!("Stored track metadata for online track: {}", track_id);
                                        }
                                    }
                                }
                            });
                        }
                    }
                    
                    // After store updates to next track (handled in core), announce new track
                    if let Ok(store) = store_arc.lock() {
                        if let Some(track) = store.get_current_track() {
                            emit_json("TrackChanged", json!({ "track": track }));
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
                            if let Some(mut track) = store.get_current_track() {
                                let app_clone = app_for_thread.clone();
                                tauri::async_runtime::spawn(async move {
                                    // Acquire AudioPlayer state
                                    let audio_state: State<'_, AudioPlayer> = app_clone.state();
                                    // Load the selected track and then play
                                    let _ = audio_state.audio_load(&mut track).await;
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


#[tracing::instrument(level = "debug", skip_all)]
#[tauri::command]
pub async fn audio_play(app: AppHandle, state: State<'_, AudioPlayer>, track: Option<types::tracks::MediaContent>) -> Result<()> {
    let mut track_ref = track;
    let result = state.audio_play(track_ref.as_mut()).await;

    // Emit events after successful play
    if result.is_ok() {
        // If a track was explicitly provided, use it directly to avoid any race with store updates
        if let Some(provided_track) = track_ref {
            // emit TrackChanged with the provided track
            let _ = app.emit(
                "audio_event",
                json!({ "type": "TrackChanged", "data": { "track": provided_track } }),
            );
            // Optionally also notify queue changed since explicit play may update index
            let _ = app.emit(
                "audio_event",
                json!({ "type": "QueueChanged", "data": {} }),
            );
        } else {
            // Fallback: no track provided, emit current track from store
            if let Ok(store) = state.get_store().lock() {
                if let Some(track) = store.get_current_track() {
                    let _ = app.emit(
                        "audio_event",
                        json!({ "type": "TrackChanged", "data": { "track": track } }),
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
pub fn get_current_track(state: State<'_, AudioPlayer>) -> Result<Option<types::tracks::MediaContent>> {
    let store_arc = state.get_store();
    let store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    // Compute current track from queue without mutating store to avoid side effects
    let q = store.get_queue();
    let track_opt = q
        .track_queue
        .get(q.current_index)
        .and_then(|id| q.data.get(id))
        .cloned()
        .or_else(|| store.get_current_track());
    Ok(track_opt)
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

#[tracing::instrument(level = "debug", skip(state, tracks))]
#[tauri::command]
pub fn add_to_queue(app: AppHandle, state: State<'_, AudioPlayer>, tracks: Vec<types::tracks::MediaContent>) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.add_to_queue(tracks);
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

#[tracing::instrument(level = "debug", skip(state, track))]
#[tauri::command]
pub fn play_now(app: AppHandle, state: State<'_, AudioPlayer>, track: types::tracks::MediaContent) -> Result<()> {
    let store_arc = state.get_store();
    let mut store = store_arc
        .lock()
        .map_err(|_| types::errors::MusicError::from("Failed to access player store"))?;
    store.play_now(track);
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
pub async fn next_track(app: AppHandle, state: State<'_, AudioPlayer>) -> Result<()> {
    // Delegate to core: updates index + load + play
    let track_opt = state.play_next().await?;

    // Emit events for UI
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    if let Some(track) = track_opt {
        let _ = app.emit(
            "audio_event",
            json!({ "type": "TrackChanged", "data": { "track": track } }),
        );
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip(state))]
#[tauri::command]
pub async fn prev_track(app: AppHandle, state: State<'_, AudioPlayer>) -> Result<()> {
    // Delegate to core: updates index + load + play
    let track_opt = state.play_prev().await?;

    // Emit events for UI
    let _ = app.emit(
        "audio_event",
        json!({ "type": "QueueChanged", "data": {} }),
    );
    if let Some(track) = track_opt {
        let _ = app.emit(
            "audio_event",
            json!({ "type": "TrackChanged", "data": { "track": track } }),
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
