use serde::{Deserialize, Serialize};

#[cfg(feature = "ts-rs")]
use ts_rs::TS;

// Frontend-facing typed view for the "general" settings domain.
// Note: This is a type contract exported for the UI. The backend stores
// values in a flexible dotpath JSON tree; this struct doesn't force the
// backend to use this shape, but defines the expected fields and types for
// the renderer.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts", rename_all = "camelCase"))]
pub struct GeneralSettings {
    // UI uses a string input for the numeric value; keep it string for now to match UI binding.
    pub language: Option<String>,
    pub minimize_to_tray: Option<bool>,
    pub launch_at_login: Option<bool>,


    // ===== Media Library · Auto Scan =====
    /// Whether to automatically scan on app start.
    pub auto_scan_enabled: Option<bool>,
    /// Folders to scan. Absolute paths.
    pub scan_folders: Option<Vec<String>>,
    /// Minimal duration rule when scanning.
    pub scan_min_duration: Option<ScanMinDuration>,
    /// File format rule when scanning.
    pub scan_formats: Option<ScanFormats>,
}

/// Minimal duration rule for library scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts", rename_all = "camelCase"))]
pub enum ScanMinDuration {
    Sec30,
    Min2,
    All,
}

/// File format filter for library scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts", rename_all = "camelCase"))]
pub enum ScanFormats {
    /// Common audio formats (e.g., mp3/flac/ape/m4a, exact set decided by scanner).
    Common,
    /// All recognized audio formats.
    All,
}
