use std::collections::HashMap;

use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::provider::{base::BaseProvider, spotify::SpotifyProvider, youtube::YoutubeProvider, bilibili::BilibiliProvider};
use types::errors::Result;

pub type ProviderBuilder = fn(key: String, cfg: serde_json::Value) -> Result<Box<dyn BaseProvider>>;

static REGISTRY: Lazy<Mutex<HashMap<&'static str, ProviderBuilder>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_builtin() {
    let mut reg = REGISTRY.lock().unwrap();
    reg.insert("spotify", |key, cfg| Ok(Box::new(SpotifyProvider::from_config(key, cfg)?)));
    reg.insert("youtube", |key, cfg| Ok(Box::new(YoutubeProvider::from_config(key, cfg)?)));
    reg.insert("bilibili", |key, cfg| Ok(Box::new(BilibiliProvider::from_config(key, cfg)?)));
}

pub fn register(name: &'static str, builder: ProviderBuilder) {
    REGISTRY.lock().unwrap().insert(name, builder);
}

pub fn create(name: &str, key: String, cfg: serde_json::Value) -> Result<Box<dyn BaseProvider>> {
    let reg = REGISTRY.lock().unwrap();
    let builder = reg.get(name).ok_or_else(|| format!("Unknown provider type: {}", name))?;
    builder(key, cfg)
}
