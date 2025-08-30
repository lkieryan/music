
//! Bilibili API response conversion functions
//! 
//! This module contains all functions for converting Bilibili API responses
//! to music-plugin-sdk compatible formats.

use chrono::Utc;
use music_plugin_sdk::types::*;
use music_plugin_sdk::errors::PluginError;

use super::types::*;

/// Convert Bilibili search response to SDK format (legacy function, kept for compatibility)
pub fn convert_search_response(response: BilibiliSearchResponse, page_num: u32, page_size: u32) -> PluginResult<SearchResult> {
    let videos = response.result.unwrap_or_default();
    let mut tracks = Vec::new();

    for video in videos {
        let author_name = video.author.clone();
        let track = Track {
            id: format!("bilibili:{}", video.bvid),
            provider: Some("bilibili".to_string()),
            provider_id: Some(video.bvid.clone()),
            title: video.title,
            artist: author_name.clone(),
            album: None,
            album_ref: None,
            disc_number: None,
            track_number: None,
            duration: Some(parse_duration(&video.duration) * 1000),
            cover_url: Some(video.pic),
            url: None,
            quality: None,
            preview_url: None,
            isrc: None,
            popularity: Some(video.play as u32),
            availability: None,
            lyrics: None,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("description".to_string(), video.description);
                meta.insert("pubdate".to_string(), video.pubdate.to_string());
                meta.insert("favorites".to_string(), video.favorites.to_string());
                meta
            },
        };
        tracks.push(track);
    }

    let page_info = PageInfo {
        limit: page_size,
        offset: ((page_num - 1) * page_size),
        next_cursor: None,
        total: Some(response.num_pages.unwrap_or(1) * page_size),
        has_more: page_num < response.num_pages.unwrap_or(1),
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


/// Convert Bilibili video details to SDK Track format
pub fn convert_track_response(track_id: &str, bvid: &str, video_details: BilibiliVideoDetails, lyrics: Option<Lyrics>) -> PluginResult<Track> {
    let owner_name = video_details.owner.name.clone();
    Ok(Track {
        id: track_id.to_string(),
        provider: Some("bilibili".to_string()),
        provider_id: Some(bvid.to_string()),
        title: video_details.title,
        artist: owner_name.clone(),
        album: None,
        album_ref: None,
        disc_number: None,
        track_number: None,
        duration: Some(video_details.duration as u32 * 1000),
        cover_url: Some(video_details.pic),
        url: None,
        quality: None,
        preview_url: None,
        isrc: None,
        popularity: Some(video_details.stat.view as u32),
        availability: None,
        lyrics, // 将歌词回填到track中
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("description".to_string(), video_details.desc);
            meta.insert("pubdate".to_string(), video_details.pubdate.to_string());
            meta.insert("cid".to_string(), video_details.cid.to_string());
            meta
        },
    })
}


/// Convert Bilibili user info to SDK Artist format
pub fn convert_artist_response(artist_id: &str, user_info: BilibiliUserInfo) -> PluginResult<Artist> {
    Ok(Artist {
        id: artist_id.to_string(),
        name: user_info.name,
        mbid: None,
        description: Some(user_info.sign),
        avatar_url: Some(user_info.face),
        followers: None,
        track_count: 0.0,
        sanitized_name: None,
        metadata: {
            let mut meta = std::collections::HashMap::new();
            meta.insert("level".to_string(), user_info.level.to_string());
            meta.insert("vip_type".to_string(), user_info.vip.vip_type.to_string());
            meta
        },
        extra_info: None,
    })
}

/// Convert Bilibili favorite list contents to SDK Playlist format
pub fn convert_playlist_response(playlist_id: &str, fav_id: u64, fav_contents: BilibiliFavoriteListContents) -> PluginResult<Playlist> {
    // 1. 转换播放列表中的音轨
    let mut tracks = Vec::new();
    for media in fav_contents.medias.unwrap_or_default() {
        let upper_name = media.upper.name.clone();
        let track = Track {
            id: format!("bilibili:{}", media.bvid),
            provider: Some("bilibili".to_string()),
            provider_id: Some(media.bvid.clone()),
            title: media.title,
            artist: upper_name.clone(),
            album: None,
            album_ref: None,
            disc_number: None,
            track_number: None,
            duration: Some(media.duration * 1000),
            cover_url: Some(media.cover),
            url: None,
            quality: None,
            preview_url: None,
            isrc: None,
            popularity: Some(media.cnt_info.play),
            availability: None,
            lyrics: None,
            metadata: {
                let mut meta = std::collections::HashMap::new();
                meta.insert("description".to_string(), media.intro);
                meta.insert("pubtime".to_string(), media.pubtime.to_string());
                meta
            },
        };
        tracks.push(track);
    }

    // 2. 转换播放列表信息
    let owner_name = fav_contents.info.upper.name.clone();
    Ok(Playlist {
        id: playlist_id.to_string(),
        provider: Some("bilibili".to_string()),
        provider_id: Some(fav_id.to_string()),
        title: fav_contents.info.title,
        description: Some(fav_contents.info.intro),
        creator: owner_name.clone(),
        owner: Some(PlaylistOwner {
            id: Some(fav_contents.info.upper.mid.to_string()),
            name: Some(owner_name),
        }),
        cover_url: Some(fav_contents.info.cover),
        images: None,
        tracks,
        track_count: fav_contents.info.media_count as f64,
        total_tracks: Some(fav_contents.info.media_count as u32),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_public: fav_contents.info.attr == 0,
        collaborative: Some(false),
        availability: None,
        external_urls: None,
        file_path: None,
        extension: None,
        icon: None,
        library_item: Some(false),
        metadata: std::collections::HashMap::new(),
    })
}


/// Convert Bilibili audio stream response to extract audio URL
/// Preference: Progressive durl -> DASH audio
pub fn convert_audio_stream_response(response: BilibiliAudioStreamResponse) -> PluginResult<String> {
    if let Some(mut durls) = response.durl {
        if !durls.is_empty() {
            durls.sort_by(|a,b| {
                let a_score = a.size.unwrap_or(0).max(a.length.unwrap_or(0));
                let b_score = b.size.unwrap_or(0).max(b.length.unwrap_or(0));
                b_score.cmp(&a_score)
            });
            return Ok(durls[0].url.clone());
        }
    }
    if let Some(dash) = response.dash {
        if let Some(audio) = dash.audio {
            if let Some(first) = audio.first() {
                return Ok(first.base_url.clone());
            }
        }
    }
    Err(PluginError::Internal("No available audio stream".to_string()))
}

/// Parse duration string in format "MM:SS" or "HH:MM:SS" to seconds
pub fn parse_duration(duration_str: &str) -> u32 {
    let parts: Vec<&str> = duration_str.split(':').collect();
    match parts.len() {
        2 => {
            let minutes = parts[0].parse::<u32>().unwrap_or(0);
            let seconds = parts[1].parse::<u32>().unwrap_or(0);
            minutes * 60 + seconds
        }
        3 => {
            let hours = parts[0].parse::<u32>().unwrap_or(0);
            let minutes = parts[1].parse::<u32>().unwrap_or(0);
            let seconds = parts[2].parse::<u32>().unwrap_or(0);
            hours * 3600 + minutes * 60 + seconds
        }
        _ => 0,
    }
}
