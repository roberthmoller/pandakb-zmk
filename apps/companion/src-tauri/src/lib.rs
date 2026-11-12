mod companion;
mod firmware;
mod hid;
mod ide;
mod lol;
mod mcp;
mod media;
mod overlay;
mod protocol;

use companion::{
    companion_enter_bootloader, companion_push_idle, companion_set_layer, companion_status,
    Companion,
};
use firmware::{
    firmware_build, firmware_flash, firmware_keymap_get, firmware_keymap_set, firmware_status,
    firmware_wait_bootloader, firmware_west_update, FirmwareBusy,
};
use mcp::spawn_mcp_server;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(FirmwareBusy::default())
        .setup(|app| {
            let companion = Companion::start(app.handle().clone())?;
            let busy = app.state::<FirmwareBusy>().inner().clone();
            spawn_mcp_server(app.handle().clone(), companion.clone(), busy);
            app.manage(companion);

            overlay::configure(app.handle());

            let show = MenuItem::with_id(app, "show", "Open Flash", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let tray_icon = Image::from_bytes(include_bytes!("../icons/tray-template.png"))
                .expect("tray template icon");
            let _tray = TrayIconBuilder::with_id("main")
                .icon(tray_icon)
                .icon_as_template(true)
                .title("…")
                .tooltip("PandaKB Companion")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            companion_status,
            companion_push_idle,
            companion_enter_bootloader,
            companion_set_layer,
            firmware_status,
            firmware_west_update,
            firmware_build,
            firmware_flash,
            firmware_wait_bootloader,
            firmware_keymap_get,
            firmware_keymap_set
        ])
        .run(tauri::generate_context!())
        .expect("error while running PandaKB companion");
}
