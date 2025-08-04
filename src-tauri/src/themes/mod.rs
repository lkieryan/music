use std::{collections::HashMap, fs, path::PathBuf, sync::Mutex};

use tauri::{App, AppHandle, Emitter, State, Manager};
use types::errors::{error_helpers, Result};
use types::themes::ThemeDetails;

use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher, Config, Event};
use regex::Regex;

#[derive(Debug)]
pub struct ThemeHolder {
    root: PathBuf,
    app: AppHandle,
    watchers: Mutex<HashMap<String, RecommendedWatcher>>, // keep watchers alive per theme id
}

impl ThemeHolder {
    pub fn new(root: PathBuf, app: AppHandle) -> Self { Self { root, app, watchers: Mutex::new(HashMap::new()) } }

    fn theme_dir(&self, id: &str) -> PathBuf { self.root.join(id) }

    pub fn save_theme(&self, theme: ThemeDetails) -> Result<()> {
        let id = theme.meta.id.clone();
        let dir = self.theme_dir(&id);
        if !dir.exists() { fs::create_dir_all(&dir).map_err(error_helpers::to_file_system_error)?; }
        let cfg_path = dir.join("config.json");
        fs::write(cfg_path, serde_json::to_string_pretty(&theme)?).map_err(error_helpers::to_file_system_error)?;
        Ok(())
    }

    pub fn load_theme(&self, id: String) -> Result<ThemeDetails> {
        let cfg = self.theme_dir(&id).join("config.json");
        if cfg.exists() {
            let data = fs::read_to_string(cfg).map_err(error_helpers::to_file_system_error)?;
            let theme: ThemeDetails = serde_json::from_str(&data)?;
            return Ok(theme);
        }
        Ok(ThemeDetails::default())
    }

    pub fn load_all_themes(&self) -> Result<HashMap<String, ThemeDetails>> {
        let mut ret = HashMap::new();
        if self.root.exists() {
            for entry in fs::read_dir(&self.root).map_err(error_helpers::to_file_system_error)? {
                if let Ok(ent) = entry {
                    let path = ent.path();
                    if path.is_dir() {
                        if let Some(id) = path.file_name().and_then(|s| s.to_str()).map(|s| s.to_string()) {
                            if let Ok(theme) = self.load_theme(id.clone()) {
                                ret.insert(id, theme);
                            }
                        }
                    }
                }
            }
        }
        Ok(ret)
    }

    pub fn remove_theme(&self, id: String) -> Result<()> {
        let dir = self.theme_dir(&id);
        if dir.exists() { fs::remove_dir_all(dir).map_err(error_helpers::to_file_system_error)?; }
        Ok(())
    }

    pub fn get_css(&self, id: String) -> Result<String> {
        // Read custom_css and expand @import lines (relative to theme dir), replace %themeDir%
        let theme = self.load_theme(id.clone())?;
        if let Some(custom_css) = theme.custom_css {
            let path = self.theme_dir(&id).join(&custom_css);
            if path.exists() {
                let (css, imports) = self.transform_css(path.clone(), Some(self.theme_dir(&id)))?;
                let _ = self.watch_theme(&id, imports);
                return Ok(css);
            }
        }
        Ok(String::new())
    }

    fn transform_css(&self, entry: PathBuf, root: Option<PathBuf>) -> Result<(String, Vec<PathBuf>)> {
        let mut imports = Vec::new();
        let mut css = fs::read_to_string(&entry).map_err(error_helpers::to_file_system_error)?;
        // Replace %themeDir%
        if let Some(parent) = entry.parent() {
            let re = Regex::new(r"%themeDir%").unwrap();
            css = re.replace_all(&css, parent.to_string_lossy().as_ref()).to_string();
        }
        // Expand @import "..."; lines
        // Use a raw string with hash delimiter to avoid escaping inner quotes
        let import_re = Regex::new(r#"@import\s+\"([^\"]+)\";\s*"#).unwrap();
        let mut out = String::new();
        let mut last = 0;
        for cap in import_re.captures_iter(&css) {
            if let Some(m) = cap.get(0) {
                out.push_str(&css[last..m.start()]);
                let rel = cap.get(1).unwrap().as_str();
                // Resolve base directory for relative imports
                let base: &std::path::Path = root
                    .as_deref()
                    .unwrap_or_else(|| entry.parent().unwrap_or_else(|| std::path::Path::new(".")));
                let imp_path = base.join(rel);
                if imp_path.exists() { imports.push(imp_path.clone()); }
                let (sub, _subimps) = self.transform_css(imp_path, root.clone())?;
                out.push_str(&sub);
                last = m.end();
            }
        }
        out.push_str(&css[last..]);
        Ok((out, imports))
    }
}

impl ThemeHolder {
    fn watch_theme(&self, id: &str, files: Vec<PathBuf>) -> Result<()> {
        let mut guard = self.watchers.lock().unwrap();
        // Drop existing watcher for this id to replace with a new one
        if let Some(mut old) = guard.remove(id) {
            let _ = old.unwatch(&self.root); // best-effort
        }

        let app = self.app.clone();
        let theme_id = id.to_string();
        let mut watcher: RecommendedWatcher = recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(_event) = res {
                // Emit theme-updated event with theme id
                let _ = app.emit("theme-updated", theme_id.clone());
            }
        }).map_err(error_helpers::to_file_system_error)?;
        watcher.configure(Config::default())
            .map_err(error_helpers::to_file_system_error)?;
        for f in files.iter() {
            if f.exists() { watcher.watch(f, RecursiveMode::NonRecursive).map_err(error_helpers::to_file_system_error)?; }
        }
        guard.insert(id.to_string(), watcher);
        Ok(())
    }
}

pub fn get_theme_handler_state(app: &mut App) -> ThemeHolder {
    let root = app.path().app_local_data_dir().unwrap().join("themes");
    if !root.exists() { fs::create_dir_all(&root).unwrap(); }
    ThemeHolder::new(root, app.app_handle().clone())
}

#[tauri::command(async)]
pub fn save_theme(theme_holder: State<ThemeHolder>, theme: ThemeDetails) -> Result<()> {
    theme_holder.save_theme(theme)
}

#[tauri::command(async)]
pub fn remove_theme(theme_holder: State<ThemeHolder>, id: String) -> Result<()> {
    theme_holder.remove_theme(id)
}

#[tauri::command(async)]
pub fn load_theme(theme_holder: State<ThemeHolder>, id: String) -> Result<ThemeDetails> {
    theme_holder.load_theme(id)
}

#[tauri::command(async)]
pub fn load_all_themes(theme_holder: State<ThemeHolder>) -> Result<HashMap<String, ThemeDetails>> {
    theme_holder.load_all_themes()
}

#[tauri::command(async)]
pub fn get_css(theme_holder: State<ThemeHolder>, id: String) -> Result<String> {
    theme_holder.get_css(id)
}

#[tauri::command(async)]
pub fn export_theme(theme_holder: State<ThemeHolder>, id: String, dest_path: String) -> Result<()> {
    use std::io::{Write};
    use zip::write::FileOptions;
    let theme = theme_holder.load_theme(id.clone())?;
    let dir = theme_holder.theme_dir(&id);
    let file = std::fs::File::create(&dest_path).map_err(error_helpers::to_file_system_error)?;
    let mut zip = zip::ZipWriter::new(file);

    // add config.json
    zip.start_file("config.json", FileOptions::default()).map_err(error_helpers::to_file_system_error)?;
    let json = serde_json::to_vec_pretty(&theme)?;
    zip.write_all(&json).map_err(error_helpers::to_file_system_error)?;

    // include custom.css if referenced
    if let Some(css_rel) = theme.custom_css.clone() {
        let css_path = dir.join(&css_rel);
        if css_path.exists() {
            zip.start_file(css_rel.replace('\\', "/"), FileOptions::default()).map_err(error_helpers::to_file_system_error)?;
            let data = std::fs::read(css_path).map_err(error_helpers::to_file_system_error)?;
            zip.write_all(&data).map_err(error_helpers::to_file_system_error)?;
        }
    }

    zip.finish().map_err(error_helpers::to_file_system_error)?;
    Ok(())
}

#[tauri::command(async)]
pub fn import_theme(theme_holder: State<ThemeHolder>, src_path: String) -> Result<()> {
    use std::io::Read;
    let file = std::fs::File::open(&src_path).map_err(error_helpers::to_file_system_error)?;
    let mut archive = zip::ZipArchive::new(file).map_err(error_helpers::to_file_system_error)?;

    // read config.json first
    let mut cfg_file = archive.by_name("config.json").map_err(error_helpers::to_file_system_error)?;
    let mut buf = Vec::new();
    cfg_file.read_to_end(&mut buf).map_err(error_helpers::to_file_system_error)?;
    let theme: ThemeDetails = serde_json::from_slice(&buf)?;
    let id = theme.meta.id.clone();
    let dst = theme_holder.theme_dir(&id);
    if !dst.exists() { fs::create_dir_all(&dst).map_err(error_helpers::to_file_system_error)?; }

    // reset archive to extract files
    let file = std::fs::File::open(&src_path).map_err(error_helpers::to_file_system_error)?;
    let mut archive = zip::ZipArchive::new(file).map_err(error_helpers::to_file_system_error)?;
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).map_err(error_helpers::to_file_system_error)?;
        let outpath = dst.join(f.mangled_name());
        if f.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(error_helpers::to_file_system_error)?;
        } else {
            if let Some(p) = outpath.parent() { fs::create_dir_all(p).map_err(error_helpers::to_file_system_error)?; }
            let mut outfile = std::fs::File::create(&outpath).map_err(error_helpers::to_file_system_error)?;
            std::io::copy(&mut f, &mut outfile).map_err(error_helpers::to_file_system_error)?;
        }
    }

    // save config.json to ensure consistency
    theme_holder.save_theme(theme)?;
    Ok(())
}
