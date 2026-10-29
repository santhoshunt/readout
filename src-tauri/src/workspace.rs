use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::seen;

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Seen,
    New,
    Changed,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub status: FileStatus,
}

/// Canonical absolute path without the \\?\ prefix that canonicalize adds on Windows.
pub fn normalize(p: &Path) -> PathBuf {
    let c = std::fs::canonicalize(p).unwrap_or_else(|_| lexical(p));
    let s = c.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC") => PathBuf::from(rest),
        _ => c,
    }
}

/// Resolves "." and ".." without touching the disk, for paths that don't exist.
fn lexical(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

pub fn path_key(p: &Path) -> String {
    p.to_string_lossy().to_lowercase()
}

pub fn is_markdown(p: &Path, extensions: &[String]) -> bool {
    p.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .is_some_and(|e| extensions.iter().any(|x| x.eq_ignore_ascii_case(&e)))
}

/// Turns command-line arguments into existing absolute paths. Flags are skipped.
/// Folders end with a backslash so the front end can tell them from files.
pub fn resolve_args(args: &[String], cwd: &Path) -> Vec<String> {
    args.iter()
        .filter(|a| !a.starts_with('-'))
        .map(|a| {
            let p = PathBuf::from(a);
            if p.is_absolute() { p } else { cwd.join(p) }
        })
        .filter(|p| p.exists())
        .map(|p| {
            let s = normalize(&p).to_string_lossy().to_string();
            if p.is_dir() && !s.ends_with('\\') {
                format!("{s}\\")
            } else {
                s
            }
        })
        .collect()
}

pub fn under_any(path: &Path, folders: &[String]) -> bool {
    let key = path_key(path);
    folders.iter().any(|f| {
        let fk = path_key(&normalize(Path::new(f)));
        key == fk || key.starts_with(&format!("{}\\", fk.trim_end_matches('\\')))
    })
}

#[tauri::command]
pub fn list_dir(app: AppHandle, path: String) -> AppResult<Vec<FileEntry>> {
    let dir = PathBuf::from(&path);
    if !dir.is_dir() {
        return Err(AppError::NotFound(path));
    }
    let settings = crate::settings::current(&app);
    let watch = &settings.watch;
    let mut out = Vec::new();
    // Hidden folders stay visible because agents write to folders such as .claude.
    let walker = ignore::WalkBuilder::new(&dir)
        .max_depth(Some(1))
        .hidden(false)
        .git_ignore(true)
        .build();
    for entry in walker.flatten() {
        let p = entry.path();
        if p == dir {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
        if is_dir {
            if watch
                .exclude_folders
                .iter()
                .any(|x| x.eq_ignore_ascii_case(&name))
            {
                continue;
            }
        } else if !is_markdown(p, &watch.extensions) {
            continue;
        }
        let status = if is_dir || !under_any(p, &watch.folders) {
            FileStatus::Seen
        } else {
            seen::status(&app, p)
        };
        out.push(FileEntry {
            path: p.to_string_lossy().to_string(),
            name,
            is_dir,
            status,
        });
    }
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(out)
}
