mod editor;
mod error;
mod protocol;
mod render;
mod search;
mod seen;
mod session;
mod settings;
mod watch;
mod workspace;

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

/// Paths given on the first launch. The front end takes them once it is listening,
/// because an event sent before then would be lost.
pub struct StartupPaths(pub Mutex<Vec<String>>);

#[derive(Serialize, Clone)]
struct FilesOpened {
    paths: Vec<String>,
}

#[tauri::command]
fn take_startup_paths(state: State<'_, StartupPaths>) -> Vec<String> {
    std::mem::take(&mut *state.0.lock().unwrap())
}

fn focus_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn run() {
    let cwd = std::env::current_dir().unwrap_or_default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let startup = workspace::resolve_args(&args, &cwd);

    tauri::Builder::default()
        // Must be the first plugin. A second launch hands its paths to this process and exits.
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            let paths = workspace::resolve_args(
                args.get(1..).unwrap_or_default(),
                std::path::Path::new(&cwd),
            );
            let _ = app.emit("files-opened", FilesOpened { paths });
            focus_main(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(StartupPaths(Mutex::new(startup)))
        .manage(settings::SettingsState(Mutex::new(None)))
        .manage(seen::SeenState(Mutex::new(None)))
        .manage(search::SearchState(Mutex::new(search::Index::default())))
        .manage(watch::WatchState(Mutex::new(None)))
        .register_asynchronous_uri_scheme_protocol("readout-file", protocol::handle)
        .setup(|app| {
            seen::cleanup(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            take_startup_paths,
            render::render_file,
            seen::mark_seen,
            workspace::list_dir,
            search::palette_query,
            settings::get_settings,
            settings::save_settings,
            session::load_session,
            session::save_session,
            editor::detect_editors,
            editor::open_in_editor,
            watch::update_watch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Readout");
}
