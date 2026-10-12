use crate::monitors::{self, Monitor};
use crate::profile::{self, Profile, Profiles};
use crate::render::{render_profile, Rendered};
use crate::wallpaper;
use base64::Engine;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader, RgbaImage};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::State;

/// Longest side of cached preview sources.
const PREVIEW_MAX: u32 = 1600;

type Cached = (Arc<RgbaImage>, (u32, u32));

/// Downscaled sources for previews, keyed by path.
// ponytail: never invalidated, an image edited on disk keeps its old preview until restart.
#[derive(Default)]
pub struct ImageCache(Arc<Mutex<HashMap<PathBuf, Cached>>>);

/// Set when profiles.json exists but could not be read at startup: `apply` must not overwrite it.
#[derive(Default)]
pub struct SaveBlocked(AtomicBool);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadResult {
    profiles: Profiles,
    warning: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    id: String,
    data_url: String,
    error: Option<String>,
}

fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("wallo")
}

fn profiles_path() -> PathBuf {
    data_dir().join("profiles.json")
}

fn decode(path: &Path) -> Result<RgbaImage, String> {
    let err = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    Ok(ImageReader::open(path).map_err(|e| err(&e))?.with_guessed_format().map_err(|e| err(&e))?.decode().map_err(|e| err(&e))?.to_rgba8())
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_monitors() -> Result<Vec<Monitor>, String> {
    blocking(monitors::list).await
}

#[tauri::command]
pub fn load_profiles(blocked: State<'_, SaveBlocked>) -> LoadResult {
    let loaded = profile::load(&profiles_path());
    blocked.0.store(loaded.save_blocked, Ordering::SeqCst);
    LoadResult {
        profiles: loaded.profiles,
        warning: loaded.warning,
    }
}

#[tauri::command]
pub async fn render_preview(cache: State<'_, ImageCache>, profile: Profile, monitors: Vec<Monitor>, width: u32) -> Result<Vec<Preview>, String> {
    let cache = cache.0.clone();
    blocking(move || {
        let used: Vec<&PathBuf> = profile.shared.image_path.iter().chain(profile.monitors.values().filter_map(|s| s.image_path.as_ref())).collect();
        cache.lock().unwrap().retain(|path, _| used.contains(&path));
        let bx = monitors.iter().map(|m| m.x).min().unwrap_or(0);
        let bx1 = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap_or(1);
        let k = (width as f64 / (bx1 - bx).max(1) as f64).min(1.0);
        let load = |p: &Path| -> Result<Cached, String> {
            if let Some(hit) = cache.lock().unwrap().get(p) {
                return Ok(hit.clone());
            }
            let full = decode(p)?;
            let native = full.dimensions();
            let small = if native.0.max(native.1) > PREVIEW_MAX {
                DynamicImage::ImageRgba8(full).thumbnail(PREVIEW_MAX, PREVIEW_MAX).to_rgba8()
            } else {
                full
            };
            let entry = (Arc::new(small), native);
            cache.lock().unwrap().insert(p.to_path_buf(), entry.clone());
            Ok(entry)
        };
        render_profile(&profile, &monitors, k, FilterType::Triangle, &load)
            .into_iter()
            .map(|Rendered { id, image, error }| {
                let mut png = Vec::new();
                image.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).map_err(|e| e.to_string())?;
                Ok(Preview {
                    id,
                    data_url: format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)),
                    error,
                })
            })
            .collect()
    })
    .await
}

#[tauri::command]
pub async fn apply(blocked: State<'_, SaveBlocked>, profiles: Profiles) -> Result<(), String> {
    if blocked.0.load(Ordering::SeqCst) {
        return Err("profiles.json could not be read at startup, restart Wallo before applying to avoid overwriting it".into());
    }
    blocking(move || {
        profile::save(&profiles_path(), &profiles).map_err(|e| format!("Cannot save profiles: {e}"))?;
        let selected = profiles.selected().ok_or("No profile selected")?;
        let monitors = monitors::list()?;
        // Same mode renders one file on every monitor: decode each path once.
        let decoded: RefCell<HashMap<PathBuf, Cached>> = RefCell::default();
        let load = |p: &Path| -> Result<Cached, String> {
            if let Some(hit) = decoded.borrow().get(p) {
                return Ok(hit.clone());
            }
            let img = decode(p)?;
            let dims = img.dimensions();
            let entry = (Arc::new(img), dims);
            decoded.borrow_mut().insert(p.to_path_buf(), entry.clone());
            Ok(entry)
        };
        let rendered = render_profile(selected, &monitors, 1.0, FilterType::Lanczos3, &load);
        if let Some(error) = rendered.iter().find_map(|r| r.error.clone()) {
            return Err(error);
        }
        let images: Vec<(String, RgbaImage)> = rendered.into_iter().map(|r| (r.id, r.image)).collect();
        wallpaper::apply(&data_dir().join("rendered"), images)
    })
    .await
}
