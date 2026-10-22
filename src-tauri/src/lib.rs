#[cfg(windows)]
mod browser;
pub mod mcp;
mod server;
pub mod students;
pub mod timeline;
pub mod video;

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, RunEvent, State, WebviewWindow, WebviewWindowBuilder};
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

/// The file the editor window opens once it is up (`timeline_open` before there
/// was a window).
#[derive(Default)]
struct Pending(Mutex<Option<String>>);

impl Pending {
    fn set(&self, path: &str) {
        if let Ok(mut pending) = self.0.lock() {
            *pending = Some(path.to_owned());
        }
    }
}

#[tauri::command]
fn take_open(pending: State<Pending>) -> Option<String> {
    pending.0.lock().ok()?.take()
}

/// The editor window, made from the config if there is none.
fn show_editor(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window("main") {
        return Ok(window);
    }
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .cloned()
        .unwrap_or_default();
    WebviewWindowBuilder::from_config(app, &config)?.build()
}

/// The browser arguments of the editor window's config. WebView2 runs every window
/// of the app in one browser, so the video window has to start it the same way.
fn browser_args(app: &AppHandle) -> Option<String> {
    app.config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .and_then(|w| w.additional_browser_args.clone())
}

/// Runs the editor, or with `mcp` the MCP server on stdin and stdout, which opens
/// windows only when a tool needs one.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run(mcp: bool) {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Pending::default())
        .invoke_handler(tauri::generate_handler![
            open_timeline,
            check_timeline,
            save_timeline,
            take_open
        ])
        .setup(move |app| {
            if mcp {
                server::start(app.handle().clone());
            } else {
                show_editor(app.handle())?;
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_, event| {
            // A server lives until its client goes, whatever windows are closed.
            if mcp
                && let RunEvent::ExitRequested {
                    code: None, api, ..
                } = event
            {
                api.prevent_exit();
            }
        });
}
