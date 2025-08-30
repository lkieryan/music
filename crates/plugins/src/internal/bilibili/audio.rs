use async_trait::async_trait;
use std::collections::BTreeMap;

use music_plugin_sdk::{
    traits::MediaPlugin,
    types::{*, media::{LyricsVersion, StreamRequest, QualityPreference, StreamFormatPreference, StreamSource, StreamProtocol}},
    errors::PluginError
};
use chrono::Utc;
use super::plugin::BilibiliPlugin;
use super::types::*;
use super::convert;
use super::wbi::wbi_request;

impl BilibiliPlugin {
    /// Fetch subtitle content from URL with caching
    async fn fetch_subtitle_content(
        &self,
        subtitle_url: &str,
    ) -> PluginResult<serde_json::Value> {
        // Check cache first
        {
            let cache = self.subtitle_cache.read().await;
            if let Some(entry) = cache.get(subtitle_url) {
                return Ok(entry.content.clone());
            }
        }

        // Parse URL to extract host and path
        let url = reqwest::Url::parse(subtitle_url)
            .map_err(|e| PluginError::Internal(format!("Invalid subtitle URL: {}", e)))?;
        
        let host = url.host_str().unwrap_or("api.bilibili.com");
        let path = url.path();
        
        // Send request using wbi_request
        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            host,
            path,
            BTreeMap::new(),
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Fetch subtitle content failed: {}", e)))?;

        // Update cache
        {
            let mut cache = self.subtitle_cache.write().await;
            use super::plugin::SubtitleCacheEntry;
            use std::time::Instant;
            
            let cache_entry = SubtitleCacheEntry {
                content: response.clone(),
                created_at: Instant::now(),
                accessed_at: Instant::now(),
                ttl: self.default_cache_ttl,
            };
            
            cache.insert(subtitle_url.to_string(), cache_entry);
        }

        Ok(response)
    }

    /// Parse subtitle content into SDK format
    fn parse_subtitle_content(
        &self,
        subtitle_info: &serde_json::Value,
        subtitle_content: &serde_json::Value,
    ) -> Option<Lyrics> {
        let lan = subtitle_info.get("lan")
            .and_then(|l| l.as_str())
            .unwrap_or("zh-CN")
            .to_string();
        
        if let Some(body) = subtitle_content.get("body").and_then(|b| b.as_array()) {
            let lines: Vec<LyricLine> = body.iter()
                .filter_map(|line| {
                    let from = line.get("from").and_then(|f| f.as_u64()).unwrap_or(0) as u32;
                    let content = line.get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    if !content.is_empty() {
                        Some(LyricLine {
                            timestamp_ms: Some(from),
                            text: content,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            
            if !lines.is_empty() {
                let text = lines.iter().map(|l| l.text.clone()).collect::<Vec<_>>().join("\n");
                let translation = LyricsTranslation {
                    language: lan.clone(),
                    synced: true,
                    format: Some("lrc".to_string()),
                    lines: lines.clone(),
                };
                
                Some(Lyrics {
                    text,
                    format: Some("lrc".to_string()),
                    synced: true,
                    language: Some(lan),
                    source: Some("bilibili".to_string()),
                    versions: Some(vec![LyricsVersion {
                        language: translation.language,
                        synced: translation.synced,
                        format: translation.format,
                        lines: translation.lines,
                    }]),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[async_trait]
impl MediaPlugin for BilibiliPlugin {
    async fn search(&self, query: &SearchQuery) -> PluginResult<SearchResult> {
        let requested_limit = query.page
            .as_ref()
            .and_then(|p| p.limit)
            .unwrap_or(50) as u32;
            
        let requested_offset = query.page
            .as_ref()
            .and_then(|p| p.offset)
            .unwrap_or(0) as u32;
            
        let bilibili_page_size = 20u32;
        let bilibili_page = (requested_offset / bilibili_page_size) + 1;
        
        let mut params = BTreeMap::new();
        params.insert("keyword".to_string(), query.query.clone());
        params.insert("search_type".to_string(), "video".to_string());
        params.insert("page".to_string(), bilibili_page.to_string());

        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/web-interface/wbi/search/type",
            params,
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Search request failed: {}", e)))?;

        let search_response: BilibiliSearchResponse = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse search response: {}", e)))?;

        let videos = search_response.result.unwrap_or_default();
        let mut tracks = Vec::new();
        
        let page_start_index = (requested_offset % bilibili_page_size) as usize;
        
        for video in videos.iter().skip(page_start_index).take(requested_limit as usize) {
            let track = Track {
                id: format!("bilibili:{}", video.bvid),
                provider: Some("bilibili".to_string()),
                provider_id: Some(video.bvid.clone()),
                title: video.title.clone(),
                artist: video.author.clone(),
                album: None,
                album_ref: None,
                disc_number: None,
                track_number: None,
                duration: Some(convert::parse_duration(&video.duration) * 1000),
                cover_url: Some(video.pic.clone()),
                url: None,
                quality: None,
                preview_url: None,
                isrc: None,
                popularity: Some(video.play as u32),
                availability: None,
                lyrics: None,
                metadata: {
                    let mut meta = std::collections::HashMap::new();
                    meta.insert("description".to_string(), video.description.clone());
                    meta.insert("pubdate".to_string(), video.pubdate.to_string());
                    meta.insert("favorites".to_string(), video.favorites.to_string());
                    meta
                },
            };
            tracks.push(track);
        }
        
        let total_results = search_response.num_results.unwrap_or(0);
        
        let page_info = PageInfo {
            limit: requested_limit,
            offset: requested_offset,
            next_cursor: None,
            total: Some(total_results),
            has_more: (requested_offset + requested_limit) < total_results,
        };

        Ok(SearchResult {
            provider: "bilibili".to_string(),
            tracks: SearchSlice { items: tracks, page: page_info.clone() },
            albums: SearchSlice { items: Vec::new(), page: page_info.clone() },
            artists: SearchSlice { items: Vec::new(), page: page_info.clone() },
            playlists: SearchSlice { items: Vec::new(), page: page_info.clone() },
            genres: SearchSlice { items: Vec::new(), page: page_info },
            suggestions: None,
            provider_context: None,
        })
    }

    async fn get_track(&self, track_id: &str) -> PluginResult<Track> {
        let bvid = track_id
            .strip_prefix("bilibili:")
            .ok_or_else(|| PluginError::InvalidInput("Invalid bilibili track ID format".to_string()))?;

        let mut params = BTreeMap::new();
        params.insert("bvid".to_string(), bvid.to_string());

        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/web-interface/view",
            params,
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get track request failed: {}", e)))?;

        let video_details: BilibiliVideoDetails = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse video details: {}", e)))?;

        // Fetch subtitle content if available
        let lyrics = if let Some(subtitle_info) = video_details.subtitle.as_ref() {
            if !subtitle_info.list.is_empty() {
                let mut translations = Vec::new();
                let mut primary_text = String::new();
                let mut is_first = true;
                
                for subtitle in &subtitle_info.list {
                    if !subtitle.subtitle_url.is_empty() {
                        match self.fetch_subtitle_content(&subtitle.subtitle_url).await {
                            Ok(content) => {
                                let subtitle_json = serde_json::to_value(subtitle).unwrap_or_default();
                                
                                if let Some(lyrics) = self.parse_subtitle_content(&subtitle_json, &content) {
                                    if is_first {
                                        primary_text = lyrics.text.clone();
                                        is_first = false;
                                    }
                                    
                                    if let Some(lyrics_versions) = lyrics.versions {
                                        for version in lyrics_versions {
                                            translations.push(LyricsTranslation {
                                                language: version.language,
                                                synced: version.synced,
                                                format: version.format,
                                                lines: version.lines,
                                            });
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Failed to fetch subtitle content: {}", e);
                            }
                        }
                    }
                }
                
                if !translations.is_empty() {
                    Some(Lyrics {
                        text: primary_text,
                        format: Some("lrc".to_string()),
                        synced: true,
                        language: Some("multi".to_string()),
                        source: Some("bilibili".to_string()),
                        versions: Some(translations.into_iter().map(|translation| LyricsVersion {
                            language: translation.language,
                            synced: translation.synced,
                            format: translation.format,
                            lines: translation.lines,
                        }).collect()),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        convert::convert_track_response(track_id, bvid, video_details, lyrics)
    }

    async fn get_album(&self, _album_id: &str) -> PluginResult<Album> {
        Err(PluginError::NotSupported("Albums not supported for Bilibili".to_string()))
    }

    async fn get_artist(&self, artist_id: &str) -> PluginResult<Artist> {
        let mid = artist_id.parse::<u64>()
            .map_err(|_| PluginError::InvalidInput("Invalid artist ID".to_string()))?;

        let mut params = BTreeMap::new();
        params.insert("mid".to_string(), mid.to_string());

        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/space/wbi/acc/info",
            params,
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get artist request failed: {}", e)))?;

        let user_info: BilibiliUserInfo = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse user info: {}", e)))?;

        convert::convert_artist_response(artist_id, user_info)
    }

    async fn get_playlist(&self, playlist_id: &str) -> PluginResult<Playlist> {
        let fav_id = playlist_id.parse::<u64>()
            .map_err(|_| PluginError::InvalidInput("Invalid playlist ID".to_string()))?;

        let mut params = BTreeMap::new();
        params.insert("media_id".to_string(), fav_id.to_string());
        params.insert("pn".to_string(), "1".to_string());
        params.insert("ps".to_string(), "100".to_string());

        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/v3/fav/resource/list",
            params,
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get playlist request failed: {}", e)))?;

        let fav_contents: BilibiliFavoriteListContents = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse playlist contents: {}", e)))?;

        convert::convert_playlist_response(playlist_id, fav_id, fav_contents)
    }

   async fn get_media_stream(&self, track_id: &str, req: &StreamRequest) -> PluginResult<StreamSource> {
        let bvid = track_id
            .strip_prefix("bilibili:")
            .ok_or_else(|| PluginError::InvalidInput("Invalid bilibili track ID format".to_string()))?;

        // Get video details to obtain cid
        let mut params = BTreeMap::new();
        params.insert("bvid".to_string(), bvid.to_string());

        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/web-interface/view",
            params,
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get video details failed: {}", e)))?;

        let video_details: BilibiliVideoDetails = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse video details: {}", e)))?;

        let cid = video_details.cid;

        // Progressive-only: 参数写死，强制 MP4（durl），忽略外部 req
        // 质量固定为 1080P（80），若接口侧降级则仍以返回的 durl 为准；不回退 DASH
        let _qn_fixed: u32 = 80;
        let _fnval_fixed: u32 = 1; // MP4 only

        let mut wbi_params = BTreeMap::new();
        wbi_params.insert("bvid".to_string(), bvid.to_string());
        wbi_params.insert("cid".to_string(), cid.to_string());
        // 固定为 MP4：fnval=1
        wbi_params.insert("fnval".to_string(), _fnval_fixed.to_string());
        wbi_params.insert("fnver".to_string(), "0".to_string());
        wbi_params.insert("fourk".to_string(), "0".to_string());
        // 平台固定为 html5，开启高画质开关；清晰度固定 80（1080P）
        wbi_params.insert("platform".to_string(), "html5".to_string());
        wbi_params.insert("high_quality".to_string(), "1".to_string());
        wbi_params.insert("qn".to_string(), _qn_fixed.to_string());

        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/player/wbi/playurl",
            wbi_params,
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get stream URL failed: {}", e)))?;

        let stream_response: BilibiliAudioStreamResponse = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse stream response: {}", e)))?;

        // Common headers for Bilibili anti-hotlinking
        let mut common_headers: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        common_headers.insert("Referer".into(), "https://www.bilibili.com".into());
        common_headers.insert("Origin".into(), "https://www.bilibili.com".into());
        common_headers.insert(
            "User-Agent".into(),
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36".into(),
        );

        // Prefer progressive durl
        if let Some(mut durls) = stream_response.durl.clone() {
            if !durls.is_empty() {
                durls.sort_by(|a,b| {
                    let a_score = a.size.unwrap_or(0).max(a.length.unwrap_or(0));
                    let b_score = b.size.unwrap_or(0).max(b.length.unwrap_or(0));
                    b_score.cmp(&a_score)
                });
                let url = durls[0].url.clone();
                return Ok(StreamSource { url, mime_type: None, container: Some("mp4".into()), codec: Some("aac".into()), bitrate: None, sample_rate: None, channels: None, protocol: Some(StreamProtocol::Progressive), expires_at: None, headers: Some(common_headers.clone()), drm: None });
            }
        }
        // 不回退 DASH：若无 durl，则视为无可用流

        Err(PluginError::Internal("No available audio stream".to_string()))
    }

    async fn is_track_available(&self, track_id: &str) -> PluginResult<bool> {
        match self.get_track(track_id).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_user_playlists(&self) -> PluginResult<Vec<Playlist>> {
        let response = wbi_request(
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/space/myinfo",
            BTreeMap::new(),
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get user info failed: {}", e)))?;

        let user_info: BilibiliUserInfo = serde_json::from_value(response)
            .map_err(|e| PluginError::SerializationError(format!("Failed to parse user info: {}", e)))?;

        let mut params = BTreeMap::new();
        params.insert("up_mid".to_string(), user_info.mid.to_string());

        let response = wbi_request( 
            &self.http,
            reqwest::Method::GET,
            "https://api.bilibili.com",
            "/x/v3/fav/folder/created/list-all",
            params,
            self.session_data.as_deref(),
            &self.wbi_salt_cache,
        ).await.map_err(|e| PluginError::Internal(format!("Get user playlists failed: {}", e)))?;

        let playlist_response: serde_json::Value = response;
        let empty_vec = vec![];
        let list = playlist_response.get("list")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        let mut playlists = Vec::new();
        for item in list {
            if let Ok(fav_info) = serde_json::from_value::<BilibiliPlaylist>(item.clone()) {
                let playlist = Playlist {
                    id: fav_info.id.to_string(),
                    provider: Some("bilibili".to_string()),
                    provider_id: Some(fav_info.id.to_string()),
                    title: fav_info.title,
                    description: None,
                    creator: user_info.name.clone(),
                    owner: Some(PlaylistOwner {
                        id: Some(user_info.mid.to_string()),
                        name: Some(user_info.name.clone()),
                    }),
                    cover_url: None,
                    images: None,
                    tracks: Vec::new(),
                    track_count: fav_info.media_count as f64,
                    total_tracks: Some(fav_info.media_count as u32),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    is_public: fav_info.attr == 0,
                    collaborative: Some(false),
                    availability: None,
                    external_urls: None,
                    file_path: None,
                    extension: None,
                    icon: None,
                    library_item: Some(false),
                    metadata: std::collections::HashMap::new(),
                };
                playlists.push(playlist);
            }
        }

        Ok(playlists)
    }
}
