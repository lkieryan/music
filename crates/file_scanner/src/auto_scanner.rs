use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, Ordering}, Arc, RwLock},
    time::{Duration, UNIX_EPOCH},
};

use crossbeam_channel::unbounded as _;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time::interval};
use tracing::{debug, error, info, warn};
use types::{
    entities::QueryablePlaylist,
    errors::Result,
    tracks::MediaContent,
};

use crate::{
    file_cache::{FileCache, FileMetadata},
    utils::{get_files_recursively, scan_file},
};

/// 扫描事件类型
#[derive(Debug, Clone)]
pub enum ScanEvent {
    /// 文件被添加
    FileAdded(PathBuf),
    /// 文件被修改
    FileModified(PathBuf),
    /// 文件被删除
    FileDeleted(PathBuf),
    /// 定时全量扫描
    ScheduledScan,
    /// 手动触发扫描
    ManualScan(Vec<PathBuf>),
}

/// 扫描结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub tracks: Vec<MediaContent>,
    pub playlists: Vec<QueryablePlaylist>,
    pub deleted_files: Vec<PathBuf>,
}

/// 自动扫描器配置
#[derive(Debug, Clone)]
pub struct AutoScannerConfig {
    /// 要扫描的路径列表
    pub scan_paths: Vec<PathBuf>,
    /// 排除的路径列表
    pub exclude_paths: Vec<PathBuf>,
    /// 扫描间隔（秒）
    pub scan_interval: u64,
    /// 是否启用文件系统监控
    pub enable_fs_watch: bool,
    /// 是否启用定时扫描
    pub enable_scheduled_scan: bool,
    /// 扫描线程数
    pub scan_threads: usize,
    /// 缩略图目录
    pub thumbnail_dir: PathBuf,
    /// 艺术家分隔符
    pub artist_splitter: String,
    /// 最小扫描时长过滤 ("sec30" | "min2" | "all")
    pub scan_min_duration: String,
    /// 扫描格式过滤 ("common" | "all")
    pub scan_formats: String,
}

impl Default for AutoScannerConfig {
    fn default() -> Self {
        Self {
            scan_paths: Vec::new(),
            exclude_paths: Vec::new(),
            scan_interval: 3600, // 1 hour
            enable_fs_watch: true,
            enable_scheduled_scan: true,
            scan_threads: num_cpus::get(),
            thumbnail_dir: PathBuf::from("thumbnails"),
            artist_splitter: ";".to_string(),
            scan_min_duration: "sec30".to_string(),
            scan_formats: "common".to_string(),
        }
    }
}

/// 自动扫描器状态
#[derive(Debug, Clone, PartialEq)]
pub enum ScannerState {
    Idle,
    Scanning,
    Watching,
    Stopped,
}

/// 自动扫描器
pub struct AutoScanner {
    config: Arc<RwLock<AutoScannerConfig>>,
    state: Arc<RwLock<ScannerState>>,
    file_cache: Arc<FileCache>,
    is_running: Arc<AtomicBool>,
    
    // 事件通道
    event_tx: mpsc::UnboundedSender<ScanEvent>,
    event_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<ScanEvent>>>,
    
    // 结果通道
    result_tx: Option<crossbeam_channel::Sender<ScanResult>>,
    
    // 文件系统监控器
    _watcher: Option<RecommendedWatcher>,
}

impl AutoScanner {
    /// 创建新的自动扫描器
    pub fn new(config: AutoScannerConfig) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        
        // 尝试从缓存文件加载文件缓存
        let cache_file_path = config.thumbnail_dir.join("file_cache.json");
        let file_cache = Arc::new(
            FileCache::load_from_file(&cache_file_path)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to load file cache from {:?}: {}, creating new cache", cache_file_path, e);
                    FileCache::new()
                })
        );
        
        
        tracing::info!("Loaded file cache with {} entries", file_cache.len());
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            state: Arc::new(RwLock::new(ScannerState::Idle)),
            file_cache,
            is_running: Arc::new(AtomicBool::new(false)),
            event_tx,
            event_rx: Arc::new(tokio::sync::Mutex::new(event_rx)),
            result_tx: None,
            _watcher: None,
        })
    }

    /// 设置结果回调通道
    pub fn set_result_channel(&mut self, tx: crossbeam_channel::Sender<ScanResult>) {
        self.result_tx = Some(tx);
    }

    /// 获取当前状态
    pub fn get_state(&self) -> ScannerState {
        self.state.read().unwrap().clone()
    }

    /// 更新配置
    pub fn update_config(&self, config: AutoScannerConfig) -> Result<()> {
        let old_roots = {
            let current = self.config.read().unwrap();
            current.scan_paths.clone()
        };

        let new_roots = config.scan_paths.clone();
        let mut current_config = self.config.write().unwrap();
        *current_config = config;

        let removed_roots: Vec<PathBuf> = old_roots
            .into_iter()
            .filter(|p| !new_roots.iter().any(|n| n == p))
            .collect();

        if !removed_roots.is_empty() {
            let mut deleted = Vec::new();
            for meta in self.file_cache.get_all_files() {
                if removed_roots.iter().any(|root| meta.path.starts_with(root)) {
                    deleted.push(meta.path.clone());
                    self.file_cache.remove_file(&meta.path);
                }
            }

            if !deleted.is_empty() {
                if let Some(tx) = &self.result_tx {
                    let _ = tx.send(ScanResult {
                        tracks: Vec::new(),
                        playlists: Vec::new(),
                        deleted_files: deleted,
                    });
                }
            }
        }
        
        if self.is_running.load(Ordering::Acquire) {
            info!("Configuration updated, restarting scanner");
            let _ = self.event_tx.send(ScanEvent::ScheduledScan);
        }
        
        Ok(())
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.is_running.load(Ordering::Acquire) {
            warn!("Scanner is already running");
            return Ok(());
        }

        self.is_running.store(true, Ordering::Release);
        *self.state.write().unwrap() = ScannerState::Watching;
        
        info!("Starting auto scanner");

        if self.config.read().unwrap().enable_fs_watch {
            self.start_file_watcher().await?;
        }
        if self.config.read().unwrap().enable_scheduled_scan {
            self.start_scheduled_scan().await;
        }

        self.start_event_loop().await;

        Ok(())
    }

    pub async fn stop(&self) {
        info!("Stopping auto scanner");
        self.is_running.store(false, Ordering::Release);
        *self.state.write().unwrap() = ScannerState::Stopped;
        self.save_file_cache().await;
    }

    async fn save_file_cache(&self) {
        let config = self.config.read().unwrap();
        let _cache_file_path = config.thumbnail_dir.join("file_cache.json");
        
        // if let Some(parent) = _cache_file_path.parent() {
        //     if let Err(e) = std::fs::create_dir_all(parent) {
        //         tracing::error!("Failed to create cache directory {:?}: {}", parent, e);
        //         return;
        //     }
        // }
        

        // self.file_cache.cleanup_invalid_entries();
        
        // if let Err(e) = self.file_cache.save_to_file(&cache_file_path) {
        //     tracing::error!("Failed to save file cache to {:?}: {}", cache_file_path, e);
        // } else {
        //     tracing::info!("Saved file cache with {} entries to {:?}", 
        //                  self.file_cache.len(), cache_file_path);
        // }
    }

    pub fn trigger_scan(&self, paths: Option<Vec<PathBuf>>) -> Result<()> {
        let scan_event = if let Some(paths) = paths {
            ScanEvent::ManualScan(paths)
        } else {
            ScanEvent::ScheduledScan
        };

        self.event_tx
            .send(scan_event)
            .map_err(|e| format!("Failed to send scan event: {}", e))?;
        
        Ok(())
    }

    async fn start_file_watcher(&mut self) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let scan_paths = self.config.read().unwrap().scan_paths.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                match res {
                    Ok(event) => {
                        match event.kind {
                            EventKind::Create(_) => {
                                for path in event.paths {
                                    if Self::is_music_file(&path) {
                                        let _ = event_tx.send(ScanEvent::FileAdded(path));
                                    }
                                }
                            }
                            EventKind::Modify(_) => {
                                for path in event.paths {
                                    if Self::is_music_file(&path) {
                                        let _ = event_tx.send(ScanEvent::FileModified(path));
                                    }
                                }
                            }
                            EventKind::Remove(_) => {
                                for path in event.paths {
                                    let _ = event_tx.send(ScanEvent::FileDeleted(path));
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("File watcher error: {:?}", e);
                    }
                }
            },
            Config::default(),
        ).map_err(|e| format!("Failed to create file watcher: {}", e))?;

        for path in scan_paths {
            if path.exists() {
                watcher
                    .watch(&path, RecursiveMode::Recursive)
                    .map_err(|e| format!("Failed to watch path {:?}: {}", path, e))?;
                info!("Watching path: {:?}", path);
            }
        }

        self._watcher = Some(watcher);
        Ok(())
    }

    async fn start_scheduled_scan(&self) {
        let event_tx = self.event_tx.clone();
        let is_running = self.is_running.clone();
        let scan_interval = self.config.read().unwrap().scan_interval;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(scan_interval));
            
            while is_running.load(Ordering::Acquire) {
                interval.tick().await;
                
                if is_running.load(Ordering::Acquire) {
                    let _ = event_tx.send(ScanEvent::ScheduledScan);
                }
            }
        });
    }

    async fn start_event_loop(&self) {
        let event_rx = self.event_rx.clone();
        let config = self.config.clone();
        let state = self.state.clone();
        let file_cache = self.file_cache.clone();
        let is_running = self.is_running.clone();
        let result_tx = self.result_tx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut rx = event_rx.lock().await;
                
                while is_running.load(Ordering::Acquire) {
                    if let Some(event) = rx.recv().await {
                        debug!("Processing scan event: {:?}", event);
                        *state.write().unwrap() = ScannerState::Scanning;
                        
                        let result = match event {
                            ScanEvent::FileAdded(path) => {
                                Self::handle_file_added(&config, &file_cache, path).await
                            }
                            ScanEvent::FileModified(path) => {
                                Self::handle_file_modified(&config, &file_cache, path).await
                            }
                            ScanEvent::FileDeleted(path) => {
                                Self::handle_file_deleted(&file_cache, path).await
                            }
                            ScanEvent::ScheduledScan => {
                                Self::handle_full_scan(&config, &file_cache).await
                            }
                            ScanEvent::ManualScan(paths) => {
                                Self::handle_manual_scan(&config, &file_cache, paths).await
                            }
                        };

                        match result {
                            Ok(scan_result) => {
                                if let Some(tx) = &result_tx {
                                    if let Err(e) = tx.send(scan_result) {
                                        error!("Failed to send scan result: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Scan error: {}", e);
                            }
                        }
                        
                        *state.write().unwrap() = ScannerState::Watching;
                    }
                }
            });
        });
    }

    async fn handle_file_added(
        config: &Arc<RwLock<AutoScannerConfig>>,
        file_cache: &Arc<FileCache>,
        path: PathBuf,
    ) -> Result<ScanResult> {
        info!("Handling file added: {:?}", path);
        
        if !Self::should_scan_file(&path, &config.read().unwrap()) {
            return Ok(ScanResult {
                tracks: Vec::new(),
                playlists: Vec::new(),
                deleted_files: Vec::new(),
            });
        }

        let config_guard = config.read().unwrap();
        let mut tracks = Self::scan_single_file(
            &path,
            &config_guard.thumbnail_dir,
            &config_guard.artist_splitter,
        ).await?;
        Self::filter_tracks_by_min_duration(&mut tracks, &config_guard.scan_min_duration);

        if let Ok(metadata) = std::fs::metadata(&path) {
            let file_meta = FileMetadata {
                path: path.clone(),
                size: metadata.len(),
                modified: metadata.modified().unwrap_or(UNIX_EPOCH),
            };
            file_cache.update_file(&path, file_meta);
        }

        Ok(ScanResult {
            tracks,
            playlists: Vec::new(),
            deleted_files: Vec::new(),
        })
    }

    async fn handle_file_modified(
        config: &Arc<RwLock<AutoScannerConfig>>,
        file_cache: &Arc<FileCache>,
        path: PathBuf,
    ) -> Result<ScanResult> {
        info!("Handling file modified: {:?}", path);
    
        // TODO:开发期：总是重新扫描，忽略缓存判断
        // 原早退逻辑（已注释，便于恢复）：基于 size + mtime 判断未变化则跳过
        // if let Ok(metadata) = std::fs::metadata(&path) {
        //     if let Some(cached) = file_cache.get_file(&path) {
        //         if cached.size == metadata.len() &&
        //            cached.modified == metadata.modified().unwrap_or(UNIX_EPOCH) {
        //             return Ok(ScanResult {
        //                 tracks: Vec::new(),
        //                 playlists: Vec::new(),
        //                 deleted_files: Vec::new(),
        //             });
        //         }
        //     }
        // }

        // 重新扫描文件
        Self::handle_file_added(config, file_cache, path).await
    }

    /// 处理文件删除事件
    async fn handle_file_deleted(
        file_cache: &Arc<FileCache>,
        path: PathBuf,
    ) -> Result<ScanResult> {
        info!("Handling file deleted: {:?}", path);
        
        file_cache.remove_file(&path);
        
        Ok(ScanResult {
            tracks: Vec::new(),
            playlists: Vec::new(),
            deleted_files: vec![path],
        })
    }

    async fn handle_full_scan(
        config: &Arc<RwLock<AutoScannerConfig>>,
        file_cache: &Arc<FileCache>,
    ) -> Result<ScanResult> {
        info!("Handling full scan");
        
        let config_guard = config.read().unwrap();
        let mut all_tracks = Vec::new();
        let all_playlists = Vec::new();
        let mut deleted_files = Vec::new();

        for scan_path in &config_guard.scan_paths {
            if !scan_path.exists() {
                continue;
            }

            let file_list = get_files_recursively(scan_path.clone())?;
            
            let current_files: HashSet<PathBuf> = file_list.file_list.iter().map(|(p, _)| p.clone()).collect();
            let cached_files: HashSet<PathBuf> = file_cache.get_all_files().into_iter().map(|f| f.path).collect();
            
            for cached_path in &cached_files {
                if cached_path.starts_with(scan_path) && !current_files.contains(cached_path) {
                    deleted_files.push(cached_path.clone());
                    file_cache.remove_file(cached_path);
                }
            }
            
            for (file_path, size) in file_list.file_list {
                if Self::should_scan_file(&file_path, &config_guard) {
                    let needs_scan = if let Some(cached) = file_cache.get_file(&file_path) {
                        if let Ok(metadata) = std::fs::metadata(&file_path) {
                            cached.size != size as u64 || 
                            cached.modified != metadata.modified().unwrap_or(UNIX_EPOCH)
                        } else {
                            true
                        }
                    } else {
                        true
                    };

                    if needs_scan {
                        match Self::scan_single_file(
                            &file_path,
                            &config_guard.thumbnail_dir,
                            &config_guard.artist_splitter,
                        ).await {
                            Ok(mut tracks) => {
                                Self::filter_tracks_by_min_duration(&mut tracks, &config_guard.scan_min_duration);
                                all_tracks.append(&mut tracks);
                                
                                if let Ok(metadata) = std::fs::metadata(&file_path) {
                                    let file_meta = FileMetadata {
                                        path: file_path.clone(),
                                        size: size as u64,
                                        modified: metadata.modified().unwrap_or(UNIX_EPOCH),
                                    };
                                    file_cache.update_file(&file_path, file_meta);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to scan file {:?}: {}", file_path, e);
                            }
                        }
                    }
                }
            }
            
            // TODO: 扫描播放列表文件
            // for playlist_path in file_list.playlist_list {
            //     // 处理播放列表
            // }
        }

        Ok(ScanResult {
            tracks: all_tracks,
            playlists: all_playlists,
            deleted_files,
        })
    }

    async fn handle_manual_scan(
        config: &Arc<RwLock<AutoScannerConfig>>,
        _file_cache: &Arc<FileCache>,
        paths: Vec<PathBuf>,
    ) -> Result<ScanResult> {
        info!("Handling manual scan for {} paths", paths.len());
        
        let config_guard = config.read().unwrap();
        let mut all_tracks = Vec::new();
        
        for path in paths {
            if path.is_file() && Self::should_scan_file(&path, &config_guard) {
                match Self::scan_single_file(
                    &path,
                    &config_guard.thumbnail_dir,
                    &config_guard.artist_splitter,
                ).await {
                    Ok(mut tracks) => {
                        Self::filter_tracks_by_min_duration(&mut tracks, &config_guard.scan_min_duration);
                        all_tracks.append(&mut tracks);
                    }
                    Err(e) => {
                        warn!("Failed to scan file {:?}: {}", path, e);
                    }
                }
            } else if path.is_dir() {
                let file_list = get_files_recursively(path)?;
                for (file_path, _) in file_list.file_list {
                    if Self::should_scan_file(&file_path, &config_guard) {
                        match Self::scan_single_file(
                            &file_path,
                            &config_guard.thumbnail_dir,
                            &config_guard.artist_splitter,
                        ).await {
                            Ok(mut tracks) => {
                                Self::filter_tracks_by_min_duration(&mut tracks, &config_guard.scan_min_duration);
                                all_tracks.append(&mut tracks);
                            }
                            Err(e) => {
                                warn!("Failed to scan file {:?}: {}", file_path, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(ScanResult {
            tracks: all_tracks,
            playlists: Vec::new(),
            deleted_files: Vec::new(),
        })
    }

    async fn scan_single_file(
        path: &Path,
        thumbnail_dir: &Path,
        artist_splitter: &str,
    ) -> Result<Vec<MediaContent>> {
        let size = std::fs::metadata(path)
            .map(|m| m.len() as f64)
            .unwrap_or(0.0);
        
        let track = scan_file(&path.to_path_buf(), thumbnail_dir, size, false, artist_splitter)?;
        Ok(vec![track])
    }

    fn should_scan_file(path: &Path, config: &AutoScannerConfig) -> bool {
        for exclude_path in &config.exclude_paths {
            if path.starts_with(exclude_path) {
                return false;
            }
        }

        Self::is_supported_music_file(path, &config.scan_formats)
    }

    fn is_music_file(path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext = ext_str.to_lowercase();
                matches!(ext.as_str(), "flac" | "mp3" | "ogg" | "m4a" | "webm" | "wav" | "wv" | "aac" | "opus")
            } else {
                false
            }
        } else {
            false
        }
    }

    fn is_supported_music_file(path: &Path, scan_formats: &str) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext = ext_str.to_lowercase();
                match scan_formats {
                    "common" => {
                        matches!(ext.as_str(), "mp3" | "flac" | "m4a" | "ogg")
                    }
                    "all" => {
                        matches!(ext.as_str(), "flac" | "mp3" | "ogg" | "m4a" | "webm" | "wav" | "wv" | "aac" | "opus")
                    }
                    _ => {
                        matches!(ext.as_str(), "mp3" | "flac" | "m4a" | "ogg")
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn filter_tracks_by_min_duration(tracks: &mut Vec<MediaContent>, scan_min_duration: &str) {
        let threshold_secs = Self::parse_min_duration_threshold(scan_min_duration);
        if threshold_secs <= 0.0 {
            return;
        }
        tracks.retain(|s| {
            let dur = s.track.duration.unwrap_or(0.0);
            dur >= threshold_secs
        });
    }

    fn parse_min_duration_threshold(scan_min_duration: &str) -> f64 {
        match scan_min_duration {
            "sec30" => 30.0,
            "min2" => 120.0,
            "all" => 0.0,
            other => {
                warn!("Unknown scan_min_duration '{}', defaulting to 30s", other);
                30.0
            }
        }
    }
}

impl Drop for AutoScanner {
    fn drop(&mut self) {
        if self.is_running.load(Ordering::Acquire) {
            self.is_running.store(false, Ordering::Release);
        }
    }
}