pub mod timeline;

use std::path::PathBuf;
use timeline::{Problem, Timeline};

/// A file read for editing, with what is wrong in it.
#[derive(serde::Serialize)]
struct Opened {
    timeline: Timeline,
    problems: Vec<Problem>,
}

/// Reads the timeline file at `path`.
#[tauri::command]
fn open_timeline(path: PathBuf) -> Result<Opened, String> {
    let yaml = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let timeline = timeline::read(&yaml)?;
    let problems = timeline::check(&timeline);
    Ok(Opened { timeline, problems })
}

/// What is wrong in `timeline`, field by field.
#[tauri::command]
fn check_timeline(timeline: Timeline) -> Vec<Problem> {
    timeline::check(&timeline)
}

/// Writes `timeline` to `path` in the docs' layout, mistakes and all: the Sensei's
/// work is never lost, and the problems stay listed until fixed.
#[tauri::command]
fn save_timeline(path: PathBuf, timeline: Timeline) -> Result<(), String> {
    std::fs::write(&path, timeline::write(&timeline))
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_timeline,
            check_timeline,
            save_timeline
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
