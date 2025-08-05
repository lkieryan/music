use super::base::*;
use async_trait::async_trait;
use types::errors::Result;

#[derive(Debug)]
pub struct YoutubeProvider {
    key: String,
}

impl YoutubeProvider {
    pub fn from_config(key: String, _cfg: serde_json::Value) -> Result<Self> { Ok(Self { key }) }
}

#[async_trait]
impl BaseProvider for YoutubeProvider {
    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "youtube".into(),
            display_name: "YouTube".into(),
            description: "YouTube provider".into(),
            capabilities: vec![
                ProviderCapability::Search,
                ProviderCapability::StreamUrl,
                ProviderCapability::UrlMatch,
            ],
            ..Default::default()
        }
    }
    fn key(&self) -> String { self.key.clone() }
}
