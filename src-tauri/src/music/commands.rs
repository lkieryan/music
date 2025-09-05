use tauri::{State, AppHandle};
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use crate::plugins::manager::PluginHandler;
use types::settings::music::MusicSourceSelection;
use music_plugin_sdk::types::{SearchResult, Track as SdkTrack, Album as SdkAlbum, Artist as SdkArtist, Playlist as SdkPlaylist, PageInfo as SdkPageInfo};
use music_plugin_sdk::types::media::Genre as SdkGenre;
use serde::{Serialize, Deserialize};
use types::tracks::MediaContent;

#[tauri::command]
pub async fn music_search(
    _app: AppHandle,
    plugin_handler: State<'_, PluginHandler>,
    search_query: music_plugin_sdk::types::SearchQuery,
    selector: Option<serde_json::Value>,
) -> Result<SearchResult, String> {
    // Parse music source selection
    let selection = parse_music_source_selection(selector)?;
    
    // Get audio providers
    let plugin_manager = plugin_handler.plugin_manager();
    let audio_providers = plugin_manager
        .get_audio_providers_by_selection(&selection)
        .await
        .map_err(|e| format!("Failed to get audio providers: {}", e))?;
    
    if audio_providers.is_empty() {
        return Ok(SearchResult::default());
    }
    
    println!("Searching '{}' across {} providers", search_query.query, audio_providers.len());
    
    // Search all providers concurrently
    let search_tasks = audio_providers.into_iter().map(|(provider_id, provider_plugin)| {
        search_provider(provider_id, provider_plugin, search_query.clone())
    });
    
    let results = futures::future::join_all(search_tasks).await;
    
    // Merge results
    let merged_result = merge_search_results(results);
    
    println!("Search completed: {} tracks, {} albums, {} artists", 
             merged_result.tracks.items.len(), 
             merged_result.albums.items.len(), 
             merged_result.artists.items.len());
    
    Ok(merged_result)
}

/// Parse music source selection from frontend
fn parse_music_source_selection(selector: Option<serde_json::Value>) -> Result<MusicSourceSelection, String> {
    match selector {
        Some(val) => serde_json::from_value::<MusicSourceSelection>(val)
            .map_err(|e| format!("Invalid selector format: {}", e)),
        None => Ok(MusicSourceSelection::default()),
    }
}



/// Search a single provider
async fn search_provider(
    provider_id: Uuid,
    provider_plugin: std::sync::Arc<tokio::sync::Mutex<dyn music_plugin_sdk::traits::MediaPlugin + Send + Sync>>,
    search_query: music_plugin_sdk::types::SearchQuery,
) -> Result<music_plugin_sdk::types::SearchResult, String> {
    let plugin_guard = provider_plugin.lock().await;
    // 防止某个提供者长时间无响应导致整体卡死
    match timeout(Duration::from_secs(5), plugin_guard.search(&search_query)).await {
        Ok(res) => res.map_err(|e| format!("Provider {} search failed: {}", provider_id, e)),
        Err(_) => Err(format!("Provider {} search timeout", provider_id)),
    }
}

/// Merge multiple search results into one
fn merge_search_results(results: Vec<Result<music_plugin_sdk::types::SearchResult, String>>) -> music_plugin_sdk::types::SearchResult {
    let mut merged = music_plugin_sdk::types::SearchResult::default();
    let mut errors = Vec::new();
    
    for result in results {
        match result {
            Ok(search_result) => {
                merge_single_search_result(&mut merged, search_result);
            }
            Err(error) => {
                errors.push(error);
            }
        }
    }
    
    // Log errors but don't fail the entire search
    if !errors.is_empty() {
        eprintln!("Some providers failed: {:?}", errors);
    }
    
    merged
}

/// Merge a single search result into the merged result
fn merge_single_search_result(
    merged: &mut music_plugin_sdk::types::SearchResult,
    search_result: music_plugin_sdk::types::SearchResult,
) {
    // Merge all media collections and update pagination info
    merged.tracks.items.extend(search_result.tracks.items);
    merged.artists.items.extend(search_result.artists.items);
    merged.albums.items.extend(search_result.albums.items);
    merged.playlists.items.extend(search_result.playlists.items);
    merged.genres.items.extend(search_result.genres.items);
    
    // Update pagination info - use the first non-default page info we encounter
    if merged.tracks.page.total.is_none() && search_result.tracks.page.total.is_some() {
        merged.tracks.page = search_result.tracks.page;
    }
    if merged.artists.page.total.is_none() && search_result.artists.page.total.is_some() {
        merged.artists.page = search_result.artists.page;
    }
    if merged.albums.page.total.is_none() && search_result.albums.page.total.is_some() {
        merged.albums.page = search_result.albums.page;
    }
    if merged.playlists.page.total.is_none() && search_result.playlists.page.total.is_some() {
        merged.playlists.page = search_result.playlists.page;
    }
    if merged.genres.page.total.is_none() && search_result.genres.page.total.is_some() {
        merged.genres.page = search_result.genres.page;
    }
    
    // Merge search suggestions
    if let Some(suggestions) = search_result.suggestions {
        merged.suggestions.get_or_insert_with(Vec::new).extend(suggestions);
    }
    
    // Merge provider context
    merge_provider_context(&mut merged.provider_context, search_result.provider_context);
}

/// Merge provider context JSON objects
fn merge_provider_context(
    merged_context: &mut Option<serde_json::Value>,
    new_context: Option<serde_json::Value>,
) {
    if let Some(context) = new_context {
        if let Some(context_obj) = context.as_object() {
            match merged_context {
                Some(merged_obj) => {
                    if let Some(merged_map) = merged_obj.as_object_mut() {
                        // Extend existing object
                        merged_map.extend(context_obj.clone());
                    } else {
                        // Replace with new object if current is not an object
                        *merged_context = Some(serde_json::Value::Object(context_obj.clone()));
                    }
                }
                None => {
                    // Set new object if no context exists
                    *merged_context = Some(serde_json::Value::Object(context_obj.clone()));
                }
            }
        }
    }
}
