use settings::{
  get_settings_state, get_secure, handle_settings_changes, initial, load_selective,
  load_selective_array, save_selective, set_secure, load_domain, save_domain_partial,
};
use tauri::Manager;
use providers::handler::{
  provider_search, provider_playback_url, provider_list_keys,
};
use scanner::{get_scanner_state, ScanTask};
use database::database::Database;
use std::sync::Arc;

mod settings;
mod themes;
mod providers;
mod scanner;
mod audio;



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let mut builder = tauri::Builder::default();

  builder = builder
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
      providers::handler::provider_search,
      providers::handler::provider_playback_url,
      providers::handler::provider_list_keys,
      // Audio Player Commands
      audio::play_song,
      audio::pause_playback,
      audio::resume_playback,
      audio::stop_playback,
      audio::seek_to_position,
      audio::set_volume,
      audio::next_track,
      audio::previous_track,
      audio::set_play_mode,
      audio::add_to_queue,
      audio::remove_from_queue,
      audio::get_queue,
      audio::get_player_status,
      audio::toggle_playback,
      audio::get_current_song,
      audio::clear_queue,
      audio::add_songs_to_queue,
      audio::play_playlist,
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

      // Initialize audio player runtime handle (Send + Sync)
      let player_handle = audio::PlayerHandle::spawn(db.clone())
        .expect("Failed to initialize player handle");
      app.manage(player_handle.clone());

      // Initialize theme subsystem
      let theme_handler_state = themes::get_theme_handler_state(app);
      app.manage(theme_handler_state);

      // Initialize providers subsystem (state + bootstrap from settings)
      providers::initialize_providers(app);

      // Setup audio player event handling
      audio::setup_player_events(app.handle().clone(), player_handle.clone());

      initial(app);
      handle_settings_changes(app.handle().clone());
      Ok(())
    });



  builder
    .run(tauri::generate_context!())
    .expect("error while running tauri application")
}
