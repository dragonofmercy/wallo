mod commands;
mod monitors;
mod profile;
mod render;
mod wallpaper;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::ImageCache::default())
        .invoke_handler(tauri::generate_handler![commands::list_monitors, commands::load_profiles, commands::render_preview, commands::apply])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
