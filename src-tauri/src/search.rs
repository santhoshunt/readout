use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use comrak::nodes::NodeValue;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::error::AppResult;
use crate::workspace::is_markdown;

const MAX_FILES: usize = 20_000;
const MAX_HITS: usize = 60;

#[derive(Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PaletteMode {
    Files,
    Headings,
}

#[derive(Serialize, Clone, Debug)]
pub struct PaletteHit {
    pub path: String,
    pub label: String,
    pub detail: String,
    pub line: Option<u32>,
    pub score: u32,
}

struct IndexedFile {
    path: PathBuf,
    rel: String,
    mtime: Option<SystemTime>,
    headings: Option<Vec<(String, u32)>>,
}

#[derive(Default)]
pub struct Index {
    roots: Vec<String>,
    files: Vec<IndexedFile>,
    stale: bool,
    heading_cache: HashMap<PathBuf, (Option<SystemTime>, Vec<(String, u32)>)>,
}

pub struct SearchState(pub Mutex<Index>);

/// Called by the watcher. The next query walks the roots again.
pub fn invalidate(app: &AppHandle) {
    app.state::<SearchState>().0.lock().unwrap().stale = true;
}

fn walk(roots: &[String], extensions: &[String], exclude: &[String]) -> Vec<IndexedFile> {
    let mut out = Vec::new();
    for root in roots {
        let base = Path::new(root);
        let excl = exclude.to_vec();
        let walker = ignore::WalkBuilder::new(base)
            .hidden(false)
            .filter_entry(move |e| {
                !excl
                    .iter()
                    .any(|x| e.file_name().to_string_lossy().eq_ignore_ascii_case(x))
            })
            .build();
        for e in walker.flatten() {
            if out.len() >= MAX_FILES {
                return out;
            }
            let p = e.path();
            if e.file_type().is_some_and(|t| t.is_file()) && is_markdown(p, extensions) {
                let rel = p
                    .strip_prefix(base)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .to_string();
                let mtime = e.metadata().ok().and_then(|m| m.modified().ok());
                out.push(IndexedFile {
                    path: p.to_path_buf(),
                    rel,
                    mtime,
                    headings: None,
                });
            }
        }
    }
    out
}

/// Brings the file index up to date for the explorer roots plus the watched folders.
fn refresh(app: &AppHandle, roots: Vec<String>) {
    let watch = crate::settings::current(app).watch;
    let mut all_roots = roots;
    all_roots.extend(watch.folders.iter().cloned());
    all_roots.sort();
    all_roots.dedup();
    let state = app.state::<SearchState>();
    let mut idx = state.0.lock().unwrap();
    if idx.stale || idx.roots != all_roots {
        idx.files = walk(&all_roots, &watch.extensions, &watch.exclude_folders);
        idx.roots = all_roots;
        idx.stale = false;
    }
}

const MAX_TEXT_HITS: usize = 1000;
const MAX_HITS_PER_FILE: usize = 50;
const PREVIEW_CHARS: usize = 160;

#[derive(Serialize, Clone, Debug)]
pub struct TextHit {
    pub path: String,
    pub rel: String,
    pub line: u32,
    /// The matching line, trimmed around the match when it is long.
    pub text: String,
    /// Match position inside `text`, in UTF-16 units so JavaScript can slice it directly.
    pub start: u32,
    pub end: u32,
}

fn utf16_len(s: &str) -> u32 {
    s.encode_utf16().count() as u32
}

/// Plain substring search through every indexed Markdown file, line by line.
#[tauri::command]
pub async fn search_text(
    app: AppHandle,
    query: String,
    roots: Vec<String>,
    case_sensitive: bool,
) -> AppResult<Vec<TextHit>> {
    let app2 = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        refresh(&app2, roots);
        let files: Vec<(PathBuf, String)> = {
            let state = app2.state::<SearchState>();
            let idx = state.0.lock().unwrap();
            idx.files
                .iter()
                .map(|f| (f.path.clone(), f.rel.clone()))
                .collect()
        };
        let needle = if case_sensitive {
            query.clone()
        } else {
            query.to_lowercase()
        };
        let mut hits = Vec::new();
        for (path, rel) in files {
            let Ok((text, _)) = crate::render::read_text(&path) else {
                continue;
            };
            if text.len() > crate::render::PLAIN_TEXT_LIMIT {
                continue;
            }
            let mut in_file = 0;
            for (n, line) in text.lines().enumerate() {
                let hay = if case_sensitive {
                    line.to_string()
                } else {
                    line.to_lowercase()
                };
                // Lowercasing can change byte lengths for some scripts; only use positions when they line up.
                let Some(at) = hay.find(&needle).filter(|_| hay.len() == line.len()) else {
                    if hay.contains(&needle) {
                        hits.push(TextHit {
                            path: path.to_string_lossy().to_string(),
                            rel: rel.clone(),
                            line: n as u32 + 1,
                            text: line.chars().take(PREVIEW_CHARS).collect(),
                            start: 0,
                            end: 0,
                        });
                        in_file += 1;
                    }
                    continue;
                };
                let end = at + needle.len();
                // Keep about 40 characters of context before a match deep in a long line.
                let from = line[..at]
                    .char_indices()
                    .rev()
                    .nth(40)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let shown: String = line[from..].chars().take(PREVIEW_CHARS).collect();
                hits.push(TextHit {
                    path: path.to_string_lossy().to_string(),
                    rel: rel.clone(),
                    line: n as u32 + 1,
                    start: utf16_len(&line[from..at]),
                    end: utf16_len(&line[from..end]).min(utf16_len(&shown)),
                    text: shown,
                });
                in_file += 1;
                if in_file >= MAX_HITS_PER_FILE || hits.len() >= MAX_TEXT_HITS {
                    break;
                }
            }
            if hits.len() >= MAX_TEXT_HITS {
                break;
            }
        }
        Ok(hits)
    })
    .await
    .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?
}

fn headings_of(path: &Path) -> Vec<(String, u32)> {
    let Ok((text, _)) = crate::render::read_text(path) else {
        return Vec::new();
    };
    let arena = comrak::Arena::new();
    let mut opts = comrak::Options::default();
    opts.extension.front_matter_delimiter = Some("---".into());
    let root = comrak::parse_document(&arena, &text, &opts);
    root.descendants()
        .filter(|n| matches!(n.data.borrow().value, NodeValue::Heading(_)))
        .map(|n| {
            let mut s = String::new();
            for d in n.descendants() {
                match &d.data.borrow().value {
                    NodeValue::Text(t) => s.push_str(t),
                    NodeValue::Code(c) => s.push_str(&c.literal),
                    _ => {}
                }
            }
            (s, n.data.borrow().sourcepos.start.line as u32)
        })
        .collect()
}

#[tauri::command]
pub async fn palette_query(
    app: AppHandle,
    query: String,
    mode: PaletteMode,
    roots: Vec<String>,
) -> AppResult<Vec<PaletteHit>> {
    let app2 = app.clone();
    let hits = tauri::async_runtime::spawn_blocking(move || {
        refresh(&app2, roots);
        let state = app2.state::<SearchState>();
        let mut idx = state.0.lock().unwrap();

        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        let pattern = Pattern::parse(&query, CaseMatching::Ignore, Normalization::Smart);
        let mut buf = Vec::new();
        let mut hits = Vec::new();
        match mode {
            PaletteMode::Files => {
                for f in &idx.files {
                    let score = if query.is_empty() {
                        Some(0)
                    } else {
                        pattern.score(Utf32Str::new(&f.rel, &mut buf), &mut matcher)
                    };
                    if let Some(score) = score {
                        let label = f
                            .path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        hits.push(PaletteHit {
                            path: f.path.to_string_lossy().to_string(),
                            label,
                            detail: f.rel.clone(),
                            line: None,
                            score,
                        });
                    }
                }
            }
            PaletteMode::Headings => {
                let Index {
                    files,
                    heading_cache,
                    ..
                } = &mut *idx;
                for f in files.iter_mut() {
                    let cached = heading_cache
                        .get(&f.path)
                        .filter(|(m, _)| *m == f.mtime)
                        .map(|(_, h)| h.clone());
                    let hs = cached.unwrap_or_else(|| {
                        let h = headings_of(&f.path);
                        heading_cache.insert(f.path.clone(), (f.mtime, h.clone()));
                        h
                    });
                    for (text, line) in &hs {
                        let score = if query.is_empty() {
                            Some(0)
                        } else {
                            pattern.score(Utf32Str::new(text, &mut buf), &mut matcher)
                        };
                        if let Some(score) = score {
                            hits.push(PaletteHit {
                                path: f.path.to_string_lossy().to_string(),
                                label: text.clone(),
                                detail: f.rel.clone(),
                                line: Some(*line),
                                score,
                            });
                        }
                    }
                    f.headings = Some(hs);
                }
            }
        }
        hits.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.detail.len().cmp(&b.detail.len()))
        });
        hits.truncate(MAX_HITS);
        hits
    })
    .await
    .map_err(|e| crate::error::AppError::Io(std::io::Error::other(e.to_string())))?;
    Ok(hits)
}
