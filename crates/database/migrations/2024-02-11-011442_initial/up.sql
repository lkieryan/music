-- Your SQL goes here
CREATE TABLE `artist_bridge`(
	`id` INTEGER PRIMARY KEY,
	`track` TEXT,
	`artist` TEXT,
	FOREIGN KEY (`track`) REFERENCES `tracks`(`_id`),
	FOREIGN KEY (`artist`) REFERENCES `artists`(`artist_id`)
);

CREATE TABLE `playlist_bridge`(
	`id` INTEGER PRIMARY KEY,
	`track` TEXT,
	`playlist` TEXT,
	FOREIGN KEY (`track`) REFERENCES `tracks`(`_id`),
	FOREIGN KEY (`playlist`) REFERENCES `playlists`(`playlist_id`)
);

CREATE TABLE `artists`(
	`artist_id` TEXT PRIMARY KEY,
	`artist_mbid` TEXT,
	`artist_name` TEXT,
	`artist_coverpath` TEXT,
	`artist_track_count` DOUBLE NOT NULL,
	`artist_extra_info` TEXT,
	`sanitized_artist_name` TEXT
);

CREATE TABLE `album_bridge`(
	`id` INTEGER PRIMARY KEY,
	`track` TEXT,
	`album` TEXT,
	FOREIGN KEY (`track`) REFERENCES `tracks`(`_id`),
	FOREIGN KEY (`album`) REFERENCES `albums`(`album_id`)
);

CREATE TABLE `genres`(
	`genre_id` TEXT PRIMARY KEY,
	`genre_name` TEXT,
	`genre_track_count` DOUBLE NOT NULL
);

CREATE TABLE `playlists`(
	`playlist_id` TEXT PRIMARY KEY,
	`playlist_name` TEXT NOT NULL,
	`playlist_coverpath` TEXT,
	`playlist_track_count` DOUBLE NOT NULL,
	`playlist_desc` TEXT,
	`playlist_path` TEXT,
	`extension` TEXT,
	`icon` TEXT
);

CREATE TABLE `tracks`(
	`_id` TEXT PRIMARY KEY,
	`path` TEXT,
	`size` DOUBLE,
	`inode` TEXT,
	`deviceno` TEXT,
	`title` TEXT,
	`date` TEXT,
	`year` TEXT,
	`lyrics` TEXT,
	`releasetype` TEXT,
	`bitrate` DOUBLE,
	`codec` TEXT,
	`container` TEXT,
	`duration` DOUBLE,
	`samplerate` DOUBLE,
	`hash` TEXT,
	`type` TEXT NOT NULL,
	`url` TEXT,
	`track_coverpath_high` TEXT,
	`playbackurl` TEXT,
	`track_coverpath_low` TEXT,
	`date_added` UNSIGNED BIG INT,
	`provider_extension` TEXT,
	`icon` TEXT,
	`show_in_library` BOOL,
	`track_no` DOUBLE,

	`provider` TEXT,                    -- Provider name (e.g. spotify, bilibili, local)
	`provider_id` TEXT,                 -- Provider original identifier
	`disc_number` INTEGER,              -- Disc number
	`preview_url` TEXT,                 -- Preview clip URL
	`isrc` TEXT,                        -- ISRC code
	`popularity` INTEGER,               -- Popularity score 0-100
	`metadata` TEXT,                    -- Additional metadata (JSON serialized)
	`quality` TEXT,                     -- Audio quality information (JSON serialized)
	`availability` TEXT,                -- Availability constraints (JSON serialized)
	`album_ref` TEXT,                   -- Album reference (JSON serialized)
	`sample_rate` REAL,                 -- Sample rate (rename from samplerate for consistency)
	`created_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	`updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE `genre_bridge`(
	`id` INTEGER PRIMARY KEY,
	`track` TEXT,
	`genre` TEXT,
	FOREIGN KEY (`track`) REFERENCES `tracks`(`_id`),
	FOREIGN KEY (`genre`) REFERENCES `genres`(`genre_id`)
);

CREATE TABLE `albums`(
	`album_id` TEXT PRIMARY KEY,
	`album_name` TEXT,
	`album_artist` TEXT,
	`album_coverpath_high` TEXT,
	`album_track_count` DOUBLE NOT NULL,
	`year` TEXT,
	`album_coverpath_low` TEXT,
	`album_extra_info` TEXT
);


-- Plugin states table for managing plugin lifecycle and configuration
CREATE TABLE plugin_states (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    version TEXT NOT NULL,
    plugin_type TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT false,
    installed BOOLEAN NOT NULL DEFAULT true,
    builtin BOOLEAN NOT NULL DEFAULT false,
    config TEXT NOT NULL,
	icon TEXT,
    manifest TEXT,
    installed_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used TIMESTAMP
);

-- Create indices for better query performance
CREATE INDEX idx_plugin_states_name ON plugin_states(name);
CREATE INDEX idx_plugin_states_enabled ON plugin_states(enabled);
CREATE INDEX idx_plugin_states_plugin_type ON plugin_states(plugin_type);
CREATE INDEX idx_plugin_states_installed_at ON plugin_states(installed_at);
CREATE INDEX idx_plugin_states_last_updated ON plugin_states(last_updated);
CREATE INDEX idx_plugin_states_last_used ON plugin_states(last_used);