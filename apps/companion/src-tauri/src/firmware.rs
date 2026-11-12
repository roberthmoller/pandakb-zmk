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

/// nice!nano mounts as NICENANO; some UF2 boards use RPI-RP2.
const BOOTLOADER_VOLUMES: &[&str] = &["/Volumes/NICENANO", "/Volumes/RPI-RP2"];
const DOCKER_IMAGE: &str = "zmkfirmware/zmk-build-arm:stable";
/// Stamp next to published UF2s — flash source of truth is `artifacts/`, not ~/.zmk.
const ARTIFACT_STAMP: &str = "artifacts/.pandakb-companion.stamp";
const WEST_SOURCE_STAMP: &str = "build/pandakb-companion.stamp";
const SOURCE_PATHS: &[&str] = &[
    "config",
    "boards",
    "modules/app-layer",
    "build.yaml",
    "zephyr/module.yml",
];
/// West build cache only. Published firmware lives in `<repo>/artifacts/`.
const DEFAULT_WEST_ROOT: &str = "~/.zmk";

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

#[derive(Clone)]
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
    pub bootloader_path: Option<String>,
    pub left_uf2: Option<String>,
    pub right_uf2: Option<String>,
    pub sources_stale: bool,
    pub busy: bool,
}

fn bootloader_path() -> Option<PathBuf> {
    for p in BOOTLOADER_VOLUMES {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Some(path);
        }
    }
    // Numbered duplicates when both halves are in bootloader: NICENANO 1, etc.
    if let Ok(entries) = fs::read_dir("/Volumes") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("NICENANO") || name.starts_with("RPI-RP2") {
                return Some(entry.path());
            }
        }
    }
    None
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

fn is_legacy_west_root(p: &str) -> bool {
    p.contains("referenced-chatgpt") || p.contains("pandakb-lily")
}

fn resolve_west_root(preferred: Option<String>) -> PathBuf {
    if let Some(p) = preferred {
        let t = p.trim();
        if !t.is_empty() && !is_legacy_west_root(t) {
            return expand_home(t);
        }
    }
    if let Ok(env) = std::env::var("PANDAKB_WEST_ROOT") {
        let t = env.trim();
        if !t.is_empty() {
            return expand_home(t);
        }
    }
    expand_home(DEFAULT_WEST_ROOT)
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

fn west_uf2_path(west: &Path, half: &str) -> PathBuf {
    let build = if half == "left" {
        LEFT_BUILD_DIR
    } else {
        RIGHT_BUILD_DIR
    };
    west.join(build).join("zephyr/zmk.uf2")
}

fn artifact_uf2_path(repo: &Path, half: &str) -> PathBuf {
    repo.join(format!("artifacts/Lily58_{half}_oled.uf2"))
}

/// Prefer repo `artifacts/` (what we flash); fall back to west build output.
fn resolve_flash_uf2(west: &Path, repo: Option<&Path>, half: &str) -> Option<PathBuf> {
    if let Some(repo) = repo {
        let art = artifact_uf2_path(repo, half);
        if art.is_file() {
            return Some(art);
        }
    }
    let west_uf2 = west_uf2_path(west, half);
    west_uf2.is_file().then_some(west_uf2)
}

fn max_mtime(path: &Path, acc: &mut Option<std::time::SystemTime>) {
    let Ok(meta) = fs::metadata(path) else {
        return;
    };
    if meta.is_file() {
        if let Ok(m) = meta.modified() {
            *acc = Some(acc.map_or(m, |cur| cur.max(m)));
        }
        return;
    }
    if !meta.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name == ".DS_Store" || name == ".git" || name == "node_modules" {
            continue;
        }
        max_mtime(&entry.path(), acc);
    }
}

fn sources_fingerprint(repo: &Path) -> String {
    let mut latest: Option<std::time::SystemTime> = None;
    for rel in SOURCE_PATHS {
        max_mtime(&repo.join(rel), &mut latest);
    }
    match latest {
        Some(t) => format!(
            "{}",
            t.duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        ),
        None => "missing".into(),
    }
}

fn read_source_stamp(repo: &Path, west: &Path) -> Option<String> {
    let artifact_stamp = repo.join(ARTIFACT_STAMP);
    if let Ok(prev) = fs::read_to_string(&artifact_stamp) {
        return Some(prev);
    }
    fs::read_to_string(west.join(WEST_SOURCE_STAMP)).ok()
}

fn sources_are_stale(west: &Path, repo: Option<&Path>) -> bool {
    let Some(repo) = repo else {
        return !west_uf2_path(west, "left").is_file() || !west_uf2_path(west, "right").is_file();
    };
    let left = artifact_uf2_path(repo, "left");
    let right = artifact_uf2_path(repo, "right");
    if !left.is_file() || !right.is_file() {
        return true;
    }
    let Some(prev) = read_source_stamp(repo, west) else {
        return true;
    };
    prev.trim() != sources_fingerprint(repo)
}

fn write_source_stamp(west: &Path, repo: Option<&Path>) -> FwResult<()> {
    let Some(repo) = repo else {
        return Ok(());
    };
    let fp = sources_fingerprint(repo);
    let artifact_stamp = repo.join(ARTIFACT_STAMP);
    if let Some(parent) = artifact_stamp.parent() {
        fs::create_dir_all(parent).map_err(|e| err(e.to_string()))?;
    }
    fs::write(&artifact_stamp, &fp).map_err(|e| err(e.to_string()))?;
    let west_stamp = west.join(WEST_SOURCE_STAMP);
    if let Some(parent) = west_stamp.parent() {
        fs::create_dir_all(parent).map_err(|e| err(e.to_string()))?;
    }
    fs::write(&west_stamp, fp).map_err(|e| err(e.to_string()))
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

    // ZMK_CONFIG discovers shields under config/boards/shields. Copy only the
    // companion shields there so BOARD_ROOT=/work is unnecessary (and so broken
    // experimental shields under boards/ don't poison Kconfig).
    let cfg_shields = west.join("config/boards/shields");
    fs::create_dir_all(&cfg_shields).map_err(|e| err(e.to_string()))?;
    for name in ["Lily58", "app_layer", "game_badge"] {
        let from = repo.join("boards/shields").join(name);
        if from.is_dir() {
            let to = cfg_shields.join(name);
            if to.exists() {
                fs::remove_dir_all(&to).map_err(|e| err(e.to_string()))?;
            }
            copy_dir_recursive(&from, &to).map_err(|e| err(e.to_string()))?;
            emit_log(app, format!("  synced config/boards/shields/{name}"));
        }
    }

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
                    .env("ZEPHYR_BASE", west_root.join("zephyr"))
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                stream_command(app, &mut cmd)
            }
            Self::Docker => {
                // Bind-mounts are owned by the host user; git inside the container
                // (root) refuses them as "dubious ownership", which breaks west's
                // manifest import — so `west build` never registers. Mark the tree safe.
                let mount = format!("{}:/work", west_root.display());
                let west_cmdline = shell_join(args);
                let script = format!(
                    "git config --global --add safe.directory '*' && \
                     git config --global --add safe.directory /work && \
                     export ZEPHYR_BASE=/work/zephyr && \
                     west {west_cmdline}"
                );
                let docker_args = [
                    "run".to_string(),
                    "--rm".into(),
                    "-v".into(),
                    mount,
                    "-w".into(),
                    "/work".into(),
                    "-e".into(),
                    "ZEPHYR_BASE=/work/zephyr".into(),
                    DOCKER_IMAGE.into(),
                    "bash".into(),
                    "-lc".into(),
                    script.clone(),
                ];
                emit_log(app, format!("$ docker run … bash -lc {script:?}"));
                let mut cmd = Command::new("docker");
                cmd.args(&docker_args)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                stream_command(app, &mut cmd)
            }
        }
    }
}

/// Quote args for `bash -lc` so spaces in `-DSHIELD=…` stay one token.
fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|a| {
            if a.is_empty() {
                "''".into()
            } else if a.chars().any(|c| c.is_whitespace() || "\"'\\$`!".contains(c)) {
                format!("'{}'", a.replace('\'', "'\"'\"'"))
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
        // Zephyr hardware model (nice!nano v2). Legacy alias `nice_nano_v2` no longer resolves.
        "nice_nano/nrf52840/zmk".into(),
        "--".into(),
        format!("-DSHIELD={shield}"),
        "-DZMK_CONFIG=/work/config".into(),
        "-DZephyr_DIR=/work/zephyr/share/zephyr-package/cmake".into(),
        // Must include zmk app modules — a lone EXTRA_MODULES path replaces them and
        // nice_nano boards disappear.
        "-DZEPHYR_EXTRA_MODULES=/work/zmk/app/module;/work/zmk/app/keymap-module;/work/modules/app-layer".into(),
    ];
    for part in cmake.split_whitespace() {
        args.push(part.to_string());
    }
    args
}

fn build_args_for_runner(runner: &WestRunner, half: &str, west_root: &Path) -> Vec<String> {
    let mut args = build_args(half);
    if let WestRunner::Native(_) = runner {
        let root = west_root.display().to_string();
        for a in &mut args {
            if a.starts_with("-DZMK_CONFIG=") {
                *a = format!("-DZMK_CONFIG={root}/config");
            } else if a.starts_with("-DZephyr_DIR=") {
                *a = format!("-DZephyr_DIR={root}/zephyr/share/zephyr-package/cmake");
            } else if a.starts_with("-DZEPHYR_EXTRA_MODULES=") {
                *a = format!(
                    "-DZEPHYR_EXTRA_MODULES={root}/zmk/app/module;{root}/zmk/app/keymap-module;{root}/modules/app-layer"
                );
            }
        }
    }
    args
}

/// Keep zephyr.base relative so docker (/work) and native hosts both resolve it.
fn ensure_west_config(west: &Path) -> FwResult<()> {
    let cfg = west.join(".west/config");
    if !cfg.is_file() {
        return Ok(());
    }
    let text = fs::read_to_string(&cfg).map_err(|e| err(e.to_string()))?;
    if text.contains("base = /work/zephyr") {
        let fixed = text.replace("base = /work/zephyr", "base = zephyr");
        fs::write(&cfg, fixed).map_err(|e| err(e.to_string()))?;
    }
    Ok(())
}

fn copy_artifacts(app: &AppHandle, west: &Path, repo: Option<&Path>) -> FwResult<()> {
    let Some(repo) = repo else {
        return Ok(());
    };
    let out = repo.join("artifacts");
    fs::create_dir_all(&out).map_err(|e| err(e.to_string()))?;
    let pair = out.join("flash-companion-oled");
    fs::create_dir_all(&pair).map_err(|e| err(e.to_string()))?;

    let left = west_uf2_path(west, "left");
    let right = west_uf2_path(west, "right");
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
    write_source_stamp(west, Some(repo))?;
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

pub fn firmware_status_of(west_root: Option<String>, busy: &FirmwareBusy) -> FirmwareStatus {
    let west = resolve_west_root(west_root);
    let repo = detect_repo_root();
    let left = resolve_flash_uf2(&west, repo.as_deref(), "left");
    let right = resolve_flash_uf2(&west, repo.as_deref(), "right");
    let boot = bootloader_path();
    FirmwareStatus {
        west_root: west.display().to_string(),
        west_root_ok: is_west_root(&west),
        repo_root: repo.as_ref().map(|p| p.display().to_string()),
        west_bin: which("west").map(|p| p.display().to_string()),
        docker_ok: which("docker").is_some() && docker_ok(),
        bootloader_mounted: boot.is_some(),
        bootloader_path: boot.as_ref().map(|p| p.display().to_string()),
        left_uf2: left.as_ref().map(|p| p.display().to_string()),
        right_uf2: right.as_ref().map(|p| p.display().to_string()),
        sources_stale: sources_are_stale(&west, repo.as_deref()),
        busy: busy.0.load(Ordering::SeqCst),
    }
}

#[tauri::command]
pub fn firmware_status(west_root: Option<String>, busy: State<'_, FirmwareBusy>) -> FirmwareStatus {
    firmware_status_of(west_root, &busy)
}

pub fn run_firmware_west_update(
    app: &AppHandle,
    busy: &FirmwareBusy,
    west_root: Option<String>,
) -> FwResult<()> {
    try_begin(busy)?;
    let result = (|| {
        let west = resolve_west_root(west_root);
        fs::create_dir_all(&west).map_err(|e| err(e.to_string()))?;
        ensure_west_config(&west)?;
        let runner = WestRunner::detect()?;
        if let Some(repo) = detect_repo_root() {
            sync_repo_into_west(app, &repo, &west)?;
        } else {
            emit_log(
                app,
                "No pandakb-zmk repo root detected — updating west tree as-is",
            );
        }
        if !west.join(".west").is_dir() {
            let code = runner.run_stream(
                app,
                &west,
                &["init".into(), "-l".into(), "config".into()],
                "west init -l config",
            )?;
            if code != 0 {
                return Err(err(format!("west init failed ({code})")));
            }
        }
        let code = runner.run_stream(app, &west, &["update".into()], "west update")?;
        if code != 0 {
            return Err(err(format!("west update failed ({code})")));
        }
        emit_log(app, "\nOK west update\n");
        let _ = app.emit("firmware://done", "update-ok");
        Ok(())
    })();
    end(busy);
    result
}

#[tauri::command]
pub fn firmware_west_update(
    app: AppHandle,
    busy: State<'_, FirmwareBusy>,
    west_root: Option<String>,
) -> FwResult<()> {
    run_firmware_west_update(&app, &busy, west_root)
}

pub fn run_firmware_build(
    app: &AppHandle,
    busy: &FirmwareBusy,
    west_root: Option<String>,
    update_first: bool,
    force: Option<bool>,
) -> FwResult<()> {
    try_begin(busy)?;
    let result = (|| {
        let west = resolve_west_root(west_root);
        let repo = detect_repo_root().ok_or_else(|| {
            err("Could not find pandakb-zmk repo (expected scripts/build-companion.sh)")
        })?;
        let script = repo.join("scripts/build-companion.sh");
        if !script.is_file() {
            return Err(err(format!("Missing {}", script.display())));
        }

        if update_first {
            ensure_west_config(&west)?;
            let runner = WestRunner::detect()?;
            let code = runner.run_stream(app, &west, &["update".into()], "west update")?;
            if code != 0 {
                return Err(err(format!("west update failed ({code})")));
            }
        }

        emit_log(app, "\n── scripts/build-companion.sh ──");
        let mut cmdline = format!("bash {} ", script.display());
        if force.unwrap_or(false) {
            cmdline.push_str("--force ");
        }
        cmdline.push_str(&format!("(WEST_ROOT={})", west.display()));
        emit_log(app, format!("$ {cmdline}"));

        let mut cmd = Command::new("bash");
        cmd.arg(&script);
        if force.unwrap_or(false) {
            cmd.arg("--force");
        }
        cmd.current_dir(&repo)
            .env("WEST_ROOT", &west)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let code = stream_command(app, &mut cmd)?;
        if code != 0 {
            return Err(err(format!(
                "build-companion.sh failed ({code}). Native west required — see script output / README."
            )));
        }

        let left = repo.join("artifacts/Lily58_left_oled.uf2");
        let right = repo.join("artifacts/Lily58_right_oled.uf2");
        if !left.is_file() || !right.is_file() {
            return Err(err(
                "Build finished but artifacts/Lily58_*_oled.uf2 missing",
            ));
        }
        emit_log(app, format!("LEFT_UF2={}", left.display()));
        emit_log(app, format!("RIGHT_UF2={}", right.display()));
        emit_log(app, "\nOK\n");
        let _ = app.emit("firmware://done", "build-ok");
        Ok(())
    })();
    end(busy);
    if result.is_err() {
        let _ = app.emit("firmware://done", "build-error");
    }
    result
}

#[tauri::command]
pub fn firmware_build(
    app: AppHandle,
    busy: State<'_, FirmwareBusy>,
    west_root: Option<String>,
    update_first: bool,
    force: Option<bool>,
) -> FwResult<()> {
    run_firmware_build(&app, &busy, west_root, update_first, force)
}

#[tauri::command]
pub fn firmware_flash(half: String, west_root: Option<String>) -> FwResult<FlashResult> {
    if half != "left" && half != "right" {
        return Err(err("half must be left or right"));
    }
    let west = resolve_west_root(west_root);
    let repo = detect_repo_root();
    let Some(uf2) = resolve_flash_uf2(&west, repo.as_deref(), &half) else {
        return Err(err(format!(
            "UF2 missing for {half} — expected artifacts/Lily58_{half}_oled.uf2 (run ./scripts/build-companion.sh)"
        )));
    };
    flash_file(&half, &uf2)
}

fn flash_file(half: &str, uf2: &Path) -> FwResult<FlashResult> {
    let Some(boot) = bootloader_path() else {
        return Err(err(
            "Bootloader not mounted (expected /Volumes/NICENANO or /Volumes/RPI-RP2) — soft-enter or double-reset, then retry",
        ));
    };
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
        if bootloader_path().is_some() {
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
