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
