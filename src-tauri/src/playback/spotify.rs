use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};
use types::errors::Result;

use audio_player::players::librespot::LibrespotAdapter;

#[derive(Debug, Default)]
pub(crate) struct SpotifySessionState {
    pub initialized: bool,
    pub logged_in: bool,
    pub user_name: Option<String>,
    pub volume: f32, // linear [0.0, 1.0]
}

#[tracing::instrument(level = "debug", skip(app))]
pub(crate) fn get_spotify_state(app: AppHandle) -> Arc<Mutex<SpotifySessionState>> {
    if let Some(state) = app.try_state::<Arc<Mutex<SpotifySessionState>>>() {
        return state.inner().clone();
    }
    let state = Arc::new(Mutex::new(SpotifySessionState::default()));
    app.manage(state.clone());
    state
}

#[tracing::instrument(level = "debug", skip(app))]
pub(crate) fn spotify_initialize(app: AppHandle) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let mut s = state.lock().unwrap();
    s.initialized = true;
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app, account_id))]
pub(crate) fn spotify_login(app: AppHandle, account_id: String) -> Result<String> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let mut s = state.lock().unwrap();
    s.logged_in = true;
    s.user_name = Some(account_id.clone());
    Ok(account_id)
}

#[tracing::instrument(level = "debug", skip(app, _code))]
pub(crate) fn spotify_authorize(app: AppHandle, _code: String) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let _s = state.lock().unwrap();
    // TODO: Exchange code for tokens; persist securely
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app, _src))]
pub(crate) fn spotify_load(app: AppHandle, _src: String, _autoplay: bool) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let _s = state.lock().unwrap();
    // TODO: Resolve URI, prepare session track, optionally autoplay
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app))]
pub(crate) fn spotify_play(app: AppHandle) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let _s = state.lock().unwrap();
    // TODO: Resume playback on session
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app))]
pub(crate) fn spotify_pause(app: AppHandle) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let _s = state.lock().unwrap();
    // TODO: Pause playback on session
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app, pos))]
pub(crate) fn spotify_seek(app: AppHandle, pos: f64) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let _s = state.lock().unwrap();
    let _ = pos; // seconds
    // TODO: Seek to position
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app, volume))]
pub(crate) fn spotify_set_volume(app: AppHandle, volume: f32) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let mut s = state.lock().unwrap();
    s.volume = volume.clamp(0.0, 1.0);
    // TODO: Apply to session/device
    Ok(())
}

#[tracing::instrument(level = "debug", skip(app))]
pub(crate) fn spotify_get_volume(app: AppHandle) -> Result<f32> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let s = state.lock().unwrap();
    Ok(s.volume)
}

#[tracing::instrument(level = "debug", skip(app))]
pub(crate) fn spotify_stop(app: AppHandle) -> Result<()> {
    let state: State<'_, Arc<Mutex<SpotifySessionState>>> = app.state();
    let _s = state.lock().unwrap();
    // TODO: Stop playback and teardown track
    Ok(())
}

/// Build adapter closures for LibrespotPlayer. Internal only.
#[tracing::instrument(level = "debug", skip(app))]
pub(crate) fn make_librespot_adapter(app: AppHandle) -> LibrespotAdapter {
    use tokio::sync::oneshot::Sender;

    let app1 = app.clone();
    let load_cb = move |src: String, autoplay: bool, resolver: Sender<()>| {
        let app = app1.clone();
        // NOTE: run in background; resolve immediately for now
        let _ = spotify_load(app.clone(), src, autoplay);
        let _ = resolver.send(());
    };

    let app2 = app.clone();
    let play_cb = move || spotify_play(app2.clone());
    let app3 = app.clone();
    let pause_cb = move || spotify_pause(app3.clone());
    let app4 = app.clone();
    let seek_cb = move |pos: f64| spotify_seek(app4.clone(), pos);
    let app5 = app.clone();
    let stop_cb = move || spotify_stop(app5.clone());
    let app6 = app.clone();
    let set_volume_cb = move |vol: f64| spotify_set_volume(app6.clone(), vol as f32);
    let app7 = app.clone();
    let get_volume_cb = move || spotify_get_volume(app7.clone()).map(|v| v as f64);

    LibrespotAdapter {
        load: Some(Arc::new(load_cb)),
        play: Some(Arc::new(play_cb)),
        pause: Some(Arc::new(pause_cb)),
        seek: Some(Arc::new(seek_cb)),
        stop: Some(Arc::new(stop_cb)),
        set_volume: Some(Arc::new(set_volume_cb)),
        get_volume: Some(Arc::new(get_volume_cb)),
    }
}
