//! External plugin support for WASM and dynamic libraries

pub mod wasm;
pub mod dynamic;

pub use wasm::WasmPluginLoader;
pub use dynamic::DynamicPluginLoader;