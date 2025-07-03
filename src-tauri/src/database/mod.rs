pub mod models;
pub mod local_library;
pub mod cache;
pub mod migration;

use sqlx::{SqlitePool, migrate::MigrateDatabase, Sqlite};
use anyhow::Result;
use dirs::data_dir;
use std::path::PathBuf;

pub use models::*;
pub use local_library::*;
pub use cache::*;

/// 初始化数据库
pub async fn init_database() -> Result<SqlitePool> {
    let db_path = get_db_path()?;
    let db_url = format!("sqlite:{}", db_path.display());
    
    // 创建数据库文件（如果不存在）
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        log::info!("Creating database at: {}", db_url);
        Sqlite::create_database(&db_url).await?;
    }
    
    let pool = SqlitePool::connect(&db_url).await?;
    
    // 运行数据库迁移
    migration::run_migrations(&pool).await?;
    
    log::info!("Database initialized successfully");
    Ok(pool)
}

/// 获取数据库文件路径
fn get_db_path() -> Result<PathBuf> {
    let mut path = data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("moekoe-music-rust");
    
    // 确保目录存在
    std::fs::create_dir_all(&path)?;
    
    path.push("database.db");
    Ok(path)
}