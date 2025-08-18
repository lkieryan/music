use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::any::Any;

use types::{
    errors::Result,
    songs::{Song, SongType},
};

use super::base::{BasePlayer, PlayerEventsSender};

// Backend control callbacks. All fields are optional and can be injected by the integrator.
#[derive(Default, Clone)]
pub struct LibrespotAdapter {
    pub load: Option<Arc<dyn Fn(String, bool, tokio::sync::oneshot::Sender<()>) + Send + Sync>>,
    pub play: Option<Arc<dyn Fn() -> Result<()> + Send + Sync>>,
    pub pause: Option<Arc<dyn Fn() -> Result<()> + Send + Sync>>,
    pub seek: Option<Arc<dyn Fn(f64) -> Result<()> + Send + Sync>>,
    pub stop: Option<Arc<dyn Fn() -> Result<()> + Send + Sync>>,
    pub set_volume: Option<Arc<dyn Fn(f64) -> Result<()> + Send + Sync>>,
    pub get_volume: Option<Arc<dyn Fn() -> Result<f64> + Send + Sync>>,
}

#[derive(Clone, Default)]
pub struct LibrespotPlayer {
    // Backend event callback to report player state
    player_state_tx: Option<PlayerEventsSender>,

    // Backend controls injected from the system integration layer
    adapter: Arc<LibrespotAdapter>,

    // Basic runtime state placeholders
    enabled: Arc<AtomicBool>,
    initialized: Arc<AtomicBool>,
    volume: Arc<Mutex<f64>>, // store linear volume in [0.0, 1.0]
}

impl std::fmt::Debug for LibrespotPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibrespotPlayer").finish()
    }
}

impl LibrespotPlayer {
    /// Create a new adapter instance
    pub fn new() -> Self {
        Self {
            player_state_tx: None,
            adapter: Arc::new(LibrespotAdapter::default()),
            enabled: Arc::new(AtomicBool::new(true)),
            initialized: Arc::new(AtomicBool::new(false)),
            volume: Arc::new(Mutex::new(1.0)),
        }
    }

    /// Inject backend control callbacks
    /// NOTE: Caller should ensure callbacks are valid for the player's lifetime
    pub fn with_adapter(mut self, adapter: LibrespotAdapter) -> Self {
        self.adapter = Arc::new(adapter);
        self
    }

    /// Set backend callbacks on an existing instance
    pub fn set_adapter(&mut self, adapter: LibrespotAdapter) {
        self.adapter = Arc::new(adapter);
    }
}

static PROVIDES: [SongType; 1] = [SongType::SPOTIFY];

impl BasePlayer for LibrespotPlayer {
    #[tracing::instrument(level = "debug", skip(self))]
    fn initialize(&self) {
        self.initialized.store(true, Ordering::SeqCst);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn key(&self) -> String { "spotify".into() }

    #[tracing::instrument(level = "debug", skip(self, src, resolver))]
    fn load(&self, src: String, autoplay: bool, resolver: tokio::sync::oneshot::Sender<()>) {
        if let Some(cb) = &self.adapter.load { let _ = cb(src, autoplay, resolver); return; }
        // Fallback: resolve immediately to keep UI unblocked
        let _ = resolver.send(());
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&mut self) -> Result<()> {
        if let Some(cb) = &self.adapter.stop { return cb(); }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn play(&self) -> Result<()> {
        if let Some(cb) = &self.adapter.play { return cb(); }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn pause(&self) -> Result<()> {
        if let Some(cb) = &self.adapter.pause { return cb(); }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, pos))]
    fn seek(&self, pos: f64) -> Result<()> {
        if let Some(cb) = &self.adapter.seek { return cb(pos); }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn provides(&self) -> &[SongType] { &PROVIDES }

    #[tracing::instrument(level = "debug", skip(self, song))]
    fn can_play(&self, song: &Song) -> bool { song.song.type_ == SongType::SPOTIFY }

    #[tracing::instrument(level = "debug", skip(self, volume))]
    fn set_volume(&self, volume: f64) -> Result<()> {
        let vol = volume.clamp(0.0, 1.0);
        if let Some(cb) = &self.adapter.set_volume { cb(vol)?; }
        if let Ok(mut v) = self.volume.lock() { *v = vol; }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_volume(&self) -> Result<f64> {
        if let Some(cb) = &self.adapter.get_volume { return cb(); }
        Ok(*self.volume.lock().unwrap_or_else(|e| e.into_inner()))
    }

    #[tracing::instrument(level = "debug", skip(self, tx))]
    fn add_listeners(&mut self, tx: PlayerEventsSender) { self.player_state_tx = Some(tx); }

    #[tracing::instrument(level = "debug", skip(self, opaque))]
    fn configure(&mut self, key: &str, opaque: &dyn Any) {
        // Inject backend callbacks when asked
        if key == "spotify.adapter" {
            if let Some(adapter) = opaque.downcast_ref::<LibrespotAdapter>() {
                self.set_adapter(adapter.clone());
            }
        }
    }
}
