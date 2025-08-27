// crates/audio-player/src/events.rs
// Centralized event handling for AudioPlayer to improve readability and avoid duplication.
// NOTE: This module intentionally mirrors existing behaviors in core.rs without changing logic.

use std::sync::Arc;

use crate::store::PlayerStore;
use types::tracks::MediaContent;
use types::ui::player_details::{PlayerEvents, PlayerState, PlayerMode};

/// Hooks used when UI/MPRIS callbacks are needed during event application.
pub struct EventHooks {
    /// Called when playback state changes
    pub on_state: Option<Arc<dyn Fn(PlayerState) + Send + Sync>>,
    /// Called when current track metadata should be announced
    pub on_metadata: Option<Arc<dyn Fn(&MediaContent) + Send + Sync>>,
    /// Called when position/time updates
    pub on_position: Option<Arc<dyn Fn(f64) + Send + Sync>>,
}

impl Default for EventHooks {
    fn default() -> Self {
        Self { on_state: None, on_metadata: None, on_position: None }
    }
}

/// Apply player event in the same way as create_player_event_handler() used to do.
/// This function does NOT handle PlayerEvents::Error, because error handling may
/// depend on caller context (e.g., blacklisting policy based on player key).
pub fn apply_event_basic(store: &mut PlayerStore, ev: &PlayerEvents) {
    match ev {
        PlayerEvents::Play => {
            store.set_state(PlayerState::Playing);
        }
        PlayerEvents::Pause => {
            store.set_state(PlayerState::Paused);
        }
        PlayerEvents::Loading => {
            store.set_state(PlayerState::Loading);
        }
        PlayerEvents::Ended => {
            handle_playback_ended_basic(store);
        }
        PlayerEvents::TimeUpdate(time) => {
            store.update_time(*time);
        }
        PlayerEvents::Error(_) => {
            // Intentionally left for caller to handle
        }
    }
}

/// Apply player event with hooks, matching the behavior inside audio_load()'s closure.
/// This version triggers metadata/state/position callbacks when appropriate.
/// It also does NOT handle PlayerEvents::Error; the caller must handle it.
pub fn apply_event_with_hooks(store: &mut PlayerStore, ev: &PlayerEvents, hooks: &EventHooks) {
    match ev {
        PlayerEvents::Play => {
            store.set_state(PlayerState::Playing);
            if let Some(cb) = &hooks.on_state { cb(PlayerState::Playing); }
        }
        PlayerEvents::Pause => {
            store.set_state(PlayerState::Paused);
            if let Some(cb) = &hooks.on_state { cb(PlayerState::Paused); }
        }
        PlayerEvents::Loading => {
            store.set_state(PlayerState::Loading);
            if let Some(cb) = &hooks.on_state { cb(PlayerState::Loading); }
        }
        PlayerEvents::Ended => {
            match store.get_repeat() {
                PlayerMode::Sequential => {
                    // Normal sequential playback: go to next track, stop if at end
                    if store.data.queue.current_index + 1 >= store.data.queue.track_queue.len() {
                        // Reached end of queue, stop playback
                        store.set_state(PlayerState::Stopped);
                        if let Some(cb) = &hooks.on_state { cb(PlayerState::Stopped); }
                    } else {
                        store.next_track();
                        store.set_state(PlayerState::Playing);
                        if let Some(track) = store.get_current_track() {
                            if let Some(cb) = &hooks.on_metadata { cb(&track); }
                        }
                        if let Some(cb) = &hooks.on_state { cb(PlayerState::Playing); }
                    }
                }
                PlayerMode::Single => {
                    // Single track repeat: always repeat current track
                    let idx = store.data.queue.current_index;
                    store.change_index(idx, true);
                    store.set_state(PlayerState::Playing);
                    if let Some(cb) = &hooks.on_state { cb(PlayerState::Playing); }
                }
                PlayerMode::Shuffle => {
                    // Random playback: get next shuffled index
                    if let Some(next_idx) = store.get_next_shuffle_index() {
                        store.change_index(next_idx, true);
                        store.set_state(PlayerState::Playing);
                        if let Some(track) = store.get_current_track() {
                            if let Some(cb) = &hooks.on_metadata { cb(&track); }
                        }
                        if let Some(cb) = &hooks.on_state { cb(PlayerState::Playing); }
                    } else {
                        // Empty queue or single track, treat as single repeat
                        let idx = store.data.queue.current_index;
                        store.change_index(idx, true);
                        store.set_state(PlayerState::Playing);
                        if let Some(cb) = &hooks.on_state { cb(PlayerState::Playing); }
                    }
                }
                PlayerMode::ListLoop => {
                    // List loop: go to next track, wrap to beginning if at end
                    if store.data.queue.current_index + 1 >= store.data.queue.track_queue.len() {
                        // Wrap to beginning
                        store.change_index(0, true);
                    } else {
                        store.next_track();
                    }
                    store.set_state(PlayerState::Playing);
                    if let Some(track) = store.get_current_track() {
                        if let Some(cb) = &hooks.on_metadata { cb(&track); }
                    }
                    if let Some(cb) = &hooks.on_state { cb(PlayerState::Playing); }
                }
            }
        }
        PlayerEvents::TimeUpdate(time) => {
            store.update_time(*time);
            if let Some(cb) = &hooks.on_position { cb(*time); }
        }
        PlayerEvents::Error(_) => {
            // Intentionally left for caller to handle
        }
    }
}

// --- internal helpers ---

/// Mirrors core::handle_playback_ended() logic exactly for the basic path.
fn handle_playback_ended_basic(store: &mut PlayerStore) {
    match store.get_repeat() {
        PlayerMode::Sequential => {
            if store.data.queue.current_index + 1 >= store.data.queue.track_queue.len() {
                store.set_state(PlayerState::Stopped);
            } else {
                store.next_track();
                store.set_state(PlayerState::Playing);
            }
        }
        PlayerMode::Single => {
            store.change_index(store.data.queue.current_index, true);
            store.set_state(PlayerState::Playing);
        }
        PlayerMode::Shuffle => {
            if let Some(next_idx) = store.get_next_shuffle_index() {
                store.change_index(next_idx, true);
                store.set_state(PlayerState::Playing);
            } else {
                store.change_index(store.data.queue.current_index, true);
                store.set_state(PlayerState::Playing);
            }
        }
        PlayerMode::ListLoop => {
            if store.data.queue.current_index + 1 >= store.data.queue.track_queue.len() {
                store.change_index(0, true);
            } else {
                store.next_track();
            }
            store.set_state(PlayerState::Playing);
        }
    }
}
