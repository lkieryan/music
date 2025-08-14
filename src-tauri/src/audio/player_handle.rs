use std::sync::{Arc, mpsc};
use std::thread;
use tokio::runtime::Builder as TokioBuilder;
use tokio::sync::{broadcast, oneshot};
use crate::audio::player_service::PlayerService;
use database::database::Database;
use audio_player::{QueueItem, PlayMode};
use types::songs::Song;
use super::player_events::PlayerEvent;
use super::player_service::PlayerState;

// Command enum sent from UI thread to the audio runtime thread
enum AudioCmd {
    PlaySong { song: Song, resp: oneshot::Sender<Result<(), String>> },
    Pause { resp: oneshot::Sender<Result<(), String>> },
    Resume { resp: oneshot::Sender<Result<(), String>> },
    Stop { resp: oneshot::Sender<Result<(), String>> },
    Seek { position_secs: f64, resp: oneshot::Sender<Result<(), String>> },
    SetVolume { volume: f32, resp: oneshot::Sender<Result<(), String>> },
    Next { resp: oneshot::Sender<Result<(), String>> },
    Previous { resp: oneshot::Sender<Result<(), String>> },
    SetPlayMode { mode: PlayMode, resp: oneshot::Sender<Result<(), String>> },
    AddToQueue { song: Song, resp: oneshot::Sender<Result<String, String>> },
    RemoveFromQueue { index: usize, resp: oneshot::Sender<Result<QueueItem, String>> },
    GetQueue { resp: oneshot::Sender<Result<Vec<QueueItem>, String>> },
    GetStatus { resp: oneshot::Sender<Result<PlayerState, String>> },
    GetCurrentSong { resp: oneshot::Sender<Result<Option<Song>, String>> },
    ClearQueue { resp: oneshot::Sender<Result<(), String>> },
}

#[derive(Clone)]
pub struct PlayerHandle {
    tx: mpsc::Sender<AudioCmd>,
    events_tx: broadcast::Sender<PlayerEvent>,
}

impl PlayerHandle {
    pub fn spawn(database: Arc<Database>) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel::<AudioCmd>();
        let (events_tx, _events_rx) = broadcast::channel::<PlayerEvent>(128);

        // Spawn a dedicated OS thread, host a single-thread Tokio runtime inside it.
        let events_tx_clone = events_tx.clone();
        thread::Builder::new()
            .name("audio-runtime".into())
            .spawn(move || {
                // Single-thread runtime so non-Send types stay confined
                let rt = TokioBuilder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build tokio current_thread runtime");

                // Create the non-Send PlayerService within this thread
                let svc_res = PlayerService::new(database).map_err(|e| e.to_string());
                let mut svc = match svc_res {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Failed to init PlayerService: {}", e);
                        return;
                    }
                };

                // Forward internal events to the external broadcast channel
                let mut internal_rx = rt.block_on(async { svc.subscribe_events() });
                let forward_tx = events_tx_clone.clone();
                rt.spawn(async move {
                    while let Ok(ev) = internal_rx.recv().await {
                        let _ = forward_tx.send(ev);
                    }
                });

                // Process commands from the UI thread in a blocking loop
                while let Ok(cmd) = rx.recv() {
                    match cmd {
                        AudioCmd::PlaySong { song, resp } => {
                            let res = rt.block_on(async { svc.play_song(song).await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::Pause { resp } => {
                            let res = rt.block_on(async { svc.pause().await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::Resume { resp } => {
                            let res = rt.block_on(async { svc.resume().await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::Stop { resp } => {
                            let res = rt.block_on(async { svc.stop().await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::Seek { position_secs, resp } => {
                            let pos = std::time::Duration::from_secs_f64(position_secs);
                            let res = rt.block_on(async { svc.seek(pos).await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::SetVolume { volume, resp } => {
                            let res = rt.block_on(async { svc.set_volume(volume).await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::Next { resp } => {
                            let res = rt.block_on(async { svc.next().await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::Previous { resp } => {
                            let res = rt.block_on(async { svc.previous().await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::SetPlayMode { mode, resp } => {
                            let res = rt.block_on(async { svc.set_play_mode(mode).await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::AddToQueue { song, resp } => {
                            let res = rt.block_on(async { svc.add_to_queue(song).await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::RemoveFromQueue { index, resp } => {
                            let res = rt.block_on(async { svc.remove_from_queue(index).await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::GetQueue { resp } => {
                            let res: Result<Vec<QueueItem>, String> = Ok(svc.get_queue());
                            let _ = resp.send(res);
                        }
                        AudioCmd::GetStatus { resp } => {
                            let res = rt.block_on(async { svc.get_status().await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::GetCurrentSong { resp } => {
                            let res = rt.block_on(async { svc.get_status().await.map_err(|e| e.to_string()).map(|s| s.current_song) });
                            let _ = resp.send(res);
                        }
                        AudioCmd::ClearQueue { resp } => {
                            let res = rt.block_on(async { svc.clear_queue().await.map_err(|e| e.to_string()) });
                            let _ = resp.send(res);
                        }
                    }
                }
            })
            .map_err(|e| format!("failed to spawn audio runtime thread: {}", e))?;

        Ok(PlayerHandle { tx, events_tx })
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<PlayerEvent> {
        self.events_tx.subscribe()
    }

    // Async API used by Tauri commands
    pub async fn play_song(&self, song: Song) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::PlaySong { song, resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn pause(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::Pause { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn resume(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::Resume { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn stop(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::Stop { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn seek(&self, position: std::time::Duration) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::Seek { position_secs: position.as_secs_f64(), resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn set_volume(&self, volume: f32) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::SetVolume { volume, resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn next(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::Next { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn previous(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::Previous { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn set_play_mode(&self, mode: PlayMode) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::SetPlayMode { mode, resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn add_to_queue(&self, song: Song) -> Result<String, String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::AddToQueue { song, resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn remove_from_queue(&self, index: usize) -> Result<QueueItem, String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::RemoveFromQueue { index, resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn get_queue(&self) -> Result<Vec<QueueItem>, String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::GetQueue { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn get_status(&self) -> Result<PlayerState, String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::GetStatus { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn get_current_song(&self) -> Result<Option<Song>, String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::GetCurrentSong { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
    pub async fn clear_queue(&self) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.tx.send(AudioCmd::ClearQueue { resp: tx }).map_err(|_| "audio runtime closed".to_string())?;
        rx.await.map_err(|_| "audio runtime closed".to_string())?
    }
}
