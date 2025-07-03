use std::path::PathBuf;
use std::collections::HashSet;
use std::time::SystemTime;
use anyhow::Result;
use walkdir::WalkDir;
use super::{UnifiedSong, MusicSourceType, SongMetadata};
use crate::audio::AudioFormat;

/// 本地音乐源
pub struct LocalMusicSource {
    library_paths: Vec<PathBuf>,
    supported_formats: HashSet<String>,
}

impl LocalMusicSource {
    pub fn new() -> Self {
        let mut supported_formats = HashSet::new();
        supported_formats.insert("mp3".to_string());
        supported_formats.insert("flac".to_string());
        supported_formats.insert("ogg".to_string());
        supported_formats.insert("m4a".to_string());
        supported_formats.insert("aac".to_string());
        supported_formats.insert("wav".to_string());
        supported_formats.insert("ape".to_string());
        supported_formats.insert("wma".to_string());

        Self {
            library_paths: vec![],
            supported_formats,
        }
    }

    pub fn add_library_path(&mut self, path: PathBuf) {
        if !self.library_paths.contains(&path) {
            self.library_paths.push(path);
        }
    }

    pub fn remove_library_path(&mut self, path: &PathBuf) {
        self.library_paths.retain(|p| p != path);
    }

    pub async fn scan_library(&self) -> Result<Vec<LocalSong>> {
        let mut songs = Vec::new();

        for library_path in &self.library_paths {
            if !library_path.exists() {
                log::warn!("Library path does not exist: {:?}", library_path);
                continue;
            }

            log::info!("Scanning library path: {:?}", library_path);
            
            for entry in WalkDir::new(library_path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(extension) = path.extension() {
                        if let Some(ext_str) = extension.to_str() {
                            if self.supported_formats.contains(&ext_str.to_lowercase()) {
                                match self.process_audio_file(path).await {
                                    Ok(song) => songs.push(song),
                                    Err(e) => {
                                        log::warn!("Failed to process file {:?}: {}", path, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        log::info!("Found {} songs in local library", songs.len());
        Ok(songs)
    }

    async fn process_audio_file(&self, file_path: &std::path::Path) -> Result<LocalSong> {
        let metadata = self.extract_metadata(file_path).await?;
        let file_metadata = std::fs::metadata(file_path)?;
        
        let id = uuid::Uuid::new_v4().to_string();
        
        Ok(LocalSong {
            id,
            file_path: file_path.to_path_buf(),
            title: metadata.title.unwrap_or_else(|| {
                file_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            }),
            artist: metadata.artist.unwrap_or_else(|| "Unknown Artist".to_string()),
            album: metadata.album.unwrap_or_else(|| "Unknown Album".to_string()),
            duration: metadata.duration.unwrap_or_default(),
            file_size: file_metadata.len(),
            format: AudioFormat::from_extension(
                file_path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            ).unwrap_or(AudioFormat::MP3),
            bitrate: metadata.bitrate,
            sample_rate: metadata.sample_rate,
            last_modified: file_metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        })
    }

    async fn extract_metadata(&self, file_path: &std::path::Path) -> Result<ExtractedMetadata> {
        // TODO: 使用 lofty 提取音频元数据
        Ok(ExtractedMetadata {
            title: None,
            artist: None,
            album: None,
            duration: None,
            bitrate: None,
            sample_rate: None,
        })
    }

    pub async fn search_local(&self, query: &str) -> Result<Vec<LocalSong>> {
        let all_songs = self.scan_library().await?;
        let query_lower = query.to_lowercase();
        
        let filtered_songs: Vec<LocalSong> = all_songs
            .into_iter()
            .filter(|song| {
                song.title.to_lowercase().contains(&query_lower) ||
                song.artist.to_lowercase().contains(&query_lower) ||
                song.album.to_lowercase().contains(&query_lower)
            })
            .collect();
        
        Ok(filtered_songs)
    }
}

/// 本地歌曲数据结构
#[derive(Debug, Clone)]
pub struct LocalSong {
    pub id: String,
    pub file_path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: std::time::Duration,
    pub file_size: u64,
    pub format: AudioFormat,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub last_modified: SystemTime,
}

impl From<LocalSong> for UnifiedSong {
    fn from(local_song: LocalSong) -> Self {
        UnifiedSong {
            id: local_song.id,
            title: local_song.title,
            artist: local_song.artist,
            album: local_song.album,
            duration: local_song.duration,
            source: MusicSourceType::Local,
            source_id: local_song.file_path.to_string_lossy().to_string(),
            cover_url: None,
            play_url: Some(local_song.file_path.to_string_lossy().to_string()),
            local_path: Some(local_song.file_path),
            metadata: SongMetadata {
                bitrate: local_song.bitrate,
                sample_rate: local_song.sample_rate,
                channels: None,
                codec: Some(format!("{:?}", local_song.format)),
                file_size: Some(local_song.file_size),
                lyrics: None,
            },
        }
    }
}

/// 提取的元数据
#[derive(Debug)]
struct ExtractedMetadata {
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration: Option<std::time::Duration>,
    bitrate: Option<u32>,
    sample_rate: Option<u32>,
}