use std::process::Command;

use crate::protocol::{Playback, Widget};

/// Snapshot of macOS now-playing + output volume.
pub fn poll_media() -> Widget {
    let volume = system_volume_percent();
    let (playback, title) = now_playing();
    Widget::Media {
        playback,
        volume,
        title,
    }
}

fn system_volume_percent() -> u8 {
    let output = Command::new("osascript")
        .args(["-e", "output volume of (get volume settings)"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout);
            s.trim().parse::<u8>().unwrap_or(0).min(100)
        }
        _ => 0,
    }
}

fn now_playing() -> (Playback, String) {
    if let Some(info) = app_now_playing("Spotify") {
        return info;
    }
    if let Some(info) = app_now_playing("Music") {
        return info;
    }
    (Playback::Stopped, String::new())
}

fn app_now_playing(app: &str) -> Option<(Playback, String)> {
    let script = format!(
        r#"tell application "{app}"
  if it is running then
    try
      set p to player state as string
      set t to name of current track
      return p & tab & t
    end try
  end if
end tell"#
    );
    let output = Command::new("osascript").args(["-e", &script]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let mut parts = text.splitn(2, '\t');
    let state = parts.next().unwrap_or("");
    let title = parts.next().unwrap_or("").trim().to_string();
    let playback = match state {
        "playing" => Playback::Playing,
        "paused" => Playback::Paused,
        _ => Playback::Stopped,
    };
    if playback == Playback::Stopped && title.is_empty() {
        return None;
    }
    Some((playback, title))
}
