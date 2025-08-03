use preference::{
  get_preference_state, get_secure, handle_pref_changes, initial, load_selective,
  load_selective_array, save_selective, set_secure,
};
use providers::handler::{
  provider_search, provider_playback_url, provider_list_keys,
};
use scanner::{get_scanner_state, ScanTask};
use tauri::Manager;


mod preference;
mod themes;
mod providers;
mod scanner;



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let mut builder = tauri::Builder::default();

  builder = builder
    .invoke_handler(tauri::generate_handler![
     // Themes
     themes::save_theme,
     themes::remove_theme,
     themes::load_theme,
     themes::load_all_themes,
     themes::get_css,
     themes::export_theme,
     themes::import_theme,
     // Preferences
      save_selective,
      load_selective,
      load_selective_array,
      get_secure,
      set_secure,
      // Providers
           providers::handler::provider_search,
     providers::handler::provider_playback_url,
     providers::handler::provider_list_keys,
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


      let config = get_preference_state(app)?;
      app.manage(config);

      // Initialize theme subsystem
      let theme_handler_state = themes::get_theme_handler_state(app);
      app.manage(theme_handler_state);

      // Initialize providers subsystem (state + bootstrap from preferences)
      providers::initialize_providers(app);

      initial(app);
      handle_pref_changes(app.handle().clone());
      Ok(())
    });



  builder
    .run(tauri::generate_context!())
    .expect("error while running tauri application")
}
