use std::sync::Arc;
use std::thread;

use parking_lot::Mutex;
use serde::Deserialize;
use tiny_http::{Header, Method, Response, Server};

use crate::protocol::{IdeBuild, Widget};

const IDE_PORT: u16 = 17302;

#[derive(Debug, Clone, Default)]
pub struct IdeState {
    pub active: bool,
    pub errors: u8,
    pub warnings: u8,
    pub build: IdeBuild,
    pub tag: String,
    pub updated_at: Option<std::time::Instant>,
}

impl IdeState {
    pub fn as_widget(&self) -> Option<Widget> {
        if !self.active {
            return None;
        }
        // Expire after 30s without updates.
        if let Some(at) = self.updated_at {
            if at.elapsed() > std::time::Duration::from_secs(30) {
                return None;
            }
        }
        Some(Widget::Ide {
            errors: self.errors,
            warnings: self.warnings,
            build: self.build,
            tag: self.tag.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct IdePayload {
    #[serde(default)]
    errors: u8,
    #[serde(default)]
    warnings: u8,
    #[serde(default)]
    build: String,
    #[serde(default)]
    tag: String,
    #[serde(default)]
    active: Option<bool>,
}

pub fn spawn_ide_server(state: Arc<Mutex<IdeState>>) {
    thread::spawn(move || {
        let server = match Server::http(format!("127.0.0.1:{IDE_PORT}")) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("IDE status server failed to bind :{IDE_PORT}: {e}");
                return;
            }
        };
        log::info!("IDE status listening on http://127.0.0.1:{IDE_PORT}/ide");

        for mut request in server.incoming_requests() {
            let url = request.url().to_string();
            let method = request.method().clone();

            if method == Method::Get && (url == "/ide" || url.starts_with("/ide?")) {
                let snap = state.lock().clone();
                let body = serde_json::json!({
                    "active": snap.active,
                    "errors": snap.errors,
                    "warnings": snap.warnings,
                    "build": match snap.build {
                        IdeBuild::Idle => "idle",
                        IdeBuild::Building => "building",
                        IdeBuild::Ok => "ok",
                        IdeBuild::Fail => "fail",
                    },
                    "tag": snap.tag,
                })
                .to_string();
                let _ = request.respond(
                    Response::from_string(body)
                        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap()),
                );
                continue;
            }

            if method == Method::Post && url.starts_with("/ide") {
                let mut body = String::new();
                if request.as_reader().read_to_string(&mut body).is_err() {
                    let _ = request.respond(Response::from_string("bad body").with_status_code(400));
                    continue;
                }
                match serde_json::from_str::<IdePayload>(&body) {
                    Ok(payload) => {
                        let mut guard = state.lock();
                        guard.active = payload.active.unwrap_or(true);
                        guard.errors = payload.errors;
                        guard.warnings = payload.warnings;
                        guard.tag = payload.tag.chars().take(4).collect();
                        guard.build = match payload.build.to_ascii_lowercase().as_str() {
                            "building" | "bld" => IdeBuild::Building,
                            "ok" | "success" => IdeBuild::Ok,
                            "fail" | "failed" | "error" => IdeBuild::Fail,
                            _ => IdeBuild::Idle,
                        };
                        guard.updated_at = Some(std::time::Instant::now());
                        let _ = request.respond(Response::from_string("ok"));
                    }
                    Err(e) => {
                        let _ = request
                            .respond(Response::from_string(format!("bad json: {e}")).with_status_code(400));
                    }
                }
                continue;
            }

            let _ = request.respond(Response::from_string("not found").with_status_code(404));
        }
    });
}
