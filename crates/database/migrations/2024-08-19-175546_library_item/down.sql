-- This file should undo anything in `up.sql`
ALTER TABLE tracks
DROP COLUMN library_item;
