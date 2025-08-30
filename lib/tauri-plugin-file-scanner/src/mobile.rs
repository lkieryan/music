
use std::{collections::HashMap, sync::mpsc::channel as mpsc_channel};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tauri::{
    async_runtime::channel,
    ipc::Channel,
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use types::{
    errors::{MusicError, Result},
    songs::Song,
};
use types::errors::error_helpers;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_file_scanner);

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.kieran.filescanner";

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<FileScanner<R>> {
    #[cfg(target_os = "android")]
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "FileScannerPlugin")
        .map_err(|e| MusicError::String(e.to_string()))?;
    #[cfg(target_os = "ios")]
    let handle = api
        .register_ios_plugin(init_plugin_file_scanner)
        .map_err(|e| MusicError::String(e.to_string()))?;
    Ok(FileScanner(handle))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanArgs {
    pub channel: Channel,
}

/// Access to the file-scanner APIs.
pub struct FileScanner<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> FileScanner<R> {
    pub fn scan_music(&self) -> Result<Vec<Song>> {
        let (tx, rx) = mpsc_channel();
        let ret: serde_json::Value = self
            .0
            .run_mobile_plugin(
                "android_scan_music",
                ScanArgs {
                    channel: Channel::new(move |event| match event {
                        tauri::ipc::InvokeResponseBody::Json(payload) => {
                            let songs: Value = serde_json::from_str(&payload).unwrap();
                            let songs = songs.get("songs");
                            if let Some(songs) = songs {
                                let songs: Vec<Song> =
                                    serde_json::from_str(songs.as_str().unwrap())?;
                                tx.send(songs).unwrap();
                            }
                            Ok(())
                        }
                        _ => Ok(()),
                    }),
                },
            )
            .map_err(error_helpers::to_plugin_error)?;

        let resp = rx.recv().unwrap();
        Ok(resp)
    }
}
