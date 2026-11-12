//! Streamable-HTTP MCP server embedded in the companion tray app.
//!
//! Listens on `127.0.0.1:17303/mcp` (override with `PANDAKB_MCP_PORT`).
//! Requires the companion to be running — BB/Cursor connect via URL, not stdio.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
        session::local::LocalSessionManager,
    },
};
use serde::Deserialize;
use tauri::AppHandle;

use crate::companion::{parse_layer_arg, Companion};
use crate::firmware::{
    firmware_flash, firmware_keymap_get, firmware_keymap_set, firmware_status_of,
    firmware_wait_bootloader, run_firmware_build, run_firmware_west_update, FirmwareBusy,
};
use crate::protocol;

pub const DEFAULT_MCP_PORT: u16 = 17303;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct WestRootArgs {
    /// Optional west workspace path (defaults to companion resolution).
    #[serde(default)]
    west_root: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct HalfArgs {
    /// `left` or `right`.
    half: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct LayerArgs {
    /// Layer name (`base`, `lower`, `raise`, `adjust`, `wow`, `lol`) or index `0`–`5`.
    layer: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FlashHalfArgs {
    /// `left` or `right`.
    half: String,
    #[serde(default)]
    west_root: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BuildArgs {
    #[serde(default)]
    west_root: Option<String>,
    /// Run `west update` before building.
    #[serde(default)]
    update_first: Option<bool>,
    /// Rebuild even when UF2s look up to date.
    #[serde(default)]
    force: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct WaitBootloaderArgs {
    /// Seconds to wait (default 60).
    #[serde(default)]
    timeout_secs: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct KeymapSetArgs {
    /// Full contents of `config/Lily58.keymap`.
    contents: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FlashKeyboardArgs {
    #[serde(default)]
    west_root: Option<String>,
    #[serde(default)]
    update_first: Option<bool>,
    #[serde(default)]
    force: Option<bool>,
    /// Skip west build and only soft-enter + flash existing UF2s.
    #[serde(default)]
    skip_build: Option<bool>,
}

#[derive(Clone)]
pub struct PandaKbMcp {
    app: AppHandle,
    companion: Companion,
    busy: FirmwareBusy,
    #[allow(dead_code)] // retained for #[tool_handler] routing
    tool_router: ToolRouter<Self>,
}

fn json_ok(value: impl serde::Serialize) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

fn fw_err(e: impl ToString) -> McpError {
    McpError::internal_error(e.to_string(), None)
}

fn enter_bootloader(companion: &Companion, half: &str) -> Result<(), String> {
    let code = match half {
        "left" | "self" => protocol::BOOTLOADER_LEFT,
        "right" => protocol::BOOTLOADER_RIGHT,
        _ => return Err("half must be left or right".into()),
    };
    companion.hid.enter_bootloader(code)
}

#[tool_router]
impl PandaKbMcp {
    pub fn new(app: AppHandle, companion: Companion, busy: FirmwareBusy) -> Self {
        Self {
            app,
            companion,
            busy,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Companion HID/OLED status: connected, active widget, mode, last error, preview layer"
    )]
    fn companion_status(&self) -> Result<CallToolResult, McpError> {
        json_ok(self.companion.snapshot.lock().clone())
    }

    #[tool(description = "Force-push the idle OLED widget over Raw HID")]
    fn companion_push_idle(&self) -> Result<CallToolResult, McpError> {
        self.companion
            .hid
            .push_if_changed(&crate::protocol::Widget::Idle)
            .map_err(fw_err)?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    #[tool(
        description = "Soft-enter UF2 bootloader on left (USB central) or right (peripheral via split). Needs Raw HID 0x30 firmware."
    )]
    fn companion_enter_bootloader(
        &self,
        Parameters(args): Parameters<HalfArgs>,
    ) -> Result<CallToolResult, McpError> {
        enter_bootloader(&self.companion, &args.half).map_err(fw_err)?;
        json_ok(serde_json::json!({ "ok": true, "half": args.half }))
    }

    #[tool(
        description = "Sticky layer switch on the keyboard (Raw HID 0x31, same as &to). layer: base|lower|raise|adjust|wow|lol or 0–5. Needs firmware with SET_LAYER."
    )]
    fn companion_set_layer(
        &self,
        Parameters(args): Parameters<LayerArgs>,
    ) -> Result<CallToolResult, McpError> {
        let index = parse_layer_arg(&args.layer).map_err(fw_err)?;
        self.companion.hid.set_layer(index).map_err(fw_err)?;
        json_ok(serde_json::json!({
            "ok": true,
            "layer": args.layer,
            "index": index,
        }))
    }

    #[tool(
        description = "Firmware/west status: west root, UF2 paths, bootloader mount, sources stale, busy"
    )]
    fn firmware_status(
        &self,
        Parameters(args): Parameters<WestRootArgs>,
    ) -> Result<CallToolResult, McpError> {
        json_ok(firmware_status_of(args.west_root, &self.busy))
    }

    #[tool(description = "Run west update in the companion west workspace (syncs repo first)")]
    async fn firmware_west_update(
        &self,
        Parameters(args): Parameters<WestRootArgs>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let busy = self.busy.clone();
        let west_root = args.west_root;
        tokio::task::spawn_blocking(move || run_firmware_west_update(&app, &busy, west_root))
            .await
            .map_err(|e| fw_err(e))?
            .map_err(fw_err)?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    #[tool(
        description = "Build left+right companion OLED UF2s (west or Docker). Skips rebuild when sources are unchanged unless force=true."
    )]
    async fn firmware_build(
        &self,
        Parameters(args): Parameters<BuildArgs>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let busy = self.busy.clone();
        let west_root = args.west_root;
        let update_first = args.update_first.unwrap_or(false);
        let force = args.force;
        tokio::task::spawn_blocking(move || {
            run_firmware_build(&app, &busy, west_root, update_first, force)
        })
        .await
        .map_err(|e| fw_err(e))?
        .map_err(fw_err)?;
        json_ok(serde_json::json!({ "ok": true }))
    }

    #[tool(
        description = "Copy one half's UF2 to the mounted bootloader volume (NICENANO / RPI-RP2). Soft-enter or double-reset first."
    )]
    fn firmware_flash(
        &self,
        Parameters(args): Parameters<FlashHalfArgs>,
    ) -> Result<CallToolResult, McpError> {
        json_ok(firmware_flash(args.half, args.west_root).map_err(fw_err)?)
    }

    #[tool(description = "Poll until a UF2 bootloader volume appears (or timeout)")]
    async fn firmware_wait_bootloader(
        &self,
        Parameters(args): Parameters<WaitBootloaderArgs>,
    ) -> Result<CallToolResult, McpError> {
        let timeout = args.timeout_secs;
        let mounted = tokio::task::spawn_blocking(move || firmware_wait_bootloader(timeout))
            .await
            .map_err(|e| fw_err(e))?
            .map_err(fw_err)?;
        json_ok(serde_json::json!({ "mounted": mounted }))
    }

    #[tool(description = "Read config/Lily58.keymap (source of truth for layers / layouts)")]
    fn firmware_keymap_get(&self) -> Result<CallToolResult, McpError> {
        json_ok(firmware_keymap_get().map_err(fw_err)?)
    }

    #[tool(
        description = "Write config/Lily58.keymap. Flash afterward for the keyboard to pick it up."
    )]
    fn firmware_keymap_set(
        &self,
        Parameters(args): Parameters<KeymapSetArgs>,
    ) -> Result<CallToolResult, McpError> {
        json_ok(firmware_keymap_set(args.contents).map_err(fw_err)?)
    }

    #[tool(
        description = "Full flash flow: build (unless skip_build), then soft-enter + flash left, then right. Returns wizard hint if a half needs a cable or double-reset."
    )]
    async fn flash_keyboard(
        &self,
        Parameters(args): Parameters<FlashKeyboardArgs>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let busy = self.busy.clone();
        let companion = self.companion.clone();
        let west_root = args.west_root.clone();
        let update_first = args.update_first.unwrap_or(false);
        let force = args.force;
        let skip_build = args.skip_build.unwrap_or(false);

        let result = tokio::task::spawn_blocking(move || {
            if !skip_build {
                run_firmware_build(&app, &busy, west_root.clone(), update_first, force)?;
            }

            let mut flashed = Vec::new();
            for half in ["left", "right"] {
                match enter_bootloader(&companion, half) {
                    Ok(()) => {}
                    Err(e) => {
                        log::warn!("soft-enter {half}: {e}");
                    }
                }

                let mounted = firmware_wait_bootloader(Some(45))?;
                if !mounted {
                    return Ok(serde_json::json!({
                        "ok": false,
                        "wizard": "bootloader",
                        "half": half,
                        "flashed": flashed,
                        "message": format!(
                            "Bootloader not mounted for {half}. Plug USB (or double-reset), then retry flash_keyboard with skip_build=true or firmware_flash."
                        ),
                    }));
                }

                let flash = firmware_flash(half.to_string(), west_root.clone())?;
                flashed.push(serde_json::json!({
                    "half": flash.half,
                    "from": flash.from,
                    "to": flash.to,
                }));
                std::thread::sleep(Duration::from_millis(1200));
            }

            Ok::<_, crate::firmware::FirmwareError>(serde_json::json!({
                "ok": true,
                "flashed": flashed,
            }))
        })
        .await
        .map_err(|e| fw_err(e))?
        .map_err(fw_err)?;

        json_ok(result)
    }
}

#[tool_handler]
impl ServerHandler for PandaKbMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("pandakb-companion", env!("CARGO_PKG_VERSION"))
                    .with_title("PandaKB Companion")
                    .with_description(
                        "Raw HID OLED companion — keymap, west build, and UF2 flash tools",
                    ),
            )
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "PandaKB companion MCP: inspect HID/OLED status, sticky companion_set_layer (wow/lol/base/…), edit Lily58.keymap, west build, soft-enter bootloader, and flash both halves. The tray app must be running. Prefer flash_keyboard for end-to-end flashing; use firmware_keymap_set then flash_keyboard after keymap edits. SET_LAYER needs left firmware with 0x31.",
            )
    }
}

fn mcp_port() -> u16 {
    std::env::var("PANDAKB_MCP_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MCP_PORT)
}

/// Bind streamable-HTTP MCP on localhost. Failures are logged; the tray app still runs.
pub fn spawn_mcp_server(app: AppHandle, companion: Companion, busy: FirmwareBusy) {
    let port = mcp_port();
    let state = Arc::new(PandaKbMcp::new(app, companion, busy));

    tauri::async_runtime::spawn(async move {
        let service = StreamableHttpService::new(
            {
                let state = state.clone();
                move || Ok(state.as_ref().clone())
            },
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig::default()
                .with_legacy_session_mode(false)
                .with_json_response(true),
        );

        let router = axum::Router::new()
            .route(
                "/",
                axum::routing::get(|| async {
                    (
                        [(
                            axum::http::header::CONTENT_TYPE,
                            "application/json",
                        )],
                        serde_json::json!({
                            "service": "pandakb-companion-mcp",
                            "mcp": "/mcp",
                            "tools": true,
                        })
                        .to_string(),
                    )
                }),
            )
            .nest_service("/mcp", service);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                log::warn!("MCP server failed to bind {addr}: {e}");
                return;
            }
        };
        log::info!("MCP listening on http://127.0.0.1:{port}/mcp");
        if let Err(e) = axum::serve(listener, router).await {
            log::warn!("MCP server stopped: {e}");
        }
    });
}
