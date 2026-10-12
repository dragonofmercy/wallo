use crate::monitors::desktop_wallpaper;
use image::buffer::ConvertBuffer;
use image::{RgbImage, RgbaImage};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::HSTRING;
use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::DWPOS_FILL;

/// Writes one PNG per monitor (unique names, Windows caches wallpapers by path), sets them, then removes older files.
/// All-or-nothing: on failure the files written here are deleted and the previous wallpapers and position restored.
pub fn apply(dir: &Path, images: Vec<(String, RgbaImage)>) -> Result<(), String> {
    if images.is_empty() {
        return Err("No monitor detected".into());
    }
    fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or_default();
    let mut written: Vec<PathBuf> = Vec::new();
    for (i, (_, image)) in images.iter().enumerate() {
        let path = dir.join(format!("{stamp}-{i}.png"));
        let rgb: RgbImage = image.convert();
        if let Err(e) = rgb.save(&path) {
            // The failed file may exist partially written.
            written.push(path.clone());
            remove(&written);
            return Err(format!("Cannot write {}: {e}", path.display()));
        }
        written.push(path);
    }
    if let Err(e) = set(&images, &written) {
        remove(&written);
        return Err(e);
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            // Only our own output: never delete something a user dropped in the folder.
            if !written.contains(&entry.path()) && entry.file_name().to_str().is_some_and(is_rendered_name) {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
}

/// Sets position and wallpapers; on failure puts back what was there before (best effort).
fn set(images: &[(String, RgbaImage)], written: &[PathBuf]) -> Result<(), String> {
    let dw = desktop_wallpaper().map_err(|e| e.to_string())?;
    let ids: Vec<HSTRING> = images.iter().map(|(id, _)| HSTRING::from(id.as_str())).collect();
    unsafe {
        // Read before changing anything. An unreadable value is just not restored.
        let position = dw.GetPosition().ok();
        let previous: Vec<Option<HSTRING>> = ids
            .iter()
            .map(|id| {
                dw.GetWallpaper(id).ok().and_then(|p| {
                    // A null PWSTR means no previous wallpaper; to_hstring would call wcslen(NULL).
                    if p.is_null() {
                        return None;
                    }
                    let s = p.to_hstring();
                    CoTaskMemFree(Some(p.0 as _));
                    Some(s)
                })
            })
            .collect();
        let mut changed = 0;
        let result = (|| -> windows::core::Result<()> {
            // Images match each monitor's resolution exactly, so Fill shows them 1:1.
            dw.SetPosition(DWPOS_FILL)?;
            for (id, path) in ids.iter().zip(written) {
                dw.SetWallpaper(id, &HSTRING::from(path.as_os_str()))?;
                changed += 1;
            }
            Ok(())
        })();
        if let Err(e) = result {
            for (id, old) in ids.iter().zip(&previous).take(changed) {
                if let Some(old) = old {
                    let _ = dw.SetWallpaper(id, old);
                }
            }
            if let Some(position) = position {
                let _ = dw.SetPosition(position);
            }
            return Err(e.to_string());
        }
    }
    Ok(())
}

/// Files written by `apply`: `<millis>-<index>.png`.
fn is_rendered_name(name: &str) -> bool {
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    name.strip_suffix(".png").and_then(|s| s.split_once('-')).is_some_and(|(a, b)| digits(a) && digits(b))
}

fn remove(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::is_rendered_name;

    #[test]
    fn rendered_name_rule() {
        assert!(is_rendered_name("1700000000000-0.png"));
        for name in ["notes.txt", "1-a.png", "-0.png", "1-0.PNG.bak"] {
            assert!(!is_rendered_name(name), "{name}");
        }
    }
}
