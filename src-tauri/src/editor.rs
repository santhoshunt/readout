use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use serde::Serialize;
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::settings::EditorSettings;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Serialize, Clone, Debug)]
pub struct EditorPreset {
    pub id: String,
    pub label: String,
    pub command: String,
}

fn preset_command(id: &str) -> String {
    format!("{id} --goto \"{{file}}:{{line}}\"")
}

/// Cursor first, then VS Code, then Notepad, which every Windows PC has.
pub fn default_editor() -> EditorSettings {
    for id in ["cursor", "code"] {
        if which::which(id).is_ok() {
            return EditorSettings {
                preset: id.into(),
                command: preset_command(id),
            };
        }
    }
    EditorSettings {
        preset: "custom".into(),
        command: "notepad \"{file}\"".into(),
    }
}

#[tauri::command]
pub fn detect_editors() -> Vec<EditorPreset> {
    let mut found: Vec<EditorPreset> = [("cursor", "Cursor"), ("code", "VS Code")]
        .into_iter()
        .filter(|(id, _)| which::which(id).is_ok())
        .map(|(id, label)| EditorPreset {
            id: id.into(),
            label: label.into(),
            command: preset_command(id),
        })
        .collect();
    found.push(EditorPreset {
        id: "custom".into(),
        label: "Custom command".into(),
        command: String::new(),
    });
    found
}

#[tauri::command]
pub fn open_in_editor(app: AppHandle, path: String, line: u32) -> AppResult<()> {
    let template = crate::settings::current(&app).editor.command;
    // Split the template first and substitute per argument, so backslashes in the
    // file path are never read as shell escapes.
    let parts = shell_words::split(&template).map_err(|e| AppError::Editor(e.to_string()))?;
    let (program, args) = parts
        .split_first()
        .ok_or_else(|| AppError::Editor("empty command".into()))?;
    let folder = Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let fill = |s: &str| {
        s.replace("{file}", &path)
            .replace("{line}", &line.max(1).to_string())
            .replace("{folder}", &folder)
    };
    // which applies PATHEXT, so code.cmd and cursor.cmd resolve. No cmd /C, so no injection.
    let resolved = which::which(fill(program))
        .map_err(|_| AppError::Editor(format!("{program} not found")))?;
    Command::new(resolved)
        .args(args.iter().map(|a| fill(a)))
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| AppError::Editor(e.to_string()))?;
    Ok(())
}
