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

pub(crate) fn mtime_ms(meta: &std::fs::Metadata) -> u64 {
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

/// The snapshot before the current one, for the "what changed" view.
fn previous_path(app: &AppHandle, path: &Path) -> AppResult<PathBuf> {
    Ok(snapshot_dir(app)?.join(format!("{:016x}.prev.md", fnv1a(&path_key(path)))))
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
    // Compare without line endings: the split editor works in LF while the file may be CRLF, and
    // that alone must not mark every line as changed. Line numbers are the same either way.
    let (old, text) = (old.replace("\r\n", "\n"), text.replace("\r\n", "\n"));
    if old == text {
        return Vec::new();
    }
    let diff = similar::TextDiff::from_lines(&old, &text);
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

/// One row of the side-by-side "what changed" view.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiffRow {
    /// "same", "added", "removed" or "changed".
    pub kind: &'static str,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub old_text: String,
    pub new_text: String,
}

fn row(kind: &'static str, old: Option<(usize, &str)>, new: Option<(usize, &str)>) -> DiffRow {
    let clean = |s: &str| s.trim_end_matches(['\r', '\n']).to_string();
    DiffRow {
        kind,
        old_line: old.map(|(i, _)| i as u32 + 1),
        new_line: new.map(|(i, _)| i as u32 + 1),
        old_text: old.map(|(_, s)| clean(s)).unwrap_or_default(),
        new_text: new.map(|(_, s)| clean(s)).unwrap_or_default(),
    }
}

/// The file as it is now against the snapshot from the last time it was read. With no snapshot,
/// every line counts as added.
#[tauri::command]
pub fn diff_file(app: AppHandle, path: String) -> AppResult<Vec<DiffRow>> {
    let p = crate::workspace::normalize(Path::new(&path));
    let (new, _) = crate::render::read_text(&p)?;
    let read = |s: AppResult<PathBuf>| s.ok().and_then(|s| std::fs::read_to_string(s).ok());
    // A file you are looking at was already marked read, so its snapshot equals the file. The
    // changes you were shown are then against the version before that, kept as the previous snapshot.
    let old = match read(snapshot_path(&app, &p)) {
        Some(snap) if snap != new => snap,
        _ => read(previous_path(&app, &p)).unwrap_or_default(),
    };
    let old_lines: Vec<&str> = old.split_inclusive('\n').collect();
    let new_lines: Vec<&str> = new.split_inclusive('\n').collect();
    let diff = similar::TextDiff::from_lines(&old, &new);
    let mut rows = Vec::new();
    for op in diff.ops() {
        let (tag, o, n) = op.as_tag_tuple();
        match tag {
            similar::DiffTag::Equal => {
                for (a, b) in o.clone().zip(n.clone()) {
                    rows.push(row(
                        "same",
                        Some((a, old_lines[a])),
                        Some((b, new_lines[b])),
                    ));
                }
            }
            similar::DiffTag::Delete => rows.extend(
                o.clone()
                    .map(|a| row("removed", Some((a, old_lines[a])), None)),
            ),
            similar::DiffTag::Insert => rows.extend(
                n.clone()
                    .map(|b| row("added", None, Some((b, new_lines[b])))),
            ),
            similar::DiffTag::Replace => {
                // Pair lines up as "changed" side by side; the longer side's extra lines stand alone.
                let (os, ns): (Vec<usize>, Vec<usize>) = (o.clone().collect(), n.clone().collect());
                for k in 0..os.len().max(ns.len()) {
                    let a = os.get(k).map(|&a| (a, old_lines[a]));
                    let b = ns.get(k).map(|&b| (b, new_lines[b]));
                    let kind = match (a, b) {
                        (Some(_), Some(_)) => "changed",
                        (Some(_), None) => "removed",
                        _ => "added",
                    };
                    rows.push(row(kind, a, b));
                }
            }
        }
    }
    Ok(rows)
}

#[tauri::command]
pub fn mark_seen(app: AppHandle, path: String) -> AppResult<()> {
    let p = PathBuf::from(&path);
    let meta = std::fs::metadata(&p)?;
    let text = std::fs::read(&p)?;
    let snap = snapshot_path(&app, &p)?;
    // Keep the outgoing snapshot when the content moved on, so the diff view can still show those changes.
    if std::fs::read(&snap).is_ok_and(|old| old != text) {
        let _ = std::fs::rename(&snap, previous_path(&app, &p)?);
    }
    std::fs::write(&snap, &text)?;
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
            // "<hash>.prev.md" belongs to the same file as "<hash>.md".
            let name = f.file_name().to_string_lossy().replace(".prev.md", ".md");
            if !keep.contains(&name) {
                let _ = std::fs::remove_file(f.path());
            }
        }
    }
    if let Ok(d) = data_dir(app) {
        let _ = write_json(&d.join("seen.json"), &all);
    }
}
