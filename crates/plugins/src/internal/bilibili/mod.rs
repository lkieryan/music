//! Bilibili provider with minimal WBI implementation.

mod plugin;
mod wbi;
mod audio;
mod auth;
mod types;
mod convert;

#[cfg(test)]
mod test_api;

#[cfg(test)]
mod test_qr_login;

pub use plugin::BilibiliPlugin;
pub use wbi::*;

