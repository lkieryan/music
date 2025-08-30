//! Validation and formatting utilities
//! 
//! This module provides validation and formatting utilities for plugin development.

/// URL validation utilities
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Format duration in human readable format
pub fn format_duration(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

/// Validate plugin ID format
pub fn is_valid_plugin_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok()
}

/// Generate new plugin ID
pub fn generate_plugin_id() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}