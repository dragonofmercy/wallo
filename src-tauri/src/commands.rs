use crate::monitors::{self, Monitor};
use crate::profile::{self, Profile, Profiles, Source};
use crate::render::{self, render_profile, Cached, Rendered};
use crate::{settings, wallpaper, watcher};
use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};
use base64::Engine;
use image::codecs::png::{self, CompressionType, PngEncoder};
use image::imageops::FilterType;
use image::ImageReader;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use tauri::{AppHandle, State};

/// Longest side of cached preview sources.
const PREVIEW_MAX: u32 = 1600;

/// Downscaled sources for previews, keyed by path.
// ponytail: only cleared when the window is closed, an image edited on disk keeps its old preview until then.
#[derive(Default)]
pub struct ImageCache(Arc<Mutex<HashMap<PathBuf, Cached>>>);

impl ImageCache {
    /// Frees the previews (the app lives on in the tray after the window is closed).
    pub(crate) fn clear(&self) {
        if let Ok(mut map) = self.0.lock() {
            map.clear();
        }
    }
}

/// Set when profiles.json exists but could not be read at startup: `apply` must not overwrite it.
static SAVE_BLOCKED: AtomicBool = AtomicBool::new(false);

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

/// Serializes applies: the window's Apply, tray clicks and the watcher must not write wallpapers concurrently.
pub(crate) static APPLY_LOCK: Mutex<()> = Mutex::new(());

/// None when another apply holds the lock. A poisoned lock (panic during an apply) is still usable.
pub(crate) fn lock_apply(lock: &Mutex<()>) -> Option<MutexGuard<'_, ()>> {
    match lock.try_lock() {
        Ok(guard) => Some(guard),
        Err(TryLockError::Poisoned(e)) => Some(e.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

/// Same folder as Tauri's app config dir (named after the bundle identifier), where the window-state plugin writes.
pub(crate) fn data_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default()).join("com.wallo.app")
}

pub(crate) fn profiles_path() -> PathBuf {
    data_dir().join("profiles.json")
}

/// Where `apply` writes its PNGs.
pub(crate) fn rendered_dir() -> PathBuf {
    data_dir().join("rendered")
}

fn reader(path: &Path) -> Result<ImageReader<BufReader<File>>, String> {
    ImageReader::open(path).and_then(|r| r.with_guessed_format()).map_err(|e| format!("{}: {e}", path.display()))
}

/// Memoized loader over `cache`; `max` downscales to that longest side (previews).
fn cached_loader(cache: &Mutex<HashMap<PathBuf, Cached>>, max: Option<u32>) -> impl Fn(&Path) -> Result<Cached, String> + '_ {
    move |p| {
        if let Some(hit) = cache.lock().unwrap().get(p) {
            return Ok(hit.clone());
        }
        let img = reader(p)?.decode().map_err(|e| format!("{}: {e}", p.display()))?;
        let native = (img.width(), img.height());
        // Downscale before converting to RGBA; into_rgba8 does not copy an image already in RGBA.
        let img = match max {
            Some(max) if native.0.max(native.1) > max => img.thumbnail(max, max),
            _ => img,
        };
        let entry = (Arc::new(img.into_rgba8()), native);
        cache.lock().unwrap().insert(p.to_path_buf(), entry.clone());
        Ok(entry)
    }
}

async fn blocking<T: Send + 'static>(f: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(f).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_monitors() -> Result<Vec<Monitor>, String> {
    blocking(monitors::list).await
}

#[tauri::command(async)]
pub fn load_profiles() -> LoadResult {
    let loaded = profile::load(&profiles_path());
    SAVE_BLOCKED.store(loaded.save_blocked, Ordering::SeqCst);
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
        render_profile(&profile, &monitors, k, FilterType::Triangle, &cached_loader(&cache, Some(PREVIEW_MAX)))
            .into_iter()
            .map(|Rendered { id, image, error }| {
                let mut png = Vec::new();
                // Throwaway image: fast compression.
                image.write_with_encoder(PngEncoder::new_with_quality(&mut png, CompressionType::Fast, png::FilterType::Adaptive)).map_err(|e| e.to_string())?;
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

/// Offsets aligning `source` in an `area` of screen pixels (see `render::align_offset`). Reads only the image header.
#[tauri::command]
pub async fn align_offset(source: Source, area: (u32, u32), anchor: (i8, i8)) -> Result<(i32, i32), String> {
    blocking(move || {
        let path = source.image_path.clone().ok_or("No image selected")?;
        let native = reader(&path)?.into_dimensions().map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(render::align_offset(&source.clamped(), native, area, anchor))
    })
    .await
}

pub(crate) enum ApplyError {
    /// A source image could not be loaded (missing file, network drive not mounted yet...).
    Source(String),
    Other(String),
}

impl From<String> for ApplyError {
    fn from(e: String) -> Self {
        ApplyError::Other(e)
    }
}

impl From<&str> for ApplyError {
    fn from(e: &str) -> Self {
        ApplyError::Other(e.into())
    }
}

impl From<ApplyError> for String {
    fn from(e: ApplyError) -> Self {
        match e {
            ApplyError::Source(e) | ApplyError::Other(e) => e,
        }
    }
}

/// Renders and applies the selected profile, then saves `profiles` (a failed apply saves nothing). Blocking; the caller holds the apply lock.
pub(crate) fn apply_now(profiles: &Profiles) -> Result<(), ApplyError> {
    let selected = profiles.selected().ok_or("No profile selected")?;
    let monitors = monitors::list()?;
    if monitors.is_empty() {
        return Err("No monitor detected".into());
    }
    // Monitors sharing an image file decode it once.
    let decoded = Mutex::default();
    let rendered = render_profile(selected, &monitors, 1.0, FilterType::Lanczos3, &cached_loader(&decoded, None));
    if let Some(error) = rendered.iter().find_map(|r| r.error.clone()) {
        return Err(ApplyError::Source(error));
    }
    let files = wallpaper::apply(&rendered_dir(), &rendered)?;
    // Before saving: the wallpapers are on screen even if profiles.json cannot be written.
    watcher::record(monitors, files);
    profile::save(&profiles_path(), profiles).map_err(|e| format!("Wallpapers applied, but profiles could not be saved: {e}").into())
}

/// Applies profiles.json with `select` as the selected profile (None = its lastSelected). Blocking; the caller holds the apply lock.
pub(crate) fn apply_saved(select: Option<String>) -> Result<(), ApplyError> {
    let loaded = profile::load(&profiles_path());
    // Corrupt (just moved to .bak) or unreadable file: applying would save defaults over it.
    if let Some(warning) = loaded.warning {
        return Err(warning.into());
    }
    let mut profiles = loaded.profiles;
    if select.is_some() {
        profiles.last_selected = select;
    }
    apply_now(&profiles)
}

#[tauri::command]
pub async fn apply(app: AppHandle, profiles: Profiles) -> Result<(), String> {
    if SAVE_BLOCKED.load(Ordering::SeqCst) {
        return Err("profiles.json could not be read, close and reopen the Wallo window before applying to avoid overwriting it".into());
    }
    blocking(move || {
        let _guard = lock_apply(&APPLY_LOCK).ok_or("An apply is already running")?;
        Ok(apply_now(&profiles)?)
    })
    .await?;
    crate::tray::refresh_menu(&app);
    Ok(())
}

/// Per-user Run entry (HKCU, never HKLM) with a quoted exe path, so a path containing spaces still launches.
fn autolaunch() -> Result<AutoLaunch, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    AutoLaunchBuilder::new()
        .set_app_name("Wallo")
        .set_app_path(&format!("\"{}\"", exe.display()))
        .set_args(&[crate::TRAY_ARG])
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        .build()
        .map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn get_autostart() -> Result<bool, String> {
    autolaunch()?.is_enabled().map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    // Dev and release share the "Wallo" Run entry: enabling from dev would register target\debug\wallo.exe (a console at every sign-in), disabling would remove the release's entry.
    if cfg!(debug_assertions) {
        return Err("Start with Windows cannot be changed from a dev build".into());
    }
    let launch = autolaunch()?;
    if enabled { launch.enable() } else { launch.disable() }.map_err(|e| e.to_string())?;
    if !enabled && launch.is_enabled().map_err(|e| e.to_string())? {
        return Err("Wallo is still started for all users (registry HKLM); remove that entry as administrator".into());
    }
    Ok(())
}

#[tauri::command]
pub fn get_keep_wallpaper() -> bool {
    watcher::enabled()
}

/// Saved first: a setting that cannot be saved is not turned on.
#[tauri::command(async)]
pub fn set_keep_wallpaper(enabled: bool) -> Result<(), String> {
    save_setting(|s| s.keep_wallpaper = enabled)?;
    watcher::set_enabled(enabled);
    Ok(())
}

#[tauri::command(async)]
pub fn get_check_updates() -> bool {
    settings::load(&settings::path()).check_updates
}

#[tauri::command(async)]
pub fn set_check_updates(enabled: bool) -> Result<(), String> {
    save_setting(|s| s.check_updates = enabled)
}

/// Read-modify-write so changing one setting keeps the others.
fn save_setting(change: impl FnOnce(&mut settings::Settings)) -> Result<(), String> {
    let path = settings::path();
    let mut s = settings::load(&path);
    change(&mut s);
    profile::save(&path, &s).map_err(|e| format!("Cannot save settings: {e}"))
}

const REPO_URL: &str = "https://github.com/dragonofmercy/wallo";

/// Opens a fixed Wallo GitHub page in the default browser, never a URL coming from the frontend.
#[tauri::command]
pub fn open_github(page: &str) -> Result<(), String> {
    let url = match page {
        "repo" => REPO_URL.to_string(),
        "releases" => format!("{REPO_URL}/releases"),
        _ => return Err(format!("Unknown page: {page}")),
    };
    std::process::Command::new("explorer").arg(url).spawn().map(|_| ()).map_err(|e| format!("Cannot open the browser: {e}"))
}

/// True once after the tray "Options" item was clicked.
#[tauri::command]
pub fn take_open_options() -> bool {
    crate::tray::OPEN_OPTIONS.swap(false, Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_apply_is_exclusive() {
        let lock = Mutex::new(());
        let guard = lock_apply(&lock);
        assert!(guard.is_some());
        assert!(lock_apply(&lock).is_none());
        drop(guard);
        assert!(lock_apply(&lock).is_some());
    }

    #[test]
    fn lock_apply_survives_poison() {
        let lock = Arc::new(Mutex::new(()));
        let l = lock.clone();
        let _ = std::thread::spawn(move || {
            let _g = l.lock().unwrap();
            panic!("apply panicked");
        })
        .join();
        assert!(lock.is_poisoned());
        assert!(lock_apply(&lock).is_some());
    }
}
