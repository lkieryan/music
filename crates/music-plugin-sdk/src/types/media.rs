//! Core data types for music plugins
//! 
//! This module contains all the data structures used in plugin development,
//! completely independent from the main application types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[cfg(feature = "ts-rs")]
use ts_rs::TS;

/// Plugin execution result type
pub type PluginResult<T> = Result<T, crate::errors::PluginError>;

/// Generic image with optional dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Image {
    /// Image URL
    pub url: String,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
}

/// Lightweight artist reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct ArtistRef {
    /// Unique artist identifier (provider-scoped or global)
    pub id: String,
    /// Display name
    pub name: String,
}

/// Lightweight album reference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct AlbumRef {
    /// Unique album identifier (provider-scoped or global)
    pub id: String,
    /// Display name
    pub name: String,
    /// Album art images
    pub images: Vec<Image>,
}

/// Streaming protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamProtocol {
    /// HTTP Live Streaming
    Hls,
    /// MPEG-DASH
    Dash,
    /// Progressive download (direct file)
    Progressive,
    /// Other protocol
    Other(String),
}

/// Stream source representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamSource {
    /// Stream URL
    pub url: String,
    /// MIME type (e.g., audio/mpeg, application/x-mpegURL)
    pub mime_type: Option<String>,
    /// Container (e.g., mp3, flac, m4a, ts, mpd)
    pub container: Option<String>,
    /// Codec (e.g., aac, opus, flac)
    pub codec: Option<String>,
    /// Bitrate in kbps
    pub bitrate: Option<u32>,
    /// Sample rate in Hz
    pub sample_rate: Option<u32>,
    /// Number of channels
    pub channels: Option<u8>,
    /// Streaming protocol
    pub protocol: Option<StreamProtocol>,
    /// Expiry time of signed URLs
    pub expires_at: Option<DateTime<Utc>>,
    /// Required headers (Cookie/Referer/User-Agent etc.)
    pub headers: Option<HashMap<String, String>>,
    /// DRM/License info (reserved)
    pub drm: Option<String>,
}

/// Preferred stream format (hint from caller)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamFormatPreference {
    Auto,
    Progressive,
    Hls,
    Dash,
}

/// Preferred quality (provider maps to concrete value as needed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityPreference {
    Auto,
    Low,
    Medium,
    High,
    /// Provider-specific numeric quality, e.g. bilibili `qn`
    Qn(u32),
}

/// Stream request options (format/quality hints)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamRequest {
    pub format: StreamFormatPreference,
    pub quality: QualityPreference,
    /// Provider-specific extra parameters
    pub extra: Option<HashMap<String, String>>,
}

impl Default for StreamRequest {
    fn default() -> Self {
        Self { format: StreamFormatPreference::Auto, quality: QualityPreference::Auto, extra: None }
    }
}

/// Availability/permission info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Availability {
    /// Markets/regions where content is available
    pub markets: Option<Vec<String>>,
    /// Markets/regions where content is blocked
    pub blocked_markets: Option<Vec<String>>,
    /// Requires login to access
    pub requires_login: bool,
    /// Requires premium/subscription
    pub requires_premium: bool,
    /// Can stream
    pub can_stream: bool,
    /// Can download
    pub can_download: bool,
}

/// Music track information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Track {
    /// Unique track identifier
    pub id: String,
    /// Provider name (e.g., spotify, bilibili)
    pub provider: Option<String>,
    /// Provider original identifier
    pub provider_id: Option<String>,
    /// Track title
    pub title: String,
    /// Artist name
    pub artist: String,
    /// Album name
    pub album: Option<String>,
    /// Album reference with images
    pub album_ref: Option<AlbumRef>,
    /// Disc number
    pub disc_number: Option<u32>,
    /// Track number
    pub track_number: Option<u32>,
    /// Track duration in milliseconds
    pub duration: Option<u32>,
    /// Cover art URL
    pub cover_url: Option<String>,
    /// Playback URL
    pub url: Option<String>,
    /// Track quality information
    pub quality: Option<AudioQuality>,
    /// Preview clip URL (short preview)
    pub preview_url: Option<String>,
    /// ISRC code
    pub isrc: Option<String>,
    /// Popularity score 0-100
    pub popularity: Option<u32>,
    /// Availability constraints
    pub availability: Option<Availability>,
    /// Lyrics information (if available)
    pub lyrics: Option<Lyrics>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Audio quality information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct AudioQuality {
    /// Bitrate in kbps
    pub bitrate: Option<u32>,
    /// Sample rate in Hz
    pub sample_rate: Option<u32>,
    /// Number of channels
    pub channels: Option<u8>,
    /// Audio format (mp3, flac, etc.)
    pub format: Option<String>,
    /// Whether it's lossless
    pub lossless: bool,
}

/// Album information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Album {
    /// Unique album identifier
    pub id: String,
    /// Album title
    pub title: String,
    /// Artist name
    pub artist: String,
    /// Release date
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub release_date: Option<DateTime<Utc>>,
    /// Release year
    pub year: Option<String>,
    /// Cover art URL (high resolution)
    pub cover_url: Option<String>,
    /// Cover art URL (low resolution)
    pub cover_url_low: Option<String>,
    /// Album tracks
    pub tracks: Vec<Track>,
    /// Track count
    pub track_count: f64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Extra information
    pub extra_info: Option<String>,
}

/// Artist information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Artist {
    /// Unique artist identifier
    pub id: String,
    /// Artist name
    pub name: String,
    /// MusicBrainz ID
    pub mbid: Option<String>,
    /// Artist description
    pub description: Option<String>,
    /// Artist avatar URL
    pub avatar_url: Option<String>,
    /// Follower count
    pub followers: Option<u64>,
    /// Track count
    pub track_count: f64,
    /// Sanitized name for search matching
    pub sanitized_name: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Extra information
    pub extra_info: Option<String>,
}

/// Playlist information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Playlist {
    /// Unique playlist identifier
    pub id: String,
    /// Provider name (e.g., spotify, bilibili)
    pub provider: Option<String>,
    /// Provider original identifier
    pub provider_id: Option<String>,
    /// Playlist title
    pub title: String,
    /// Playlist description
    pub description: Option<String>,
    /// Creator name
    pub creator: String,
    /// Owner info
    pub owner: Option<PlaylistOwner>,
    /// Cover art URL
    pub cover_url: Option<String>,
    /// Multiple sized images
    pub images: Option<Vec<Image>>,
    /// Playlist tracks
    pub tracks: Vec<Track>,
    /// Track count
    pub track_count: f64,
    /// Total tracks (if tracks list is partial)
    pub total_tracks: Option<u32>,
    /// Creation date
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub created_at: DateTime<Utc>,
    /// Last updated date
    #[cfg_attr(feature = "ts-rs", ts(type = "string"))]
    pub updated_at: DateTime<Utc>,
    /// Whether playlist is public
    pub is_public: bool,
    /// Collaborative playlist
    pub collaborative: Option<bool>,
    /// Availability constraints
    pub availability: Option<Availability>,
    /// External URLs (by provider)
    pub external_urls: Option<HashMap<String, String>>,
    /// Local file path (for local playlists)
    pub file_path: Option<String>,
    /// File extension
    pub extension: Option<String>,
    /// Playlist icon
    pub icon: Option<String>,
    /// Whether it's a library item
    pub library_item: Option<bool>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Genre information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Genre {
    /// Unique genre identifier
    pub id: Option<String>,
    /// Genre name
    pub name: Option<String>,
    /// Track count in this genre
    pub track_count: f64,
}

/// Playlist owner
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct PlaylistOwner {
    /// Owner id
    pub id: Option<String>,
    /// Owner display name
    pub name: Option<String>,
}

/// Pagination input for search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct PageInput {
    /// Page size
    pub limit: Option<u32>,
    /// Offset-based pagination
    pub offset: Option<u32>,
    /// Cursor-based pagination (provider-specific)
    pub cursor: Option<String>,
}

/// Pagination info for returned slices
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct PageInfo {
    /// Page size
    pub limit: u32,
    /// Offset used
    pub offset: u32,
    /// Next cursor (if cursor-based pagination is used)
    pub next_cursor: Option<String>,
    /// Total count if known
    pub total: Option<u32>,
    /// Whether more results are available
    pub has_more: bool,
}

impl Default for PageInfo {
    fn default() -> Self {
        Self {
            limit: 0,
            offset: 0,
            next_cursor: None,
            total: None,
            has_more: false,
        }
    }
}

/// Sort orders for search
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub enum SearchSort { Relevance, Recent, Popularity, Alphabetical, DurationAsc, DurationDesc, Custom(String) }

/// Search query parameters (provider-scoped)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct SearchQuery {
    /// Search term
    pub query: String,
    /// Requested result types
    pub types: Vec<SearchType>,
    /// Global pagination defaults
    pub page: Option<PageInput>,
    /// Per-type pagination overrides
    pub per_type_page: Option<HashMap<SearchType, PageInput>>,
    /// Global sort order
    pub sort: Option<SearchSort>,
    /// Per-type sort overrides
    pub per_type_sort: Option<HashMap<SearchType, SearchSort>>,
    /// Additional generic filters (provider best-effort)
    pub filters: HashMap<String, String>,
    /// Provider-specific params passthrough
    #[cfg_attr(feature = "ts-rs", ts(type = "Record<string, any>"))]
    pub provider_params: HashMap<String, serde_json::Value>,
}

/// Search result types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub enum SearchType {
    /// Search for tracks
    Track,
    /// Search for albums
    Album,
    /// Search for artists
    Artist,
    /// Search for playlists
    Playlist,
    /// Search everything
    All,
}

/// Generic search slice with pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct SearchSlice<T> {
    pub items: Vec<T>,
    pub page: PageInfo,
}

impl<T> Default for SearchSlice<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            page: PageInfo::default(),
        }
    }
}

/// Provider-scoped search result returned by plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct SearchResult {
    /// Provider name (e.g., spotify, bilibili)
    pub provider: String,
    /// Track results
    pub tracks: SearchSlice<Track>,
    /// Album results
    pub albums: SearchSlice<Album>,
    /// Artist results
    pub artists: SearchSlice<Artist>,
    /// Playlist results
    pub playlists: SearchSlice<Playlist>,
    /// Genre results
    pub genres: SearchSlice<Genre>,
    /// Provider-suggested queries (optional)
    pub suggestions: Option<Vec<String>>,
    /// Provider-specific context for follow-up requests
    #[cfg_attr(feature = "ts-rs", ts(type = "Record<string, any>"))]
    pub provider_context: Option<serde_json::Value>,
}

impl Default for SearchResult {
    fn default() -> Self {
        Self {
            provider: String::new(),
            tracks: SearchSlice::default(),
            albums: SearchSlice::default(),
            artists: SearchSlice::default(),
            playlists: SearchSlice::default(),
            genres: SearchSlice::default(),
            suggestions: None,
            provider_context: None,
        }
    }
}

/// Lyrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct Lyrics {
    /// Raw lyrics text (LRC/SRT/plain)
    pub text: String,
    /// Lyrics format (e.g., "lrc", "srt", "plain")
    pub format: Option<String>,
    /// Whether lyrics are time-synced
    pub synced: bool,
    /// Language code (e.g., "en", "zh-CN")
    pub language: Option<String>,
    /// Source/provider of the lyrics
    pub source: Option<String>,
    /// All language versions of lyrics (including original and translations)
    pub versions: Option<Vec<LyricsVersion>>,
}

/// A lyrics version (original language or translation)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct LyricsVersion {
    /// Language code, e.g., en, zh-CN
    pub language: String,
    /// Whether time-synced
    pub synced: bool,
    /// Format (lrc/krc/plain)
    pub format: Option<String>,
    /// Lines of lyrics
    pub lines: Vec<LyricLine>,
}

/// A single lyric line with optional timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub struct LyricLine {
    /// Timestamp in milliseconds
    pub timestamp_ms: Option<u32>,
    /// Line text
    pub text: String,
}

/// A translated lyrics variant (deprecated - use LyricsVersion instead)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsTranslation {
    /// Language code, e.g., zh-CN
    pub language: String,
    /// Whether time-synced
    pub synced: bool,
    /// Format (lrc/krc/plain)
    pub format: Option<String>,
    /// Lines of lyrics
    pub lines: Vec<LyricLine>,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthMethod {
    /// Username + password
    Password,
    /// Phone number + verification code
    Phone,
    /// QR code based login (scan on another device)
    QrCode,
}

/// Authentication session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    /// Opaque session identifier to correlate steps
    pub id: String,
    /// Optional expiration for the session
    pub expires_at: Option<DateTime<Utc>>,
}

/// Authentication challenge to be presented/fulfilled by the host UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthChallenge {
    /// Show a QR code to the user for scanning
    QrCode {
        /// Raw content to encode as QR (URL or token)
        content: String,
        /// Optional QR image URL prepared by provider
        image_url: Option<String>,
        /// Optional expiry
        expires_at: Option<DateTime<Utc>>,
    },
    /// Request credentials input (field names for UI rendering)
    Credentials {
        /// Required field keys, e.g. ["username", "password"]
        fields: Vec<String>,
    },
    /// Request OTP verification
    Otp {
        /// Channel, e.g. "sms", "email", "app"
        channel: String,
        /// Optional hint (e.g., masked phone/email)
        hint: Option<String>,
        /// Optional resend interval in ms
        resend_in_ms: Option<u32>,
    },
}

/// Authentication status for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthStatus {
    /// Still pending (poll again or continue flow)
    Pending,
    /// Authorized successfully with optional user info
    Authorized { user_id: Option<String>, display_name: Option<String> },
    /// Failed with reason
    Failed(String),
}

/// Authentication progress returned by provider for stepwise flows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthProgress {
    /// Awaiting user input to a presented challenge
    AwaitingInput {
        session: AuthSession,
        challenge: AuthChallenge,
    },
    /// Provider suggests polling status periodically
    Polling {
        session: AuthSession,
        /// Suggested polling interval in milliseconds
        interval_ms: u32,
    },
    /// Authentication completed
    Completed {
        user_id: Option<String>,
        display_name: Option<String>,
    },
    /// Authentication failed
    Failed {
        reason: String,
    },
}

/// User information for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserInfo {
    /// User ID
    pub user_id: String,
    /// Display name
    pub display_name: Option<String>,
    /// Avatar URL
    pub avatar_url: Option<String>,
    /// Additional user metadata
    pub metadata: HashMap<String, String>,
}

/// QR code response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeResponse {
    /// QR code content (URL or token)
    pub content: String,
    /// QR code image URL (optional)
    pub image_url: Option<String>,
    /// QR code key for polling
    pub qrcode_key: String,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

/// QR code status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCodeStatus {
    /// Status of the QR code
    pub status: QrCodeState,
    /// User information if login successful
    pub user_info: Option<AuthUserInfo>,
    /// Session token if login successful
    pub session_token: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// QR code state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QrCodeState {
    /// QR code generated, waiting for scan
    WaitingForScan,
    /// QR code scanned, waiting for confirmation
    WaitingForConfirmation,
    /// Login successful
    Success,
    /// QR code expired
    Expired,
    /// Login failed
    Failed,
}

/// SMS response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsResponse {
    /// Session ID for verification
    pub session_id: String,
    /// Masked phone number
    pub masked_phone: Option<String>,
    /// Resend interval in seconds
    pub resend_interval: Option<u32>,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    /// Whether authentication was successful
    pub success: bool,
    /// User information if successful
    pub user_info: Option<AuthUserInfo>,
    /// Session token
    pub session_token: Option<String>,
    /// Refresh token
    pub refresh_token: Option<String>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Additional authentication data
    pub auth_data: HashMap<String, String>,
}
