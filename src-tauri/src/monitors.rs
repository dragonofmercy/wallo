use serde::{Deserialize, Serialize};
use windows::core::PCWSTR;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED};
use windows::Win32::UI::Shell::{DesktopWallpaper, IDesktopWallpaper};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    /// Device path from IDesktopWallpaper, stable across reboots.
    pub id: String,
    /// 1-based display number shown in the UI.
    pub number: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub fn desktop_wallpaper() -> windows::core::Result<IDesktopWallpaper> {
    unsafe {
        // Already initialized (S_FALSE or RPC_E_CHANGED_MODE) is fine: we only need COM usable on this thread.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL)
    }
}

/// Attached monitors with their rect in virtual desktop coordinates.
pub fn list() -> Result<Vec<Monitor>, String> {
    let dw = desktop_wallpaper().map_err(|e| e.to_string())?;
    let mut monitors = Vec::new();
    unsafe {
        let count = dw.GetMonitorDevicePathCount().map_err(|e| e.to_string())?;
        for i in 0..count {
            let path = dw.GetMonitorDevicePathAt(i).map_err(|e| e.to_string())?;
            let id = path.to_string().unwrap_or_default();
            let rect = dw.GetMonitorRECT(PCWSTR(path.0));
            CoTaskMemFree(Some(path.0 as _));
            // Detached monitors stay in the list but have no rect.
            let Ok(r) = rect else { continue };
            if r.right <= r.left || r.bottom <= r.top {
                continue;
            }
            monitors.push(Monitor {
                id,
                number: monitors.len() as u32 + 1,
                x: r.left,
                y: r.top,
                width: (r.right - r.left) as u32,
                height: (r.bottom - r.top) as u32,
            });
        }
    }
    Ok(monitors)
}

#[cfg(test)]
mod tests {
    /// Needs a real desktop session: `cargo test monitors -- --ignored`.
    #[test]
    #[ignore]
    fn lists_at_least_one_monitor() {
        let monitors = super::list().unwrap();
        assert!(!monitors.is_empty());
        assert!(monitors.iter().all(|m| m.width > 0 && m.height > 0 && !m.id.is_empty()));
        assert_eq!(monitors[0].number, 1);
    }
}
