use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::error::AppResult;
use crate::settings::{data_dir, read_json, write_json};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Session {
    pub tabs: Vec<TabState>,
    pub active: i64,
    pub closed: Vec<TabState>,
    pub recent_files: Vec<RecentFile>,
    pub recent_folders: Vec<String>,
    pub roots: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct TabState {
    pub path: String,
    pub pinned: bool,
    pub scroll_line: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RecentFile {
    pub path: String,
    /// RFC 3339 time stamp written by the front end.
    pub opened_at: String,
}

#[tauri::command]
pub fn load_session(app: AppHandle) -> Session {
    data_dir(&app)
        .map(|d| read_json(&d.join("session.json")))
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_session(app: AppHandle, session: Session) -> AppResult<()> {
    write_json(&data_dir(&app)?.join("session.json"), &session)
}
