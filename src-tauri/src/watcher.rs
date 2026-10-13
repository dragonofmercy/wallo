use crate::commands::{self, ApplyLock};
use crate::monitors::{self, Monitor};
use crate::{profile, settings, wallpaper};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread::Thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};

const INTERVAL: Duration = Duration::from_secs(10);

static ENABLED: AtomicBool = AtomicBool::new(false);
/// Set after a failed reapply so its error shows once; cleared by any successful apply or by re-enabling.
static PAUSED: AtomicBool = AtomicBool::new(false);
static LAST: Mutex<Option<Applied>> = Mutex::new(None);
static THREAD: OnceLock<Thread> = OnceLock::new();

/// What the last successful apply put on screen.
#[derive(Clone, Debug)]
pub struct Applied {
    pub monitors: Vec<Monitor>,
    /// (monitor id, PNG path), in apply order.
    pub files: Vec<(String, PathBuf)>,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Nothing,
    /// Same monitors, wallpaper changed by someone else: set the recorded PNGs again.
    Reset,
    Rerender,
}

/// Called by every successful apply (window, tray, watcher).
pub fn record(monitors: Vec<Monitor>, files: Vec<(String, PathBuf)>) {
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = Some(Applied { monitors, files });
    PAUSED.store(false, Ordering::SeqCst);
}

/// Same ids and rects; the display number is only a label.
fn same_layout(a: &[Monitor], b: &[Monitor]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.id == b.id && (a.x, a.y, a.width, a.height) == (b.x, b.y, b.width, b.height))
}

/// `current[i]` is the wallpaper now shown on the monitor of `last.files[i]`.
pub fn decide(last: Option<&Applied>, monitors: &[Monitor], current: &[Option<PathBuf>], exists: impl Fn(&Path) -> bool) -> Action {
    let Some(last) = last else { return Action::Rerender };
    if !same_layout(&last.monitors, monitors) {
        return Action::Rerender;
    }
    let shown = current.len() == last.files.len() && last.files.iter().zip(current).all(|((_, file), cur)| cur.as_ref().is_some_and(|c| c.as_os_str().eq_ignore_ascii_case(file.as_os_str())));
    if shown {
        Action::Nothing
    } else if last.files.iter().all(|(_, file)| exists(file)) {
        Action::Reset
    } else {
        Action::Rerender
    }
}

/// One check. Busy apply lock, unlisted monitors or no monitor (display sleep): skip until the next tick.
fn tick(app: &AppHandle) -> Result<(), String> {
    let lock = app.state::<ApplyLock>().0.clone();
    let Some(_guard) = commands::lock_apply(&lock) else { return Ok(()) };
    let Ok(monitors) = monitors::list() else { return Ok(()) };
    if monitors.is_empty() {
        return Ok(());
    }
    let last = LAST.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let current = match &last {
        Some(l) => wallpaper::current(&l.files.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>())?,
        None => Vec::new(),
    };
    match decide(last.as_ref(), &monitors, &current, Path::exists) {
        Action::Nothing => Ok(()),
        Action::Reset => {
            let Some(last) = last else { return Ok(()) };
            let (ids, paths): (Vec<String>, Vec<PathBuf>) = last.files.into_iter().unzip();
            wallpaper::set(&ids, &paths)
        }
        Action::Rerender => {
            // No profiles.json means no successful apply yet (it is only written by one): nothing to keep, and load() would return a default black profile.
            if !commands::profiles_path().exists() {
                return Ok(());
            }
            let loaded = profile::load(&commands::profiles_path());
            // Corrupt or unreadable file: applying would save defaults over it.
            if let Some(warning) = loaded.warning {
                return Err(warning);
            }
            commands::apply_now(&loaded.profiles)
        }
    }
}

/// Starts the polling thread; the first check runs at once (reapplies the profile at startup when enabled).
pub fn start(app: &AppHandle) {
    ENABLED.store(settings::load(&settings::path()).keep_wallpaper, Ordering::SeqCst);
    let app = app.clone();
    let handle = std::thread::spawn(move || loop {
        if ENABLED.load(Ordering::SeqCst) && !PAUSED.load(Ordering::SeqCst) {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| tick(&app))).unwrap_or_else(|_| Err("unexpected error".to_string()));
            if let Err(e) = result {
                PAUSED.store(true, Ordering::SeqCst);
                crate::tray::error(&app, format!("Keep wallpaper paused: {e}"));
            }
        }
        std::thread::park_timeout(INTERVAL);
    });
    let _ = THREAD.set(handle.thread().clone());
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

/// Also clears a pause and checks at once instead of waiting for the next tick.
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::SeqCst);
    PAUSED.store(false, Ordering::SeqCst);
    if let Some(thread) = THREAD.get() {
        thread.unpark();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(id: &str, x: i32, number: u32) -> Monitor {
        Monitor {
            id: id.into(),
            number,
            x,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    fn applied() -> Applied {
        Applied {
            monitors: vec![mon("A", 0, 1), mon("B", 1920, 2)],
            files: vec![("A".into(), PathBuf::from(r"C:\r\1-0.png")), ("B".into(), PathBuf::from(r"C:\r\1-1.png"))],
        }
    }

    fn shown() -> Vec<Option<PathBuf>> {
        vec![Some(PathBuf::from(r"C:\r\1-0.png")), Some(PathBuf::from(r"C:\r\1-1.png"))]
    }

    #[test]
    fn decide_rerenders_when_nothing_recorded() {
        assert_eq!(decide(None, &[mon("A", 0, 1)], &[], |_| true), Action::Rerender);
    }

    #[test]
    fn decide_nothing_when_unchanged() {
        let a = applied();
        assert_eq!(decide(Some(&a), &a.monitors, &shown(), |_| true), Action::Nothing);
    }

    #[test]
    fn decide_ignores_path_case() {
        let a = applied();
        let cur = vec![Some(PathBuf::from(r"c:\R\1-0.PNG")), Some(PathBuf::from(r"C:\r\1-1.png"))];
        assert_eq!(decide(Some(&a), &a.monitors, &cur, |_| true), Action::Nothing);
    }

    #[test]
    fn decide_ignores_display_number() {
        let a = applied();
        let renumbered = vec![mon("A", 0, 2), mon("B", 1920, 1)];
        assert_eq!(decide(Some(&a), &renumbered, &shown(), |_| true), Action::Nothing);
    }

    #[test]
    fn decide_resets_when_wallpaper_changed() {
        let a = applied();
        let cur = vec![Some(PathBuf::from(r"C:\other.jpg")), Some(PathBuf::from(r"C:\r\1-1.png"))];
        assert_eq!(decide(Some(&a), &a.monitors, &cur, |_| true), Action::Reset);
        assert_eq!(decide(Some(&a), &a.monitors, &[None, None], |_| true), Action::Reset);
    }

    #[test]
    fn decide_rerenders_when_png_missing() {
        let a = applied();
        let cur = vec![Some(PathBuf::from(r"C:\other.jpg")), None];
        assert_eq!(decide(Some(&a), &a.monitors, &cur, |p| !p.ends_with("1-0.png")), Action::Rerender);
    }

    #[test]
    fn decide_rerenders_on_layout_change() {
        let a = applied();
        assert_eq!(decide(Some(&a), &[mon("A", 0, 1)], &shown(), |_| true), Action::Rerender);
        assert_eq!(decide(Some(&a), &[mon("A", 0, 1), mon("C", 1920, 2)], &shown(), |_| true), Action::Rerender);
        let mut resized = a.monitors.clone();
        resized[1].width = 2560;
        assert_eq!(decide(Some(&a), &resized, &shown(), |_| true), Action::Rerender);
        assert_eq!(decide(Some(&a), &[mon("A", 0, 1), mon("B", -1920, 2)], &shown(), |_| true), Action::Rerender);
    }
}
