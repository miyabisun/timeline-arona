//! The MCP server mode (`timeline-arona mcp`): the client's lines come on stdin,
//! the answers go to stdout, and the tools are carried out in the app.

use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

use crate::mcp::{self, Tools};
use crate::{students, timeline};

/// Serves the client on stdin and stdout until it goes, then ends the app.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tools = AppTools { app: app.clone() };
        let mut out = std::io::stdout().lock();
        for line in std::io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            if let Some(answer) = mcp::respond(&line, &mut tools)
                && writeln!(out, "{answer}")
                    .and_then(|()| out.flush())
                    .is_err()
            {
                break;
            }
        }
        app.exit(0);
    });
}

struct AppTools {
    app: AppHandle,
}

impl Tools for AppTools {
    fn call(&mut self, name: &str, args: &Value) -> Result<Value, String> {
        let text = |key: &str| {
            args[key]
                .as_str()
                .ok_or_else(|| format!("{key} を入れてください"))
        };
        match name {
            "video_open" => video_open(&self.app, text("url")?),
            "video_frames" => video_frames(&self.app, args),
            "students" => find_students(&self.app, args["query"].as_str().unwrap_or_default()),
            "timeline_check" => {
                let (yaml, problems) = timeline::tidy(text("yaml")?)?;
                Ok(json!({"problems": problems, "yaml": yaml}))
            }
            "timeline_open" => open_in_editor(&self.app, text("path")?),
            _ => Err(format!("no tool named {name}")),
        }
    }
}

#[cfg(windows)]
fn video_open(app: &AppHandle, url: &str) -> Result<Value, String> {
    crate::browser::open(app, url)
}

#[cfg(windows)]
fn video_frames(app: &AppHandle, args: &Value) -> Result<Value, String> {
    crate::browser::frames(app, args)
}

#[cfg(not(windows))]
fn video_open(_: &AppHandle, _: &str) -> Result<Value, String> {
    Err("動画は Windows 版だけで読めます".to_owned())
}

#[cfg(not(windows))]
fn video_frames(_: &AppHandle, _: &Value) -> Result<Value, String> {
    Err("動画は Windows 版だけで読めます".to_owned())
}

/// The students `query` may mean, each with its face icon written out to a file.
fn find_students(app: &AppHandle, query: &str) -> Result<Value, String> {
    let json = String::from_utf8(asset(app, "schaledb/students.json")?)
        .map_err(|e| format!("students.json: {e}"))?;
    let all = students::parse(&json)?;
    let dir = std::env::temp_dir().join("timeline-arona").join("students");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    students::search(&all, query)
        .into_iter()
        .map(|s| {
            let icon = dir.join(format!("{}.webp", s.id));
            if !icon.is_file() {
                let bytes = asset(app, &format!("schaledb/students/{}.webp", s.id))?;
                std::fs::write(&icon, bytes).map_err(|e| format!("{}: {e}", icon.display()))?;
            }
            Ok(json!({"id": s.id, "name": s.name, "squad": s.squad, "released": s.released, "icon": icon}))
        })
        .collect::<Result<Vec<Value>, String>>()
        .map(Value::from)
}

/// A file the pages are built with (`public/`), as embedded in the app.
fn asset(app: &AppHandle, path: &str) -> Result<Vec<u8>, String> {
    if let Some(asset) = app.asset_resolver().get(path.to_owned()) {
        return Ok(asset.bytes);
    }
    // `tauri dev` serves the pages from Vite and embeds nothing.
    #[cfg(debug_assertions)]
    if let Ok(bytes) = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../public")
            .join(path),
    ) {
        return Ok(bytes);
    }
    Err(format!("{path} がアプリに入っていません"))
}

/// Shows the file at `path` in the editor window, made if there is none.
fn open_in_editor(app: &AppHandle, path: &str) -> Result<Value, String> {
    let file = PathBuf::from(path);
    if !file.is_absolute() || !file.is_file() {
        return Err(format!(
            "タイムラインのファイルの絶対パスを渡してください: {path}"
        ));
    }
    if let Some(window) = app.get_webview_window("main") {
        app.emit_to("main", "open", path)
            .map_err(|e| e.to_string())?;
        let _ = window.unminimize();
        let _ = window.set_focus();
    } else {
        app.state::<crate::Pending>().set(path);
        crate::show_editor(app).map_err(|e| e.to_string())?;
    }
    Ok(json!({"opened": path}))
}
