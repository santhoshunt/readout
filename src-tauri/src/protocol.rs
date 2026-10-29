use std::path::PathBuf;

use percent_encoding::percent_decode_str;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Runtime, UriSchemeContext, UriSchemeResponder};

/// Serves local images for http://readout-file.localhost/<percent-encoded absolute path>.
/// Only image types are served; anything else gets a 404.
pub fn handle<R: Runtime>(
    _ctx: UriSchemeContext<'_, R>,
    req: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let encoded = req.uri().path().trim_start_matches('/').to_string();
    std::thread::spawn(move || responder.respond(serve(&encoded)));
}

fn mime_for(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        _ => return None,
    })
}

fn not_found() -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Vec::new())
        .unwrap()
}

fn serve(encoded: &str) -> Response<Vec<u8>> {
    let path = PathBuf::from(percent_decode_str(encoded).decode_utf8_lossy().to_string());
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let Some(mime) = mime_for(&ext) else {
        return not_found();
    };
    match std::fs::read(&path) {
        Ok(bytes) => Response::builder()
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, "no-cache")
            .body(bytes)
            .unwrap_or_else(|_| not_found()),
        Err(_) => not_found(),
    }
}
