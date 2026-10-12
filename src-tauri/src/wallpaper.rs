use crate::monitors::desktop_wallpaper;
use image::{DynamicImage, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::HSTRING;
use windows::Win32::UI::Shell::DWPOS_FILL;

/// Writes one PNG per monitor (unique names, Windows caches wallpapers by path), sets them, then removes older files.
pub fn apply(dir: &Path, images: &[(String, RgbaImage)]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or_default();
    let mut written: Vec<PathBuf> = Vec::new();
    for (i, (_, image)) in images.iter().enumerate() {
        let path = dir.join(format!("{stamp}-{i}.png"));
        DynamicImage::ImageRgba8(image.clone()).into_rgb8().save(&path).map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
        written.push(path);
    }
    let dw = desktop_wallpaper().map_err(|e| e.to_string())?;
    unsafe {
        // Images match each monitor's resolution exactly, so Fill shows them 1:1.
        dw.SetPosition(DWPOS_FILL).map_err(|e| e.to_string())?;
        for ((id, _), path) in images.iter().zip(&written) {
            dw.SetWallpaper(&HSTRING::from(id.as_str()), &HSTRING::from(path.as_os_str())).map_err(|e| e.to_string())?;
        }
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if !written.contains(&entry.path()) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}
