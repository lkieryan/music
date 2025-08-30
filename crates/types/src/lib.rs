pub mod errors;
pub mod settings;
pub mod providers;
pub mod themes;
pub mod tracks;
pub mod entities;
#[cfg(feature = "db")]
pub mod schema;
pub mod common;
pub mod cache;
#[cfg(feature = "db")]
pub mod cache_schema;
pub mod ui;
pub mod mpris;

#[cfg(all(test, feature = "ts-rs"))]
mod tests {
    #[test]
    fn export_bindings() {
        // This test triggers ts-rs to generate TypeScript bindings
        // when running: cargo test --features ts-rs export_bindings
    }
}