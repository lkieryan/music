use std::{
    collections::HashMap,
    path::PathBuf,
    sync::RwLock,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

/// 文件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
}

/// 文件缓存，用于跟踪已扫描的文件状态
/// 避免重复扫描未修改的文件
pub struct FileCache {
    cache: RwLock<HashMap<PathBuf, FileMetadata>>,
}

impl FileCache {
    /// 创建新的文件缓存
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 从现有的缓存数据创建
    pub fn from_data(data: HashMap<PathBuf, FileMetadata>) -> Self {
        Self {
            cache: RwLock::new(data),
        }
    }

    /// 获取文件信息
    pub fn get_file(&self, path: &PathBuf) -> Option<FileMetadata> {
        let cache = self.cache.read().unwrap();
        cache.get(path).cloned()
    }

    /// 更新文件信息
    pub fn update_file(&self, path: &PathBuf, metadata: FileMetadata) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(path.clone(), metadata);
    }

    /// 移除文件信息
    pub fn remove_file(&self, path: &PathBuf) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(path);
    }

    /// 获取所有缓存的文件
    pub fn get_all_files(&self) -> Vec<FileMetadata> {
        let cache = self.cache.read().unwrap();
        cache.values().cloned().collect()
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    /// 获取缓存大小
    pub fn len(&self) -> usize {
        let cache = self.cache.read().unwrap();
        cache.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        let cache = self.cache.read().unwrap();
        cache.is_empty()
    }

    /// 检查文件是否需要重新扫描
    pub fn needs_rescan(&self, path: &PathBuf) -> bool {
        if let Some(cached) = self.get_file(path) {
            if let Ok(metadata) = std::fs::metadata(path) {
                // 检查文件大小和修改时间是否变化
                return cached.size != metadata.len() || 
                       cached.modified != metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            }
        }
        
        // 如果没有缓存或无法获取文件信息，需要扫描
        true
    }

    /// 批量检查多个文件是否需要重新扫描
    pub fn batch_needs_rescan(&self, paths: &[(PathBuf, u64)]) -> Vec<(PathBuf, u64)> {
        paths
            .iter()
            .filter(|(path, size)| {
                if let Some(cached) = self.get_file(path) {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        // 文件大小或修改时间发生变化
                        cached.size != *size || 
                        cached.modified != metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH)
                    } else {
                        true // 无法获取文件信息，需要扫描
                    }
                } else {
                    true // 没有缓存，需要扫描
                }
            })
            .cloned()
            .collect()
    }

    /// 序列化缓存数据
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let cache = self.cache.read().unwrap();
        let serialized = serde_json::to_vec(&*cache)?;
        Ok(serialized)
    }

    /// 反序列化缓存数据
    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let cache_data: HashMap<PathBuf, FileMetadata> = serde_json::from_slice(data)?;
        Ok(Self::from_data(cache_data))
    }

    /// 将缓存保存到文件
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = self.serialize()?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// 从文件加载缓存
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if !path.exists() {
            return Ok(Self::new());
        }
        
        let data = std::fs::read(path)?;
        Self::deserialize(&data)
    }

    /// 清理无效的缓存条目（对应的文件已不存在）
    pub fn cleanup_invalid_entries(&self) {
        let mut cache = self.cache.write().unwrap();
        let mut to_remove = Vec::new();
        
        for (path, _) in cache.iter() {
            if !path.exists() {
                to_remove.push(path.clone());
            }
        }
        
        for path in to_remove {
            cache.remove(&path);
        }
    }

    /// 获取缓存统计信息
    pub fn get_stats(&self) -> CacheStats {
        let cache = self.cache.read().unwrap();
        let mut total_size = 0u64;
        let mut valid_files = 0usize;
        let mut invalid_files = 0usize;
        
        for (path, metadata) in cache.iter() {
            if path.exists() {
                valid_files += 1;
                total_size += metadata.size;
            } else {
                invalid_files += 1;
            }
        }
        
        CacheStats {
            total_files: cache.len(),
            valid_files,
            invalid_files,
            total_size,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_files: usize,
    pub valid_files: usize,
    pub invalid_files: usize,
    pub total_size: u64,
}

impl Default for FileCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::SystemTime;

    #[test]
    fn test_file_cache_basic_operations() {
        let cache = FileCache::new();
        let path = PathBuf::from("test_file.mp3");
        let metadata = FileMetadata {
            path: path.clone(),
            size: 1024,
            modified: SystemTime::now(),
        };

        // 测试添加和获取
        cache.update_file(&path, metadata.clone());
        let retrieved = cache.get_file(&path).unwrap();
        assert_eq!(retrieved.path, metadata.path);
        assert_eq!(retrieved.size, metadata.size);

        // 测试移除
        cache.remove_file(&path);
        assert!(cache.get_file(&path).is_none());
    }

    #[test]
    fn test_cache_serialization() {
        let cache = FileCache::new();
        let path = PathBuf::from("test_file.mp3");
        let metadata = FileMetadata {
            path: path.clone(),
            size: 2048,
            modified: SystemTime::now(),
        };

        cache.update_file(&path, metadata);
        
        // 序列化
        let serialized = cache.serialize().unwrap();
        
        // 反序列化
        let restored_cache = FileCache::deserialize(&serialized).unwrap();
        let retrieved = restored_cache.get_file(&path).unwrap();
        assert_eq!(retrieved.size, 2048);
    }
}