//! Local west build + UF2 flash for the companion OLED pair.
//!
//! Prefers `west` on PATH; otherwise runs `west` inside
//! `zmkfirmware/zmk-build-arm:stable` with the west tree mounted at `/work`.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;

const BOOTLOADER: &str = "/Volumes/RPI-RP2";
const DOCKER_IMAGE: &str = "zmkfirmware/zmk-build-arm:stable";
const DEFAULT_WEST_ROOT: &str =
    "/Users/robert/Documents/Codex/2026-11-05/referenced-chatgpt-conversation-this-is-an/work/pandakb-lily";

const LEFT_SHIELD: &str = "Lily58_L nice_oled raw_hid_adapter app_layer";
const RIGHT_SHIELD: &str = "Lily58_R game_badge";
const LEFT_CMAKE: &str = "-DCONFIG_ZMK_STUDIO=n -DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_CUSTOM=y -DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_BUILT_IN=n";
const RIGHT_CMAKE: &str = "-DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_CUSTOM=y -DCONFIG_ZMK_DISPLAY_STATUS_SCREEN_BUILT_IN=n";

const LEFT_BUILD_DIR: &str = "build/lily58_left_oled";
const RIGHT_BUILD_DIR: &str = "build/lily58_right_oled";

#[derive(Debug, Error)]
pub enum FirmwareError {
    #[error("{0}")]
    Msg(String),
}

impl Serialize for FirmwareError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

type FwResult<T> = std::result::Result<T, FirmwareError>;

fn err(msg: impl Into<String>) -> FirmwareError {
    FirmwareError::Msg(msg.into())
}

pub struct FirmwareBusy(pub Arc<AtomicBool>);

impl Default for FirmwareBusy {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareStatus {
    pub west_root: String,
    pub west_root_ok: bool,
    pub repo_root: Option<String>,
    pub west_bin: Option<String>,
    pub docker_ok: bool,
    pub bootloader_mounted: bool,
    pub left_uf2: Option<String>,
    pub right_uf2: Option<String>,
    pub busy: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlashResult {
    pub half: String,
    pub from: String,
    pub to: String,
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs_home() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn detect_repo_root() -> Option<PathBuf> {
    if let Ok(env) = std::env::var("PANDAKB_REPO") {
        let p = expand_home(&env);
        if is_repo_root(&p) {
            return p.canonicalize().ok();
        }
    }
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..8 {
        if let Ok(c) = dir.canonicalize() {
            if is_repo_root(&c) {
                return Some(c);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn is_repo_root(p: &Path) -> bool {
    p.join("config/west.yml").is_file()
        && p.join("modules/app-layer").is_dir()
        && p.join("boards/shields/app_layer").is_dir()
}

fn is_west_root(p: &Path) -> bool {
    p.join(".west").is_dir() && (p.join("zmk").is_dir() || p.join("zephyr").is_dir())
}

fn resolve_west_root(preferred: Option<String>) -> PathBuf {
    if let Some(p) = preferred {
        let t = p.trim();
        if !t.is_empty() {
            return expand_home(t);
        }
    }
    if let Ok(env) = std::env::var("PANDAKB_WEST_ROOT") {
        let t = env.trim();
        if !t.is_empty() {
            return expand_home(t);
        }
    }
    let default = PathBuf::from(DEFAULT_WEST_ROOT);
    if default.is_dir() {
        return default;
    }
    if let Some(repo) = detect_repo_root() {
        return repo;
    }
    dirs_home()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("pandakb-west")
}

fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn docker_ok() -> bool {
    Command::new("docker")
        .args(["image", "inspect", DOCKER_IMAGE])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn uf2_path(west: &Path, half: &str) -> PathBuf {
    let build = if half == "left" {
        LEFT_BUILD_DIR
    } else {
        RIGHT_BUILD_DIR
    };
    west.join(build).join("zephyr/zmk.uf2")
}

fn emit_log(app: &AppHandle, line: impl AsRef<str>) {
    let _ = app.emit("firmware://log", line.as_ref().to_string());
}

/// Sync this repo's firmware sources into the west workspace so shields/modules resolve.
fn sync_repo_into_west(app: &AppHandle, repo: &Path, west: &Path) -> FwResult<()> {
    emit_log(
        app,
        format!(
            "Syncing {} → {} (config, boards, modules/app-layer)",
            repo.display(),
            west.display()
        ),
    );

    copy_dir_contents(&repo.join("config"), &west.join("config"), app)?;
    copy_dir_contents(&repo.join("boards"), &west.join("boards"), app)?;

    let app_layer_src = repo.join("modules/app-layer");
    let app_layer_dst = west.join("modules/app-layer");
    if app_layer_src.is_dir() {
        if app_layer_dst.exists() {
            fs::remove_dir_all(&app_layer_dst).map_err(|e| err(e.to_string()))?;
        }
        copy_dir_recursive(&app_layer_src, &app_layer_dst).map_err(|e| err(e.to_string()))?;
        emit_log(app, "  synced modules/app-layer");
    }

    // Manifest must match this repo so west update pulls raw_hid / nice_oled / etc.
    let west_yml = repo.join("config/west.yml");
    if west_yml.is_file() {
        fs::copy(&west_yml, west.join("config/west.yml")).map_err(|e| err(e.to_string()))?;
    }

    // Zephyr discovers EXTRA_MODULES via <module>/zephyr/module.yml. Keep this
    // under modules/app-layer so we never touch the real zephyr/ checkout.
    let module_meta = west.join("modules/app-layer/zephyr");
    fs::create_dir_all(&module_meta).map_err(|e| err(e.to_string()))?;
    fs::write(
        module_meta.join("module.yml"),
        "name: pandakb-app-layer\n\
build:\n\
  cmake: .\n\
  kconfig: Kconfig\n\
  settings:\n\
    dts_root: .\n",
    )
    .map_err(|e| err(e.to_string()))?;

    Ok(())
}

fn copy_dir_contents(src: &Path, dst: &Path, app: &AppHandle) -> FwResult<()> {
    if !src.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|e| err(e.to_string()))?;
    for entry in fs::read_dir(src).map_err(|e| err(e.to_string()))? {
        let entry = entry.map_err(|e| err(e.to_string()))?;
        let name = entry.file_name();
        if name == ".DS_Store" {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        if from.is_dir() {
            if to.exists() {
                fs::remove_dir_all(&to).map_err(|e| err(e.to_string()))?;
            }
            copy_dir_recursive(&from, &to).map_err(|e| err(e.to_string()))?;
        } else {
            fs::copy(&from, &to).map_err(|e| err(e.to_string()))?;
        }
    }
    emit_log(
        app,
        format!("  synced {}/", src.file_name().unwrap_or_default().to_string_lossy()),
    );
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if entry.file_name() != ".DS_Store" {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

enum WestRunner {
    Native(PathBuf),
    Docker,
}

impl WestRunner {
    fn detect() -> FwResult<Self> {
        if let Some(w) = which("west") {
            return Ok(Self::Native(w));
        }
        if which("docker").is_some() && docker_ok() {
            return Ok(Self::Docker);
        }
        if which("docker").is_some() {
            return Err(err(format!(
                "west not on PATH; docker found but image {DOCKER_IMAGE} is missing — run: docker pull {DOCKER_IMAGE}"
            )));
        }
        Err(err(
            "Neither west nor docker is available. Install west, or: docker pull zmkfirmware/zmk-build-arm:stable",
        ))
    }

    fn run_stream(
        &self,
        app: &AppHandle,
        west_root: &Path,
        args: &[String],
        label: &str,
    ) -> FwResult<i32> {
        emit_log(app, format!("\n── {label} ──"));
        match self {
            Self::Native(west) => {
                emit_log(app, format!("$ west {}", args.join(" ")));
                let mut cmd = Command::new(west);
                cmd.args(args)
                    .current_dir(west_root)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                stream_command(app, &mut cmd)
            }
            Self::Docker => {
                let mount = format!("{}:/work", west_root.display());
                let mut docker_args: Vec<String> = vec![
                    "run".into(),
                    "--rm".into(),
                    "-v".into(),
                    mount,
                    "-w".into(),
                    "/work".into(),
                    DOCKER_IMAGE.into(),
                    "west".into(),
                ];
                docker_args.extend(args.iter().cloned());
                emit_log(app, format!("$ docker {}", docker_args.join(" ")));
                let mut cmd = Command::new("docker");
                cmd.args(&docker_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                stream_command(app, &mut cmd)
            }
        }
    }
}

fn stream_command(app: &AppHandle, cmd: &mut Command) -> FwResult<i32> {
    let mut child = cmd.spawn().map_err(|e| err(format!("spawn failed: {e}")))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let app_out = app.clone();
    let app_err = app.clone();
    let t_out = thread::spawn(move || {
        if let Some(out) = stdout {
            for line in BufReader::new(out).lines().flatten() {
                emit_log(&app_out, line);
            }
        }
    });
    let t_err = thread::spawn(move || {
        if let Some(err_pipe) = stderr {
            for line in BufReader::new(err_pipe).lines().flatten() {
                emit_log(&app_err, line);
            }
        }
    });
    let status = child.wait().map_err(|e| err(e.to_string()))?;
    let _ = t_out.join();
    let _ = t_err.join();
    Ok(status.code().unwrap_or(1))
}

fn build_args(half: &str) -> Vec<String> {
    let (dir, shield, cmake) = if half == "left" {
        (LEFT_BUILD_DIR, LEFT_SHIELD, LEFT_CMAKE)
    } else {
        (RIGHT_BUILD_DIR, RIGHT_SHIELD, RIGHT_CMAKE)
    };
    let mut args = vec![
        "build".into(),
        "-s".into(),
        "zmk/app".into(),
        "-d".into(),
        dir.into(),
        "-b".into(),
        "nice_nano_v2".into(),
        "--".into(),
        format!("-DSHIELD={shield}"),
        "-DZMK_CONFIG=/work/config".into(),
        "-DBOARD_ROOT=/work".into(),
        "-DZEPHYR_EXTRA_MODULES=/work/modules/app-layer".into(),
    ];
    // Native west: paths must be host paths — fix up below in caller if needed.
    for part in cmake.split_whitespace() {
        args.push(part.to_string());
    }
    args
}

fn build_args_for_runner(runner: &WestRunner, half: &str, west_root: &Path) -> Vec<String> {
    let mut args = build_args(half);
    if let WestRunner::Native(_) = runner {
        // Rewrite docker-style absolute /work paths to host west root.
        let root = west_root.display().to_string();
        for a in &mut args {
            if a.starts_with("-DZMK_CONFIG=") {
                *a = format!("-DZMK_CONFIG={root}/config");
            } else if a.starts_with("-DBOARD_ROOT=") {
                *a = format!("-DBOARD_ROOT={root}");
            } else if a.starts_with("-DZEPHYR_EXTRA_MODULES=") {
                *a = format!("-DZEPHYR_EXTRA_MODULES={root}/modules/app-layer");
            }
        }
    }
    args
}

fn copy_artifacts(app: &AppHandle, west: &Path, repo: Option<&Path>) -> FwResult<()> {
    let Some(repo) = repo else {
        return Ok(());
    };
    let out = repo.join("artifacts");
    fs::create_dir_all(&out).map_err(|e| err(e.to_string()))?;
    let pair = out.join("flash-companion-oled");
    fs::create_dir_all(&pair).map_err(|e| err(e.to_string()))?;

    let left = uf2_path(west, "left");
    let right = uf2_path(west, "right");
    if left.is_file() {
        let dest = out.join("Lily58_left_oled.uf2");
        fs::copy(&left, &dest).map_err(|e| err(e.to_string()))?;
        fs::copy(&left, pair.join("Lily58_left_oled.uf2")).map_err(|e| err(e.to_string()))?;
        emit_log(app, format!("Copied left → {}", dest.display()));
    }
    if right.is_file() {
        let dest = out.join("Lily58_right_oled.uf2");
        fs::copy(&right, &dest).map_err(|e| err(e.to_string()))?;
        fs::copy(&right, pair.join("Lily58_right_oled.uf2")).map_err(|e| err(e.to_string()))?;
        emit_log(app, format!("Copied right → {}", dest.display()));
    }
    Ok(())
}

fn try_begin(busy: &FirmwareBusy) -> FwResult<()> {
    if busy
        .0
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(err("A firmware operation is already running"));
    }
    Ok(())
}

fn end(busy: &FirmwareBusy) {
    busy.0.store(false, Ordering::SeqCst);
}

#[tauri::command]
pub fn firmware_status(west_root: Option<String>, busy: State<'_, FirmwareBusy>) -> FirmwareStatus {
    let west = resolve_west_root(west_root);
    let left = uf2_path(&west, "left");
    let right = uf2_path(&west, "right");
    FirmwareStatus {
        west_root: west.display().to_string(),
        west_root_ok: is_west_root(&west),
        repo_root: detect_repo_root().map(|p| p.display().to_string()),
        west_bin: which("west").map(|p| p.display().to_string()),
        docker_ok: which("docker").is_some() && docker_ok(),
        bootloader_mounted: Path::new(BOOTLOADER).is_dir(),
        left_uf2: left.is_file().then(|| left.display().to_string()),
        right_uf2: right.is_file().then(|| right.display().to_string()),
        busy: busy.0.load(Ordering::SeqCst),
    }
}

#[tauri::command]
pub fn firmware_west_update(
    app: AppHandle,
    busy: State<'_, FirmwareBusy>,
    west_root: Option<String>,
) -> FwResult<()> {
    try_begin(&busy)?;
    let result = (|| {
        let west = resolve_west_root(west_root);
        fs::create_dir_all(&west).map_err(|e| err(e.to_string()))?;
        let runner = WestRunner::detect()?;
        if let Some(repo) = detect_repo_root() {
            sync_repo_into_west(&app, &repo, &west)?;
        } else {
            emit_log(
                &app,
                "No pandakb-zmk repo root detected — updating west tree as-is",
            );
        }
        if !west.join(".west").is_dir() {
            let code = runner.run_stream(
                &app,
                &west,
                &["init".into(), "-l".into(), "config".into()],
                "west init -l config",
            )?;
            if code != 0 {
                return Err(err(format!("west init failed ({code})")));
            }
        }
        let code = runner.run_stream(&app, &west, &["update".into()], "west update")?;
        if code != 0 {
            return Err(err(format!("west update failed ({code})")));
        }
        emit_log(&app, "\nOK west update\n");
        let _ = app.emit("firmware://done", "update-ok");
        Ok(())
    })();
    end(&busy);
    result
}

#[tauri::command]
pub fn firmware_build(
    app: AppHandle,
    busy: State<'_, FirmwareBusy>,
    west_root: Option<String>,
    update_first: bool,
) -> FwResult<()> {
    try_begin(&busy)?;
    let result = (|| {
        let west = resolve_west_root(west_root);
        fs::create_dir_all(&west).map_err(|e| err(e.to_string()))?;
        let runner = WestRunner::detect()?;
        let repo = detect_repo_root();

        if let Some(ref repo) = repo {
            sync_repo_into_west(&app, repo, &west)?;
        } else {
            emit_log(
                &app,
                "Warning: pandakb-zmk repo not found — building whatever is in the west tree",
            );
        }

        if !is_west_root(&west) {
            if !west.join("config/west.yml").is_file() {
                return Err(err(format!(
                    "Not a west workspace and no config/west.yml at {}",
                    west.display()
                )));
            }
            let code = runner.run_stream(
                &app,
                &west,
                &["init".into(), "-l".into(), "config".into()],
                "west init -l config",
            )?;
            if code != 0 {
                return Err(err(format!("west init failed ({code})")));
            }
            let code = runner.run_stream(&app, &west, &["update".into()], "west update")?;
            if code != 0 {
                return Err(err(format!("west update failed ({code})")));
            }
        } else if update_first {
            let code = runner.run_stream(&app, &west, &["update".into()], "west update")?;
            if code != 0 {
                return Err(err(format!("west update failed ({code})")));
            }
        }

        for half in ["left", "right"] {
            let args = build_args_for_runner(&runner, half, &west);
            let code = runner.run_stream(&app, &west, &args, &format!("{half} companion OLED"))?;
            if code != 0 {
                return Err(err(format!("{half} build failed ({code})")));
            }
            let uf2 = uf2_path(&west, half);
            if !uf2.is_file() {
                return Err(err(format!("Build OK but UF2 missing: {}", uf2.display())));
            }
            emit_log(&app, format!("{}_UF2={}", half.to_uppercase(), uf2.display()));
        }

        copy_artifacts(&app, &west, repo.as_deref())?;
        emit_log(&app, "\nOK\n");
        let _ = app.emit("firmware://done", "build-ok");
        Ok(())
    })();
    end(&busy);
    if result.is_err() {
        let _ = app.emit("firmware://done", "build-error");
    }
    result
}

#[tauri::command]
pub fn firmware_flash(half: String, west_root: Option<String>) -> FwResult<FlashResult> {
    if half != "left" && half != "right" {
        return Err(err("half must be left or right"));
    }
    let west = resolve_west_root(west_root);
    let uf2 = uf2_path(&west, &half);
    if !uf2.is_file() {
        // Fall back to artifacts/ from the repo if present.
        if let Some(repo) = detect_repo_root() {
            let alt = repo.join(format!("artifacts/Lily58_{half}_oled.uf2"));
            if alt.is_file() {
                return flash_file(&half, &alt);
            }
        }
        return Err(err(format!(
            "UF2 missing: {} — build first",
            uf2.display()
        )));
    }
    flash_file(&half, &uf2)
}

fn flash_file(half: &str, uf2: &Path) -> FwResult<FlashResult> {
    let boot = Path::new(BOOTLOADER);
    if !boot.is_dir() {
        return Err(err(
            "Bootloader not mounted at /Volumes/RPI-RP2 — double-reset the nice!nano, then retry",
        ));
    }
    let dest = boot.join(format!("zmk_{half}.uf2"));
    fs::copy(uf2, &dest).map_err(|e| {
        err(format!(
            "Copy to bootloader failed: {e} (is the volume still mounted?)"
        ))
    })?;
    Ok(FlashResult {
        half: half.to_string(),
        from: uf2.display().to_string(),
        to: dest.display().to_string(),
    })
}

#[tauri::command]
pub fn firmware_wait_bootloader(timeout_secs: Option<u64>) -> FwResult<bool> {
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(60));
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if Path::new(BOOTLOADER).is_dir() {
            return Ok(true);
        }
        thread::sleep(Duration::from_millis(400));
    }
    Ok(false)
}

/// Local `.keymap` that the companion parses for layers / layouts / edits.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeymapFile {
    pub path: String,
    pub repo_root: String,
    pub contents: String,
}

const KEYMAP_CANDIDATES: &[&str] = &["config/Lily58.keymap", "config/lily58.keymap"];

fn resolve_keymap_path(repo: &Path) -> FwResult<PathBuf> {
    for rel in KEYMAP_CANDIDATES {
        let p = repo.join(rel);
        if p.is_file() {
            return Ok(p);
        }
    }
    Err(err(format!(
        "No Lily58.keymap under {}/config",
        repo.display()
    )))
}

fn require_repo_root() -> FwResult<PathBuf> {
    detect_repo_root().ok_or_else(|| {
        err(
            "Could not find pandakb-zmk repo root (expected config/west.yml + modules/app-layer). Set PANDAKB_REPO.",
        )
    })
}

/// Read the repo's Lily58 keymap — source of truth for companion layers.
#[tauri::command]
pub fn firmware_keymap_get() -> FwResult<KeymapFile> {
    let repo = require_repo_root()?;
    let path = resolve_keymap_path(&repo)?;
    let contents = fs::read_to_string(&path).map_err(|e| err(e.to_string()))?;
    Ok(KeymapFile {
        path: path.display().to_string(),
        repo_root: repo.display().to_string(),
        contents,
    })
}

/// Write the repo's Lily58 keymap (companion edits land here before flash).
#[tauri::command]
pub fn firmware_keymap_set(contents: String) -> FwResult<KeymapFile> {
    let repo = require_repo_root()?;
    let path = resolve_keymap_path(&repo).unwrap_or_else(|_| repo.join("config/Lily58.keymap"));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| err(e.to_string()))?;
    }
    fs::write(&path, contents.as_bytes()).map_err(|e| err(e.to_string()))?;
    Ok(KeymapFile {
        path: path.display().to_string(),
        repo_root: repo.display().to_string(),
        contents,
    })
}
