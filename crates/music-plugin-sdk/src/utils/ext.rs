//! Extension traits and implementations
//! 
//! This module provides extension methods for various types used in plugin development.

use crate::types::base::{PluginMetadata, PluginCapability, PluginConfig};
use crate::types::media::{
    SearchQuery, SearchType, PageInput, Track, AudioQuality, Album, Artist, Playlist
};
use std::collections::HashMap;

/// Plugin metadata helpers
impl PluginMetadata {
    /// Create minimal metadata
    pub fn minimal(id: uuid::Uuid, name: &str, version: &str, author: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            version: version.to_string(),
            description: String::new(),
            author: author.to_string(),
            website: None,
            icon: None,
            capabilities: vec![],
            min_sdk_version: "0.1.0".to_string(),
            config_schema: None,
        }
    }
    
    /// Check if plugin has capability
    pub fn has_capability(&self, capability: &PluginCapability) -> bool {
        self.capabilities.contains(capability)
    }
    
    /// Add capability
    pub fn with_capability(mut self, capability: PluginCapability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
        self
    }
}

/// Search query helpers
impl SearchQuery {
    /// Create simple text search
    pub fn simple(query: &str) -> Self {
        Self {
            query: query.to_string(),
            types: vec![SearchType::All],
            page: None,
            per_type_page: None,
            sort: None,
            per_type_sort: None,
            filters: HashMap::new(),
            provider_params: HashMap::new(),
        }
    }
    
    /// Create typed search
    pub fn typed(query: &str, search_type: SearchType) -> Self {
        Self { types: vec![search_type], ..Self::simple(query) }
    }
    
    /// Add limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        let mut page = self.page.unwrap_or(PageInput { limit: None, offset: None, cursor: None });
        page.limit = Some(limit);
        self.page = Some(page);
        self
    }
    
    /// Add offset for pagination
    pub fn with_offset(mut self, offset: u32) -> Self {
        let mut page = self.page.unwrap_or(PageInput { limit: None, offset: None, cursor: None });
        page.offset = Some(offset);
        page.cursor = None; // offset-based; clear cursor
        self.page = Some(page);
        self
    }
    
    /// Add filter
    pub fn with_filter(mut self, key: &str, value: &str) -> Self {
        self.filters.insert(key.to_string(), value.to_string());
        self
    }
}

/// Track helpers
impl Track {
    /// Create minimal track
    pub fn minimal(id: &str, title: &str, artist: &str) -> Self {
        Self {
            id: id.to_string(),
            provider: None,
            provider_id: None,
            title: title.to_string(),
            artist: artist.to_string(),
            album: None,
            album_ref: None,
            disc_number: None,
            track_number: None,
            duration: None,
            cover_url: None,
            url: None,
            quality: None,
            preview_url: None,
            isrc: None,
            popularity: None,
            availability: None,
            lyrics: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set album
    pub fn with_album(mut self, album: &str) -> Self {
        self.album = Some(album.to_string());
        self
    }
    
    /// Set duration
    pub fn with_duration(mut self, duration: u32) -> Self {
        self.duration = Some(duration);
        self
    }
    
    /// Set cover URL
    pub fn with_cover(mut self, cover_url: &str) -> Self {
        self.cover_url = Some(cover_url.to_string());
        self
    }
    
    /// Set playback URL
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }
    
    /// Set quality
    pub fn with_quality(mut self, quality: AudioQuality) -> Self {
        self.quality = Some(quality);
        self
    }
}

/// Audio quality helpers
impl AudioQuality {
    /// Create lossless quality
    pub fn lossless() -> Self {
        Self {
            bitrate: None,
            sample_rate: Some(44100),
            channels: Some(2),
            format: Some("flac".to_string()),
            lossless: true,
        }
    }
    
    /// Create high quality MP3
    pub fn high_mp3() -> Self {
        Self {
            bitrate: Some(320),
            sample_rate: Some(44100),
            channels: Some(2),
            format: Some("mp3".to_string()),
            lossless: false,
        }
    }
    
    /// Create standard quality MP3
    pub fn standard_mp3() -> Self {
        Self {
            bitrate: Some(192),
            sample_rate: Some(44100),
            channels: Some(2),
            format: Some("mp3".to_string()),
            lossless: false,
        }
    }
}

/// Plugin config helpers
impl PluginConfig {
    /// Create empty config
    pub fn empty() -> Self {
        Self {
            values: HashMap::new(),
            is_valid: true,
            errors: vec![],
        }
    }
    
    /// Create config with values
    pub fn with_values(values: HashMap<String, serde_json::Value>) -> Self {
        Self {
            values,
            is_valid: true,
            errors: vec![],
        }
    }
    
    /// Get string value
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.values.get(key)?.as_str().map(|s| s.to_string())
    }
    
    /// Get boolean value
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.values.get(key)?.as_bool()
    }
    
    /// Get number value
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.values.get(key)?.as_f64()
    }
    
    /// Set value
    pub fn set_value(&mut self, key: &str, value: serde_json::Value) {
        self.values.insert(key.to_string(), value);
    }
    
    /// Add error
    pub fn add_error(&mut self, error: &str) {
        self.errors.push(error.to_string());
        self.is_valid = false;
    }
}