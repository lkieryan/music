use sqlx::SqlitePool;
use anyhow::Result;
use crate::database::models::{LocalSongModel};
use crate::sources::LocalSong;

/// 本地音乐库数据库操作
pub struct LocalLibraryDb;

impl LocalLibraryDb {
    /// 添加本地歌曲到数据库
    pub async fn add_local_song(pool: &SqlitePool, song: &LocalSong) -> Result<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO local_songs 
            (file_path, song_id, title, artist, album, duration_seconds, file_size, format, bitrate, sample_rate, last_modified)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&song.file_path.to_string_lossy())
        .bind(&song.id)
        .bind(&song.title)
        .bind(&song.artist)
        .bind(&song.album)
        .bind(song.duration.as_secs() as i32)
        .bind(song.file_size as i64)
        .bind(&format!("{:?}", song.format))
        .bind(song.bitrate.map(|b| b as i32))
        .bind(song.sample_rate.map(|s| s as i32))
        .bind(song.last_modified)
        .execute(pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }

    /// 批量添加本地歌曲
    pub async fn add_local_songs_batch(pool: &SqlitePool, songs: &[LocalSong]) -> Result<()> {
        let mut tx = pool.begin().await?;
        
        for song in songs {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO local_songs 
                (file_path, song_id, title, artist, album, duration_seconds, file_size, format, bitrate, sample_rate, last_modified)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&song.file_path.to_string_lossy())
            .bind(&song.id)
            .bind(&song.title)
            .bind(&song.artist)
            .bind(&song.album)
            .bind(song.duration.as_secs() as i32)
            .bind(song.file_size as i64)
            .bind(&format!("{:?}", song.format))
            .bind(song.bitrate.map(|b| b as i32))
            .bind(song.sample_rate.map(|s| s as i32))
            .bind(song.last_modified)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }

    /// 获取所有本地歌曲
    pub async fn get_all_local_songs(pool: &SqlitePool) -> Result<Vec<LocalSongModel>> {
        let songs = sqlx::query_as::<_, LocalSongModel>(
            "SELECT * FROM local_songs ORDER BY artist, album, title"
        )
        .fetch_all(pool)
        .await?;
        
        Ok(songs)
    }

    /// 按艺术家搜索本地歌曲
    pub async fn search_by_artist(pool: &SqlitePool, artist: &str) -> Result<Vec<LocalSongModel>> {
        let songs = sqlx::query_as::<_, LocalSongModel>(
            "SELECT * FROM local_songs WHERE artist LIKE ? ORDER BY album, title"
        )
        .bind(format!("%{}%", artist))
        .fetch_all(pool)
        .await?;
        
        Ok(songs)
    }

    /// 按专辑搜索本地歌曲
    pub async fn search_by_album(pool: &SqlitePool, album: &str) -> Result<Vec<LocalSongModel>> {
        let songs = sqlx::query_as::<_, LocalSongModel>(
            "SELECT * FROM local_songs WHERE album LIKE ? ORDER BY title"
        )
        .bind(format!("%{}%", album))
        .fetch_all(pool)
        .await?;
        
        Ok(songs)
    }

    /// 按标题搜索本地歌曲
    pub async fn search_by_title(pool: &SqlitePool, title: &str) -> Result<Vec<LocalSongModel>> {
        let songs = sqlx::query_as::<_, LocalSongModel>(
            "SELECT * FROM local_songs WHERE title LIKE ? ORDER BY artist, album"
        )
        .bind(format!("%{}%", title))
        .fetch_all(pool)
        .await?;
        
        Ok(songs)
    }

    /// 综合搜索本地歌曲
    pub async fn search_local_songs(pool: &SqlitePool, query: &str) -> Result<Vec<LocalSongModel>> {
        let songs = sqlx::query_as::<_, LocalSongModel>(
            r#"
            SELECT * FROM local_songs 
            WHERE title LIKE ? OR artist LIKE ? OR album LIKE ?
            ORDER BY 
                CASE 
                    WHEN title LIKE ? THEN 1
                    WHEN artist LIKE ? THEN 2
                    WHEN album LIKE ? THEN 3
                    ELSE 4
                END,
                artist, album, title
            "#
        )
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .bind(format!("%{}%", query))
        .fetch_all(pool)
        .await?;
        
        Ok(songs)
    }

    /// 删除不存在的文件记录
    pub async fn remove_missing_files(pool: &SqlitePool) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM local_songs 
            WHERE file_path NOT IN (
                SELECT file_path FROM local_songs 
                WHERE file_path IS NOT NULL
            )
            "#
        )
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected() as i64)
    }

    /// 获取本地音乐库统计信息
    pub async fn get_library_stats(pool: &SqlitePool) -> Result<LibraryStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_songs,
                COUNT(DISTINCT artist) as total_artists,
                COUNT(DISTINCT album) as total_albums,
                SUM(duration_seconds) as total_duration,
                SUM(file_size) as total_size
            FROM local_songs
            "#
        )
        .fetch_one(pool)
        .await?;
        
        Ok(LibraryStats {
            total_songs: row.get("total_songs"),
            total_artists: row.get("total_artists"),
            total_albums: row.get("total_albums"),
            total_duration: row.get::<i64, _>("total_duration") as u64,
            total_size: row.get::<i64, _>("total_size") as u64,
        })
    }
}

/// 音乐库统计信息
#[derive(Debug, serde::Serialize)]
pub struct LibraryStats {
    pub total_songs: i64,
    pub total_artists: i64,
    pub total_albums: i64,
    pub total_duration: u64, // 秒
    pub total_size: u64,     // 字节
}