-- This file should undo anything in `up.sql`
DROP TABLE IF EXISTS `artist_bridge`;
DROP TABLE IF EXISTS `playlist_bridge`;
DROP TABLE IF EXISTS `artists`;
DROP TABLE IF EXISTS `album_bridge`;
DROP TABLE IF EXISTS `genres`;
DROP TABLE IF EXISTS `playlists`;
DROP TABLE IF EXISTS `tracks`;
DROP TABLE IF EXISTS `genre_bridge`;
DROP TABLE IF EXISTS `albums`;

-- Drop plugin states table
DROP INDEX IF EXISTS idx_plugin_states_name;
DROP INDEX IF EXISTS idx_plugin_states_enabled;
DROP INDEX IF EXISTS idx_plugin_states_plugin_type;
DROP INDEX IF EXISTS idx_plugin_states_installed_at;
DROP INDEX IF EXISTS idx_plugin_states_last_updated;
DROP INDEX IF EXISTS idx_plugin_states_last_used;
DROP TABLE IF EXISTS plugin_states;
