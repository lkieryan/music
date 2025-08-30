//! Plugin development macros
//! 
//! This module provides convenience macros for plugin development.

/// Plugin metadata macro
#[macro_export]
macro_rules! plugin_metadata {
    ($id:expr, $name:expr, $version:expr, $author:expr) => {
        $crate::types::PluginMetadata::minimal(
            uuid::Uuid::parse_str($id).expect("Invalid plugin ID"),
            $name,
            $version,
            $author,
        )
    };
}

/// Simple search macro
#[macro_export]
macro_rules! simple_search {
    ($query:expr) => {
        $crate::types::SearchQuery::simple($query)
    };
    ($query:expr, $type:expr) => {
        $crate::types::SearchQuery::typed($query, $type)
    };
}

/// Track creation macro
#[macro_export]
macro_rules! track {
    ($id:expr, $title:expr, $artist:expr) => {
        $crate::types::Track::minimal($id, $title, $artist)
    };
}