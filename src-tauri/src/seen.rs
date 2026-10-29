use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::AppResult;
use crate::render::LineRange;
use crate::settings::{data_dir, read_json, write_json};
use crate::workspace::{FileStatus, path_key};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SeenEntry {
    pub mtime_ms: u64,
    pub len: u64,
    pub viewed_at_ms: u64,
}

pub struct SeenState(pub Mutex<Option<HashMap<String, SeenEntry>>>);

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn mtime_ms(meta: &std::fs::Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// 64-bit FNV-1a, written out so snapshot names stay stable across Rust releases.
fn fnv1a(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn snapshot_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let d = data_dir(app)?.join("snapshots");
    std::fs::create_dir_all(&d)?;
    Ok(d)
}

fn snapshot_path(app: &AppHandle, path: &Path) -> AppResult<PathBuf> {
    Ok(snapshot_dir(app)?.join(format!("{:016x}.md", fnv1a(&path_key(path)))))
}

fn with_entries<T>(app: &AppHandle, f: impl FnOnce(&mut HashMap<String, SeenEntry>) -> T) -> T {
    let state = app.state::<SeenState>();
    let mut guard = state.0.lock().unwrap();
    if guard.is_none() {
        let loaded = data_dir(app)
            .map(|d| read_json(&d.join("seen.json")))
            .unwrap_or_default();
        *guard = Some(loaded);
    }
    f(guard.as_mut().unwrap())
}

pub fn status(app: &AppHandle, path: &Path) -> FileStatus {
    let Ok(meta) = std::fs::metadata(path) else {
        return FileStatus::Seen;
    };
    with_entries(app, |m| match m.get(&path_key(path)) {
        None => FileStatus::New,
        Some(e) if e.mtime_ms != mtime_ms(&meta) || e.len != meta.len() => FileStatus::Changed,
        Some(_) => FileStatus::Seen,
    })
}

/// 1-based line ranges of `text` that differ from the snapshot taken at the last mark_seen.
pub fn changed_ranges(app: &AppHandle, path: &Path, text: &str) -> Vec<LineRange> {
    let Ok(snap) = snapshot_path(app, path) else {
        return Vec::new();
    };
    let Ok(old) = std::fs::read_to_string(snap) else {
        return Vec::new();
    };
    if old == text {
        return Vec::new();
    }
    let diff = similar::TextDiff::from_lines(&old, text);
    diff.ops()
        .iter()
        .filter_map(|op| {
            let (tag, _old, new) = op.as_tag_tuple();
            match tag {
                similar::DiffTag::Insert | similar::DiffTag::Replace if !new.is_empty() => {
                    Some(LineRange {
                        start: new.start as u32 + 1,
                        end: new.end as u32,
                    })
                }
                _ => None,
            }
        })
        .collect()
}

#[tauri::command]
pub fn mark_seen(app: AppHandle, path: String) -> AppResult<()> {
    let p = PathBuf::from(&path);
    let meta = std::fs::metadata(&p)?;
    let text = std::fs::read(&p)?;
    std::fs::write(snapshot_path(&app, &p)?, &text)?;
    let entry = SeenEntry {
        mtime_ms: mtime_ms(&meta),
        len: meta.len(),
        viewed_at_ms: now_ms(),
    };
    let all = with_entries(&app, |m| {
        m.insert(path_key(&p), entry);
        m.clone()
    });
    write_json(&data_dir(&app)?.join("seen.json"), &all)
}

/// Drops snapshots and entries not viewed for 30 days.
pub fn cleanup(app: &AppHandle) {
    let cutoff = now_ms().saturating_sub(Duration::from_secs(30 * 24 * 3600).as_millis() as u64);
    let all = with_entries(app, |m| {
        m.retain(|_, e| e.viewed_at_ms >= cutoff);
        m.clone()
    });
    if let Ok(dir) = snapshot_dir(app) {
        let keep: std::collections::HashSet<String> = all
            .keys()
            .map(|k| format!("{:016x}.md", fnv1a(k)))
            .collect();
        for f in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            if !keep.contains(&f.file_name().to_string_lossy().to_string()) {
                let _ = std::fs::remove_file(f.path());
            }
        }
    }
    if let Ok(d) = data_dir(app) {
        let _ = write_json(&d.join("seen.json"), &all);
    }
}
