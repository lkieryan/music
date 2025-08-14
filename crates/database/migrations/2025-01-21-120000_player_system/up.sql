-- Your SQL goes here
-- 播放器状态表
CREATE TABLE `player_state`(
    `id` INTEGER PRIMARY KEY,
    `current_song_id` TEXT,
    `current_position` DOUBLE DEFAULT 0.0,
    `volume` REAL DEFAULT 1.0,
    `play_mode` TEXT DEFAULT 'Sequential',
    `is_playing` BOOL DEFAULT FALSE,
    `is_paused` BOOL DEFAULT FALSE,
    `shuffle_enabled` BOOL DEFAULT FALSE,
    `repeat_mode` TEXT DEFAULT 'None',
    `last_updated` DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (`current_song_id`) REFERENCES `allsongs`(`_id`)
);

-- 播放队列表
CREATE TABLE `play_queue`(
    `id` INTEGER PRIMARY KEY AUTOINCREMENT,
    `song_id` TEXT NOT NULL,
    `position` INTEGER NOT NULL,
    `added_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `queue_type` TEXT DEFAULT 'current',
    FOREIGN KEY (`song_id`) REFERENCES `allsongs`(`_id`),
    UNIQUE(`song_id`, `position`, `queue_type`)
);

-- 播放历史表
CREATE TABLE `play_history`(
    `id` INTEGER PRIMARY KEY AUTOINCREMENT,
    `song_id` TEXT NOT NULL,
    `played_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `play_duration` DOUBLE DEFAULT 0.0,
    `completed` BOOL DEFAULT FALSE,
    `session_id` TEXT,
    FOREIGN KEY (`song_id`) REFERENCES `allsongs`(`_id`)
);

-- 用户收藏表 (如果不存在的话)
CREATE TABLE IF NOT EXISTS `favorites`(
    `id` INTEGER PRIMARY KEY AUTOINCREMENT,
    `song_id` TEXT NOT NULL,
    `added_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (`song_id`) REFERENCES `allsongs`(`_id`),
    UNIQUE(`song_id`)
);

-- 播放器设置表
CREATE TABLE `player_settings`(
    `id` INTEGER PRIMARY KEY,
    `setting_key` TEXT NOT NULL UNIQUE,
    `setting_value` TEXT,
    `setting_type` TEXT DEFAULT 'string',
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 插入默认播放器状态
INSERT INTO `player_state` (id, volume, play_mode, is_playing, is_paused)
VALUES (1, 1.0, 'Sequential', FALSE, FALSE);

-- 插入默认播放器设置
INSERT INTO `player_settings` (setting_key, setting_value, setting_type) VALUES
('crossfade_duration', '0.0', 'float'),
('auto_play_next', 'true', 'boolean'),
('remember_position', 'true', 'boolean'),
('gapless_playback', 'false', 'boolean'),
('replay_gain', 'false', 'boolean'),
('equalizer_enabled', 'false', 'boolean'),
('audio_device', 'default', 'string');

-- 创建索引以提高查询性能
CREATE INDEX idx_play_queue_position ON play_queue(position);
CREATE INDEX idx_play_queue_song ON play_queue(song_id);
CREATE INDEX idx_play_history_song ON play_history(song_id);
CREATE INDEX idx_play_history_played_at ON play_history(played_at);
CREATE INDEX idx_favorites_song ON favorites(song_id);