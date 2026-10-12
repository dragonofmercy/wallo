#![allow(dead_code)] // ponytail: unused until Task 7 wires the module; remove then

use crate::monitors::Monitor;
use crate::profile::{Mode, Profile, Sizing, Source};
use image::imageops::{self, FilterType};
use image::{Rgba, RgbaImage};
use imageproc::geometric_transformations::{warp_into, Interpolation, Projection};
use std::path::Path;
use std::sync::Arc;

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

pub const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
const CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// `v * k` rounded, never 0.
pub fn scaled(v: u32, k: f64) -> u32 {
    ((v as f64 * k).round() as u32).max(1)
}

/// Renders `src` into an area of `area` screen pixels, output scaled by `k`.
pub fn render_area(img: &RgbaImage, native: (u32, u32), src: &Source, area: (u32, u32), k: f64, filter: FilterType) -> RgbaImage {
    let (cw, ch) = (scaled(area.0, k), scaled(area.1, k));
    let mut canvas = RgbaImage::from_pixel(cw, ch, BLACK);
    let (sx, sy) = placement(src.sizing, src.no_upscale, native.0, native.1, area.0, area.1);
    let s = src.scale as f64 / 100.0;
    let (lw, lh) = (native.0 as f64 * sx * s * k, native.1 as f64 * sy * s * k);
    let (ox, oy) = (src.offset_x as f64 * k, src.offset_y as f64 * k);

    let mut source = img.clone();
    if src.flip_h {
        imageops::flip_horizontal_in_place(&mut source);
    }
    if src.flip_v {
        imageops::flip_vertical_in_place(&mut source);
    }

    if src.sizing == Sizing::Tile {
        draw_tiles(&mut canvas, &source, lw, lh, ox, oy, filter);
    } else {
        let (lx, ly) = ((cw as f64 - lw) / 2.0 + ox, (ch as f64 - lh) / 2.0 + oy);
        let angle = (src.rotation as f64).rem_euclid(360.0);
        if angle == 0.0 {
            draw_visible(&mut canvas, &source, (lx, ly, lw, lh), (0.0, 0.0, cw as f64, ch as f64), 0.0, filter);
        } else {
            draw_rotated(&mut canvas, &source, (lx, ly, lw, lh), angle.to_radians(), filter);
        }
    }

    if src.blur > 0.0 {
        canvas = imageops::fast_blur(&canvas, src.blur * k as f32);
    }
    canvas
}

/// Draws the part of the layer `(lx, ly, lw, lh)` (canvas coordinates, before rotation) that intersects `region` `(x0, y0, x1, y1)`,
/// rotated clockwise by `theta` radians around the layer center. Only the matching source pixels are read, so cost stays bounded by the canvas.
/// Downscales resize the crop; upscales warp it bilinearly (the resized piece would otherwise be unbounded).
fn draw_visible(canvas: &mut RgbaImage, source: &RgbaImage, layer: (f64, f64, f64, f64), region: (f64, f64, f64, f64), theta: f64, filter: FilterType) {
    let (lx, ly, lw, lh) = layer;
    let (iw, ih) = source.dimensions();
    let (fx, fy) = (lw / iw as f64, lh / ih as f64);
    let sx0 = ((region.0.max(lx) - lx) / fx).floor().max(0.0) as u32;
    let sy0 = ((region.1.max(ly) - ly) / fy).floor().max(0.0) as u32;
    let sx1 = ((region.2.min(lx + lw) - lx) / fx).ceil().clamp(0.0, iw as f64) as u32;
    let sy1 = ((region.3.min(ly + lh) - ly) / fy).ceil().clamp(0.0, ih as f64) as u32;
    if sx1 <= sx0 || sy1 <= sy0 {
        return;
    }
    let crop = imageops::crop_imm(source, sx0, sy0, sx1 - sx0, sy1 - sy0).to_image();
    let (px, py) = (lx + sx0 as f64 * fx, ly + sy0 as f64 * fy);
    let (cx, cy) = (lx + lw / 2.0, ly + lh / 2.0);
    let rotation = Projection::translate(cx as f32, cy as f32) * Projection::rotate(theta as f32) * Projection::translate(-cx as f32, -cy as f32);
    let (cw, ch) = canvas.dimensions();
    let blank = || RgbaImage::from_pixel(cw, ch, CLEAR);
    let layer_px = if fx <= 1.0 && fy <= 1.0 {
        let (pw, ph) = ((((sx1 - sx0) as f64 * fx).round() as u32).max(1), (((sy1 - sy0) as f64 * fy).round() as u32).max(1));
        let piece = if (pw, ph) == crop.dimensions() { crop } else { imageops::resize(&crop, pw, ph, filter) };
        if theta == 0.0 {
            imageops::overlay(canvas, &piece, px.round() as i64, py.round() as i64);
            return;
        }
        let projection = rotation * Projection::translate(px as f32, py as f32);
        let mut layer_px = blank();
        warp_into(&piece, &projection, Interpolation::Bilinear, CLEAR, &mut layer_px);
        layer_px
    } else {
        // One replicated edge pixel on each side so bilinear sampling is valid up to the crop edges (it needs right < width).
        let (w, h) = crop.dimensions();
        let padded = RgbaImage::from_fn(w + 2, h + 2, |x, y| *crop.get_pixel(x.saturating_sub(1).min(w - 1), y.saturating_sub(1).min(h - 1)));
        // Source pixel i is centered at layer offset (i + 0.5) * f; output pixel d is sampled at d (imageproc has no half-pixel shift).
        let projection = rotation * Projection::translate((px - fx / 2.0 - 0.5) as f32, (py - fy / 2.0 - 0.5) as f32) * Projection::scale(fx as f32, fy as f32);
        let mut layer_px = blank();
        warp_into(&padded, &projection, Interpolation::Bilinear, CLEAR, &mut layer_px);
        if theta == 0.0 {
            // The replicated padding spills up to half a source pixel past the layer edge: cut it off.
            let (x0, x1, y0, y1) = (lx.round(), (lx + lw).round(), ly.round(), (ly + lh).round());
            for (x, y, p) in layer_px.enumerate_pixels_mut() {
                if (x as f64) < x0 || (x as f64) >= x1 || (y as f64) < y0 || (y as f64) >= y1 {
                    *p = CLEAR;
                }
            }
        }
        layer_px
    };
    imageops::overlay(canvas, &layer_px, 0, 0);
}

/// Rotates the layer clockwise by `theta` radians around its center and draws it on the canvas.
fn draw_rotated(canvas: &mut RgbaImage, source: &RgbaImage, layer: (f64, f64, f64, f64), theta: f64, filter: FilterType) {
    let (cw, ch) = (canvas.width() as f64, canvas.height() as f64);
    let (lx, ly, lw, lh) = layer;
    let (cx, cy) = (lx + lw / 2.0, ly + lh / 2.0);
    // Canvas corners rotated back into the unrotated layer frame give the only region that can be visible.
    let (sin, cos) = (-theta).sin_cos();
    let corners = [(0.0, 0.0), (cw, 0.0), (0.0, ch), (cw, ch)].map(|(x, y)| (cx + (x - cx) * cos - (y - cy) * sin, cy + (x - cx) * sin + (y - cy) * cos));
    let region = corners.iter().fold((f64::MAX, f64::MAX, f64::MIN, f64::MIN), |r, &(x, y)| (r.0.min(x), r.1.min(y), r.2.max(x), r.3.max(y)));
    draw_visible(canvas, source, layer, (region.0.floor(), region.1.floor(), region.2.ceil(), region.3.ceil()), theta, filter);
}

/// Tiles the layer over the canvas from its top-left, shifted by the offset. Rotation does not apply to tiles.
fn draw_tiles(canvas: &mut RgbaImage, source: &RgbaImage, lw: f64, lh: f64, ox: f64, oy: f64, filter: FilterType) {
    let (cw, ch) = canvas.dimensions();
    let (tw, th) = ((lw.round() as u32).max(1), (lh.round() as u32).max(1));
    let x0 = (ox.round() as i64).rem_euclid(tw as i64) - tw as i64;
    let y0 = (oy.round() as i64).rem_euclid(th as i64) - th as i64;
    // Small tiles: resize once. Tiles bigger than the canvas: only resize the visible part of each.
    let whole = (tw <= cw && th <= ch).then(|| imageops::resize(source, tw, th, filter));
    let mut y = y0;
    while y < ch as i64 {
        let mut x = x0;
        while x < cw as i64 {
            match &whole {
                Some(tile) => imageops::overlay(canvas, tile, x, y),
                None => {
                    draw_visible(canvas, source, (x as f64, y as f64, tw as f64, th as f64), (0.0, 0.0, cw as f64, ch as f64), 0.0, filter);
                }
            }
            x += tw as i64;
        }
        y += th as i64;
    }
}

/// Loads an image: returns the decoded pixels (possibly downscaled) and the original size. Errors include the path.
pub type Loader<'a> = dyn Fn(&Path) -> Result<(Arc<RgbaImage>, (u32, u32)), String> + 'a;

pub struct Rendered {
    pub id: String,
    pub image: RgbaImage,
    pub error: Option<String>,
}

/// One image per monitor, in `monitors` order, each `scaled(width, k) x scaled(height, k)`.
pub fn render_profile(profile: &Profile, monitors: &[Monitor], k: f64, filter: FilterType, load: &Loader) -> Vec<Rendered> {
    match profile.mode {
        Mode::Span => {
            let Some(bx) = monitors.iter().map(|m| m.x).min() else { return Vec::new() };
            let by = monitors.iter().map(|m| m.y).min().unwrap_or(0);
            let bx1 = monitors.iter().map(|m| m.x + m.width as i32).max().unwrap_or(0);
            let by1 = monitors.iter().map(|m| m.y + m.height as i32).max().unwrap_or(0);
            let (canvas, error) = render_source(&profile.shared, ((bx1 - bx) as u32, (by1 - by) as u32), k, filter, load);
            monitors
                .iter()
                .map(|m| {
                    let x = ((m.x - bx) as f64 * k).round() as u32;
                    let y = ((m.y - by) as f64 * k).round() as u32;
                    let image = imageops::crop_imm(&canvas, x, y, scaled(m.width, k), scaled(m.height, k)).to_image();
                    Rendered { id: m.id.clone(), image, error: error.clone() }
                })
                .collect()
        }
        Mode::Same | Mode::PerMonitor => monitors
            .iter()
            .map(|m| {
                let fallback = Source::default();
                let src = match profile.mode {
                    Mode::Same => &profile.shared,
                    _ => profile.monitors.get(&m.id).unwrap_or(&fallback),
                };
                let (image, error) = render_source(src, (m.width, m.height), k, filter, load);
                Rendered { id: m.id.clone(), image, error }
            })
            .collect(),
    }
}

fn render_source(src: &Source, area: (u32, u32), k: f64, filter: FilterType, load: &Loader) -> (RgbaImage, Option<String>) {
    let black = || RgbaImage::from_pixel(scaled(area.0, k), scaled(area.1, k), BLACK);
    let Some(path) = &src.image_path else { return (black(), None) };
    match load(path) {
        Ok((img, native)) => (render_area(&img, native, src, area, k, filter), None),
        Err(e) => (black(), Some(e)),
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

    use crate::profile::Source;
    use image::imageops::FilterType;
    use image::{Rgba, RgbaImage};

    const RED: Rgba<u8> = Rgba([255, 0, 0, 255]);
    const BLUE: Rgba<u8> = Rgba([0, 0, 255, 255]);

    /// 4x2 image: left half red, right half blue.
    fn halves() -> RgbaImage {
        RgbaImage::from_fn(4, 2, |x, _| if x < 2 { RED } else { BLUE })
    }

    fn src(sizing: Sizing) -> Source {
        Source { sizing, ..Source::default() }
    }

    fn area(img: &RgbaImage, s: &Source, aw: u32, ah: u32) -> RgbaImage {
        render_area(img, img.dimensions(), s, (aw, ah), 1.0, FilterType::Nearest)
    }

    fn is_red(p: &Rgba<u8>) -> bool {
        p[0] > 200 && p[1] < 60 && p[2] < 60
    }

    fn is_blue(p: &Rgba<u8>) -> bool {
        p[2] > 200 && p[0] < 60 && p[1] < 60
    }

    #[test]
    fn center_on_black() {
        let out = area(&halves(), &src(Sizing::Center), 8, 4);
        assert_eq!(out.dimensions(), (8, 4));
        assert_eq!(*out.get_pixel(0, 0), BLACK);
        assert_eq!(*out.get_pixel(2, 0), BLACK);
        assert_eq!(*out.get_pixel(2, 1), RED);
        assert_eq!(*out.get_pixel(4, 1), BLUE);
        assert_eq!(*out.get_pixel(6, 1), BLACK);
    }

    #[test]
    fn center_larger_than_area_is_cropped_centered() {
        let img = RgbaImage::from_fn(8, 2, |x, _| if x < 4 { RED } else { BLUE });
        let out = area(&img, &src(Sizing::Center), 4, 2);
        assert_eq!(*out.get_pixel(1, 0), RED);
        assert_eq!(*out.get_pixel(2, 0), BLUE);
    }

    #[test]
    fn stretch_fills_area() {
        let out = area(&halves(), &src(Sizing::Stretch), 8, 8);
        assert_eq!(*out.get_pixel(0, 0), RED);
        assert_eq!(*out.get_pixel(7, 7), BLUE);
    }

    #[test]
    fn flip_h_swaps_halves() {
        let out = area(&halves(), &Source { flip_h: true, ..src(Sizing::Stretch) }, 4, 2);
        assert_eq!(*out.get_pixel(0, 0), BLUE);
        assert_eq!(*out.get_pixel(3, 0), RED);
    }

    #[test]
    fn flip_v_swaps_top_and_bottom() {
        let img = RgbaImage::from_fn(2, 4, |_, y| if y < 2 { RED } else { BLUE });
        let out = area(&img, &Source { flip_v: true, ..src(Sizing::Stretch) }, 2, 4);
        assert_eq!(*out.get_pixel(0, 0), BLUE);
        assert_eq!(*out.get_pixel(0, 3), RED);
    }

    #[test]
    fn offset_moves_layer() {
        let out = area(&halves(), &Source { offset_x: 2, ..src(Sizing::Center) }, 8, 4);
        assert_eq!(*out.get_pixel(2, 1), BLACK);
        assert_eq!(*out.get_pixel(4, 1), RED);
        assert_eq!(*out.get_pixel(7, 1), BLUE);
    }

    #[test]
    fn scale_percent_multiplies_size() {
        let out = area(&halves(), &Source { scale: 200.0, ..src(Sizing::Center) }, 8, 4);
        assert_eq!(*out.get_pixel(0, 0), RED);
        assert_eq!(*out.get_pixel(7, 3), BLUE);
    }

    #[test]
    fn tile_repeats_from_top_left() {
        let out = area(&halves(), &src(Sizing::Tile), 8, 2);
        assert_eq!(*out.get_pixel(0, 0), RED);
        assert_eq!(*out.get_pixel(2, 0), BLUE);
        assert_eq!(*out.get_pixel(4, 0), RED);
        assert_eq!(*out.get_pixel(6, 1), BLUE);
    }

    #[test]
    fn tile_offset_shifts_origin() {
        let out = area(&halves(), &Source { offset_x: 1, ..src(Sizing::Tile) }, 8, 2);
        assert_eq!(*out.get_pixel(0, 0), BLUE);
        assert_eq!(*out.get_pixel(1, 0), RED);
    }

    #[test]
    fn rotation_90_is_clockwise() {
        let img = RgbaImage::from_fn(16, 16, |_, y| if y < 8 { RED } else { BLUE });
        let out = render_area(&img, (16, 16), &Source { rotation: 90.0, ..src(Sizing::Center) }, (16, 16), 1.0, FilterType::Triangle);
        assert!(is_red(out.get_pixel(12, 8)), "top half must end up on the right");
        assert!(is_blue(out.get_pixel(3, 8)), "bottom half must end up on the left");
    }

    #[test]
    fn huge_scale_stays_bounded() {
        let img = RgbaImage::from_pixel(2000, 2000, RED);
        let out = render_area(&img, (2000, 2000), &Source { scale: 1000.0, ..src(Sizing::Center) }, (100, 100), 1.0, FilterType::Lanczos3);
        assert!(is_red(out.get_pixel(50, 50)));
        assert!(is_red(out.get_pixel(0, 0)));
    }

    #[test]
    fn huge_scale_rotated_stays_bounded() {
        let img = RgbaImage::from_pixel(2000, 2000, RED);
        let out = render_area(
            &img,
            (2000, 2000),
            &Source {
                scale: 1000.0,
                rotation: 45.0,
                ..src(Sizing::Center)
            },
            (100, 100),
            1.0,
            FilterType::Lanczos3,
        );
        assert!(is_red(out.get_pixel(50, 50)));
        assert!(is_red(out.get_pixel(0, 0)));
    }

    #[test]
    fn huge_tile_stays_bounded() {
        let img = RgbaImage::from_pixel(2000, 2000, RED);
        let out = render_area(&img, (2000, 2000), &Source { scale: 1000.0, ..src(Sizing::Tile) }, (100, 100), 1.0, FilterType::Lanczos3);
        assert!(is_red(out.get_pixel(50, 50)));
    }

    #[test]
    fn blur_spreads_into_black() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([255, 255, 255, 255]));
        let out = area(&img, &Source { blur: 2.0, ..src(Sizing::Center) }, 12, 12);
        assert!(out.get_pixel(3, 6)[0] > 0);
        assert!(out.get_pixel(6, 6)[0] > 50);
    }

    #[test]
    fn preview_scale_shrinks_output() {
        let out = render_area(&halves(), (400, 200), &src(Sizing::Stretch), (400, 200), 0.01, FilterType::Triangle);
        assert_eq!(out.dimensions(), (4, 2));
    }

    #[test]
    fn tiny_image_huge_upscale_stays_bounded() {
        let img = RgbaImage::from_pixel(1, 1, RED);
        for rotation in [0.0, 30.0] {
            let out = render_area(&img, (1, 1), &Source { scale: 1000.0, rotation, ..src(Sizing::Stretch) }, (5120, 1440), 1.0, FilterType::Lanczos3);
            assert!(is_red(out.get_pixel(2560, 720)), "rotation {rotation}");
        }
    }

    use crate::monitors::Monitor;
    use crate::profile::{Mode, Profile};
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    fn mon(id: &str, x: i32, y: i32, width: u32, height: u32) -> Monitor {
        Monitor { id: id.into(), number: 1, x, y, width, height }
    }

    fn with_image(path: &str, sizing: Sizing) -> Source {
        Source {
            image_path: Some(PathBuf::from(path)),
            sizing,
            ..Source::default()
        }
    }

    #[allow(clippy::type_complexity)]
    fn ok_loader(img: RgbaImage) -> impl Fn(&Path) -> Result<(Arc<RgbaImage>, (u32, u32)), String> {
        let dims = img.dimensions();
        let img = Arc::new(img);
        move |_| Ok((img.clone(), dims))
    }

    #[test]
    fn span_crops_each_monitor_with_negative_x() {
        let img = RgbaImage::from_fn(8, 2, |x, _| if x < 4 { RED } else { BLUE });
        let profile = Profile {
            mode: Mode::Span,
            shared: with_image("a.png", Sizing::Stretch),
            ..Profile::default()
        };
        let monitors = [mon("left", -4, 0, 4, 2), mon("primary", 0, 0, 4, 2)];
        let out = render_profile(&profile, &monitors, 1.0, FilterType::Nearest, &ok_loader(img));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "left");
        assert_eq!(out[0].image.dimensions(), (4, 2));
        assert!(out[0].image.pixels().all(|p| *p == RED));
        assert!(out[1].image.pixels().all(|p| *p == BLUE));
        assert!(out.iter().all(|r| r.error.is_none()));
    }

    #[test]
    fn same_renders_shared_source_on_each_monitor() {
        let profile = Profile {
            mode: Mode::Same,
            shared: with_image("a.png", Sizing::Stretch),
            ..Profile::default()
        };
        let monitors = [mon("a", 0, 0, 4, 2), mon("b", 4, 0, 6, 3)];
        let out = render_profile(&profile, &monitors, 1.0, FilterType::Nearest, &ok_loader(halves()));
        assert_eq!(out[1].image.dimensions(), (6, 3));
        assert_eq!(*out[0].image.get_pixel(0, 0), RED);
        assert_eq!(*out[1].image.get_pixel(5, 2), BLUE);
    }

    #[test]
    fn per_monitor_uses_own_source_and_black_when_missing() {
        let mut profile = Profile { mode: Mode::PerMonitor, ..Profile::default() };
        profile.monitors.insert("a".into(), with_image("a.png", Sizing::Stretch));
        let monitors = [mon("a", 0, 0, 4, 2), mon("unknown", 4, 0, 4, 2)];
        let out = render_profile(&profile, &monitors, 1.0, FilterType::Nearest, &ok_loader(halves()));
        assert_eq!(*out[0].image.get_pixel(0, 0), RED);
        assert!(out[1].image.pixels().all(|p| *p == BLACK));
        assert!(out[1].error.is_none());
    }

    #[test]
    fn source_without_image_is_black_without_error() {
        let profile = Profile { mode: Mode::Same, ..Profile::default() };
        let out = render_profile(&profile, &[mon("a", 0, 0, 4, 2)], 1.0, FilterType::Nearest, &ok_loader(halves()));
        assert!(out[0].image.pixels().all(|p| *p == BLACK));
        assert!(out[0].error.is_none());
    }

    #[test]
    fn missing_image_reports_error_and_renders_black() {
        let profile = Profile {
            mode: Mode::Same,
            shared: with_image("gone.png", Sizing::Center),
            ..Profile::default()
        };
        let fail = |p: &Path| -> Result<(Arc<RgbaImage>, (u32, u32)), String> { Err(format!("{}: not found", p.display())) };
        let out = render_profile(&profile, &[mon("a", 0, 0, 4, 2)], 1.0, FilterType::Nearest, &fail);
        assert!(out[0].image.pixels().all(|p| *p == BLACK));
        assert_eq!(out[0].error.as_deref(), Some("gone.png: not found"));
    }

    #[test]
    fn span_preview_scale_keeps_slices_aligned() {
        let img = RgbaImage::from_fn(8, 2, |x, _| if x < 4 { RED } else { BLUE });
        let profile = Profile {
            mode: Mode::Span,
            shared: with_image("a.png", Sizing::Stretch),
            ..Profile::default()
        };
        let monitors = [mon("left", -400, 0, 400, 200), mon("primary", 0, 0, 400, 200)];
        let out = render_profile(&profile, &monitors, 0.01, FilterType::Nearest, &ok_loader(img));
        assert_eq!(out[0].image.dimensions(), (4, 2));
        assert_eq!(out[1].image.dimensions(), (4, 2));
        assert_eq!(*out[1].image.get_pixel(0, 0), BLUE);
    }
}
