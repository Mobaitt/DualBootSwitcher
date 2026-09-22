use crate::boot::{current_manager, visible_boot_info};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, Runtime,
};

const TRAY_ID: &str = "dualboot-tray";
const NEXT_ITEM_PREFIX: &str = "boot-next:";
const SHOW_ITEM_ID: &str = "show-window";
const HIDE_ITEM_ID: &str = "hide-window";
const REFRESH_ITEM_ID: &str = "refresh-entries";
const QUIT_ITEM_ID: &str = "quit-app";
static TRAY_ENGLISH: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
struct TrayBootNextResult {
    ok: bool,
    message: String,
}

pub fn initialize(app: &App) -> tauri::Result<()> {
    let menu = build_menu(app.handle())?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tauri::include_image!("./icons/32x32.png"))
        .tooltip("BootPilot")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().0.as_str()))
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                toggle_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

pub fn set_language<R: Runtime>(app: &AppHandle<R>, language: &str) -> tauri::Result<()> {
    TRAY_ENGLISH.store(language.eq_ignore_ascii_case("en-us") || language.eq_ignore_ascii_case("en"), Ordering::Relaxed);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        tray.set_tooltip(Some(tray_text("引导序", "BootPilot")))?;
    }
    refresh_menu(app)
}

fn tray_text<'a>(chinese: &'a str, english: &'a str) -> &'a str {
    if TRAY_ENGLISH.load(Ordering::Relaxed) { english } else { chinese }
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        SHOW_ITEM_ID => show_window(app),
        HIDE_ITEM_ID => hide_window(app),
        REFRESH_ITEM_ID => {
            if let Err(error) = refresh_menu(app) {
                log::warn!("Unable to refresh tray menu: {error}");
            }
        }
        QUIT_ITEM_ID => app.exit(0),
        _ => {
            if let Some(entry_id) = id.strip_prefix(NEXT_ITEM_PREFIX) {
                set_next_from_tray(app.clone(), entry_id.to_owned());
            }
        }
    }
}

fn set_next_from_tray<R: Runtime>(app: AppHandle<R>, entry_id: String) {
    std::thread::spawn(move || {
        let result = current_manager().and_then(|manager| {
            let exists = manager.list_entries()?.iter().any(|entry| entry.id == entry_id);
            if !exists {
                return Err(crate::error::BootError::BootEntryNotFound(entry_id.clone()));
            }
            manager.set_boot_next(&entry_id)
        });

        let payload = match result {
            Ok(()) => {
                let name = current_manager()
                    .and_then(|manager| {
                        Ok(manager
                            .list_entries()?
                            .into_iter()
                            .find(|entry| entry.id == entry_id)
                            .map(|entry| entry.name))
                    })
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| entry_id.clone());
                let _ = refresh_menu(&app);
                TrayBootNextResult {
                    ok: true,
                    message: if TRAY_ENGLISH.load(Ordering::Relaxed) {
                        format!("The next boot will start “{name}”.")
                    } else {
                        format!("下一次启动将进入「{name}」。")
                    },
                }
            }
            Err(error) => TrayBootNextResult {
                ok: false,
                message: error.to_string(),
            },
        };

        if let Err(error) = app.emit("tray-boot-next-result", payload) {
            log::warn!("Unable to notify the app about tray action: {error}");
        }
    });
}

fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let info = current_manager()
        .and_then(|manager| manager.get_boot_info().map(visible_boot_info))
        .unwrap_or_else(|error| {
            log::warn!("Unable to load boot entries for tray: {error}");
            crate::boot::models::BootInfo {
                current_os: String::new(),
                uefi: false,
                secure_boot: None,
                boot_current: None,
                boot_next: None,
                boot_order: Vec::new(),
                entries: Vec::new(),
            }
        });
    let boot_next = info.boot_next;
    let entries = info.entries;

    let has_entries = !entries.is_empty();
    let next_items = if !has_entries {
        vec![MenuItem::with_id(
            app,
            "no-entries",
            tray_text("没有可用启动项", "No usable boot entries"),
            false,
            None::<&str>,
        )?]
    } else {
        entries
            .into_iter()
            .map(|entry| {
                let selected = entry.next
                    || boot_next
                        .as_deref()
                        .is_some_and(|id| entry.id.eq_ignore_ascii_case(id));
                let marker = if selected { "●" } else { "○" };
                MenuItem::with_id(
                    app,
                    format!("{NEXT_ITEM_PREFIX}{}", entry.id),
                    format!("{marker} {}", entry.name),
                    true,
                    None::<&str>,
                )
            })
            .collect::<tauri::Result<Vec<_>>>()?
    };
    let next_refs: Vec<&dyn IsMenuItem<R>> = next_items
        .iter()
        .map(|item| item as &dyn IsMenuItem<R>)
        .collect();
    let next_menu = Submenu::with_items(app, tray_text("设置下一次启动", "Set next boot"), has_entries, &next_refs)?;
    let show = MenuItem::with_id(app, SHOW_ITEM_ID, tray_text("显示应用", "Show app"), true, None::<&str>)?;
    let hide = MenuItem::with_id(app, HIDE_ITEM_ID, tray_text("隐藏应用", "Hide app"), true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, REFRESH_ITEM_ID, tray_text("刷新启动项", "Refresh boot entries"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_ITEM_ID, tray_text("关闭应用", "Quit app"), true, None::<&str>)?;
    let divider_one = PredefinedMenuItem::separator(app)?;
    let divider_two = PredefinedMenuItem::separator(app)?;

    Menu::with_items(
        app,
        &[
            &next_menu,
            &divider_one,
            &show,
            &hide,
            &refresh,
            &divider_two,
            &quit,
        ],
    )
}

fn refresh_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return Ok(());
    };
    tray.set_menu(Some(build_menu(app)?))
}

fn show_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn toggle_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
