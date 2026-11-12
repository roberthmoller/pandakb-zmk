use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::hid::HidClient;
use crate::ide::{spawn_ide_server, IdeState};
use crate::lol::{insecure_lol_client, poll_lol};
use crate::media::{poll_media, system_volume_percent};
use crate::protocol::{
    self, format_battery, layer_menu_name, BoardStatus, InboundReport, KeyMatrix, LayerPreview,
    Playback, Widget, BATTERY_UNKNOWN,
};

/// How long the OLED keeps a full-body volume HUD after a change.
const VOLUME_FLASH_MS: u64 = 1800;

const LAYER_PREVIEW_WINDOW: &str = "layer-preview";

#[derive(Clone)]
pub struct Companion {
    pub hid: Arc<HidClient>,
    #[allow(dead_code)]
    pub ide: Arc<Mutex<IdeState>>,
    pub snapshot: Arc<Mutex<CompanionSnapshot>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompanionSnapshot {
    pub connected: bool,
    pub widget: String,
    pub mode: u8,
    pub last_error: Option<String>,
    pub preview_layer: Option<u8>,
    /// Left (central) SoC 0–100, or 0xFF unknown.
    pub left_battery: u8,
    /// Right (peripheral) SoC 0–100, or 0xFF unknown.
    pub right_battery: u8,
    /// Highest active layer from STATUS (0x22); defaults to Base until reported.
    pub active_layer: u8,
}

impl Default for CompanionSnapshot {
    fn default() -> Self {
        Self {
            connected: false,
            widget: "idle".into(),
            mode: 0,
            last_error: None,
            preview_layer: None,
            left_battery: BATTERY_UNKNOWN,
            right_battery: BATTERY_UNKNOWN,
            active_layer: protocol::LAYER_BASE,
        }
    }
}

fn tray_title(connected: bool, snap: &CompanionSnapshot) -> String {
    if !connected {
        return "…".into();
    }
    let layer = snap
        .preview_layer
        .map(layer_menu_name)
        .unwrap_or_else(|| layer_menu_name(snap.active_layer));
    format!(
        "L{} R{} · {}",
        format_battery(snap.left_battery),
        format_battery(snap.right_battery),
        layer
    )
}

fn refresh_tray(app: &AppHandle, snap: &CompanionSnapshot) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(tray_title(snap.connected, snap)));
    }
}

impl Companion {
    pub fn start(app: AppHandle) -> Result<Self, String> {
        let hid = Arc::new(HidClient::new()?);
        let ide = Arc::new(Mutex::new(IdeState::default()));
        spawn_ide_server(ide.clone());

        let snapshot = Arc::new(Mutex::new(CompanionSnapshot::default()));
        let state = Self {
            hid: hid.clone(),
            ide: ide.clone(),
            snapshot: snapshot.clone(),
        };

        let lol_client = insecure_lol_client();
        let widget_app = app.clone();
        let widget_hid = hid.clone();
        let widget_snap = snapshot.clone();
        tauri::async_runtime::spawn(async move {
            // 500ms keeps volume HUD responsive; LoL/IDE still refresh on this cadence.
            let mut ticker = tokio::time::interval(Duration::from_millis(500));
            let mut last_volume: Option<u8> = None;
            let mut volume_flash_until: Option<Instant> = None;
            loop {
                ticker.tick().await;

                let volume = system_volume_percent();
                let now = Instant::now();
                if let Some(prev) = last_volume {
                    if prev != volume {
                        volume_flash_until = Some(now + Duration::from_millis(VOLUME_FLASH_MS));
                    }
                }
                last_volume = Some(volume);

                let flashing = volume_flash_until
                    .map(|until| now < until)
                    .unwrap_or(false);
                if !flashing {
                    volume_flash_until = None;
                }

                let widget = select_widget(&lol_client, &ide, flashing, volume).await;
                let was_connected = widget_snap.lock().connected;
                let connected = widget_hid.ensure_connected();
                if connected && !was_connected {
                    let _ = widget_hid.request_status();
                }
                let push = widget_hid.push_if_changed(&widget);
                let mut snap = widget_snap.lock();
                snap.connected = connected;
                if !connected {
                    snap.left_battery = BATTERY_UNKNOWN;
                    snap.right_battery = BATTERY_UNKNOWN;
                }
                snap.widget = widget.summary();
                snap.mode = widget.mode();
                snap.last_error = match &push {
                    Ok(_) => None,
                    Err(e) => Some(e.clone()),
                };
                let emit = snap.clone();
                drop(snap);
                let _ = widget_app.emit("companion://status", &emit);
                refresh_tray(&widget_app, &emit);
            }
        });

        let preview_app = app.clone();
        let preview_hid = hid.clone();
        let preview_snap = snapshot.clone();
        tauri::async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(25));
            loop {
                ticker.tick().await;
                for report in preview_hid.poll_inbound() {
                    match report {
                        InboundReport::LayerPreview(preview) => {
                            apply_layer_preview(&preview_app, &preview_snap, preview);
                        }
                        InboundReport::KeyMatrix(matrix) => {
                            apply_key_matrix(&preview_app, matrix);
                        }
                        InboundReport::Status(status) => {
                            apply_board_status(&preview_app, &preview_snap, status);
                        }
                    }
                }
            }
        });

        Ok(state)
    }
}

fn apply_layer_preview(
    app: &AppHandle,
    snapshot: &Arc<Mutex<CompanionSnapshot>>,
    preview: LayerPreview,
) {
    let emit = {
        let mut snap = snapshot.lock();
        snap.preview_layer = if preview.show {
            Some(preview.layer)
        } else {
            None
        };
        snap.clone()
    };

    let _ = app.emit("companion://layer-preview", &preview);
    refresh_tray(app, &emit);

    if let Some(win) = app.get_webview_window(LAYER_PREVIEW_WINDOW) {
        if preview.show {
            crate::overlay::present(&win);
        } else {
            crate::overlay::dismiss(&win);
        }
    }
}

fn apply_key_matrix(app: &AppHandle, matrix: KeyMatrix) {
    let _ = app.emit("companion://key-matrix", &matrix);
}

fn apply_board_status(
    app: &AppHandle,
    snapshot: &Arc<Mutex<CompanionSnapshot>>,
    status: BoardStatus,
) {
    let emit = {
        let mut snap = snapshot.lock();
        snap.left_battery = status.left_battery;
        snap.right_battery = status.right_battery;
        snap.active_layer = status.layer;
        snap.clone()
    };
    let _ = app.emit("companion://status", &emit);
    refresh_tray(app, &emit);
}

async fn select_widget(
    lol_client: &reqwest::Client,
    ide: &Arc<Mutex<IdeState>>,
    volume_flash: bool,
    volume: u8,
) -> Widget {
    // Volume change briefly forces MEDIA so the OLED can show the full-body HUD
    // even while Idle / LoL / IDE would normally win.
    if volume_flash {
        let media = poll_media();
        return match media {
            Widget::Media {
                playback, title, ..
            } => Widget::Media {
                playback,
                volume,
                title,
            },
            other => other,
        };
    }

    if let Some(lol) = poll_lol(lol_client).await {
        return lol;
    }
    if let Some(ide_w) = ide.lock().as_widget() {
        return ide_w;
    }
    match poll_media() {
        Widget::Media {
            playback: Playback::Stopped,
            title,
            ..
        } if title.is_empty() => Widget::Idle,
        Widget::Media {
            playback, title, ..
        } => Widget::Media {
            playback,
            volume,
            title,
        },
        other => other,
    }
}

#[tauri::command]
pub fn companion_status(state: tauri::State<'_, Companion>) -> CompanionSnapshot {
    state.snapshot.lock().clone()
}

#[tauri::command]
pub fn companion_push_idle(state: tauri::State<'_, Companion>) -> Result<(), String> {
    state.hid.push_if_changed(&Widget::Idle).map(|_| ())
}

/// Soft-enter UF2 bootloader on left (central) or right (peripheral via split).
#[tauri::command]
pub fn companion_enter_bootloader(
    state: tauri::State<'_, Companion>,
    half: String,
) -> Result<(), String> {
    let code = match half.as_str() {
        "left" | "self" => protocol::BOOTLOADER_LEFT,
        "right" => protocol::BOOTLOADER_RIGHT,
        _ => return Err("half must be left or right".into()),
    };
    state.hid.enter_bootloader(code)
}

/// Resolve a layer name or index string to the protocol layer index.
pub fn parse_layer_arg(layer: &str) -> Result<u8, String> {
    let t = layer.trim();
    if let Ok(n) = t.parse::<u8>() {
        if n <= protocol::LAYER_LOL {
            return Ok(n);
        }
        return Err("layer index must be 0–5".into());
    }
    match t.to_ascii_lowercase().as_str() {
        "base" | "default" => Ok(protocol::LAYER_BASE),
        "lower" => Ok(protocol::LAYER_LOWER),
        "raise" => Ok(protocol::LAYER_RAISE),
        "adjust" => Ok(protocol::LAYER_ADJUST),
        "wow" => Ok(protocol::LAYER_WOW),
        "lol" => Ok(protocol::LAYER_LOL),
        _ => Err("layer must be base|lower|raise|adjust|wow|lol or 0–5".into()),
    }
}

/// Sticky `&to` layer switch over Raw HID (`0x31`). Needs firmware with SET_LAYER.
#[tauri::command]
pub fn companion_set_layer(
    state: tauri::State<'_, Companion>,
    layer: String,
) -> Result<u8, String> {
    let index = parse_layer_arg(&layer)?;
    state.hid.set_layer(index)?;
    Ok(index)
}
