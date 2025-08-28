// MPRIS-related methods extracted from core.rs
// This module keeps all MPRIS integration and notifications in one place.

use std::sync::{Arc, Mutex};

use types::errors::Result;
use types::tracks::MediaContent;
use types::ui::player_details::{PlayerEvents, PlayerState};
use types::mpris::MprisPlayerDetails;

use ::mpris; // external crate or root module providing MprisHolder and MediaControlEvent

use crate::AudioPlayer;

impl AudioPlayer {
    /// Initialize MPRIS integration
    pub fn initialize_mpris(&mut self) -> Result<()> {
        match mpris::MprisHolder::new() {
            Ok(holder) => {
                self.mpris_holder = Some(holder);
                tracing::info!("MPRIS initialized successfully");
                Ok(())
            }
            Err(e) => {
                tracing::warn!(
                    "MPRIS initialization failed (expected in headless environments): {:?}",
                    e
                );
                Ok(()) // Don't fail the entire player if MPRIS is unavailable
            }
        }
    }

    /// Set MPRIS app handle for mobile platforms
    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub fn set_mpris_app_handle(&mut self, app_handle: tauri::AppHandle) {
        if let Some(ref mpris) = self.mpris_holder {
            mpris.set_app_handle(app_handle);
        }
    }

    /// Start MPRIS event listener
    pub fn start_mpris_event_listener(&self) -> Option<std::thread::JoinHandle<()>> {
        if let Some(ref mpris) = self.mpris_holder {
            let event_rx = mpris.event_rx.clone();
            let events_tx = self.events_tx.clone();

            Some(std::thread::spawn(move || {
                loop {
                    if let Ok(rx) = event_rx.lock() {
                        match rx.recv() {
                            Ok(event) => {
                                tracing::debug!("Received MPRIS event: {:?}", event);
                                match event {
                                    mpris::MediaControlEvent::Play => {
                                        let _ = events_tx.send(PlayerEvents::Play);
                                    }
                                    mpris::MediaControlEvent::Pause => {
                                        let _ = events_tx.send(PlayerEvents::Pause);
                                    }
                                    mpris::MediaControlEvent::Toggle => {
                                        tracing::debug!("MPRIS toggle event received");
                                    }
                                    mpris::MediaControlEvent::Stop => {
                                        let _ = events_tx.send(PlayerEvents::Pause);
                                    }
                                    mpris::MediaControlEvent::Next => {
                                        tracing::debug!("MPRIS next event received");
                                        // TODO: Implement next track logic
                                    }
                                    mpris::MediaControlEvent::Previous => {
                                        tracing::debug!("MPRIS previous event received");
                                        // TODO: Implement previous track logic
                                    }
                                    mpris::MediaControlEvent::SetPosition(pos) => {
                                        tracing::debug!("MPRIS seek event: {:?}", pos);
                                        // TODO: Implement seek logic
                                    }
                                    _ => {
                                        tracing::debug!("Unhandled MPRIS event: {:?}", event);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!("MPRIS event listener error: {:?}", e);
                                break;
                            }
                        }
                    }
                }
                tracing::info!("MPRIS event listener stopped");
            }))
        } else {
            None
        }
    }

    /// Notify MPRIS of metadata changes
    pub fn notify_mpris_metadata(&self, track: &MediaContent) {
        // Use direct MPRIS integration if available
        if let Some(ref mpris) = self.mpris_holder {
            let metadata = MprisPlayerDetails {
                id: track.track._id.clone(),
                title: track.track.title.clone(),
                artist_name: Some(
                    track.artists
                        .as_ref()
                        .map(|artists| {
                            artists
                                .iter()
                                .filter_map(|artist| artist.artist_name.as_ref())
                                .cloned()
                                .collect::<Vec<String>>()
                                .join(", ")
                        })
                        .unwrap_or_else(|| "Unknown Artist".to_string()),
                ),
                album_name: track.album.as_ref().and_then(|a| a.album_name.clone()),
                album_artist: track.album.as_ref().and_then(|a| a.album_artist.clone()),
                genres: track
                    .genre
                    .as_ref()
                    .map(|genres| {
                        genres
                            .iter()
                            .filter_map(|g| g.genre_name.clone())
                            .collect::<Vec<String>>()
                    }),
                duration: track.track.duration,
                thumbnail: track
                    .track
                    .track_cover_path_high
                    .clone()
                    .or_else(|| track.track.track_cover_path_low.clone()),
            };

            if let Err(_e) = mpris.set_metadata(metadata) {
                // tracing::debug!("MPRIS metadata update failed (expected in headless)");
            } else {
                tracing::debug!("Updated MPRIS metadata for: {:?}", track.track.title);
            }
        }
    }

    /// Notify MPRIS of state changes
    pub fn notify_mpris_state(&self, state: PlayerState) {
        // Use direct MPRIS integration if available
        if let Some(ref mpris) = self.mpris_holder {
            if let Err(_e) = mpris.set_playback_state(state) {
                tracing::debug!(
                    "MPRIS playback state update failed (expected in headless)"
                );
            } else {
                tracing::trace!("Updated MPRIS playback state: {:?}", state);
            }
        }
    }

    /// Notify MPRIS of position changes
    pub fn notify_mpris_position(&self, position: f64) {
        // Use direct MPRIS integration if available
        if let Some(ref mpris) = self.mpris_holder {
            if let Err(_e) = mpris.set_position(position) {
                tracing::debug!(
                    "MPRIS position update failed (expected in headless)"
                );
            } else {
                tracing::trace!("Updated MPRIS position: {:.2}s", position);
            }
        }
    }
}
