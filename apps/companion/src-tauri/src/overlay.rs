//! macOS layer-preview window: native glass + level above fullscreen games.

use log::{debug, warn};
use tauri::{AppHandle, Manager, WebviewWindow};

const LAYER_PREVIEW_WINDOW: &str = "layer-preview";

/// Configure the overlay once at startup (glass + space joining).
pub fn configure(app: &AppHandle) {
    let Some(win) = app.get_webview_window(LAYER_PREVIEW_WINDOW) else {
        warn!("layer-preview window missing at configure");
        return;
    };

    let win2 = win.clone();
    let _ = win.run_on_main_thread(move || {
        let _ = win2.set_ignore_cursor_events(true);
        let _ = win2.set_always_on_top(true);

        #[cfg(target_os = "macos")]
        {
            apply_glass(&win2);
            elevate(&win2);
        }
    });
}

/// Re-assert z-order + Active vibrancy when showing.
/// Games (and app deactivation) otherwise steal the space / fade the blur.
pub fn present(win: &WebviewWindow) {
    let win2 = win.clone();
    let _ = win.run_on_main_thread(move || {
        let _ = win2.set_always_on_top(true);
        let _ = win2.show();

        #[cfg(target_os = "macos")]
        {
            // Re-apply every show: inactive-app state can clear / fade the effect,
            // and Tauri always-on-top may reset the window level.
            apply_glass(&win2);
            elevate(&win2);
            order_front(&win2);
        }
    });
}

pub fn dismiss(win: &WebviewWindow) {
    let win2 = win.clone();
    let _ = win.run_on_main_thread(move || {
        let _ = win2.hide();
    });
}

#[cfg(target_os = "macos")]
fn apply_glass(win: &WebviewWindow) {
    use window_vibrancy::{
        apply_vibrancy, clear_liquid_glass, clear_vibrancy, NSVisualEffectMaterial,
        NSVisualEffectState,
    };

    // Prefer HudWindow + Active over liquid glass: NSGlassEffectView dims when the
    // app is not frontmost (overlay never becomes key; games keep focus), which is
    // exactly the "blur disappears after a while" bug.
    let _ = clear_liquid_glass(win);
    let _ = clear_vibrancy(win);

    match apply_vibrancy(
        win,
        NSVisualEffectMaterial::HudWindow,
        Some(NSVisualEffectState::Active),
        Some(20.0),
    ) {
        Ok(()) => debug!("layer-preview: HudWindow Active vibrancy applied"),
        Err(e) => warn!("layer-preview vibrancy failed: {e}"),
    }
}

#[cfg(target_os = "macos")]
fn elevate(win: &WebviewWindow) {
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior, NSWindowLevel};

    let Ok(ptr) = win.ns_window() else {
        warn!("layer-preview: ns_window unavailable");
        return;
    };

    // SAFETY: Tauri returns a valid NSWindow* for this WebviewWindow.
    let ns_window = unsafe { &*(ptr as *const AnyObject as *const NSWindow) };

    // Above exclusive / borderless fullscreen content (PopUpMenu is not enough
    // for many Metal games). Shielding-1 is the usual overlay ceiling.
    let level = (unsafe { CGShieldingWindowLevel() } - 1) as NSWindowLevel;
    ns_window.setLevel(level);
    ns_window.setHidesOnDeactivate(false);
    ns_window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );
}

#[cfg(target_os = "macos")]
fn order_front(win: &WebviewWindow) {
    use objc2::runtime::AnyObject;
    use objc2_app_kit::NSWindow;

    let Ok(ptr) = win.ns_window() else {
        return;
    };
    let ns_window = unsafe { &*(ptr as *const AnyObject as *const NSWindow) };
    ns_window.orderFrontRegardless();
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGShieldingWindowLevel() -> i32;
}
