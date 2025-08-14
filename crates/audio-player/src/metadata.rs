use crate::{AudioError, AudioQuality, AudioResult};
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tracing::debug;

/// 音频元数据信息
#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub duration: Option<Duration>,
    pub quality: AudioQuality,
    pub format: String,
    pub codec: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

/// 音频元数据提取器
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// 从文件路径提取音频元数据
    pub fn extract_from_file(file_path: &str) -> AudioResult<AudioMetadata> {
        let path = Path::new(file_path);
        
        // 检查文件是否存在
        if !path.exists() {
            return Err(AudioError::IoError(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Audio file not found")
            ));
        }

        // 打开文件
        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        // 创建格式提示
        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                hint.with_extension(ext_str);
            }
        }

        // 探测格式
        let format_opts = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| AudioError::UnsupportedFormat(format!("Failed to probe audio format: {}", e)))?;

        let mut format = probed.format;
        let track = format.tracks()
            .iter()
            .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
            .ok_or_else(|| AudioError::UnsupportedFormat("No valid audio track found".to_string()))?;

        let codec_params = &track.codec_params;

        // 提取基本信息
        let sample_rate = codec_params.sample_rate.unwrap_or(44100);
        let channels = codec_params.channels.map(|ch| ch.count()).unwrap_or(2) as u16;
        let bits_per_sample = codec_params.bits_per_sample.unwrap_or(16) as u16;
        
        let codec_name = if let Some(codec) = symphonia::default::get_codecs().get_codec(codec_params.codec) {
            codec.short_name.to_string()
        } else {
            "unknown".to_string()
        };

        // 计算比特率
        let bitrate = if let (Some(rate), Some(bps)) = (codec_params.sample_rate, codec_params.bits_per_sample) {
            Some((rate * channels as u32 * bps as u32) / 1000) // kbps
        } else {
            None
        };

        // 计算时长
        let duration = if let Some(n_frames) = codec_params.n_frames {
            Some(Duration::from_secs_f64(n_frames as f64 / sample_rate as f64))
        } else {
            // 尝试从容器格式获取时长
            Self::estimate_duration_from_container(&mut format)
        };

        // 提取元数据
        let (title, artist, album) = Self::extract_metadata_tags(&mut format);

        let quality = AudioQuality {
            sample_rate,
            channels,
            bits_per_sample,
            codec: codec_name.clone(),
            bitrate,
        };

        // Best-effort format name: use file extension if available; Symphonia API doesn't expose a portable format name here
        let format_name = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());

        debug!("Extracted metadata for {}: duration={:?}, codec={}, quality={:?}", 
               file_path, duration, codec_name, quality);

        Ok(AudioMetadata {
            duration,
            quality,
            format: format_name,
            codec: codec_name,
            title,
            artist,
            album,
        })
    }

    /// 从容器格式估算时长
    fn estimate_duration_from_container(format: &mut Box<dyn symphonia::core::formats::FormatReader>) -> Option<Duration> {
        // 尝试读取一些数据包来估算时长
        let mut packet_count = 0;
        let mut total_ts = 0u64;
        let max_packets = 100; // 限制读取的数据包数量

        while packet_count < max_packets {
            match format.next_packet() {
                Ok(packet) => {
                    if packet.ts > total_ts {
                        total_ts = packet.ts;
                    }
                    packet_count += 1;
                }
                Err(_) => break,
            }
        }

        if total_ts > 0 && packet_count > 0 {
            // 这是一个粗略的估算
            let estimated_total_ts = (total_ts * max_packets) / packet_count as u64;
            Some(Duration::from_millis(estimated_total_ts))
        } else {
            None
        }
    }

    /// 提取元数据标签
    fn extract_metadata_tags(format: &mut Box<dyn symphonia::core::formats::FormatReader>) -> (Option<String>, Option<String>, Option<String>) {
        let mut title = None;
        let mut artist = None;
        let mut album = None;

        if let Some(metadata_rev) = format.metadata().current() {
            for tag in metadata_rev.tags() {
                match tag.key.as_str() {
                    "TITLE" | "TIT2" => title = Some(tag.value.to_string()),
                    "ARTIST" | "TPE1" => artist = Some(tag.value.to_string()),
                    "ALBUM" | "TALB" => album = Some(tag.value.to_string()),
                    _ => {}
                }
            }
        }

        (title, artist, album)
    }

    /// 验证音频文件格式
    pub fn validate_audio_file(file_path: &str) -> AudioResult<bool> {
        match Self::extract_from_file(file_path) {
            Ok(_) => Ok(true),
            Err(AudioError::UnsupportedFormat(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// 获取支持的音频格式列表
    pub fn supported_formats() -> Vec<&'static str> {
        vec![
            "mp3", "wav", "flac", "ogg", "m4a", "aac", 
            "wma", "ape", "mpc", "tak", "tta", "wv"
        ]
    }

    /// 检查文件扩展名是否受支持
    pub fn is_supported_extension(file_path: &str) -> bool {
        if let Some(extension) = Path::new(file_path).extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return Self::supported_formats().contains(&ext_lower.as_str());
            }
        }
        false
    }
}