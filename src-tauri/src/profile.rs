use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{self, Write};
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
    // Own default: a file without `profiles` must read as empty (unrecognized), not as the fresh-install set.
    #[serde(default)]
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

impl Source {
    /// Same ranges as `cleanSource` in api.ts; non-finite values fall back to the default.
    pub fn clamped(&self) -> Source {
        let fin = |v: f32, def: f32, lo: f32, hi: f32| if v.is_finite() { v.clamp(lo, hi) } else { def };
        Source {
            scale: fin(self.scale, 100.0, 1.0, 1000.0),
            rotation: fin(self.rotation, 0.0, -360.0, 360.0),
            offset_x: self.offset_x.clamp(-10000, 10000),
            offset_y: self.offset_y.clamp(-10000, 10000),
            blur: fin(self.blur, 0.0, 0.0, 100.0),
            ..self.clone()
        }
    }
}

impl Profiles {
    pub fn selected(&self) -> Option<&Profile> {
        let name = self.last_selected.as_deref()?;
        self.profiles.iter().find(|p| p.name == name)
    }
}

pub struct Loaded {
    pub profiles: Profiles,
    pub warning: Option<String>,
    /// The file exists but could not be read: saving would overwrite it.
    pub save_blocked: bool,
}

/// Never fails: a missing or empty file gives the default set, a corrupt or unrecognized one is moved to a unique `.bak` and reported as a warning,
/// an unreadable one gives the default set with `save_blocked` set.
pub fn load(path: &Path) -> Loaded {
    let fresh = |warning, save_blocked| Loaded {
        profiles: Profiles::default(),
        warning,
        save_blocked,
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return fresh(None, false),
        Err(e) => {
            let msg = format!(
                "Cannot read {} ({e}). Profiles are not saved until the file can be read (close and reopen the Wallo window to retry), to avoid overwriting it.",
                path.display()
            );
            return fresh(Some(msg), true);
        }
    };
    // Editors on Windows like to prepend a BOM, which serde_json rejects.
    let body = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    if body.iter().all(u8::is_ascii_whitespace) {
        return fresh(None, false);
    }
    let problem = match serde_json::from_slice::<Profiles>(body) {
        Ok(mut p) if !p.profiles.is_empty() => {
            for prof in &mut p.profiles {
                prof.shared = prof.shared.clamped();
                for s in prof.monitors.values_mut() {
                    *s = s.clamped();
                }
            }
            dedupe_names(&mut p.profiles);
            return Loaded { profiles: p, warning: None, save_blocked: false };
        }
        Ok(_) => "no profiles found".to_string(),
        Err(e) => e.to_string(),
    };
    let bak = backup_path(path);
    let moved = match fs::rename(path, &bak) {
        Ok(()) => format!("moved to {}", bak.display()),
        Err(re) => format!("could not be moved to {} ({re}) and will be overwritten", bak.display()),
    };
    fresh(Some(format!("profiles.json was corrupt ({problem}), {moved}")), false)
}

/// `profiles.json.<unix-millis>.bak`, bumped until free so an earlier backup is never overwritten.
fn backup_path(path: &Path) -> PathBuf {
    let mut ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
    loop {
        let bak = path.with_extension(format!("json.{ms}.bak"));
        if !bak.exists() {
            return bak;
        }
        ms += 1;
    }
}

/// Later duplicates become "<name> (2)", "<name> (3)"...; the first keeps its name so `last_selected` still resolves to it.
fn dedupe_names(profiles: &mut [Profile]) {
    let mut used: HashSet<String> = profiles.iter().map(|p| p.name.clone()).collect();
    let mut seen = HashSet::new();
    for p in profiles {
        if seen.insert(p.name.clone()) {
            continue;
        }
        let name = (2..).map(|n| format!("{} ({n})", p.name)).find(|c| !used.contains(c)).unwrap();
        used.insert(name.clone());
        seen.insert(name.clone());
        p.name = name;
    }
}

/// Atomic: writes and syncs a temp file then renames it over the target.
pub fn save<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    let mut f = fs::File::create(&tmp)?;
    f.write_all(serde_json::to_string_pretty(value)?.as_bytes())?;
    f.sync_all()?;
    drop(f);
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

    fn baks(path: &Path) -> Vec<PathBuf> {
        let rd = fs::read_dir(path.parent().unwrap()).unwrap();
        rd.map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|x| x == "bak")).collect()
    }

    fn load_str(name: &str, json: &str) -> Loaded {
        let path = tmp(name);
        fs::write(&path, json).unwrap();
        load(&path)
    }

    #[test]
    fn missing_file_gives_default_without_warning() {
        let Loaded { profiles: p, warning, .. } = load(&tmp("missing"));
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
        let Loaded { profiles: loaded, warning, .. } = load(&path);
        assert_eq!(loaded, profiles);
        assert!(warning.is_none());
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn corrupt_file_is_backed_up_and_replaced_by_default() {
        let path = tmp("corrupt");
        fs::write(&path, "{ not json").unwrap();
        let Loaded { profiles: p, warning, save_blocked } = load(&path);
        assert_eq!(p, Profiles::default());
        assert!(warning.is_some());
        assert!(!save_blocked);
        assert_eq!(baks(&path).len(), 1);
        assert!(!path.exists());
    }

    #[test]
    fn invalid_utf8_is_backed_up() {
        let path = tmp("utf8");
        fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
        let l = load(&path);
        assert_eq!(l.profiles, Profiles::default());
        assert!(l.warning.unwrap().contains("moved to"));
        assert!(!l.save_blocked);
        assert_eq!(baks(&path).len(), 1);
    }

    #[test]
    fn unreadable_file_blocks_saving() {
        let path = tmp("unreadable");
        fs::create_dir(&path).unwrap();
        let l = load(&path);
        assert!(l.save_blocked);
        assert!(l.warning.is_some());
        assert!(path.is_dir());
    }

    #[test]
    fn missing_fields_take_defaults() {
        let path = tmp("partial");
        fs::write(&path, r#"{"profiles":[{"name":"A"}]}"#).unwrap();
        let p = load(&path).profiles;
        assert_eq!(p.profiles[0].mode, Mode::PerMonitor);
        assert_eq!(p.profiles[0].shared.scale, 100.0);
        assert_eq!(p.profiles[0].shared.sizing, Sizing::FitBestClip);
    }

    #[test]
    fn empty_profile_list_gives_default_with_backup() {
        let path = tmp("empty");
        fs::write(&path, r#"{"profiles":[]}"#).unwrap();
        let l = load(&path);
        assert_eq!(l.profiles, Profiles::default());
        assert!(l.warning.is_some());
        assert_eq!(baks(&path).len(), 1);
    }

    #[test]
    fn second_corruption_keeps_first_backup() {
        let path = tmp("twobak");
        fs::write(&path, "{ one").unwrap();
        load(&path);
        fs::write(&path, "{ two").unwrap();
        load(&path);
        let mut contents: Vec<_> = baks(&path).iter().map(|b| fs::read_to_string(b).unwrap()).collect();
        contents.sort();
        assert_eq!(contents, ["{ one", "{ two"]);
    }

    #[test]
    fn bom_file_loads() {
        let path = tmp("bom");
        let mut data = vec![0xEF, 0xBB, 0xBF];
        data.extend_from_slice(br#"{"profiles":[{"name":"A"}]}"#);
        fs::write(&path, data).unwrap();
        let l = load(&path);
        assert_eq!(l.profiles.profiles[0].name, "A");
        assert!(l.warning.is_none());
    }

    #[test]
    fn unrecognized_content_is_backed_up() {
        for (i, json) in ["{}", r#"{"Profiles":[{"name":"A"}]}"#].into_iter().enumerate() {
            let path = tmp(&format!("unrec{i}"));
            fs::write(&path, json).unwrap();
            let l = load(&path);
            assert_eq!(l.profiles, Profiles::default());
            assert!(l.warning.is_some());
            assert_eq!(baks(&path).len(), 1);
        }
    }

    #[test]
    fn blank_file_gives_default_without_warning() {
        for (i, content) in ["", "  \r\n"].into_iter().enumerate() {
            let l = load_str(&format!("blank{i}"), content);
            assert_eq!(l.profiles, Profiles::default());
            assert!(l.warning.is_none());
        }
    }

    #[test]
    fn out_of_range_values_are_clamped() {
        let l = load_str("clamp", r#"{"profiles":[{"name":"A","shared":{"blur":1e9,"scale":-5,"rotation":999,"offsetX":99999},"monitors":{"m":{"scale":1e39,"blur":-3}}}]}"#);
        let p = &l.profiles.profiles[0];
        assert_eq!((p.shared.blur, p.shared.scale, p.shared.rotation, p.shared.offset_x), (100.0, 1.0, 360.0, 10000));
        assert_eq!((p.monitors["m"].scale, p.monitors["m"].blur), (100.0, 0.0));
    }

    #[test]
    fn duplicate_names_get_suffixes() {
        let l = load_str("dups", r#"{"lastSelected":"A","profiles":[{"name":"A"},{"name":"A"},{"name":"A (2)"},{"name":"A"}]}"#);
        let names: Vec<_> = l.profiles.profiles.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["A", "A (3)", "A (2)", "A (4)"]);
        assert_eq!(l.profiles.selected().unwrap().name, "A");
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
