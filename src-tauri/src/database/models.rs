use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// 歌单数据模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlaylistModel {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub song_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 歌单歌曲关联模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlaylistSongModel {
    pub id: i64,
    pub playlist_id: i64,
    pub song_id: String,
    pub song_title: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration_seconds: i32,
    pub source_type: String,
    pub source_id: String,
    pub added_at: DateTime<Utc>,
}

/// 收藏歌曲模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FavoriteSongModel {
    pub id: i64,
    pub song_id: String,
    pub song_title: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration_seconds: i32,
    pub source_type: String,
    pub source_id: String,
    pub cover_url: Option<String>,
    pub added_at: DateTime<Utc>,
}

/// 播放历史模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PlayHistoryModel {
    pub id: i64,
    pub song_id: String,
    pub song_title: String,
    pub artist_name: String,
    pub album_name: Option<String>,
    pub duration_seconds: i32,
    pub source_type: String,
    pub source_id: String,
    pub played_at: DateTime<Utc>,
    pub play_duration: Option<i32>, // 实际播放时长（秒）
}

/// 搜索历史模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchHistoryModel {
    pub id: i64,
    pub keyword: String,
    pub search_count: i32,
    pub last_searched: DateTime<Utc>,
}

/// 本地音乐库模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LocalSongModel {
    pub id: i64,
    pub file_path: String,
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_seconds: i32,
    pub file_size: i64,
    pub format: String,
    pub bitrate: Option<i32>,
    pub sample_rate: Option<i32>,
    pub last_modified: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// 下载任务模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DownloadTaskModel {
    pub id: i64,
    pub song_id: String,
    pub song_title: String,
    pub artist_name: String,
    pub source_type: String,
    pub source_id: String,
    pub download_url: String,
    pub save_path: String,
    pub status: String, // pending, downloading, completed, failed
    pub progress: f32,
    pub total_size: Option<i64>,
    pub downloaded_size: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 应用配置模型
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AppConfigModel {
    pub id: i64,
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}