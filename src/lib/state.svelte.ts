import { check, type Update } from "@tauri-apps/plugin-updater";
import { alignOffset, applyProfiles, getCheckUpdates, listMonitors, loadProfiles } from "./api";
import { defaultSource, monitorBounds, type Monitor, type Preview, type Profile, type Source } from "./types";

class AppState {
  profiles = $state<Profile[]>([]);
  selectedName = $state("");
  monitors = $state<Monitor[]>([]);
  selectedMonitorId = $state("");
  previews = $state<Record<string, Preview>>({});
  banner = $state<{ kind: "error" | "warning"; text: string } | null>(null);
  busy = $state(false);
  optionsOpen = $state(false);
  update = $state<Update | null>(null);

  /** Shows `e` in the error banner. */
  fail(e: unknown, prefix = "") {
    this.banner = { kind: "error", text: prefix + String(e) };
  }

  get profile(): Profile {
    return this.profiles.find((p) => p.name === this.selectedName) ?? this.profiles[0];
  }

  /** Source edited by Wallpaper Source / Adjustments: the selected monitor's in PerMonitor mode, the shared one otherwise. */
  get source(): Source {
    const p = this.profile;
    return p.mode === "perMonitor" ? (p.monitors[this.selectedMonitorId] ?? p.shared) : p.shared;
  }

  /** Load error of the image shown in Wallpaper Source, if any. */
  get sourceError(): string | null {
    if (this.profile.mode === "perMonitor") return this.previews[this.selectedMonitorId]?.error ?? null;
    return Object.values(this.previews).find((p) => p.error)?.error ?? null;
  }

  async init() {
    const [{ profiles, warning }, monitors] = await Promise.all([loadProfiles(), listMonitors()]);
    this.monitors = monitors;
    this.selectedMonitorId = monitors[0]?.id ?? "";
    profiles.profiles.forEach((p) => this.fillMonitors(p));
    this.profiles = profiles.profiles;
    this.selectedName = profiles.profiles.some((p) => p.name === profiles.lastSelected) ? profiles.lastSelected! : profiles.profiles[0].name;
    if (warning) this.banner = { kind: "warning", text: warning };
  }

  /** Looks for a newer release (latest.json on GitHub) unless turned off in Options. Offline, or no release yet: stays silent. */
  async checkUpdate() {
    if (!(await getCheckUpdates().catch(() => true))) return;
    this.update = await check({ timeout: 10000 }).catch(() => null);
  }

  /** Every connected monitor gets an entry, so `source` never has to create one while rendering. */
  private fillMonitors(p: Profile) {
    for (const m of this.monitors) p.monitors[m.id] ??= defaultSource();
  }

  nameError(name: string, except?: string): string | null {
    const n = name.trim();
    if (!n) return "Name cannot be empty";
    if (this.profiles.some((p) => p.name === n && p.name !== except)) return "A profile with this name already exists";
    return null;
  }

  newProfile(name: string) {
    const p: Profile = { name, mode: "perMonitor", shared: defaultSource(), monitors: {} };
    this.fillMonitors(p);
    this.profiles.push(p);
    this.selectedName = p.name;
  }

  rename(name: string) {
    this.profile.name = this.selectedName = name;
  }

  saveAs(name: string) {
    const copy = $state.snapshot(this.profile) as Profile;
    copy.name = name;
    this.profiles.push(copy);
    this.selectedName = copy.name;
  }

  deleteCurrent() {
    const name = this.profile.name;
    this.profiles = this.profiles.filter((p) => p.name !== name);
    if (this.profiles.length === 0) this.newProfile("Default");
    this.selectedName = this.profiles[0].name;
  }

  /** Sets the offsets so the image touches an edge (-1/1) or is centered (0) on each axis, in the area it is rendered in. */
  async align(ax: number, ay: number) {
    const src = this.source;
    const ms = this.profile.mode === "span" ? this.monitors : this.monitors.filter((m) => m.id === this.selectedMonitorId);
    const { w, h } = monitorBounds(ms);
    try {
      [src.offsetX, src.offsetY] = await alignOffset(src, [w, h], [ax, ay]);
    } catch (e) {
      this.fail(e);
    }
  }

  /** Saves every profile and applies the selected one. Returns false (and shows the error) on failure. */
  async apply(): Promise<boolean> {
    this.busy = true;
    this.banner = null;
    try {
      await applyProfiles({ lastSelected: this.selectedName, profiles: $state.snapshot(this.profiles) as Profile[] });
      return true;
    } catch (e) {
      this.fail(e);
      return false;
    } finally {
      this.busy = false;
    }
  }
}

export const app = new AppState();
