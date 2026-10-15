export type Mode = "span" | "perMonitor" | "same";
export type Sizing = "center" | "stretch" | "fitBestNoClip" | "fitBestClip" | "fitWidth" | "fitHeight" | "tile";

export interface Source {
  imagePath: string | null;
  sizing: Sizing;
  noUpscale: boolean;
  scale: number;
  rotation: number;
  offsetX: number;
  offsetY: number;
  blur: number;
  flipH: boolean;
  flipV: boolean;
}

export interface Profile {
  name: string;
  mode: Mode;
  shared: Source;
  monitors: Record<string, Source>;
}

export interface Profiles {
  lastSelected: string | null;
  profiles: Profile[];
}

export interface Monitor {
  id: string;
  number: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Preview {
  id: string;
  dataUrl: string;
  error: string | null;
}

/** Bounding box of `ms` in virtual desktop pixels. */
export function monitorBounds(ms: Monitor[]) {
  const x0 = Math.min(...ms.map((m) => m.x));
  const y0 = Math.min(...ms.map((m) => m.y));
  return { x0, y0, w: Math.max(...ms.map((m) => m.x + m.width)) - x0, h: Math.max(...ms.map((m) => m.y + m.height)) - y0 };
}

/** Input ranges, same as `Source::clamped` in Rust. */
export const RANGES = { scale: { min: 1, max: 1000 }, rotation: { min: -360, max: 360 }, offset: { min: -10000, max: 10000 }, blur: { min: 0, max: 100 } };

export const defaultAdjustments = { scale: 100, rotation: 0, offsetX: 0, offsetY: 0, blur: 0, flipH: false, flipV: false };

export function defaultSource(): Source {
  return { imagePath: null, sizing: "fitBestClip", noUpscale: false, ...defaultAdjustments };
}

export const SIZINGS: { value: Sizing; label: string }[] = [
  { value: "center", label: "Center" },
  { value: "stretch", label: "Stretch" },
  { value: "fitBestNoClip", label: "Fit Best and maintain Aspect Ratio (No Clipping)" },
  { value: "fitBestClip", label: "Fit Best and maintain Aspect Ratio (Clip Edges)" },
  { value: "fitWidth", label: "Fit Width and maintain Aspect Ratio" },
  { value: "fitHeight", label: "Fit Height and maintain Aspect Ratio" },
  { value: "tile", label: "Tile" },
];
