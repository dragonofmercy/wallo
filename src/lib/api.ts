import { invoke } from "@tauri-apps/api/core";
import { defaultAdjustments as d, RANGES, type Monitor, type Preview, type Profile, type Profiles, type Source } from "./types";

/** Coerces empty/invalid inputs (Svelte binds null for an empty number field) to defaults and clamps to the UI ranges. */
function num(v: unknown, def: number, { min, max }: { min: number; max: number }, int = false): number {
  const n = typeof v === "number" && Number.isFinite(v) ? v : def;
  const c = Math.min(max, Math.max(min, n));
  return int ? Math.round(c) : c;
}

export function cleanSource(s: Source): Source {
  return {
    imagePath: s.imagePath || null,
    sizing: s.sizing,
    noUpscale: !!s.noUpscale,
    scale: num(s.scale, d.scale, RANGES.scale),
    rotation: num(s.rotation, d.rotation, RANGES.rotation),
    offsetX: num(s.offsetX, d.offsetX, RANGES.offset, true),
    offsetY: num(s.offsetY, d.offsetY, RANGES.offset, true),
    blur: num(s.blur, d.blur, RANGES.blur),
    flipH: !!s.flipH,
    flipV: !!s.flipV,
  };
}

export function cleanProfile(p: Profile): Profile {
  return { name: p.name, mode: p.mode, shared: cleanSource(p.shared), monitors: Object.fromEntries(Object.entries(p.monitors).map(([id, s]) => [id, cleanSource(s)])) };
}

export const listMonitors = () => invoke<Monitor[]>("list_monitors");
export const loadProfiles = () => invoke<{ profiles: Profiles; warning: string | null }>("load_profiles");
export const renderPreview = (profile: Profile, monitors: Monitor[], width: number) => invoke<Preview[]>("render_preview", { profile: cleanProfile(profile), monitors, width });
export const alignOffset = (source: Source, area: [number, number], anchor: [number, number]) => invoke<[number, number]>("align_offset", { source: cleanSource(source), area, anchor });
export const applyProfiles = (profiles: Profiles) => invoke<void>("apply", { profiles: { lastSelected: profiles.lastSelected, profiles: profiles.profiles.map(cleanProfile) } });
export const getAutostart = () => invoke<boolean>("get_autostart");
export const setAutostart = (enabled: boolean) => invoke<void>("set_autostart", { enabled });
export const getKeepWallpaper = () => invoke<boolean>("get_keep_wallpaper");
export const setKeepWallpaper = (enabled: boolean) => invoke<void>("set_keep_wallpaper", { enabled });
export const getCheckUpdates = () => invoke<boolean>("get_check_updates");
export const setCheckUpdates = (enabled: boolean) => invoke<void>("set_check_updates", { enabled });
export const openGithub = (page: "repo" | "releases") => invoke<void>("open_github", { page });
export const takeOpenOptions = () => invoke<boolean>("take_open_options");
