mod commands;
mod monitors;
mod profile;
mod render;
mod tray;
mod wallpaper;

/// Command-line flag used by the autostart entry: start in the tray, no window.
const TRAY_ARG: &str = "--tray";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let hidden = std::env::args().any(|a| a == TRAY_ARG);
    tauri::Builder::default()
        // Must be the first plugin: a second launch only shows this instance's window.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| tray::show_window(app)))
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![TRAY_ARG])))
        .plugin(tauri_plugin_dialog::init())
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
            commands::apply,
            commands::get_autostart,
            commands::set_autostart
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Closing the window keeps Wallo in the tray; only "Quit" (app.exit) carries an exit code.
            if let tauri::RunEvent::ExitRequested { code: None, api, .. } = event {
                api.prevent_exit();
            }
        });
}
