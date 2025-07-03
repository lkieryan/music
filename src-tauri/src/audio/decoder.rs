use std::path::Path;
use anyhow::Result;

/// 音频格式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    MP3,
    FLAC,
    OGG,
    M4A,
    WAV,
    APE,
    WMA,
}

impl AudioFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "mp3" => Some(AudioFormat::MP3),
            "flac" => Some(AudioFormat::FLAC),
            "ogg" => Some(AudioFormat::OGG),
            "m4a" | "aac" => Some(AudioFormat::M4A),
            "wav" => Some(AudioFormat::WAV),
            "ape" => Some(AudioFormat::APE),
            "wma" => Some(AudioFormat::WMA),
            _ => None,
        }
    }
}

/// 音频解码器
pub struct AudioDecoder;

impl AudioDecoder {
    pub fn detect_format(file_path: &Path) -> Option<AudioFormat> {
        if let Some(ext) = file_path.extension() {
            if let Some(ext_str) = ext.to_str() {
                return AudioFormat::from_extension(ext_str);
            }
        }
        None
    }

    pub fn decode_file(file_path: &Path) -> Result<()> {
        // TODO: 使用 symphonia 解码音频文件
        Ok(())
    }
}