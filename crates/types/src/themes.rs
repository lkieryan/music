use serde::{Deserialize, Serialize};

#[cfg(feature = "ts-rs")]
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ThemeMeta {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub is_dark: Option<bool>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ThemeTokens {
    pub color_bg: String,
    pub color_surface: String,
    pub color_border: String,
    pub color_divider: String,

    pub text_primary: String,
    pub text_secondary: String,
    pub text_tertiary: String,
    pub text_disabled: String,

    pub accent: String,
    pub on_accent: String,

    pub state_hover_bg: String,
    pub state_pressed_bg: String,
    pub state_selected_bg: String,

    pub focus_ring: String,
    pub shadow_1: String,

    // Optional extensions used by some components
    pub colors_border: Option<String>,
    pub menu_accent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub enum LayerType {
    Linear,
    Radial,
    Conic,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ColorStop {
    pub color: String,
    pub pos: Option<f32>, // 0..100
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct BackgroundLayerBase {
    pub opacity: Option<f32>,
    pub blend_mode: Option<String>,
    pub position: Option<String>,
    pub size: Option<String>,
    pub repeat: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct LinearLayer {
    #[serde(rename = "type")]
    pub kind: LayerType,
    pub angle: Option<f32>,
    pub stops: Vec<ColorStop>,
    #[serde(flatten)]
    pub base: BackgroundLayerBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct RadialLayer {
    #[serde(rename = "type")]
    pub kind: LayerType,
    pub shape: Option<String>, // circle | ellipse
    pub at: Option<String>,
    pub stops: Vec<ColorStop>,
    #[serde(flatten)]
    pub base: BackgroundLayerBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ConicLayer {
    #[serde(rename = "type")]
    pub kind: LayerType,
    pub angle: Option<f32>,
    pub at: Option<String>,
    pub stops: Vec<ColorStop>,
    #[serde(flatten)]
    pub base: BackgroundLayerBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ImageLayer {
    #[serde(rename = "type")]
    pub kind: LayerType,
    pub url: String,
    #[serde(flatten)]
    pub base: BackgroundLayerBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub enum BackgroundLayer {
    Linear(LinearLayer),
    Radial(RadialLayer),
    Conic(ConicLayer),
    Image(ImageLayer),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct BackgroundConfig {
    pub layers: Vec<BackgroundLayer>,
    pub opacity: Option<f32>,
    pub blur: Option<f32>,
    pub grain_opacity: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ThemeBackground {
    pub app: BackgroundConfig,
    pub toolbar: Option<BackgroundConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ThemeDetails {
    pub meta: ThemeMeta,
    pub tokens: ThemeTokens,
    pub background: ThemeBackground,
    pub custom_css: Option<String>, // relative path within theme dir or raw css? here we store a file path
}
