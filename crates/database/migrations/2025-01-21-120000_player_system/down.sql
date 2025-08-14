-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_favorites_song;
DROP INDEX IF EXISTS idx_play_history_played_at;
DROP INDEX IF EXISTS idx_play_history_song;
DROP INDEX IF EXISTS idx_play_queue_song;
DROP INDEX IF EXISTS idx_play_queue_position;

DROP TABLE IF EXISTS `player_settings`;
DROP TABLE IF EXISTS `favorites`;
DROP TABLE IF EXISTS `play_history`;
DROP TABLE IF EXISTS `play_queue`;
DROP TABLE IF EXISTS `player_state`;