//! The plain computation behind the video tools: which URLs are YouTube videos,
//! which times to take, the part of the picture to keep, and whether two pictures
//! differ. `browser.rs` drives the page with `video.js`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The most frames one `video_frames` call takes.
pub const MAX_FRAMES: u32 = 600;

/// The canonical watch URL of a YouTube video URL (`youtube.com/watch?v=`,
/// `youtu.be/`), without the start time or playlist.
pub fn watch_url(text: &str) -> Result<String, String> {
    let refused = || {
        format!("YouTube の動画の URL（youtube.com/watch?v=… か youtu.be/…）ではありません: {text}")
    };
    let rest = text
        .trim()
        .strip_prefix("https://")
        .or_else(|| text.trim().strip_prefix("http://"))
        .ok_or_else(refused)?;
    let (host, path) = rest.split_once('/').ok_or_else(refused)?;
    let (path, query) = path.split_once('?').unwrap_or((path, ""));
    let id = match host {
        "youtu.be" => path,
        "www.youtube.com" | "youtube.com" | "m.youtube.com" if path == "watch" => query
            .split('&')
            .find_map(|pair| pair.strip_prefix("v="))
            .unwrap_or_default(),
        _ => "",
    };
    let valid = id.len() == 11
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    if valid {
        Ok(format!("https://www.youtube.com/watch?v={id}"))
    } else {
        Err(refused())
    }
}

/// The times from `from` to `to` (seconds, both included) every `every` seconds.
pub fn frame_times(from: f64, to: f64, every: f64) -> Result<Vec<f64>, String> {
    if !(from >= 0.0 && to >= from && every > 0.0) {
        return Err(format!(
            "from は 0 以上、to は from 以上、every は 0 より大きくしてください（from {from}、to {to}、every {every}）"
        ));
    }
    // A thousandth of a second keeps 1.0 + 0.4 × 2 from missing 1.8 by rounding.
    let count = ((to - from) / every + 1e-3).floor() + 1.0;
    if count > f64::from(MAX_FRAMES) {
        return Err(format!(
            "一度に取れるのは {MAX_FRAMES} 枚までです（{count} 枚）。範囲を分けるか every を大きくしてください"
        ));
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "count is a whole number from 1 to MAX_FRAMES"
    )]
    let count = count as u32;
    Ok((0..count)
        .map(|i| ((from + every * f64::from(i)) * 1000.0).round() / 1000.0)
        .collect())
}

/// A part of the picture, as fractions of its width and height.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Crop {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Crop {
    pub const WHOLE: Crop = Crop {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };

    /// The crop a tool was given; the whole picture when none.
    pub fn parse(value: Option<&Value>) -> Result<Crop, String> {
        let Some(value) = value else {
            return Ok(Crop::WHOLE);
        };
        let crop = Crop::deserialize(value)
            .map_err(|e| format!("crop は {{x, y, w, h}}（画面に対する 0〜1 の割合）です: {e}"))?;
        let inside =
            |start: f64, size: f64| start >= 0.0 && size > 0.0 && start + size <= 1.0 + 1e-9;
        if inside(crop.x, crop.w) && inside(crop.y, crop.h) {
            Ok(crop)
        } else {
            Err(format!(
                "crop が画面からはみ出しています（x {}、y {}、w {}、h {}）",
                crop.x, crop.y, crop.w, crop.h
            ))
        }
    }
}

/// The PNG in a `data:image/png;base64,` URL, as the page's canvas gives it.
pub fn png_bytes(data_url: &str) -> Result<Vec<u8>, String> {
    use base64::Engine as _;
    let data = data_url
        .strip_prefix("data:image/png;base64,")
        .ok_or("the page gave no PNG")?;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("the page's PNG is broken: {e}"))
}

/// How much a grey pixel has to change to count (of 255).
const CHANGED_BY: u8 = 48;
/// More than one pixel in this many has to change (2%).
const CHANGED_ONE_IN: usize = 50;

/// Whether two grey thumbnails of the same part of the picture show something else:
/// enough pixels changed by much, as text appearing or going does, unlike the
/// game moving faintly behind a dark panel.
pub fn differs(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return true;
    }
    let changed = a
        .iter()
        .zip(b)
        .filter(|(p, q)| p.abs_diff(**q) >= CHANGED_BY)
        .count();
    changed * CHANGED_ONE_IN > a.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn youtube_urls_become_the_watch_url() {
        let canonical = "https://www.youtube.com/watch?v=n75QlmuJcnM";
        for url in [
            "https://www.youtube.com/watch?v=n75QlmuJcnM&t=5s",
            "https://youtube.com/watch?v=n75QlmuJcnM",
            "https://m.youtube.com/watch?feature=share&v=n75QlmuJcnM",
            "http://www.youtube.com/watch?v=n75QlmuJcnM",
            "https://youtu.be/n75QlmuJcnM?t=5",
            " https://youtu.be/n75QlmuJcnM ",
        ] {
            assert_eq!(watch_url(url).as_deref(), Ok(canonical), "{url}");
        }
    }

    #[test]
    fn other_urls_are_refused() {
        for url in [
            "https://www.youtube.com/",
            "https://www.youtube.com/watch?v=short",
            "https://www.youtube.com/watch?v=n75QlmuJcn!",
            "https://example.com/watch?v=n75QlmuJcnM",
            "https://youtube.com.example.com/watch?v=n75QlmuJcnM",
            "https://youtu.be/",
            "file:///C:/video.mp4",
            "n75QlmuJcnM",
        ] {
            assert!(watch_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn frames_are_taken_every_so_many_seconds_both_ends_included() {
        assert_eq!(frame_times(0.0, 10.0, 5.0), Ok(vec![0.0, 5.0, 10.0]));
        assert_eq!(frame_times(1.0, 2.0, 0.4), Ok(vec![1.0, 1.4, 1.8]));
        assert_eq!(frame_times(3.0, 3.0, 1.0), Ok(vec![3.0]));
    }

    #[test]
    fn frame_times_out_of_reason_are_refused() {
        for (from, to, every) in [
            (-1.0, 3.0, 1.0),
            (5.0, 3.0, 1.0),
            (0.0, 3.0, 0.0),
            (0.0, 3.0, f64::NAN),
            (0.0, 600.0, 1.0),
        ] {
            assert!(frame_times(from, to, every).is_err(), "{from} {to} {every}");
        }
        assert_eq!(
            frame_times(0.0, 599.0, 1.0).map(|t| t.len()),
            Ok(MAX_FRAMES as usize)
        );
    }

    #[test]
    fn a_crop_is_a_part_inside_the_picture() {
        assert_eq!(Crop::parse(None), Ok(Crop::WHOLE));
        assert_eq!(
            Crop::parse(Some(&json!({"x": 0.0, "y": 0.1, "w": 0.4, "h": 0.9}))),
            Ok(Crop {
                x: 0.0,
                y: 0.1,
                w: 0.4,
                h: 0.9
            })
        );
        for bad in [
            json!({"x": 0.7, "y": 0.0, "w": 0.4, "h": 1.0}),
            json!({"x": 0.0, "y": 0.0, "w": 0.0, "h": 1.0}),
            json!({"x": -0.1, "y": 0.0, "w": 0.5, "h": 0.5}),
            json!({"x": 0.0, "y": 0.0, "w": 0.5}),
            json!("left"),
        ] {
            assert!(Crop::parse(Some(&bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn pictures_differ_when_enough_pixels_change_by_much() {
        let still = vec![40u8; 64 * 36];
        assert!(!differs(&still, &still));

        let mut faint = still.clone();
        for p in &mut faint {
            *p += 12;
        }
        assert!(!differs(&still, &faint), "the game moving behind a panel");

        let mut speck = still.clone();
        speck[..10].fill(255);
        assert!(!differs(&still, &speck), "a few pixels");

        let mut line = still.clone();
        line[64 * 10..64 * 12].fill(255);
        assert!(differs(&still, &line), "a new line of text");

        assert!(differs(&still, &still[1..]), "pictures of other sizes");
    }

    #[test]
    fn a_png_data_url_gives_the_png() {
        assert_eq!(
            png_bytes("data:image/png;base64,iVBORw0KGgo="),
            Ok(b"\x89PNG\r\n\x1a\n".to_vec())
        );
        for bad in [
            "data:image/jpeg;base64,iVBORw0KGgo=",
            "data:image/png;base64,!!",
            "data:,",
            "",
        ] {
            assert!(png_bytes(bad).is_err(), "{bad}");
        }
    }
}
