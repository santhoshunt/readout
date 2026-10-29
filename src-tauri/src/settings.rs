use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// "dark", "light" or "system".
    pub theme: String,
    /// Per-theme element color overrides, keyed by element key from elements.ts.
    pub colors: BTreeMap<String, BTreeMap<String, String>>,
    pub editor: EditorSettings,
    pub watch: WatchSettings,
    pub reading: ReadingSettings,
    pub explorer: ExplorerSettings,
    pub layout: LayoutSettings,
    pub updates: UpdateSettings,
    pub editing: EditingSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct LayoutSettings {
    pub explorer_width: u32,
    pub outline_width: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct UpdateSettings {
    /// Off until the user opts in; nothing contacts the network before that.
    pub auto_check: bool,
    pub last_check_ms: u64,
    pub skipped_version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct EditingSettings {
    pub allow_edits: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        LayoutSettings {
            explorer_width: 250,
            outline_width: 220,
        }
    }
}

impl Default for EditingSettings {
    fn default() -> Self {
        EditingSettings { allow_edits: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct EditorSettings {
    /// "cursor", "code" or "custom".
    pub preset: String,
    /// Command template with {file}, {line} and {folder} placeholders.
    pub command: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct WatchSettings {
    pub folders: Vec<String>,
    /// File extensions the watcher reports, without the dot.
    pub extensions: Vec<String>,
    /// Folder names the watcher and the file index skip.
    pub exclude_folders: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct ReadingSettings {
    pub font_family: String,
    pub font_size: u32,
    pub max_width: u32,
    pub follow_mode: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct ExplorerSettings {
    pub show_explorer: bool,
    pub show_outline: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "dark".into(),
            colors: BTreeMap::new(),
            editor: EditorSettings::default(),
            watch: WatchSettings::default(),
            reading: ReadingSettings::default(),
            explorer: ExplorerSettings::default(),
            layout: LayoutSettings::default(),
            updates: UpdateSettings::default(),
            editing: EditingSettings::default(),
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        crate::editor::default_editor()
    }
}

impl Default for WatchSettings {
    fn default() -> Self {
        WatchSettings {
            folders: Vec::new(),
            extensions: vec!["md".into(), "markdown".into()],
            exclude_folders: vec!["node_modules".into(), ".git".into(), "target".into()],
        }
    }
}

impl Default for ReadingSettings {
    fn default() -> Self {
        ReadingSettings {
            font_family: "Segoe UI".into(),
            font_size: 15,
            max_width: 860,
            follow_mode: false,
        }
    }
}

impl Default for ExplorerSettings {
    fn default() -> Self {
        ExplorerSettings {
            show_explorer: true,
            show_outline: true,
        }
    }
}

pub struct SettingsState(pub Mutex<Option<Settings>>);

pub fn data_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Settings(e.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Reads a JSON file. A missing file gives the default; a corrupt one is renamed to *.bad first.
pub fn read_json<T: DeserializeOwned + Default>(path: &Path) -> T {
    let Ok(text) = std::fs::read_to_string(path) else {
        return T::default();
    };
    match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => {
            let _ = std::fs::rename(path, path.with_extension("json.bad"));
            T::default()
        }
    }
}

/// Writes to a temp file and renames it, so a crash mid-write never leaves a half file.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let text =
        serde_json::to_string_pretty(value).map_err(|e| AppError::Settings(e.to_string()))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub fn current(app: &AppHandle) -> Settings {
    let state = app.state::<SettingsState>();
    let mut guard = state.0.lock().unwrap();
    if guard.is_none() {
        let mut loaded: Settings = data_dir(app)
            .map(|d| read_json(&d.join("settings.json")))
            .unwrap_or_default();
        // A hand-edited or very old file must not leave a panel too narrow to grab or wider than the window.
        loaded.layout.explorer_width = loaded.layout.explorer_width.clamp(160, 480);
        loaded.layout.outline_width = loaded.layout.outline_width.clamp(160, 480);
        *guard = Some(loaded);
    }
    guard.clone().unwrap()
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    current(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> AppResult<()> {
    write_json(&data_dir(&app)?.join("settings.json"), &settings)?;
    *app.state::<SettingsState>().0.lock().unwrap() = Some(settings);
    Ok(())
}
