use serde::Deserialize;
use tokio::sync::oneshot::Sender as OneShotSender;
use types::{errors::Result, songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen::JsValue;

use crate::utils::{
    common::listen_plugin_event,
    invoke::{mobile_load, mobile_pause, mobile_play, mobile_seek, mobile_stop},
};

use super::base::{BasePlayer, PlayerEventsSender};

#[derive(Deserialize)]
struct TimeChangeEvent {
    key: String,
    pos: f64,
}

#[derive(Deserialize)]
struct KeyEvent {
    key: String,
}

#[derive(Clone)]
pub struct MobilePlayer {
    key: String,
    listeners: Vec<js_sys::Function>,
    event_tx: Option<PlayerEventsSender>,
}

macro_rules! listen_event {
    ($self:expr, $tx:expr, $event:expr, $typ:ident, $handler:expr) => {{
        let key = $self.key.clone();
        let unlisten = listen_plugin_event("audioplayer", $event, move |evt| {
            let tx = $tx.clone();
            let key = key.clone();
            let data = serde_wasm_bindgen::from_value::<$typ>(evt).unwrap();
            if data.key == key {
                spawn_local(async move {
                    let val = $handler(data);
                    let _ = tx(key, val);
                    // if let Err(res) = res {
                    //     console_log!("Error sending event: {:?}", res);
                    // }
                });
            }
        });
        $self.listeners.push(unlisten);
    }};
}

macro_rules! generate_event_listeners {
    ($($method:tt => ($event:expr, $typ:ident) => $handler:expr),*) => {
        $(
            fn $method(&mut self, tx: PlayerEventsSender) {
                listen_event!(self, tx, $event, $typ, $handler);
            }
        )*
    };
}

impl std::fmt::Debug for MobilePlayer {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalPlayer").finish()
    }
}

impl MobilePlayer {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new(key: String) -> Self {
        MobilePlayer {
            key,
            listeners: vec![],
            event_tx: None,
        }
    }

    generate_event_listeners!(
        listen_onplay => ("onPlay", KeyEvent) => |_| PlayerEvents::Play,
        listen_onpause => ("onPause", KeyEvent) => |_| PlayerEvents::Pause,
        listen_onended => ("onSongEnded", KeyEvent) => |_| PlayerEvents::Ended,
        listen_ontimeupdate => ("onTimeChange", TimeChangeEvent) => |evt: TimeChangeEvent|{
            PlayerEvents::TimeUpdate(evt.pos / 1000f64)
        }
    );
}

impl BasePlayer for MobilePlayer {
    #[tracing::instrument(level = "debug", skip(self))]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn key(&self) -> String {
        self.key.clone()
    }

    #[tracing::instrument(level = "debug", skip(self, src, resolver))]
    fn load(&self, src: String, autoplay: bool, resolver: OneShotSender<()>) {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn play(&self) -> Result<()> {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn pause(&self) -> Result<()> {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn provides(&self) -> &[SongType] {}

    #[tracing::instrument(level = "debug", skip(self, _volume))]
    fn set_volume(&self, _volume: f64) -> Result<()> {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_volume(&self) -> Result<f64> {}

    #[tracing::instrument(level = "debug", skip(self, tx))]
    fn add_listeners(&mut self, tx: PlayerEventsSender) {}

    #[tracing::instrument(level = "debug", skip(self, pos))]
    fn seek(&self, pos: f64) -> Result<()> {}

    #[tracing::instrument(level = "debug", skip(self, song))]
    fn can_play(&self, song: &types::songs::Song) -> bool {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&mut self) -> Result<()> {}
}
