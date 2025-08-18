-- Rollback player store KV table
DROP INDEX IF EXISTS idx_player_store_kv_updated_at;
DROP TABLE IF EXISTS player_store_kv;
