pub mod auto_scanner;
pub mod file_cache;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod playlist_scanner;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod scanner;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use scanner::{ScanState, ScannerHolder};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod track_scanner;

mod types;
mod utils;

#[cfg(test)]
mod tests;

#[cfg(target_os = "android")]
mod scanner_android;
#[cfg(target_os = "android")]
pub use scanner_android::{ScanState, ScannerHolder};

pub use auto_scanner::{AutoScanner, AutoScannerConfig, ScanEvent, ScanResult, ScannerState as AutoScannerState};
pub use file_cache::{FileCache, FileMetadata, CacheStats};
pub use utils::{get_files_recursively, scan_file};
pub use types::FileList;
