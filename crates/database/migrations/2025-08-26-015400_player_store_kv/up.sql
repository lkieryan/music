-- Player store KV table for local persistence (SQLite)
-- Keys mirror previous IndexedDB keys:
--  - player_state: PlayerDetails (JSON)
--  - song_queue:   Vec<String> (JSON)
--  - current_index: usize (JSON or stringified number)
--  - queue_data:   HashMap<String, Song> (JSON)

CREATE TABLE IF NOT EXISTS player_store_kv (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Seed defaults if not present (optional, safe no-ops on conflict)
INSERT OR IGNORE INTO player_store_kv (key, value) VALUES
  ("player_state", '{"current_time":0.0,"last_song":null,"last_song_played_duration":0.0,"force_seek":0.0,"state":"STOPPED","has_repeated":false,"repeat":"NONE","old_volume":0.0,"volume":100.0,"volume_mode":"NORMAL","volume_map":{},"clamp_map":{}}'),
  ("song_queue", '[]'),
  ("current_index", '0'),
  ("queue_data", '{}');

CREATE INDEX IF NOT EXISTS idx_player_store_kv_updated_at ON player_store_kv(updated_at);
