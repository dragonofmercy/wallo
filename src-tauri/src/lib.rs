use tauri::Manager;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

mod commands;
mod monitors;
mod profile;
mod render;
mod tray;
mod wallpaper;

/// Command-line flag used by the autostart entry: start in the tray, no window.
const TRAY_ARG: &str = "--tray";

pub fn run() {
    let hidden = std::env::args().any(|a| a == TRAY_ARG);
    tauri::Builder::default()
        // Must be the first plugin: a second launch only shows this instance's window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Runs inside the WM_COPYDATA window proc: building the WebView2 window there can fail, so go through a thread.
            let app = app.clone();
            std::thread::spawn(move || tray::show_window(&app));
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        // Fixed-size window: only the position is remembered.
        .plugin(tauri_plugin_window_state::Builder::new().with_state_flags(StateFlags::POSITION).build())
        .manage(commands::ImageCache::default())
        .manage(commands::SaveBlocked::default())
        .manage(commands::ApplyLock::default())
        .setup(move |app| {
            tray::create(app.handle())?;
            if !hidden {
                tray::show_window(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_monitors,
            commands::load_profiles,
            commands::render_preview,
            commands::align_offset,
            commands::apply,
            commands::get_autostart,
            commands::set_autostart
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            // Closing the window keeps Wallo in the tray; only "Quit" (app.exit) carries an exit code.
            tauri::RunEvent::ExitRequested { code: None, api, .. } => api.prevent_exit(),
            tauri::RunEvent::WindowEvent { event: tauri::WindowEvent::Destroyed, .. } => {
                app.state::<commands::ImageCache>().clear();
                // The plugin only writes to disk on exit; Wallo rarely exits (tray), and a logoff kills it without Exit.
                let _ = app.save_window_state(StateFlags::POSITION);
            }
            _ => {}
        });
}
