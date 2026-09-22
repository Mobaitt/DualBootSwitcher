use crate::boot::{
    current_manager,
    models::{BootEntry, BootInfo},
    visible_boot_info,
};
use crate::error::BootError;
use tauri::Manager;

#[tauri::command]
pub fn set_tray_language(app: tauri::AppHandle, language: String) -> Result<(), String> {
    crate::tray::set_language(&app, &language).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_window_title(app: tauri::AppHandle, title: String) -> Result<(), String> {
    app.get_webview_window("main")
        .ok_or_else(|| "主窗口不存在".to_string())?
        .set_title(&title)
        .map_err(|error| error.to_string())
}

fn readable(error: BootError) -> String {
    error.to_string()
}

#[tauri::command]
pub async fn get_boot_info() -> Result<BootInfo, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let result = current_manager()
            .and_then(|manager| manager.get_boot_info())
            .map(visible_boot_info);
        match &result {
            Ok(info) => log::info!("scanned {} UEFI boot entries", info.entries.len()),
            Err(error) => log::error!("boot scan failed: {error}"),
        }
        result.map_err(readable)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_boot_entries() -> Result<Vec<BootEntry>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        current_manager()
            .and_then(|manager| manager.list_entries())
            .map_err(readable)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_boot_next(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        log::info!("requesting BootNext for a selected target");
        let manager = current_manager().map_err(readable)?;
        if !manager
            .list_entries()
            .map_err(readable)?
            .iter()
            .any(|entry| entry.id == id)
        {
            return Err(readable(BootError::BootEntryNotFound(id)));
        }
        manager.set_boot_next(&id).map_err(|error| {
            log::error!("setting BootNext failed: {error}");
            readable(error)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn reboot() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| {
        current_manager()
            .and_then(|manager| manager.reboot())
            .map_err(readable)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_boot_next_and_reboot(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let manager = current_manager().map_err(readable)?;
        if !manager
            .list_entries()
            .map_err(readable)?
            .iter()
            .any(|entry| entry.id == id)
        {
            return Err(readable(BootError::BootEntryNotFound(id)));
        }
        manager.set_boot_next(&id).map_err(readable)?;
        log::info!("BootNext set successfully; rebooting only after confirmation");
        manager.reboot().map_err(readable)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_default_boot(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let manager = current_manager().map_err(readable)?;
        if !manager
            .list_entries()
            .map_err(readable)?
            .iter()
            .any(|entry| entry.id == id)
        {
            return Err(readable(BootError::BootEntryNotFound(id)));
        }
        log::info!("setting a validated target as the permanent default boot entry");
        manager.set_default_boot(&id).map_err(|error| {
            log::error!("setting permanent boot order failed: {error}");
            readable(error)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn refresh_boot_info() -> Result<BootInfo, String> {
    get_boot_info().await
}
