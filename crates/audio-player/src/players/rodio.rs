use std::{
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread,
    time::Duration,
    fs::File,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use tracing::{trace, debug, info, error};
use types::{errors::{Result, error_helpers}, tracks::{TrackType}, ui::player_details::PlayerEvents};
use stream_download::{StreamDownload, Settings};
use stream_download::storage::temp::TempStorageProvider;
use hls_client::{config::ConfigBuilder, stream::HLSStream};
use rodio::Sink;

use super::base::{BasePlayer, PlayerEventsSender};

// Supported track types for Rodio backend (DASH handled by dash backend)
static PROVIDES: [TrackType; 3] = [TrackType::LOCAL, TrackType::URL, TrackType::HLS];

#[derive(Debug, Clone)]
pub struct RodioPlayer {
    tx: Sender<RodioCommand>,
    events_rx: Arc<Mutex<Receiver<PlayerEvents>>>,
    // comments: ensure forwarding thread is spawned only once
    forward_started: Arc<AtomicBool>,
    // playback state tracking for periodic TimeUpdate
    playing: Arc<AtomicBool>,
    position: Arc<Mutex<f64>>, // seconds
}

#[derive(Debug, Clone)]
enum RodioCommand {
    SetSrc(String),
    Play,
    Pause,
    Stop,
    SetVolume(f64),
    Seek(u64),
}

impl RodioPlayer {

    #[tracing::instrument(level = "debug", skip(cache_dir))]
    pub fn new(cache_dir: PathBuf) -> Self {
        let (events_tx, events_rx) = unbounded::<PlayerEvents>();
        let cache_dir = cache_dir.join("rodio");
        if !cache_dir.exists() {
            std::fs::create_dir(cache_dir.clone()).unwrap();
        }
        // shared state
        let playing = Arc::new(AtomicBool::new(false));
        let position = Arc::new(Mutex::new(0.0f64));

        let tx = Self::initialize(events_tx, cache_dir, playing.clone(), position.clone());
        Self {
            tx,
            events_rx: Arc::new(Mutex::new(events_rx)),
            forward_started: Arc::new(AtomicBool::new(false)),
            playing,
            position,
        }
    }

    async fn set_src(cache_dir: PathBuf, src: String, sink: &Arc<Sink>) -> Result<()> {
        if src.ends_with(".m3u8") || src.contains(".m3u8") {
            Self::handle_hls_stream(cache_dir.clone(), &src, sink).await?;
        } else if src.starts_with("http") {
            Self::handle_http_stream(cache_dir.clone(), &src, sink).await?;
        } else {
            Self::handle_local_file(&src, sink).await?;
        }

        Ok(())
    }

    async fn handle_hls_stream(cache_dir: PathBuf, src: &str, sink: &Arc<Sink>) -> Result<()> {
        let reader = StreamDownload::new::<HLSStream>(
            ConfigBuilder::new().url(src).map_err(error_helpers::to_playback_error)?.build().map_err(error_helpers::to_playback_error)?,
            TempStorageProvider::new_in(cache_dir.clone()),
            Settings::default(),
        )
        .await
        .map_err(error_helpers::to_playback_error)?;

        info!("HLS Stream content length {:?}", reader.content_length());
        trace!("Stream created");

        let decoder = rodio::Decoder::new(reader).map_err(error_helpers::to_playback_error)?;
        trace!("Decoder created");
        sink.append(decoder);
        trace!("Decoder appended");

        Ok(())
    }

    async fn handle_http_stream(cache_dir: PathBuf, src: &str, sink: &Arc<Sink>) -> Result<()> {
        trace!("Creating HTTP stream");

        match StreamDownload::new_http(
            src.parse().unwrap(),
            TempStorageProvider::new_in(cache_dir.clone()),
            Settings::default()
                .on_progress(move |_cl, state, _c| {
                    tracing::debug!("Progress: {}", state.current_position)
                })
                .prefetch_bytes(512),
        )
        .await
        {
            Ok(reader) => {
                trace!("Stream created");

                let decoder = rodio::Decoder::new(reader).map_err(error_helpers::to_playback_error)?;
                trace!("Decoder created");
                sink.append(decoder);
                trace!("Decoder appended");

                Ok(())
            }
            Err(e) => Err(e.to_string().into()),
        }
    }

    async fn handle_local_file(src: &str, sink: &Arc<Sink>) -> Result<()> {
        let path = PathBuf::from_str(src).unwrap();
        if path.exists() {
            let file = File::open(path)?;
            let decoder = rodio::Decoder::try_from(file).map_err(error_helpers::to_playback_error)?;
            sink.append(decoder);

            trace!("Local file {} appended", src);

            return Ok(());
        }

        Err("Failed to read local file".into())
    }

    pub fn get_events_rx(&self) -> Arc<Mutex<Receiver<PlayerEvents>>> {
        self.events_rx.clone()
    }

    fn send_event(events_tx: Sender<PlayerEvents>, event: PlayerEvents) {
        events_tx.send(event).unwrap();
    }

    fn initialize(
        events_tx: Sender<PlayerEvents>,
        cache_dir: PathBuf,
        playing_flag: Arc<AtomicBool>,
        position_ref: Arc<Mutex<f64>>,
    ) -> Sender<RodioCommand> {
        let (tx, rx) = unbounded::<RodioCommand>();
        let ret = tx.clone();

        thread::spawn(move || {
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
            let sink = Arc::new(rodio::Sink::connect_new(stream_handle.mixer()));

            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            let events_tx = events_tx.clone();
            runtime.block_on(async move {
                let last_src = Arc::new(Mutex::new(None));

                // periodic timer for TimeUpdate
                let ticker_events = events_tx.clone();
                let ticker_playing = playing_flag.clone();
                let ticker_pos = position_ref.clone();
                thread::spawn(move || {
                    loop {
                        thread::sleep(Duration::from_millis(500));
                        if ticker_playing.load(Ordering::SeqCst) {
                            // increment position ~0.5s
                            let mut pos = ticker_pos.lock().unwrap();
                            *pos += 0.5;
                            // fire event
                            RodioPlayer::send_event(
                                ticker_events.clone(),
                                PlayerEvents::TimeUpdate(*pos),
                            );
                        }
                    }
                });
                while let Ok(command) = rx.recv() {
                    let sink = sink.clone();

                    match command {
                        RodioCommand::SetSrc(src) => {
                            let last_src = last_src.clone();
                            {
                                let mut last_src = last_src.lock().unwrap();
                                *last_src = Some(src.clone());
                            }

                            sink.clear();
                            // reset tracking state on new source
                            {
                                let mut p = position_ref.lock().unwrap();
                                *p = 0.0;
                            }
                            playing_flag.store(false, Ordering::SeqCst);
                            Self::send_event(events_tx.clone(), PlayerEvents::TimeUpdate(0f64));
                            Self::send_event(events_tx.clone(), PlayerEvents::Loading);

                            // TODO
                            if let Err(err) =
                                Self::set_src(cache_dir.clone(), src.clone(), &sink).await
                            {
                                error!("Failed to set src: {:?}", err);
                                Self::send_event(events_tx.clone(), PlayerEvents::Error(err))
                            } else {
                                debug!("Set src");
                                let src_clone = src.clone();

                                let events_tx = events_tx.clone();
                                let sink = sink.clone();
                                // clone playing flag for move into thread
                                let ended_playing_flag = playing_flag.clone();

                                // Send ended event only if track hasn't changed yet
                                thread::spawn(move || {
                                    sink.sleep_until_end();
                                    let last_src = last_src.clone();
                                    let last_src = last_src.lock().unwrap();
                                    if let Some(last_src) = last_src.clone() {
                                        info!("last src={}, current src={}", last_src, src_clone);
                                        if last_src == src_clone {
                                            // stop ticker when ended
                                            ended_playing_flag.store(false, Ordering::SeqCst);
                                            Self::send_event(
                                                events_tx.clone(),
                                                PlayerEvents::Ended,
                                            );
                                        }
                                    }
                                });
                            }
                        }
                        RodioCommand::Play => {
                            if !sink.empty() {
                                sink.play();
                                // start ticker
                                playing_flag.store(true, Ordering::SeqCst);
                                Self::send_event(events_tx.clone(), PlayerEvents::Play)
                            }
                        }
                        RodioCommand::Pause => {
                            if !sink.empty() {
                                sink.pause();
                                playing_flag.store(false, Ordering::SeqCst);
                                Self::send_event(events_tx.clone(), PlayerEvents::Pause)
                            }
                        }
                        RodioCommand::Stop => {
                            if !sink.empty() {
                                sink.stop();
                                sink.clear();
                                playing_flag.store(false, Ordering::SeqCst);
                                // reset position on stop
                                {
                                    let mut p = position_ref.lock().unwrap();
                                    *p = 0.0;
                                }
                                Self::send_event(events_tx.clone(), PlayerEvents::Pause)
                            }
                        }
                        RodioCommand::SetVolume(volume) => {
                            if !sink.empty() {
                                sink.set_volume(volume as f32);
                            }
                        }
                        RodioCommand::Seek(pos) => {
                            if !sink.empty() {
                                if let Err(err) = sink.try_seek(Duration::from_secs(pos)) {
                                    error!("Failed to seek: {:?}", err)
                                } else {
                                    // update tracked position
                                    {
                                        let mut p = position_ref.lock().unwrap();
                                        *p = pos as f64;
                                    }
                                    Self::send_event(
                                        events_tx.clone(),
                                        PlayerEvents::TimeUpdate(pos as f64),
                                    )
                                }
                            } else {
                                let last_src = last_src.clone();
                                let last_src = last_src.lock().unwrap();
                                if let Some(last_src) = last_src.clone() {
                                    tx.send(RodioCommand::SetSrc(last_src.clone())).unwrap();
                                    tx.send(RodioCommand::Seek(pos)).unwrap();
                                    tx.send(RodioCommand::Play).unwrap();
                                }
                            }
                        }
                    }
                }
            });
        });

        ret
    }
}

impl BasePlayer for RodioPlayer {
    #[tracing::instrument(level = "debug", skip(self))]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn key(&self) -> String {
        "rodio".into()
    }

    #[tracing::instrument(level = "debug", skip(self, src, resolver))]
    fn load(&self, src: String, _autoplay: bool, resolver: tokio::sync::oneshot::Sender<()>) {
        let _ = self.tx.send(RodioCommand::SetSrc(src.clone()));
        // Resolve immediately to avoid blocking caller
        let _ = resolver.send(());
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&mut self) -> types::errors::Result<()> {
        self.tx.send(RodioCommand::Stop).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn play(&self) -> types::errors::Result<()> {
        self.tx.send(RodioCommand::Play).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn pause(&self) -> types::errors::Result<()> {
        self.tx.send(RodioCommand::Pause).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, pos))]
    fn seek(&self, pos: f64) -> types::errors::Result<()> {
        self.tx
        .send(RodioCommand::Seek(pos.abs().round() as u64))
        .unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn provides(&self) -> &[types::tracks::TrackType] { &PROVIDES }

    #[tracing::instrument(level = "debug", skip(self, _track))]
    fn can_play(&self, _track: &types::tracks::MediaContent) -> bool { true }

    #[tracing::instrument(level = "debug", skip(self, volume))]
    fn set_volume(&self, volume: f64) -> types::errors::Result<()> {
        self.tx.send(RodioCommand::SetVolume(volume)).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_volume(&self) -> types::errors::Result<f64> { Ok(0f64) }

    #[tracing::instrument(level = "debug", skip(self, _state_setter))]
    fn add_listeners(&mut self, _state_setter: PlayerEventsSender) {
        // comments: start forwarding only once
        if self
            .forward_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        // comments: Bridge internal Rodio events to the upstream state_setter
        let rx = self.events_rx.clone();
        let player_key = self.key();

        std::thread::spawn(move || {
            let rx_guard = rx.lock().expect("lock rodio events_rx");
            while let Ok(ev) = rx_guard.recv() {
                _state_setter(player_key.clone(), ev);
            }
        });
    }
}
