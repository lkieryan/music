//! # Music Plugin SDK
//! 
//! This SDK provides all the necessary interfaces and types for developing 
//! music player plugins. It includes audio provider traits, plugin lifecycle
//! management, and common data structures.

pub mod types;
pub mod traits;
pub mod errors;
pub mod base;
pub mod core;
pub mod utils;

/// Prelude module containing commonly used items
pub mod prelude {
    pub use crate::traits::*;
    pub use crate::errors::*;
    pub use crate::base::*;
    pub use crate::core::*;
    pub use crate::utils::*;
    pub use async_trait::async_trait;
}

// Re-export commonly used external types
pub use uuid::Uuid;
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use anyhow::Result as AnyhowResult;