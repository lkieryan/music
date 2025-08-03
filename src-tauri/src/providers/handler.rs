use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use providers::{
    factory,
    provider::base::{ProviderCapability, SearchResult, Song, ProviderStatus},
    registry::ProviderRegistry,
    router::{self, ProviderSelector},
};
use serde::{Deserialize, Serialize};
use types::providers::ProviderSelectorArg;
use tauri::{AppHandle, Emitter, State};
use types::errors::Result;

#[derive(Clone)]
pub struct ProviderHandler {
    reg: ProviderRegistry,
    app: AppHandle,
}

impl ProviderHandler {
    pub fn new(app: AppHandle) -> Self {
        // Register built-in providers (spotify, youtube, ...)
        factory::register_builtin();
        Self {
            reg: ProviderRegistry::new(),
            app,
        }
    }

    /// Initialize a provider instance and add it into the registry.
    /// - `name`: provider type registered in factory (e.g., "spotify", "youtube").
    /// - `key`: optional instance key. If None, use the same as name.
    /// - `cfg`: provider-specific JSON config (optional).
    pub async fn initialize(&self, name: String, key: Option<String>, cfg: Option<serde_json::Value>) -> Result<()> {
        let key = key.unwrap_or_else(|| name.clone());
        let cfg = cfg.unwrap_or(serde_json::json!({}));
        let p = factory::create(&name, key.clone(), cfg)?;
       self.reg.add(key, Arc::from(p)).await;
       // emit providers-updated and provider-status-update after adding
       let _ = self.app.emit("providers-updated", serde_json::Value::Null);
       if let Ok(statuses) = self.get_all_statuses().await {
           let _ = self.app.emit("provider-status-update", statuses);
       }
       Ok(())
   }

    /// Ensure that the given provider key exists and supports the capability.
    pub async fn ensure_supports(&self, key: &str, cap: ProviderCapability) -> Result<()> {
        if let Some(p) = self.reg.get(key).await {
            if p.supports(&cap) {
                Ok(())
            } else {
                Err(format!("provider '{}' does not support capability '{:?}'", key, cap).into())
            }
        } else {
            Err(format!("unknown provider '{}'", key).into())
        }
    }

    pub async fn search(&self, selector: ProviderSelectorArg, term: String) -> Result<SearchResult> {
        let selector: ProviderSelector = self.map_selector(selector).await?;
        router::search_with_selector(selector, term, &self.reg).await
    }

    pub async fn playback_url(&self, selector: ProviderSelectorArg, song: Song, player: String) -> Result<String> {
        let selector: ProviderSelector = self.map_selector(selector).await?;
        router::playback_url_with_selector(selector, song, player, &self.reg).await
    }

    pub async fn list_keys(&self) -> Vec<String> { self.reg.keys().await }

   pub async fn remove_instance(&self, key: &str) -> bool {
       self.reg.remove(key).await.is_some()
   }

   pub async fn get_all_statuses(&self) -> Result<Vec<ProviderStatus>> {
       let mut res = Vec::new();
       for key in self.reg.keys().await {
           if let Some(p) = self.reg.get(&key).await {
               if let Ok(mut st) = p.get_status().await {
                   // ensure capabilities present even if provider overrides get_status without them
                   if st.capabilities.is_empty() { st.capabilities = p.capabilities(); }
                   res.push(st);
               }
           }
       }
       Ok(res)
   }
}

impl ProviderHandler {
    async fn map_selector(&self, val: ProviderSelectorArg) -> Result<ProviderSelector> {
        use types::providers::{ProviderKind, ProviderSelectorArg as Sel};
        Ok(match val {
            Sel::Single { provider } => ProviderSelector::Single(self.kind_to_key(&provider).await?),
            Sel::All => ProviderSelector::All,
            Sel::Many { providers } => {
                let mut keys = vec![];
                for k in providers {
                    keys.push(self.kind_to_key(&k).await?);
                }
                ProviderSelector::Many(keys)
            }
        })
    }

    async fn kind_to_key(&self, k: &types::providers::ProviderKind) -> Result<String> {
        // Strategy: if an instance with the same name exists (e.g., "spotify"), use it; otherwise error.
        // Later you can extend this to a mapping table if you allow multiple instances per kind.
        let name = k.as_str();
        if self.reg.get(name).await.is_some() {
            Ok(name.to_string())
        } else {
            Err(format!("provider instance for kind '{}' not found; initialize it first via provider_initialize", name).into())
        }
    }
}


#[tauri::command(async)]
pub async fn provider_search(
    handler: State<'_, ProviderHandler>,
    selector: ProviderSelectorArg,
    term: String,
) -> Result<SearchResult> {
    handler.search(selector, term).await
}

#[tauri::command(async)]
pub async fn provider_playback_url(
    handler: State<'_, ProviderHandler>,
    selector: ProviderSelectorArg,
    song: Song,
    player: String,
) -> Result<String> {
    handler.playback_url(selector, song, player).await
}

#[tauri::command(async)]
pub async fn provider_list_keys(handler: State<'_, ProviderHandler>) -> Result<Vec<String>> {
    Ok(handler.list_keys().await)
}

#[tauri::command(async)]
pub async fn provider_list_statuses(handler: State<'_, ProviderHandler>) -> Result<Vec<ProviderStatus>> {
    handler.get_all_statuses().await
}
