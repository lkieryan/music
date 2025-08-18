// crates/audio-player/src/lib.rs
// Re-export minimal, backend-only surface for Tauri integration
pub mod players;
pub mod core;
pub mod store;
pub mod events;
pub mod mpris;

// Public facade for backend usage
pub use core::AudioPlayer;