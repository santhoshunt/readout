use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::render::{PLAIN_TEXT_LIMIT, read_text};
use crate::seen;
use crate::workspace::{normalize, path_key};

/// The last write Readout made to each file, as (mtime, length). The watcher compares events
/// against it so the app's own saves don't reload the tab or show up as "changed".
pub struct OwnWrites(pub Mutex<HashMap<String, (u64, u64)>>);

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SourceFile {
    pub text: String,
    pub mtime_ms: u64,
    pub crlf: bool,
    pub lossy: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SavedFile {
    pub mtime_ms: u64,
}

pub fn is_own_write(app: &AppHandle, path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let now = (seen::mtime_ms(&meta), meta.len());
    app.state::<OwnWrites>()
        .0
        .lock()
        .unwrap()
        .get(&path_key(path))
        == Some(&now)
}

fn mtime_of(path: &Path) -> u64 {
    std::fs::metadata(path)
        .map(|m| seen::mtime_ms(&m))
        .unwrap_or(0)
}

fn check_writable(app: &AppHandle, text: &str, lossy: bool) -> AppResult<()> {
    if !crate::settings::current(app).editing.allow_edits {
        return Err(AppError::ReadOnly(
            "Editing is switched off in Settings.".into(),
        ));
    }
    if lossy {
        return Err(AppError::ReadOnly(
            "This file isn't UTF-8, so Readout won't write it.".into(),
        ));
    }
    if text.len() > PLAIN_TEXT_LIMIT {
        return Err(AppError::ReadOnly(
            "Files over 10 MB are read-only in Readout.".into(),
        ));
    }
    Ok(())
}

fn check_unchanged(path: &Path, base_mtime_ms: u64) -> AppResult<()> {
    if base_mtime_ms != 0 && mtime_of(path) != base_mtime_ms {
        return Err(AppError::Conflict(
            "The file changed on disk since Readout last read it.".into(),
        ));
    }
    Ok(())
}

/// Writes next to the file and renames over it, so a crash never leaves a half-written file.
fn write_atomic(app: &AppHandle, path: &Path, text: &str) -> AppResult<SavedFile> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!("{name}.readout.tmp"));
    std::fs::write(&tmp, text)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
    let meta = std::fs::metadata(path)?;
    let mtime_ms = seen::mtime_ms(&meta);
    app.state::<OwnWrites>()
        .0
        .lock()
        .unwrap()
        .insert(path_key(path), (mtime_ms, meta.len()));
    // Your own edit is, by definition, seen.
    let _ = seen::mark_seen(app.clone(), path.to_string_lossy().to_string());
    Ok(SavedFile { mtime_ms })
}

/// Editors work in "\n"; files that used CRLF get it back on save.
fn with_line_endings(text: &str, crlf: bool) -> String {
    let lf = text.replace("\r\n", "\n");
    if crlf { lf.replace('\n', "\r\n") } else { lf }
}

#[tauri::command]
pub fn read_source(path: String) -> AppResult<SourceFile> {
    let p = normalize(Path::new(&path));
    let (text, lossy) = read_text(&p)?;
    Ok(SourceFile {
        crlf: text.contains("\r\n"),
        mtime_ms: mtime_of(&p),
        text,
        lossy,
    })
}

#[tauri::command]
pub fn save_file(
    app: AppHandle,
    path: String,
    text: String,
    base_mtime_ms: u64,
    force: bool,
) -> AppResult<SavedFile> {
    let p = normalize(Path::new(&path));
    let (current, lossy) = read_text(&p)?;
    check_writable(&app, &current, lossy)?;
    if !force {
        check_unchanged(&p, base_mtime_ms)?;
    }
    write_atomic(
        &app,
        &p,
        &with_line_endings(&text, current.contains("\r\n")),
    )
}

/// Byte index of the space or x inside "[ ]" when `line` is a task list item, including tasks
/// inside block quotes and alerts ("> - [ ] ..."). None for anything else.
fn task_box(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    loop {
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'>' {
            i += 1;
        } else {
            break;
        }
    }
    match bytes.get(i) {
        Some(b'-' | b'*' | b'+') => i += 1,
        Some(b'0'..=b'9') => {
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            match bytes.get(i) {
                Some(b'.' | b')') => i += 1,
                _ => return None,
            }
        }
        _ => return None,
    }
    let spaces = bytes[i..]
        .iter()
        .take_while(|&&b| b == b' ' || b == b'\t')
        .count();
    if spaces == 0 {
        return None;
    }
    i += spaces;
    match bytes.get(i..i + 3) {
        Some([b'[', b' ' | b'x' | b'X', b']']) => Some(i + 1),
        _ => None,
    }
}

#[tauri::command]
pub fn toggle_task(
    app: AppHandle,
    path: String,
    line: u32,
    checked: bool,
    base_mtime_ms: u64,
) -> AppResult<SavedFile> {
    let p = normalize(Path::new(&path));
    let (text, lossy) = read_text(&p)?;
    check_writable(&app, &text, lossy)?;
    check_unchanged(&p, base_mtime_ms)?;
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let idx = (line as usize).checked_sub(1).filter(|&i| i < lines.len());
    let target = idx.and_then(|i| task_box(&lines[i]).map(|b| (i, b)));
    let Some((i, at)) = target else {
        return Err(AppError::Conflict(
            "That line no longer holds a task.".into(),
        ));
    };
    lines[i].replace_range(at..at + 1, if checked { "x" } else { " " });
    write_atomic(&app, &p, &lines.concat())
}

/// Writes an exported page. Only .html targets, which come from the save dialog.
#[tauri::command]
pub fn write_export(dest: String, content: String) -> AppResult<()> {
    if !dest.to_lowercase().ends_with(".html") {
        return Err(AppError::ReadOnly(
            "Exports are saved as .html files.".into(),
        ));
    }
    std::fs::write(&dest, content)?;
    Ok(())
}
