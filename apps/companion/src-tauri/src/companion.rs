use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::hid::HidClient;
use crate::ide::{spawn_ide_server, IdeState};
use crate::lol::{insecure_lol_client, poll_lol};
use crate::media::poll_media;
use crate::protocol::{LayerPreview, Playback, Widget};

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
}

impl Default for CompanionSnapshot {
    fn default() -> Self {
        Self {
            connected: false,
            widget: "idle".into(),
            mode: 0,
            last_error: None,
            preview_layer: None,
        }
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
            let mut ticker = tokio::time::interval(Duration::from_secs(2));
            loop {
                ticker.tick().await;
                let widget = select_widget(&lol_client, &ide).await;
                let connected = widget_hid.ensure_connected();
                let push = widget_hid.push_if_changed(&widget);
                let mut snap = widget_snap.lock();
                snap.connected = connected;
                snap.widget = widget.summary();
                snap.mode = widget.mode();
                snap.last_error = match &push {
                    Ok(_) => None,
                    Err(e) => Some(e.clone()),
                };
                let emit = snap.clone();
                drop(snap);
                let _ = widget_app.emit("companion://status", &emit);
                if let Some(tray) = widget_app.tray_by_id("main") {
                    let title = format!(
                        "KB {}",
                        if connected {
                            emit.widget.chars().take(18).collect::<String>()
                        } else {
                            "…".into()
                        }
                    );
                    let _ = tray.set_title(Some(title));
                }
            }
        });

        let preview_app = app.clone();
        let preview_hid = hid.clone();
        let preview_snap = snapshot.clone();
        tauri::async_runtime::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_millis(25));
            loop {
                ticker.tick().await;
                if let Some(preview) = preview_hid.poll_layer_preview() {
                    apply_layer_preview(&preview_app, &preview_snap, preview);
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
    {
        let mut snap = snapshot.lock();
        snap.preview_layer = if preview.show {
            Some(preview.layer)
        } else {
            None
        };
    }

    let _ = app.emit("companion://layer-preview", &preview);

    if let Some(win) = app.get_webview_window(LAYER_PREVIEW_WINDOW) {
        if preview.show {
            crate::overlay::present(&win);
        } else {
            crate::overlay::dismiss(&win);
        }
    }
}

async fn select_widget(lol_client: &reqwest::Client, ide: &Arc<Mutex<IdeState>>) -> Widget {
    if let Some(lol) = poll_lol(lol_client).await {
        return lol;
    }
    if let Some(ide_w) = ide.lock().as_widget() {
        return ide_w;
    }
    let media = poll_media();
    match &media {
        Widget::Media {
            playback: Playback::Stopped,
            title,
            ..
        } if title.is_empty() => Widget::Idle,
        _ => media,
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
