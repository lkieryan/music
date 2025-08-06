use serde::{Deserialize, Serialize};

#[cfg(feature = "ts-rs")]
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts") )]
pub enum ProviderKind {
    #[serde(rename = "spotify")] Spotify,
    #[serde(rename = "youtube")] Youtube,
    #[serde(rename = "bilibili")] Bilibili,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::Spotify => "spotify",
            ProviderKind::Youtube => "youtube",
            ProviderKind::Bilibili => "bilibili",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts") )]
pub struct ProviderInstancePref {
    pub key: String,
    pub kind: ProviderKind,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    #[cfg_attr(feature = "ts-rs", ts(type = "Record<string, any>"))]
    pub cfg: Option<serde_json::Value>,
    #[serde(default)]
    pub secure_ref: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts") )]
pub enum ProviderSelectorArg {
    Single { provider: ProviderKind },
    All,
    Many { providers: Vec<ProviderKind> },
}
