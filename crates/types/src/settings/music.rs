use serde::{Deserialize, Serialize};

#[cfg(feature = "ts-rs")]
use ts_rs::TS;

// Frontend-facing typed view for the "music" settings domain.
// This defines player/platform selection and audio-related preferences
// to be stored under prefs.music.* via the settings service.

/// Platform selection mode for music source routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts-rs",
    derive(TS),
    ts(export, export_to = "bindings.d.ts", rename_all = "camelCase")
)]
pub enum MusicSourceMode {
    All,
    Single,
    Many,
}

impl Default for MusicSourceMode {
    fn default() -> Self { Self::All }
}

/// Selected platform(s) used by the app when searching/playing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts-rs",
    derive(TS),
    ts(export, export_to = "bindings.d.ts", rename_all = "camelCase")
)]
pub struct MusicSourceSelection {
    pub mode: MusicSourceMode,
    /// Plugin IDs when mode is Single/Many. Empty for All.
    #[serde(default)]
    pub ids: Vec<String>,
}

impl Default for MusicSourceSelection {
    fn default() -> Self {
        Self { mode: MusicSourceMode::All, ids: Vec::new() }
    }
}

/// Playback related preferences (kept minimal; extend as needed).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts-rs",
    derive(TS),
    ts(export, export_to = "bindings.d.ts", rename_all = "camelCase")
)]
pub struct MusicPlaybackSettings {
    /// Enable loudness normalization if supported by source.
    pub normalize: Option<bool>,
    /// Crossfade duration in milliseconds.
    pub crossfade_ms: Option<u32>,
    /// Prefer seamless (gapless) playback when possible.
    pub gapless: Option<bool>,
}

/// A single audio effect unit in the processing chain.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts-rs",
    derive(TS),
    ts(export, export_to = "bindings.d.ts", rename_all = "camelCase")
)]
pub struct MusicEffectUnit {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub enabled: Option<bool>,
    /// Effect parameters (implementation-defined).
    #[cfg_attr(feature = "ts-rs", ts(type = "Record<string, any>"))]
    pub params: Option<serde_json::Value>,
}

/// Audio effects chain configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts-rs",
    derive(TS),
    ts(export, export_to = "bindings.d.ts", rename_all = "camelCase")
)]
pub struct MusicEffectsSettings {
    pub enabled: Option<bool>,
    #[serde(default)]
    pub chain: Vec<MusicEffectUnit>,
}

/// Root of the "music" settings domain.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts-rs",
    derive(TS),
    ts(export, export_to = "bindings.d.ts", rename_all = "camelCase")
)]
pub struct MusicSettings {
    /// Current platform selection (Home/all or specified plugins).
    pub source: Option<MusicSourceSelection>,
    /// Preferred order for non-Home sources (plugin IDs).
    pub sources_order: Option<Vec<String>>,
    /// Playback preferences.
    pub playback: Option<MusicPlaybackSettings>,
    /// Effects chain configuration.
    pub effects: Option<MusicEffectsSettings>,
}
