use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use notify_debouncer_full::notify::{EventKind, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{DebounceEventResult, Debouncer, RecommendedCache, new_debouncer};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::workspace::{is_markdown, normalize, under_any};

pub struct WatchState(pub Mutex<Option<Debouncer<RecommendedWatcher, RecommendedCache>>>);

#[derive(Serialize, Clone)]
struct FileChanged {
    path: String,
    kind: &'static str,
}

fn excluded(path: &Path, names: &[String]) -> bool {
    path.components().any(|c| {
        names
            .iter()
            .any(|n| c.as_os_str().to_string_lossy().eq_ignore_ascii_case(n))
    })
}

fn on_events(app: &AppHandle, result: DebounceEventResult) {
    let Ok(events) = result else { return };
    let watch = crate::settings::current(app).watch;
    for ev in events {
        let kind = match ev.kind {
            EventKind::Create(_) => "created",
            EventKind::Modify(_) => "modified",
            EventKind::Remove(_) => "removed",
            _ => continue,
        };
        for p in &ev.paths {
            if excluded(p, &watch.exclude_folders) {
                continue;
            }
            let is_dir = p.is_dir();
            if !is_dir && !is_markdown(p, &watch.extensions) {
                continue;
            }
            crate::search::invalidate(app);
            let _ = app.emit(
                "file-changed",
                FileChanged {
                    path: p.to_string_lossy().to_string(),
                    kind,
                },
            );
        }
    }
}

/// Replaces all watches: `recursive` for watched folders and explorer roots,
/// `flat` for the folders of open tabs that are not already covered.
#[tauri::command]
pub fn update_watch(app: AppHandle, recursive: Vec<String>, flat: Vec<String>) -> AppResult<()> {
    let handle = app.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(300),
        None,
        move |r: DebounceEventResult| on_events(&handle, r),
    )
    .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    let roots: Vec<PathBuf> = recursive
        .iter()
        .map(|r| normalize(Path::new(r)))
        .filter(|p| p.is_dir())
        .collect();
    let root_strings: Vec<String> = roots
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    for r in &roots {
        // An unreachable folder (a network drive, say) must not stop the others.
        let _ = debouncer.watch(r, RecursiveMode::Recursive);
    }
    for f in flat
        .iter()
        .map(|f| normalize(Path::new(f)))
        .filter(|p| p.is_dir())
    {
        if !under_any(&f, &root_strings) {
            let _ = debouncer.watch(&f, RecursiveMode::NonRecursive);
        }
    }
    *app.state::<WatchState>().0.lock().unwrap() = Some(debouncer);
    Ok(())
}
