use std::sync::Arc;

use futures::future::join_all;
use tokio::time::{timeout, Duration};
use crate::provider::base::{BaseProvider, ProviderCapability, SearchResult, Song}; // supports() kept local
use crate::registry::ProviderRegistry;
use types::errors::{Result, MoosyncError};

const DEFAULT_TIMEOUT_SECS: u64 = 5;

fn supports(p: &dyn BaseProvider, cap: &ProviderCapability) -> bool {
    p.capabilities().contains(&cap)
}

// 并发聚合：要求传入的 providers 已经按能力过滤
pub async fn search_all(term: String, providers: Vec<Arc<dyn BaseProvider>>) -> Result<SearchResult> {
    let tasks = providers
        .into_iter()
        .map(|p| {
            let value = term.clone();
            async move {
                let key = p.key();
                let term_cloned = value.clone();
                match timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), p.search(term_cloned)).await {
                    Ok(res) => (key, res),
                    Err(_) => (key, Err(MoosyncError::String("timeout".into()))),
                }
            }
        });
    let results = join_all(tasks).await;

    let mut merged: Vec<_> = vec![];
    for (key, res) in results {
        match res {
            Ok(mut r) => {
                for s in r.songs.iter_mut() {
                    s.provider_extension.get_or_insert(key.clone());
                }
                merged.extend(r.songs);
            }
            Err(_e) => { /* 局部失败容忍，可记录日志 */ }
        }
    }
    Ok(SearchResult { songs: merged })
}

// 选择器：单个 / 全部 / 若干
#[derive(Debug, Clone)]
pub enum ProviderSelector {
    Single(String),
    All,
    Many(Vec<String>),
}

async fn select_providers(
    selector: ProviderSelector,
    capability: ProviderCapability,
    registry: &ProviderRegistry,
) -> Vec<Arc<dyn BaseProvider>> {
    match selector {
        ProviderSelector::Single(key) => registry
            .get(&key)
            .await
           .filter(|p| supports(p.as_ref(), &capability))
            .into_iter()
            .collect(),
       ProviderSelector::All => {
           let cap = capability.clone();
            let mut res = vec![];
            for k in registry.keys().await {
                if let Some(p) = registry.get(&k).await {
                    if supports(p.as_ref(), &capability) {
                        res.push(p);
                    }
                }
            }
            res
        }
        ProviderSelector::Many(list) => {
            let mut res = vec![];
            for k in list {
                if let Some(p) = registry.get(&k).await {
                    if supports(p.as_ref(), &capability) {
                        res.push(p);
                    }
                }
            }
            res
        }
    }
}

// 统一搜索入口：基于 selector 和能力过滤
pub async fn search_with_selector(
    selector: ProviderSelector,
    term: String,
    registry: &ProviderRegistry,
) -> Result<SearchResult> {
   let providers = select_providers(selector.clone(), ProviderCapability::Search, registry).await;
   if providers.len() == 1 {
       let p = providers[0].clone();
       match timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), p.search(term.clone())).await {
           Ok(Ok(res)) => return Ok(res),
           Ok(Err(MoosyncError::SwitchProviders(next_key))) => {
               if let Some(np) = registry.get(&next_key).await {
                   if supports(np.as_ref(), &ProviderCapability::Search) {
                       return timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), np.search(term)).await
                           .unwrap_or_else(|_| Err(MoosyncError::String("timeout".into())));
                   }
               }
               return Err(MoosyncError::String(format!("delegated provider '{}' unavailable or does not support Search", next_key)));
           }
           Ok(Err(e)) => return Err(e),
           Err(_) => return Err(MoosyncError::String("timeout".into())),
       }
   }
   search_all(term, providers).await
}

// 播放 URL：优先来源 provider，失败后在候选中回退
pub async fn playback_url_with_selector(
    selector: ProviderSelector,
    song: Song,
    player: String,
    registry: &ProviderRegistry,
) -> Result<String> {
    match selector {
        ProviderSelector::Single(key) => {
            let p = registry
                .get(&key)
                .await
                .ok_or_else(|| format!("unknown provider {}", key))?;
            if !supports(p.as_ref(), &ProviderCapability::StreamUrl) {
                return Err("provider does not support StreamUrl".into());
            }
           match p.get_playback_url(song.clone(), player.clone()).await {
               Ok(url) => Ok(url),
               Err(MoosyncError::SwitchProviders(next_key)) => {
                   // Try delegated provider once
                   if let Some(np) = registry.get(&next_key).await {
                       if supports(np.as_ref(), &ProviderCapability::StreamUrl) {
                           return np.get_playback_url(song.clone(), player.clone()).await;
                       }
                   }
                   Err(MoosyncError::String(format!("delegated provider '{}' unavailable or does not support StreamUrl", next_key)))
               }
               Err(e) => Err(e),
           }
        }
        ProviderSelector::All | ProviderSelector::Many(_) => {
            // 优先来源
            if let Some(src) = song.provider_extension.clone() {
                if let Some(p) = registry.get(&src).await {
                    if supports(p.as_ref(), &ProviderCapability::StreamUrl) {
                        if let Ok(url) = timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), p.get_playback_url(song.clone(), player.clone())).await
                            .unwrap_or_else(|_| Err(MoosyncError::String("timeout".into()))) {
                            return Ok(url);
                        }
                    }
                }
            }
            
            // 回退候选集
            let providers = match selector {
                ProviderSelector::Many(list) => select_providers(ProviderSelector::Many(list), ProviderCapability::StreamUrl, registry).await,
                _ => select_providers(ProviderSelector::All, ProviderCapability::StreamUrl, registry).await,
            };
            for p in providers {
                if Some(p.key()) == song.provider_extension { continue; }
                // Try with timeout and handle delegation once.
                match timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), p.get_playback_url(song.clone(), player.clone())).await {
                    Ok(Ok(url)) => return Ok(url),
                    Ok(Err(MoosyncError::SwitchProviders(next_key))) => {
                        if let Some(np) = registry.get(&next_key).await {
                            if supports(np.as_ref(), &ProviderCapability::StreamUrl) {
                                if let Ok(url) = timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS), np.get_playback_url(song.clone(), player.clone())).await
                                    .unwrap_or_else(|_| Err(MoosyncError::String("timeout".into()))) {
                                    return Ok(url);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Err("no provider could produce a playback url".into())
        }
    }
}
