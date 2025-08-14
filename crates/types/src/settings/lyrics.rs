use serde::{Deserialize, Serialize};

#[cfg(feature = "ts-rs")]
use ts_rs::TS;

// Frontend-facing typed view for the "lyrics" settings domain.
// Note: This is a type contract exported for the UI. The backend stores
// values in a flexible dotpath JSON tree; this struct doesn't force the
// backend to use this shape, but defines the expected fields and types for
// the renderer.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "ts-rs",
    derive(TS),
    ts(export, export_to = "bindings.d.ts", rename_all = "camelCase")
)]
pub struct LyricsSettings {
    // Appearance / implementation
    pub player_implementation: Option<String>,
    pub font_family: Option<String>,
    pub font_weight: Option<String>,
    pub letter_spacing: Option<String>,
    pub size_preset: Option<String>,
    pub line_blur_effect: Option<bool>,
    pub line_scale_effect: Option<bool>,
    pub line_spring_animation: Option<bool>,
    pub advance_line_timing: Option<bool>,
    pub word_fade_width: Option<f32>,

    // Content toggles
    pub translation_line: Option<bool>,
    pub roman_line: Option<bool>,
    pub swap_trans_roman_line: Option<bool>,

    // Background
    pub background_renderer: Option<String>,
    pub css_background_property: Option<String>,
    pub background_fps: Option<u32>,
    pub background_render_scale: Option<f32>,
    pub background_static_mode: Option<bool>,
}


