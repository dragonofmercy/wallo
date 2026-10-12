use crate::commands::{self, ApplyLock};
use crate::profile;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder, Wry};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

const TRAY_ID: &str = "main";
const PROFILE_ITEM: &str = "profile:";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().expect("bundle has an icon"))
        .tooltip("Wallo")
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => show_window(app),
            "quit" => app.exit(0),
            id => {
                if let Some(name) = id.strip_prefix(PROFILE_ITEM) {
                    apply_from_tray(app, name.to_string());
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

/// Profiles from disk, the last applied one checked.
fn menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let loaded = profile::load(&commands::profiles_path());
    // load() moves a corrupt file to .bak: the window loading after us would not see the warning.
    if let Some(warning) = loaded.warning {
        error(app, warning);
    }
    let menu = Menu::new(app)?;
    for p in &loaded.profiles.profiles {
        let checked = loaded.profiles.last_selected.as_deref() == Some(p.name.as_str());
        menu.append(&CheckMenuItem::with_id(app, format!("{PROFILE_ITEM}{}", p.name), &p.name, true, checked, None::<&str>)?)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "open", "Open Wallo", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?)?;
    Ok(menu)
}

/// Rebuilds the menu from profiles.json (check mark = lastSelected).
pub fn refresh_menu(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY_ID), menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// Brings the window to front, or recreates it (it is destroyed on close to free WebView2).
pub fn show_window(app: &AppHandle) {
    let window = match app.get_webview_window("main") {
        Some(window) => window,
        None => {
            // Built hidden ("visible": false): the window-state plugin moves it before it first appears.
            let config = app.config().app.windows[0].clone();
            match WebviewWindowBuilder::from_config(app, &config).and_then(|b| b.build()) {
                Ok(window) => window,
                // A concurrent call may have built it first (duplicate label): show that one.
                Err(e) => match app.get_webview_window("main") {
                    Some(window) => window,
                    None => return error(app, format!("Cannot open the window: {e}")),
                },
            }
        }
    };
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn apply_from_tray(app: &AppHandle, name: String) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let lock = app.state::<ApplyLock>().0.clone();
        // Busy: ignore the click. Either way the menu is rebuilt, Windows toggled the clicked check mark itself.
        if let Some(_guard) = commands::lock_apply(&lock) {
            let loaded = profile::load(&commands::profiles_path());
            let result = if let Some(warning) = loaded.warning {
                // Corrupt file was just moved to .bak: applying would save defaults over it.
                Err(warning)
            } else if loaded.save_blocked {
                Err("profiles.json could not be read, fix or remove it before applying".to_string())
            } else {
                let mut profiles = loaded.profiles;
                profiles.last_selected = Some(name);
                commands::apply_now(&profiles)
            };
            match result {
                Ok(()) => {
                    let _ = app.emit("profiles-changed", ());
                }
                Err(e) => error(&app, e),
            }
        }
        refresh_menu(&app);
    });
}

fn error(app: &AppHandle, text: String) {
    app.dialog().message(text).kind(MessageDialogKind::Error).title("Wallo").show(|_| {});
}
