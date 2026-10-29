use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use comrak::nodes::{AstNode, NodeHtmlBlock, NodeValue};
use comrak::options::Plugins;
use comrak::plugins::syntect::{SyntectAdapter, SyntectAdapterBuilder};
use comrak::{Arena, Options};
use percent_encoding::{NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};
use serde::Serialize;
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::seen;
use crate::workspace::normalize;

pub const PLAIN_TEXT_LIMIT: usize = 10 * 1024 * 1024;
const DIAGRAM_LANGS: [&str; 5] = ["mermaid", "dot", "graphviz", "plantuml", "puml"];
const CLASS_PREFIXES: [&str; 7] = [
    "syn-",
    "markdown-alert",
    "task-list-item",
    "contains-task-list",
    "footnote",
    "diagram",
    "anchor",
];
const IMAGE_ORIGIN: &str = "http://readout-file.localhost/";
const DRIVE_LETTERS: [&str; 26] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
    "t", "u", "v", "w", "x", "y", "z",
];

/// syntect's built-in syntaxes lack some common languages; these map to the closest one it has.
fn syntax_alias(lang: &str) -> Option<&'static str> {
    Some(match lang {
        "ts" | "typescript" | "tsx" | "jsx" | "mjs" | "cjs" => "js",
        "ps1" | "powershell" | "pwsh" | "shell" | "zsh" | "console" => "sh",
        "svelte" | "vue" => "html",
        "jsonc" | "json5" => "json",
        _ => return None,
    })
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RenderedDoc {
    pub path: String,
    pub html: String,
    pub headings: Vec<Heading>,
    pub front_matter: Vec<(String, String)>,
    pub word_count: usize,
    pub changed_ranges: Vec<LineRange>,
    pub resolved_paths: HashMap<String, ResolvedPath>,
    pub plain_text_fallback: bool,
    pub lossy: bool,
    /// Modification time of the file when it was read; writes send it back to detect changes on disk.
    pub mtime_ms: u64,
    /// The file uses CRLF line endings, which saves keep.
    pub crlf: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct Heading {
    pub level: u8,
    pub text: String,
    pub id: String,
    pub line: u32,
}

#[derive(Serialize, Clone, Copy, Debug)]
pub struct LineRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct ResolvedPath {
    pub abs: String,
    pub line: Option<u32>,
}

fn adapter() -> &'static SyntectAdapter {
    static ADAPTER: OnceLock<SyntectAdapter> = OnceLock::new();
    ADAPTER.get_or_init(|| {
        SyntectAdapterBuilder::new()
            .css_with_class_prefix("syn-")
            .build()
    })
}

fn options() -> Options<'static> {
    let mut o = Options::default();
    o.extension.strikethrough = true;
    o.extension.table = true;
    o.extension.autolink = true;
    o.extension.tasklist = true;
    o.extension.footnotes = true;
    o.extension.alerts = true;
    o.extension.math_dollars = true;
    o.extension.front_matter_delimiter = Some("---".into());
    o.extension.header_id_prefix = Some(String::new());
    o.render.sourcepos = true;
    o.render.r#unsafe = true;
    o.render.github_pre_lang = true;
    o.render.tasklist_classes = true;
    o
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn inline_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut s = String::new();
    for d in node.descendants() {
        match &d.data.borrow().value {
            NodeValue::Text(t) => s.push_str(t),
            NodeValue::Code(c) => s.push_str(&c.literal),
            _ => {}
        }
    }
    s
}

/// Top-level YAML keys as (key, value). Nested values are kept as YAML text.
fn parse_front_matter(raw: &str) -> Vec<(String, String)> {
    use yaml_rust2::{Yaml, YamlEmitter, YamlLoader};
    let body: String = raw
        .lines()
        .filter(|l| l.trim() != "---")
        .collect::<Vec<_>>()
        .join("\n");
    let Ok(docs) = YamlLoader::load_from_str(&body) else {
        return vec![("front matter".into(), body)];
    };
    let Some(Yaml::Hash(map)) = docs.into_iter().next() else {
        return Vec::new();
    };
    map.into_iter()
        .map(|(k, v)| {
            let key = k
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| format!("{k:?}"));
            let value = match &v {
                Yaml::String(s) => s.clone(),
                Yaml::Integer(i) => i.to_string(),
                Yaml::Real(r) => r.clone(),
                Yaml::Boolean(b) => b.to_string(),
                Yaml::Null => String::new(),
                other => {
                    let mut out = String::new();
                    let _ = YamlEmitter::new(&mut out).dump(other);
                    out.trim_start_matches("---").trim().to_string()
                }
            };
            (key, value)
        })
        .collect()
}

/// Splits "path:line:col" into the path and the line. Returns None for text that is not path-shaped.
fn split_path_line(code: &str) -> Option<(&str, Option<u32>)> {
    let code = code.trim();
    if code.is_empty() || code.len() > 260 || code.contains('\n') || code.contains("://") {
        return None;
    }
    let mut path = code;
    let mut line = None;
    for _ in 0..2 {
        if let Some((head, tail)) = path.rsplit_once(':') {
            if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) && head.len() > 1 {
                line = tail.parse().ok().or(line);
                path = head;
                continue;
            }
        }
        break;
    }
    let file = Path::new(path).file_name()?.to_string_lossy().to_string();
    file.contains('.').then_some((path, line))
}

/// Looks for the file next to the document first, then in each parent folder,
/// because agents usually write repo-relative paths into files in sub-folders.
fn resolve_candidate(code: &str, doc_dir: &Path) -> Option<ResolvedPath> {
    let (path, line) = split_path_line(code)?;
    let p = PathBuf::from(path);
    let found = if p.is_absolute() {
        p.is_file().then_some(p)
    } else {
        doc_dir
            .ancestors()
            .take(8)
            .map(|d| d.join(&p))
            .find(|c| c.is_file())
    }?;
    Some(ResolvedPath {
        abs: normalize(&found).to_string_lossy().to_string(),
        line,
    })
}

fn decode(s: &str) -> String {
    percent_decode_str(s).decode_utf8_lossy().to_string()
}

/// Maps every <img src> to the web, a data URI, or the readout-file protocol.
fn rewrite_img_src(src: &str, base: &Path) -> Option<String> {
    let s = src.trim();
    if s.starts_with("https://") || s.starts_with("data:image/") {
        return Some(s.to_string());
    }
    if let Some(rest) = s.strip_prefix("http://") {
        return Some(format!("https://{rest}"));
    }
    let local = if let Some(rest) = s.strip_prefix("file:///") {
        PathBuf::from(decode(rest))
    } else if s.contains("://") || s.contains(':') && !looks_like_drive(s) {
        return None;
    } else {
        let clean = s.split(['?', '#']).next().unwrap_or(s);
        let p = PathBuf::from(decode(clean));
        if p.is_absolute() { p } else { base.join(p) }
    };
    let full = normalize(&local);
    Some(format!(
        "{IMAGE_ORIGIN}{}",
        utf8_percent_encode(&full.to_string_lossy(), NON_ALPHANUMERIC)
    ))
}

fn looks_like_drive(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/')
}

fn clean_html(html: &str, base: PathBuf) -> String {
    let mut b = ammonia::Builder::default();
    b.add_tags(["input", "section"])
        .add_tag_attributes("input", ["type", "checked", "disabled"])
        .add_generic_attributes([
            "data-sourcepos",
            "data-math-style",
            "data-diagram",
            "data-footnotes",
            "data-footnote-ref",
            "data-footnote-backref",
            "lang",
            "id",
            "class",
            "aria-hidden",
            "aria-label",
        ])
        .add_url_schemes(["data"])
        // ammonia checks schemes before attribute_filter runs, and "D:/x.png" parses as scheme "d".
        // Allowing single letters lets drive paths reach the filter, which rewrites them.
        .add_url_schemes(DRIVE_LETTERS)
        .attribute_filter(
            move |element, attribute, value| match (element, attribute) {
                (_, "class") => {
                    let kept: Vec<&str> = value
                        .split_whitespace()
                        .filter(|c| CLASS_PREFIXES.iter().any(|p| c.starts_with(p)))
                        .collect();
                    (!kept.is_empty()).then(|| Cow::Owned(kept.join(" ")))
                }
                ("img", "src") => rewrite_img_src(value, &base).map(Cow::Owned),
                (_, "href") if value.trim_start().starts_with("data:") => None,
                (_, "src") if value.trim_start().starts_with("data:") => None,
                _ => Some(Cow::Borrowed(value)),
            },
        );
    b.clean(html).to_string()
}

pub fn render_text(
    app: &AppHandle,
    path: &Path,
    text: &str,
    lossy: bool,
    mtime_ms: u64,
) -> RenderedDoc {
    let doc_dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let changed_ranges = seen::changed_ranges(app, path, text);
    let path_str = path.to_string_lossy().to_string();
    let crlf = text.contains("\r\n");

    if text.len() > PLAIN_TEXT_LIMIT {
        return RenderedDoc {
            path: path_str,
            html: format!(
                "<pre data-sourcepos=\"1:1-1:1\">{}</pre>",
                escape_html(text)
            ),
            headings: Vec::new(),
            front_matter: Vec::new(),
            word_count: text.split_whitespace().count(),
            changed_ranges,
            resolved_paths: HashMap::new(),
            plain_text_fallback: true,
            lossy,
            mtime_ms,
            crlf,
        };
    }

    let opts = options();
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, text, &opts);
    let mut headings = Vec::new();
    let mut front_matter = Vec::new();
    let mut resolved_paths = HashMap::new();
    let mut word_count = 0;
    let mut anchorizer = comrak::Anchorizer::new();

    for node in root.descendants() {
        let line = node.data.borrow().sourcepos.start.line as u32;
        let mut replace_with = None;
        let mut new_info = None;
        match &node.data.borrow().value {
            NodeValue::FrontMatter(raw) => front_matter = parse_front_matter(raw),
            NodeValue::Heading(h) => {
                let text = inline_text(node);
                let id = anchorizer.anchorize(&text);
                headings.push(Heading {
                    level: h.level,
                    text,
                    id,
                    line,
                });
            }
            NodeValue::Text(t) => word_count += t.split_whitespace().count(),
            NodeValue::Code(c) => {
                word_count += 1;
                if !resolved_paths.contains_key(&c.literal) {
                    if let Some(r) = resolve_candidate(&c.literal, &doc_dir) {
                        resolved_paths.insert(c.literal.clone(), r);
                    }
                }
            }
            NodeValue::CodeBlock(cb) => {
                let lang = cb
                    .info
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                if DIAGRAM_LANGS.contains(&lang.as_str()) {
                    let end = node.data.borrow().sourcepos.end.line;
                    replace_with = Some(format!(
                        "<pre class=\"diagram\" data-diagram=\"{lang}\" data-sourcepos=\"{line}:1-{end}:1\">{}</pre>\n",
                        escape_html(&cb.literal)
                    ));
                } else if let Some(alias) = syntax_alias(&lang) {
                    new_info = Some(alias.to_string());
                }
            }
            _ => {}
        }
        if let Some(literal) = replace_with {
            node.data.borrow_mut().value = NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 6,
                literal,
            });
        } else if let Some(info) = new_info {
            if let NodeValue::CodeBlock(cb) = &mut node.data.borrow_mut().value {
                cb.info = info;
            }
        }
    }

    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(adapter());
    let mut html = String::new();
    let _ = comrak::format_html_with_plugins(root, &opts, &mut html, &plugins);

    RenderedDoc {
        path: path_str,
        html: clean_html(&html, doc_dir),
        headings,
        front_matter,
        word_count,
        changed_ranges,
        resolved_paths,
        plain_text_fallback: false,
        lossy,
        mtime_ms,
        crlf,
    }
}

pub fn read_text(path: &Path) -> AppResult<(String, bool)> {
    let bytes = std::fs::read(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => AppError::NotFound(path.to_string_lossy().to_string()),
        _ => AppError::Io(e),
    })?;
    Ok(match String::from_utf8(bytes) {
        Ok(s) => (s, false),
        Err(e) => (String::from_utf8_lossy(e.as_bytes()).to_string(), true),
    })
}

#[tauri::command]
pub async fn render_file(app: AppHandle, path: String) -> AppResult<RenderedDoc> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = normalize(Path::new(&path));
        let (text, lossy) = read_text(&p)?;
        let mtime = std::fs::metadata(&p)
            .map(|m| seen::mtime_ms(&m))
            .unwrap_or(0);
        Ok(render_text(&app, &p, &text, lossy, mtime))
    })
    .await
    .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}

/// Renders unsaved editor text for the split view's live preview. Relative links and images
/// resolve against `path`, the file being edited.
#[tauri::command]
pub async fn render_markdown(app: AppHandle, path: String, text: String) -> AppResult<RenderedDoc> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = normalize(Path::new(&path));
        Ok(render_text(&app, &p, &text, false, 0))
    })
    .await
    .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?
}
