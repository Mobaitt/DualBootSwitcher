mod boot;
mod commands;
mod error;
mod privilege;
mod system;
mod tray;

use tauri::Manager;

pub fn run() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| Ok(tray::initialize(app)?))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_window_title,
            commands::set_tray_language,
            commands::get_boot_info,
            commands::get_boot_entries,
            commands::set_boot_next,
            commands::set_default_boot,
            commands::reboot,
            commands::set_boot_next_and_reboot,
            commands::refresh_boot_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BootPilot");
}
