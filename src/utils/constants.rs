// Application constants

pub const APP_NAME: &str = "Rust Leptos Tauri App";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

// Default settings
pub const DEFAULT_LANGUAGE: &str = "en";
pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "zh"];

// Theme constants
pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";
pub const THEME_SYSTEM: &str = "system";

// Storage keys
pub const STORAGE_KEY_LANGUAGE: &str = "app_language";
pub const STORAGE_KEY_THEME: &str = "app_theme";
pub const STORAGE_KEY_USER_SETTINGS: &str = "user_settings";

// API constants
pub const API_TIMEOUT_SECONDS: u64 = 30;
pub const MAX_RETRY_ATTEMPTS: u32 = 3;