use {
  db::{
    get_cache_state,
    {
      get_db_state
    },
  },
};


// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::fs;

use settings::{
  get_settings_state, get_secure, handle_settings_changes, initial, load_selective,
  load_selective_array, save_selective, set_secure, load_domain, save_domain_partial,
};
use tauri::Manager;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self},
    layer::SubscriberExt,
};
use scanner::{
  start_scan,
  get_scanner_state, ScanTask, 
  start_auto_scanner, stop_auto_scanner, trigger_manual_scan, get_auto_scanner_status, get_local_tracks
};
use plugins::{
  get_plugins, get_plugin, enable_plugin, disable_plugin, start_plugin, stop_plugin, load_plugin,
};

use music::commands::{
  music_search,
};

use audio::{
  audio_play, audio_pause, audio_stop, audio_seek, audio_set_volume, audio_get_volume,
  // PlayerStore commands
  get_current_track, get_queue, get_player_state, add_to_queue, remove_from_queue,
  play_now, shuffle_queue, clear_queue, toggle_player_mode, get_player_mode,
  set_player_mode, next_track, prev_track, change_index,
};

mod db;
use database::database::Database;
use std::sync::Arc;
use ::plugins::system::manager::PluginManager;

mod settings;
mod themes;
mod scanner;
mod audio;
mod playback;
mod plugins;
mod music;

/// run the app
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {

  let _ = rustls::crypto::ring::default_provider().install_default();

  let filter = if cfg!(mobile) {
      EnvFilter::try_new("debug").unwrap()
  } else {
      EnvFilter::from_env("MUSIC_LOG")
  };

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
      // Scanner 
      start_auto_scanner,
      stop_auto_scanner, 
      trigger_manual_scan,
      get_auto_scanner_status,
      get_local_tracks,
      start_scan,
      // Audio Player Commands
      audio_play,
      audio_pause,
      audio_stop,
      audio_seek,
      audio_set_volume,
      audio_get_volume,
      // PlayerStore Commands
      get_current_track,
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
      next_track,
      prev_track,
      change_index,
      // Plugin management
      get_plugins,
      get_plugin,
      enable_plugin,
      disable_plugin,
      start_plugin,
      stop_plugin,
      load_plugin,
      // Music API
      music_search
    ])
    .setup(|app| {
       let layer = fmt::layer()
          .pretty()
          .with_target(true)
          .with_ansi(!cfg!(mobile));
      let log_path = app.path().app_log_dir()?;
      if !log_path.exists() {
          fs::create_dir_all(log_path.clone())?;
      }

      #[cfg(desktop)]
      let subscriber = {
          let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_path, "music");
          let log_layer = fmt::layer()
              .pretty()
              .with_ansi(false)
              .with_target(true)
              .with_writer(file_appender);
          tracing_subscriber::registry()
              .with(filter)
              .with(layer)
              .with(log_layer)
      };

      #[cfg(mobile)]
      let subscriber = tracing_subscriber::registry().with(filter).with(layer);
      tracing::subscriber::set_global_default(subscriber).unwrap();

      let db = get_db_state(app);
      app.manage(db);

      let scanner_state = get_scanner_state();
      app.manage(scanner_state);

      let scan_task = ScanTask::default();
      app.manage(scan_task);


      let config = get_settings_state(app)?;
      app.manage(config);


      // Initialize plugin manager
      let plugins_root = app.path().app_data_dir().unwrap().join("plugins");
      let plugin_manager = Arc::new(PluginManager::new(app.state::<Database>().inner().clone(), plugins_root));
      app.manage(plugin_manager.clone());
      
      // Initialize plugin handler
      let plugin_handler = plugins::manager::PluginHandler::new(plugin_manager.clone());
      app.manage(plugin_handler);

      // Initialize audio player via builder (single instance) and manage it
      // Note: This must come AFTER plugin handler is managed
      let audio_state = audio::build_audio_player(app.app_handle().clone());
      app.manage(audio_state);
      
      // Initialize plugins (use Tauri's runtime to ensure a reactor exists)
      tauri::async_runtime::spawn(async move {
          if let Err(e) = plugin_manager.initialize().await {
              eprintln!("Failed to initialize plugins: {}", e);
          }
          
          // Start plugins
          if let Err(e) = plugin_manager.start_plugins().await {
              eprintln!("Failed to start plugins: {}", e);
          }
      });

      initial(app);
      handle_settings_changes(app.handle().clone());
      Ok(())
    });



  builder
    .run(tauri::generate_context!())
    .expect("error while running tauri application")
}
