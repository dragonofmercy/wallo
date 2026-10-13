use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// App-level settings, kept out of profiles.json: the window's Apply rewrites that whole file from its own copy.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub keep_wallpaper: bool,
    pub check_updates: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { keep_wallpaper: false, check_updates: true }
    }
}

pub fn path() -> PathBuf {
    crate::commands::data_dir().join("settings.json")
}

/// Missing or unreadable file: defaults (nothing worth backing up). Save with `profile::save`.
pub fn load(path: &Path) -> Settings {
    fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(s.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("wallo-settings-tests");
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        let _ = fs::remove_file(&p);
        p
    }

    #[test]
    fn missing_file_gives_defaults() {
        assert_eq!(load(&tmp("missing.json")), Settings::default());
        assert!(!Settings::default().keep_wallpaper);
        assert!(Settings::default().check_updates);
    }

    #[test]
    fn save_then_load_round_trips() {
        let p = tmp("round.json");
        crate::profile::save(&p, &Settings { keep_wallpaper: true, check_updates: false }).unwrap();
        assert!(fs::read_to_string(&p).unwrap().contains("\"keepWallpaper\": true"));
        assert_eq!(load(&p), Settings { keep_wallpaper: true, check_updates: false });
    }

    #[test]
    fn missing_field_takes_its_default() {
        let p = tmp("partial.json");
        fs::write(&p, r#"{ "keepWallpaper": true }"#).unwrap();
        assert_eq!(load(&p), Settings { keep_wallpaper: true, check_updates: true });
    }

    #[test]
    fn garbage_gives_defaults() {
        let p = tmp("garbage.json");
        fs::write(&p, "not json").unwrap();
        assert_eq!(load(&p), Settings::default());
    }
}
