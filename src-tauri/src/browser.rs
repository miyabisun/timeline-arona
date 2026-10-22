//! The video window: a YouTube page in the app's own WebView2, driven through
//! `video.js` with ExecuteScript. The page gets no IPC to the app.

use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::video::{self, Crop};

const LABEL: &str = "video";
const HELPER: &str = include_str!("video.js");
const POLL: Duration = Duration::from_millis(200);
/// How long ads may hold up one wait on top of its own time. Ads are played and
/// skipped where YouTube offers it, never cut short.
const AD_LIMIT: Duration = Duration::from_secs(15 * 60);

/// Opens `url` in the video window (made the first time) and waits past any ad
/// until the video can be read.
pub fn open(app: &AppHandle, url: &str) -> Result<Value, String> {
    let url = video::watch_url(url)?;
    let target: tauri::Url = url.parse().map_err(|e| format!("{url}: {e}"))?;
    let window = if let Some(window) = app.get_webview_window(LABEL) {
        window.navigate(target).map_err(|e| e.to_string())?;
        window
    } else {
        let mut builder = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::External(target))
            .title("動画 - Timeline Arona")
            .inner_size(1280.0, 760.0)
            .initialization_script(HELPER);
        // Windows of one app share one browser, which needs the same arguments.
        if let Some(args) = crate::browser_args(app) {
            builder = builder.additional_browser_args(&args);
        }
        builder.build().map_err(|e| e.to_string())?
    };
    let id = url.rsplit('=').next().unwrap_or_default().to_owned();
    let info = wait(
        &window,
        "window.__arona ? window.__arona.info() : null",
        Duration::from_secs(120),
        "動画が読めるようになるの",
        // The length can be missing for a moment while the player swaps in the video.
        |v| {
            v["ready"] == true
                && v["duration"].is_number()
                && v["page"].as_str().is_some_and(|p| p.contains(&id))
        },
    )?;
    Ok(json!({
        "url": url,
        "title": info["title"],
        "channel": info["channel"],
        "duration": info["duration"],
        "description": info["description"],
    }))
}

/// Takes the pictures `video_frames` asks for (see `mcp::tool_list`).
pub fn frames(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let seconds = |key: &str| {
        args[key]
            .as_f64()
            .ok_or_else(|| format!("{key} に秒数を入れてください"))
    };
    let times = video::frame_times(seconds("from")?, seconds("to")?, seconds("every")?)?;
    let crop: Crop = Crop::parse(args.get("crop"))?;
    let changes_only = args["changes_only"].as_bool().unwrap_or(false);
    let window = app
        .get_webview_window(LABEL)
        .ok_or("先に video_open で動画を開いてください")?;
    let state = wait(
        &window,
        "window.__arona ? window.__arona.info() : null",
        Duration::from_secs(60),
        "動画が読めるようになるの",
        |v| v["ready"] == true && v["duration"].is_number(),
    )
    .map_err(|e| format!("{e}。video_open で開き直してください"))?;
    let duration = state["duration"].as_f64().unwrap_or(f64::MAX);

    let dir = frame_dir()?;
    let crop = serde_json::to_string(&crop).map_err(|e| e.to_string())?;
    let (mut kept, mut dropped, mut last, mut size) = (Vec::new(), 0, None::<Vec<u8>>, Value::Null);
    for time in times.into_iter().take_while(|t| *t <= duration) {
        eval(&window, &format!("window.__arona.seek({time})"))?;
        wait(
            &window,
            "window.__arona.settled()",
            Duration::from_secs(60),
            &format!("{time} 秒へのシーク"),
            |v| *v == true,
        )?;
        let picture = eval(&window, &format!("window.__arona.grab({crop})"))?;
        if let Some(error) = picture["error"].as_str() {
            return Err(format!("{time} 秒の絵を取れません: {error}"));
        }
        let thumb: Vec<u8> =
            serde_json::from_value(picture["thumb"].clone()).map_err(|e| e.to_string())?;
        if changes_only && last.as_ref().is_some_and(|l| !video::differs(l, &thumb)) {
            dropped += 1;
            continue;
        }
        last = Some(thumb);
        let path = dir.join(format!("{time:09.3}.png"));
        let png = video::png_bytes(picture["png"].as_str().unwrap_or_default())?;
        std::fs::write(&path, png).map_err(|e| format!("{}: {e}", path.display()))?;
        size = json!({"width": picture["width"], "height": picture["height"]});
        kept.push(json!({"time": time, "path": path}));
    }
    Ok(json!({"dir": dir, "video_size": size, "frames": kept, "dropped": dropped}))
}

/// A new folder for one call's frames.
fn frame_dir() -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let dir = std::env::temp_dir()
        .join("timeline-arona")
        .join("frames")
        .join(stamp.to_string());
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(dir)
}

/// Runs `script` until `done` holds for its result, which it returns. Time an ad
/// shows (`video.js` says `"ad"` or `{ad: true}`) does not count, up to `AD_LIMIT`.
fn wait(
    window: &WebviewWindow,
    script: &str,
    timeout: Duration,
    what: &str,
    done: impl Fn(&Value) -> bool,
) -> Result<Value, String> {
    let start = Instant::now();
    let (mut counted, mut last) = (Duration::ZERO, start);
    loop {
        let value = eval(window, script)?;
        if done(&value) {
            return Ok(value);
        }
        let now = Instant::now();
        if value != "ad" && value["ad"] != true {
            counted += now - last;
        }
        last = now;
        if counted > timeout || start.elapsed() > timeout + AD_LIMIT {
            return Err(format!(
                "{what}を {} 秒待ちましたが終わりません（最後の状態: {value}）",
                start.elapsed().as_secs()
            ));
        }
        std::thread::sleep(POLL);
    }
}

/// The value of `script` in the page, which WebView2 hands back as JSON.
fn eval(window: &WebviewWindow, script: &str) -> Result<Value, String> {
    use webview2_com::ExecuteScriptCompletedHandler;
    use windows::core::HSTRING;

    let (tx, rx) = mpsc::channel::<Result<String, String>>();
    let script = HSTRING::from(script);
    window
        .with_webview(move |webview| {
            let failed = tx.clone();
            // SAFETY: with_webview runs this on the UI thread that owns the WebView2,
            // whose controller lives as long as the window; the handler owns its sender.
            let started = unsafe {
                webview.controller().CoreWebView2().and_then(|core| {
                    core.ExecuteScript(
                        &script,
                        &ExecuteScriptCompletedHandler::create(Box::new(move |error, json| {
                            let _ = tx.send(error.map(|()| json).map_err(|e| e.to_string()));
                            Ok(())
                        })),
                    )
                })
            };
            if let Err(error) = started {
                let _ = failed.send(Err(error.to_string()));
            }
        })
        .map_err(|e| e.to_string())?;
    let json = rx
        .recv_timeout(Duration::from_secs(30))
        .map_err(|_| "動画の窓が応答しません".to_owned())??;
    serde_json::from_str(&json).map_err(|e| format!("動画の窓の答えが読めません: {e}"))
}
