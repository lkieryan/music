pub mod base;
pub mod media;

// Re-export all commonly used types
pub use base::{
    PluginResult, PluginMetadata, PluginContext, PluginConfig, PluginStatus, PluginCapability
};
pub use media::{
    SearchQuery, SearchResult, Track, Album, Artist, Playlist, PageInput, SearchType,
    AuthMethod, AuthUserInfo, QrCodeResponse, QrCodeStatus, SmsResponse, AuthResult,
    AudioQuality, Image, ArtistRef, AlbumRef, StreamSource, StreamRequest, StreamProtocol, Availability, Lyrics,
    LyricLine, LyricsTranslation, AuthSession, AuthChallenge, AuthStatus, AuthProgress,
    SearchSlice, PageInfo, SearchSort,PlaylistOwner
};
