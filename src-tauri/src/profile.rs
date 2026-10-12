use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Span,
    #[default]
    PerMonitor,
    Same,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Sizing {
    Center,
    Stretch,
    FitBestNoClip,
    #[default]
    FitBestClip,
    FitWidth,
    FitHeight,
    Tile,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Source {
    pub image_path: Option<PathBuf>,
    pub sizing: Sizing,
    pub no_upscale: bool,
    pub scale: f32,
    pub rotation: f32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub blur: f32,
    pub flip_h: bool,
    pub flip_v: bool,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            image_path: None,
            sizing: Sizing::default(),
            no_upscale: false,
            scale: 100.0,
            rotation: 0.0,
            offset_x: 0,
            offset_y: 0,
            blur: 0.0,
            flip_h: false,
            flip_v: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Profile {
    pub name: String,
    pub mode: Mode,
    pub shared: Source,
    /// Keyed by monitor device path. Keeps entries of absent monitors.
    pub monitors: BTreeMap<String, Source>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Profiles {
    pub last_selected: Option<String>,
    pub profiles: Vec<Profile>,
}

impl Default for Profiles {
    fn default() -> Self {
        Self {
            last_selected: Some("Default".into()),
            profiles: vec![Profile { name: "Default".into(), ..Profile::default() }],
        }
    }
}

impl Profiles {
    pub fn selected(&self) -> Option<&Profile> {
        let name = self.last_selected.as_deref()?;
        self.profiles.iter().find(|p| p.name == name)
    }
}

/// Never fails: a missing file gives the default set, a corrupt one is moved to `.json.bak` and reported as a warning.
pub fn load(path: &Path) -> (Profiles, Option<String>) {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return (Profiles::default(), None),
        Err(e) => return (Profiles::default(), Some(format!("Cannot read {}: {e}", path.display()))),
    };
    match serde_json::from_str::<Profiles>(&text) {
        Ok(p) if !p.profiles.is_empty() => (p, None),
        Ok(_) => (Profiles::default(), None),
        Err(e) => {
            let bak = path.with_extension("json.bak");
            let _ = fs::rename(path, &bak);
            (Profiles::default(), Some(format!("profiles.json was corrupt ({e}), moved to {}", bak.display())))
        }
    }
}

/// Atomic: writes a temp file then renames it over the target.
pub fn save(path: &Path, profiles: &Profiles) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(profiles)?)?;
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tmp(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("test-tmp").join(name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.join("profiles.json")
    }

    #[test]
    fn missing_file_gives_default_without_warning() {
        let (p, warning) = load(&tmp("missing"));
        assert_eq!(p, Profiles::default());
        assert_eq!(p.selected().unwrap().name, "Default");
        assert!(warning.is_none());
    }

    #[test]
    fn save_then_load_round_trips() {
        let path = tmp("roundtrip");
        let mut profile = Profile {
            name: "Ahri".into(),
            mode: Mode::Span,
            ..Profile::default()
        };
        profile.shared.image_path = Some(PathBuf::from(r"D:\img\a.jpg"));
        profile.monitors.insert(r"\?\DISPLAY#ABSENT#1".into(), Source { scale: 150.0, flip_h: true, ..Source::default() });
        let profiles = Profiles {
            last_selected: Some("Ahri".into()),
            profiles: vec![profile],
        };
        save(&path, &profiles).unwrap();
        let (loaded, warning) = load(&path);
        assert_eq!(loaded, profiles);
        assert!(warning.is_none());
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn corrupt_file_is_backed_up_and_replaced_by_default() {
        let path = tmp("corrupt");
        fs::write(&path, "{ not json").unwrap();
        let (p, warning) = load(&path);
        assert_eq!(p, Profiles::default());
        assert!(warning.is_some());
        assert!(path.with_extension("json.bak").exists());
        assert!(!path.exists());
    }

    #[test]
    fn missing_fields_take_defaults() {
        let path = tmp("partial");
        fs::write(&path, r#"{"profiles":[{"name":"A"}]}"#).unwrap();
        let (p, _) = load(&path);
        assert_eq!(p.profiles[0].mode, Mode::PerMonitor);
        assert_eq!(p.profiles[0].shared.scale, 100.0);
        assert_eq!(p.profiles[0].shared.sizing, Sizing::FitBestClip);
    }

    #[test]
    fn empty_profile_list_gives_default() {
        let path = tmp("empty");
        fs::write(&path, r#"{"profiles":[]}"#).unwrap();
        assert_eq!(load(&path).0, Profiles::default());
    }

    #[test]
    fn json_uses_camel_case() {
        let json = serde_json::to_string(&Profile { mode: Mode::PerMonitor, ..Profile::default() }).unwrap();
        assert!(json.contains(r#""mode":"perMonitor""#));
        assert!(json.contains(r#""sizing":"fitBestClip""#));
        assert!(json.contains(r#""imagePath":null"#));
        assert!(json.contains(r#""flipH":false"#));
    }
}
