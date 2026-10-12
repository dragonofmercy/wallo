import { invoke } from "@tauri-apps/api/core";
import type { Monitor, Preview, Profile, Profiles, Source } from "./types";

/** Coerces empty/invalid inputs (Svelte binds null for an empty number field) to defaults and clamps to the UI ranges. */
function num(v: unknown, def: number, min: number, max: number, int = false): number {
  const n = typeof v === "number" && Number.isFinite(v) ? v : def;
  const c = Math.min(max, Math.max(min, n));
  return int ? Math.round(c) : c;
}

export function cleanSource(s: Source): Source {
  return {
    imagePath: s.imagePath || null,
    sizing: s.sizing,
    noUpscale: !!s.noUpscale,
    scale: num(s.scale, 100, 1, 1000),
    rotation: num(s.rotation, 0, -360, 360),
    offsetX: num(s.offsetX, 0, -10000, 10000, true),
    offsetY: num(s.offsetY, 0, -10000, 10000, true),
    blur: num(s.blur, 0, 0, 100),
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
export const applyProfiles = (profiles: Profiles) => invoke<void>("apply", { profiles: { lastSelected: profiles.lastSelected, profiles: profiles.profiles.map(cleanProfile) } });
export const getAutostart = () => invoke<boolean>("get_autostart");
export const setAutostart = (enabled: boolean) => invoke<void>("set_autostart", { enabled });
