//! Built-in plugins for the music player

pub mod spotify;
pub mod youtube;
pub mod bilibili;
// Optional legacy modules can remain but are not loaded by default
// pub mod netease;

pub use spotify::SpotifyPlugin;
pub use youtube::YoutubePlugin;
pub use bilibili::BilibiliPlugin;
