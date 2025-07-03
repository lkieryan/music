use sqlx::SqlitePool;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};

/// 缓存管理器
pub struct CacheManager;

impl CacheManager {
    /// 设置缓存值
    pub async fn set<T>(
        pool: &SqlitePool,
        key: &str,
        value: &T,
        ttl_seconds: Option<i64>,
    ) -> Result<()>
    where
        T: Serialize,
    {
        let value_json = serde_json::to_string(value)?;
        let expires_at = ttl_seconds.map(|ttl| Utc::now() + Duration::seconds(ttl));
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO cache (key, value, expires_at, created_at)
            VALUES (?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(key)
        .bind(&value_json)
        .bind(expires_at)
        .execute(pool)
        .await?;
        
        Ok(())
    }

    /// 获取缓存值
    pub async fn get<T>(pool: &SqlitePool, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        // 首先清理过期缓存
        Self::cleanup_expired(pool).await?;
        
        let row = sqlx::query(
            r#"
            SELECT value FROM cache 
            WHERE key = ? AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            "#,
        )
        .bind(key)
        .fetch_optional(pool)
        .await?;
        
        if let Some(row) = row {
            let value_json: String = row.get("value");
            let value = serde_json::from_str(&value_json)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// 删除缓存
    pub async fn delete(pool: &SqlitePool, key: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM cache WHERE key = ?")
            .bind(key)
            .execute(pool)
            .await?;
        
        Ok(result.rows_affected() > 0)
    }

    /// 检查缓存是否存在且未过期
    pub async fn exists(pool: &SqlitePool, key: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1 FROM cache 
            WHERE key = ? AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            "#,
        )
        .bind(key)
        .fetch_optional(pool)
        .await?;
        
        Ok(row.is_some())
    }

    /// 清理过期缓存
    pub async fn cleanup_expired(pool: &SqlitePool) -> Result<i64> {
        let result = sqlx::query(
            "DELETE FROM cache WHERE expires_at IS NOT NULL AND expires_at <= CURRENT_TIMESTAMP"
        )
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected() as i64)
    }

    /// 清空所有缓存
    pub async fn clear_all(pool: &SqlitePool) -> Result<i64> {
        let result = sqlx::query("DELETE FROM cache")
            .execute(pool)
            .await?;
        
        Ok(result.rows_affected() as i64)
    }

    /// 获取缓存统计信息
    pub async fn get_stats(pool: &SqlitePool) -> Result<CacheStats> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_entries,
                COUNT(CASE WHEN expires_at IS NOT NULL AND expires_at <= CURRENT_TIMESTAMP THEN 1 END) as expired_entries,
                COUNT(CASE WHEN expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP THEN 1 END) as valid_entries
            FROM cache
            "#,
        )
        .fetch_one(pool)
        .await?;
        
        Ok(CacheStats {
            total_entries: row.get("total_entries"),
            expired_entries: row.get("expired_entries"),
            valid_entries: row.get("valid_entries"),
        })
    }
}

/// 缓存统计信息
#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_entries: i64,
    pub expired_entries: i64,
    pub valid_entries: i64,
}

/// 为缓存表创建迁移
pub async fn create_cache_table(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS cache (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            key TEXT UNIQUE NOT NULL,
            value TEXT NOT NULL,
            expires_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;
    
    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_cache_key ON cache(key)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_cache_expires_at ON cache(expires_at)")
        .execute(pool)
        .await?;
    
    Ok(())
}