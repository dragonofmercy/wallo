#![allow(dead_code)] // ponytail: unused until Task 7 wires the module; remove then

use crate::profile::Sizing;

/// Scale factors (x, y) that fit an `iw x ih` image into an `aw x ah` area for the given sizing mode.
pub fn placement(sizing: Sizing, no_upscale: bool, iw: u32, ih: u32, aw: u32, ah: u32) -> (f64, f64) {
    let (rx, ry) = (aw as f64 / iw as f64, ah as f64 / ih as f64);
    let (sx, sy) = match sizing {
        Sizing::Center | Sizing::Tile => return (1.0, 1.0),
        Sizing::Stretch => (rx, ry),
        Sizing::FitBestNoClip => (rx.min(ry), rx.min(ry)),
        Sizing::FitBestClip => (rx.max(ry), rx.max(ry)),
        Sizing::FitWidth => (rx, rx),
        Sizing::FitHeight => (ry, ry),
    };
    if no_upscale {
        (sx.min(1.0), sy.min(1.0))
    } else {
        (sx, sy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_and_tile_keep_native_size() {
        assert_eq!(placement(Sizing::Center, false, 100, 50, 400, 300), (1.0, 1.0));
        assert_eq!(placement(Sizing::Tile, true, 100, 50, 400, 300), (1.0, 1.0));
    }

    #[test]
    fn stretch_scales_each_axis() {
        assert_eq!(placement(Sizing::Stretch, false, 100, 50, 400, 300), (4.0, 6.0));
    }

    #[test]
    fn fit_best_no_clip_uses_min() {
        assert_eq!(placement(Sizing::FitBestNoClip, false, 100, 50, 400, 300), (4.0, 4.0));
    }

    #[test]
    fn fit_best_clip_uses_max() {
        assert_eq!(placement(Sizing::FitBestClip, false, 100, 50, 400, 300), (6.0, 6.0));
    }

    #[test]
    fn fit_width_and_height() {
        assert_eq!(placement(Sizing::FitWidth, false, 100, 50, 400, 300), (4.0, 4.0));
        assert_eq!(placement(Sizing::FitHeight, false, 100, 50, 400, 300), (6.0, 6.0));
    }

    #[test]
    fn no_upscale_caps_at_one() {
        assert_eq!(placement(Sizing::FitBestClip, true, 100, 50, 400, 300), (1.0, 1.0));
        assert_eq!(placement(Sizing::Stretch, true, 800, 50, 400, 300), (0.5, 1.0));
        assert_eq!(placement(Sizing::FitBestNoClip, true, 800, 600, 400, 300), (0.5, 0.5));
    }
}
