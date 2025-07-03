use sqlx::SqlitePool;
use anyhow::Result;

/// 运行数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    log::info!("Running database migrations...");
    
    // 创建歌单表
    create_playlists_table(pool).await?;
    
    // 创建歌单歌曲关联表
    create_playlist_songs_table(pool).await?;
    
    // 创建收藏歌曲表
    create_favorite_songs_table(pool).await?;
    
    // 创建播放历史表
    create_play_history_table(pool).await?;
    
    // 创建搜索历史表
    create_search_history_table(pool).await?;
    
    // 创建本地音乐库表
    create_local_songs_table(pool).await?;
    
    // 创建下载任务表
    create_download_tasks_table(pool).await?;
    
    // 创建应用配置表
    create_app_config_table(pool).await?;
    
    // 创建缓存表
    super::cache::create_cache_table(pool).await?;
    
    log::info!("Database migrations completed");
    Ok(())
}

async fn create_playlists_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            cover_url TEXT,
            song_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_playlist_songs_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playlist_songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            playlist_id INTEGER NOT NULL,
            song_id TEXT NOT NULL,
            song_title TEXT NOT NULL,
            artist_name TEXT NOT NULL,
            album_name TEXT,
            duration_seconds INTEGER DEFAULT 0,
            source_type TEXT NOT NULL,
            source_id TEXT NOT NULL,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (playlist_id) REFERENCES playlists (id) ON DELETE CASCADE,
            UNIQUE(playlist_id, song_id)
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_favorite_songs_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS favorite_songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id TEXT UNIQUE NOT NULL,
            song_title TEXT NOT NULL,
            artist_name TEXT NOT NULL,
            album_name TEXT,
            duration_seconds INTEGER DEFAULT 0,
            source_type TEXT NOT NULL,
            source_id TEXT NOT NULL,
            cover_url TEXT,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_play_history_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS play_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id TEXT NOT NULL,
            song_title TEXT NOT NULL,
            artist_name TEXT NOT NULL,
            album_name TEXT,
            duration_seconds INTEGER DEFAULT 0,
            source_type TEXT NOT NULL,
            source_id TEXT NOT NULL,
            played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            play_duration INTEGER
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    // 创建索引以提高查询性能
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_play_history_played_at ON play_history(played_at)")
        .execute(pool)
        .await?;
    
    Ok(())
}

async fn create_search_history_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS search_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            keyword TEXT UNIQUE NOT NULL,
            search_count INTEGER DEFAULT 1,
            last_searched DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_local_songs_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS local_songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_path TEXT UNIQUE NOT NULL,
            song_id TEXT UNIQUE NOT NULL,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT NOT NULL,
            duration_seconds INTEGER DEFAULT 0,
            file_size INTEGER DEFAULT 0,
            format TEXT NOT NULL,
            bitrate INTEGER,
            sample_rate INTEGER,
            last_modified DATETIME NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_local_songs_artist ON local_songs(artist)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_local_songs_album ON local_songs(album)")
        .execute(pool)
        .await?;
    
    Ok(())
}

async fn create_download_tasks_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS download_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id TEXT NOT NULL,
            song_title TEXT NOT NULL,
            artist_name TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_id TEXT NOT NULL,
            download_url TEXT NOT NULL,
            save_path TEXT NOT NULL,
            status TEXT DEFAULT 'pending',
            progress REAL DEFAULT 0.0,
            total_size INTEGER,
            downloaded_size INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn create_app_config_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS app_config (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT UNIQUE NOT NULL,
            value TEXT NOT NULL,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}