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
    pub gapless_skip: Option<String>,
    pub language: Option<String>,
    pub minimize_to_tray: Option<bool>,
    pub launch_at_login: Option<bool>,
}
