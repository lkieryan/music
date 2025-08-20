use settings::{
  get_settings_state, get_secure, handle_settings_changes, initial, load_selective,
  load_selective_array, save_selective, set_secure, load_domain, save_domain_partial,
};
use tauri::Manager;
use providers::handler::{
  provider_search, provider_playback_url, provider_list_keys,
};
use scanner::{
  start_scan,
  get_scanner_state, ScanTask, 
  start_auto_scanner, stop_auto_scanner, trigger_manual_scan, get_auto_scanner_status, get_local_songs
};

use audio::{
  audio_play,
  audio_pause,
  audio_stop,
  audio_seek,
  audio_set_volume,
  audio_get_volume,
  // PlayerStore commands
  get_current_song,
  get_queue,
  get_player_state,
  add_to_queue,
  remove_from_queue,
  play_now,
  shuffle_queue,
  clear_queue,
  toggle_player_mode,
  get_player_mode,
  set_player_mode,
  next_song,
  prev_song,
  change_index,
};
use database::database::Database;
use std::sync::Arc;

mod settings;
mod themes;
mod providers;
mod scanner;
mod audio;
mod playback;

/// run the app
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

  let _ = rustls::crypto::ring::default_provider().install_default();
  let mut builder = tauri::Builder::default();

  builder = builder
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
     // Themes      themes::save_theme,      themes::remove_theme,      themes::load_theme,      themes::load_all_themes,      themes::get_css,      themes::export_theme,      themes::import_theme,
      // settings
      save_selective,
      load_domain,
      save_domain_partial,
      load_selective,
      load_selective_array,
      get_secure,
      set_secure,
      // Providers
      provider_search,
      provider_playback_url,
      provider_list_keys,
      // Scanner 
      start_auto_scanner,
      stop_auto_scanner, 
      trigger_manual_scan,
      get_auto_scanner_status,
      get_local_songs,
      start_scan,
      // Audio Player Commands
      audio_play,
      audio_pause,
      audio_stop,
      audio_seek,
      audio_set_volume,
      audio_get_volume,
      // PlayerStore Commands
      get_current_song,
      get_queue,
      get_player_state,
      add_to_queue,
      remove_from_queue,
      play_now,
      shuffle_queue,
      clear_queue,
      toggle_player_mode,
      get_player_mode,
      set_player_mode,
      next_song,
      prev_song,
      change_index
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let scanner_state = get_scanner_state();
      app.manage(scanner_state);

      let scan_task = ScanTask::default();
      app.manage(scan_task);


      let config = get_settings_state(app)?;
      app.manage(config);

      // Initialize database
      let app_data_dir = app.path().app_data_dir().expect("Failed to get app data directory");
      std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");
      let db_path = app_data_dir.join("music.db");
      let db = Arc::new(Database::new(db_path));
      app.manage(db.clone());

      // Initialize audio player via builder (single instance) and manage it
      let audio_state = audio::build_audio_player(app.app_handle().clone());
      app.manage(audio_state);

      // Initialize theme subsystem
      let theme_handler_state = themes::get_theme_handler_state(app);
      app.manage(theme_handler_state);

      // Initialize providers subsystem (state + bootstrap from settings)
      providers::initialize_providers(app);

      initial(app);
      handle_settings_changes(app.handle().clone());
      Ok(())
    });



  builder
    .run(tauri::generate_context!())
    .expect("error while running tauri application")
}
