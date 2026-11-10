use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    rc::Rc,
};

use windows::Win32::Graphics::Direct2D::{
    Common::{D2D_RECT_F, D2D1_COLOR_F},
    D2D1_ANTIALIAS_MODE_ALIASED, D2D1_DRAW_TEXT_OPTIONS_CLIP, D2D1_ELLIPSE, D2D1_ROUNDED_RECT, ID2D1DCRenderTarget,
    ID2D1SolidColorBrush,
};
use windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL;
use windows_numerics::{Matrix3x2, Vector2};

use super::{
    color::Color,
    fonts::{FontStyle, Fonts},
    geometry::{Point, Rect},
    image_brush::{self, ImageBrush, ImageCache},
    image_data::ImageData,
    raster::Mask,
};

const MIN_VISIBLE_ALPHA: f32 = 0.01;

/// What a page draws with. Coordinates are device pixels, so multiply device-independent sizes by `scale`; inside
/// `in_units` they are device-independent units and `scale` is 1.
pub struct Canvas<'a> {
    target: &'a ID2D1DCRenderTarget,
    brush: &'a ID2D1SolidColorBrush,
    images: &'a ImageCache,
    fonts: &'a Fonts,
    scale: f32,
    raster_scale: f32,
    origin: Point,
    in_unit_space: bool,
}

impl From<Rect> for D2D_RECT_F {
    fn from(rect: Rect) -> Self {
        D2D_RECT_F { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom }
    }
}

#[derive(Clone, Copy, Hash)]
enum FlareSide {
    Left,
    Right,
}

impl<'a> Canvas<'a> {
    pub(super) fn new(
        target: &'a ID2D1DCRenderTarget,
        brush: &'a ID2D1SolidColorBrush,
        images: &'a ImageCache,
        fonts: &'a Fonts,
        scale: f32,
    ) -> Self {
        Self {
            target,
            brush,
            images,
            fonts,
            scale,
            raster_scale: 1.0,
            origin: Point::new(0.0, 0.0),
            in_unit_space: false,
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Device pixels per drawing unit for images and masks built at pixel size: 1 outside `in_units`.
    pub fn raster_scale(&self) -> f32 {
        self.raster_scale
    }

    /// Where unit space starts, in device pixels; zero outside `in_units`.
    pub fn origin(&self) -> Point {
        self.origin
    }

    /// Draws in device-independent units with `origin` (device pixels) as the zero point; it cannot be nested.
    pub fn in_units(&self, origin: Point, draw: impl FnOnce(&Canvas)) {
        debug_assert!(!self.in_unit_space, "in_units cannot be nested");
        let previous = self.transform();
        let unit_space =
            Matrix3x2 { M11: self.scale, M12: 0.0, M21: 0.0, M22: self.scale, M31: origin.x, M32: origin.y };
        self.set_transform(&then(&unit_space, &previous));
        draw(&Canvas {
            target: self.target,
            brush: self.brush,
            images: self.images,
            fonts: self.fonts.unit(),
            scale: 1.0,
            raster_scale: self.scale,
            origin,
            in_unit_space: true,
        });
        self.set_transform(&previous);
    }

    pub fn fill_rounded_rect(&self, rect: Rect, radius: f32, color: Color) {
        self.use_color(color);
        unsafe {
            self.target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT { rect: rect.into(), radiusX: radius, radiusY: radius },
                self.brush,
            );
        }
    }

    /// Uploaded once per key; include every visual input in the cache key.
    pub fn cached_image(&self, key: impl Hash, build: impl FnOnce() -> ImageData) -> Option<Rc<ImageBrush>> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        self.images.get_or_insert(hasher.finish(), || image_brush::upload(self.target, &build()).ok())
    }

    pub fn upload(&self, image: &ImageData) -> Option<ImageBrush> {
        image_brush::upload(self.target, image).ok()
    }

    /// Concave top fillets, cached as images to avoid the initial 10 MB cost of Direct2D geometry.
    pub fn fill_top_flares(&self, bounds: Rect, flare: f32, color: Color) {
        let size = flare.round().max(1.0) as u32;
        let rgb = color.rgb_bytes();
        for side in [FlareSide::Left, FlareSide::Right] {
            let Some(brush) = self.cached_image(("flare", side, size, rgb), || flare_image(size, side, color)) else {
                continue;
            };
            // Each image has one extra column that overlaps the shape, so no seam shows between them.
            let left = match side {
                FlareSide::Left => bounds.left.round() - size as f32,
                FlareSide::Right => bounds.right.round() - 1.0,
            };
            let rect = Rect::new(left, bounds.top, left + size as f32 + 1.0, bounds.top + size as f32);
            self.image(&brush, rect, 0.0, color.a);
        }
    }

    pub fn fill_circle(&self, center: Point, radius: f32, color: Color) {
        self.use_color(color);
        unsafe {
            self.target.FillEllipse(
                &D2D1_ELLIPSE { point: Vector2 { X: center.x, Y: center.y }, radiusX: radius, radiusY: radius },
                self.brush,
            );
        }
    }

    /// Returns device-independent pixels, unlike drawing coordinates.
    pub fn measure(&self, style: &FontStyle, text: &str) -> f32 {
        self.fonts.measure(style, text)
    }

    pub fn text(&self, style: &FontStyle, text: &str, rect: Rect, color: Color) {
        if color.a <= MIN_VISIBLE_ALPHA || text.is_empty() {
            return;
        }
        self.use_color(color);
        if style.tabular {
            let Some(layout) = self.fonts.layout(style, text, rect.width(), rect.height()) else { return };
            unsafe {
                self.target.DrawTextLayout(
                    Vector2 { X: rect.left, Y: rect.top },
                    &layout,
                    self.brush,
                    D2D1_DRAW_TEXT_OPTIONS_CLIP,
                );
            }
            return;
        }
        let Some(format) = self.fonts.format(style) else { return };
        let bounds: D2D_RECT_F = rect.into();
        self.fonts.with_wide_text(text, |wide_text| unsafe {
            self.target.DrawText(
                wide_text,
                &format,
                &bounds,
                self.brush,
                D2D1_DRAW_TEXT_OPTIONS_CLIP,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        });
    }

    pub fn image(&self, image: &ImageBrush, rect: Rect, radius: f32, opacity: f32) {
        let pixel_scale = rect.width() / image.width as f32;
        let brush = image.brush();
        unsafe {
            brush.SetTransform(&Matrix3x2 {
                M11: pixel_scale,
                M12: 0.0,
                M21: 0.0,
                M22: pixel_scale,
                M31: rect.left,
                M32: rect.top,
            });
            brush.SetOpacity(opacity);
            self.target.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT { rect: rect.into(), radiusX: radius, radiusY: radius },
                brush,
            );
        }
    }

    pub fn clipped(&self, rect: Rect, draw: impl FnOnce()) {
        unsafe { self.target.PushAxisAlignedClip(&rect.into(), D2D1_ANTIALIAS_MODE_ALIASED) };
        draw();
        unsafe { self.target.PopAxisAlignedClip() };
    }

    /// Zooms what `draw` paints around `origin`, in the coordinates this canvas draws in, on top of the current transform.
    pub fn scaled_about(&self, origin: Point, factor: f32, draw: impl FnOnce()) {
        let previous = self.transform();
        let zoom = Matrix3x2 {
            M11: factor,
            M12: 0.0,
            M21: 0.0,
            M22: factor,
            M31: origin.x * (1.0 - factor),
            M32: origin.y * (1.0 - factor),
        };
        self.set_transform(&then(&zoom, &previous));
        draw();
        self.set_transform(&previous);
    }

    fn transform(&self) -> Matrix3x2 {
        let mut transform = Matrix3x2::identity();
        unsafe { self.target.GetTransform(&mut transform) };
        transform
    }

    fn set_transform(&self, transform: &Matrix3x2) {
        unsafe { self.target.SetTransform(transform) };
    }

    fn use_color(&self, color: Color) {
        unsafe { self.brush.SetColor(&D2D1_COLOR_F { r: color.r, g: color.g, b: color.b, a: color.a }) };
    }
}

/// A transform that applies `first`, then `second` (Direct2D multiplies row vectors).
fn then(first: &Matrix3x2, second: &Matrix3x2) -> Matrix3x2 {
    Matrix3x2 {
        M11: first.M11 * second.M11 + first.M12 * second.M21,
        M12: first.M11 * second.M12 + first.M12 * second.M22,
        M21: first.M21 * second.M11 + first.M22 * second.M21,
        M22: first.M21 * second.M12 + first.M22 * second.M22,
        M31: first.M31 * second.M11 + first.M32 * second.M21 + second.M31,
        M32: first.M31 * second.M12 + first.M32 * second.M22 + second.M32,
    }
}

/// The solid region is outside the circle; an extra column overlaps the adjoining shape.
fn flare_image(size: u32, side: FlareSide, color: Color) -> ImageData {
    let (width, radius) = (size + 1, size as f32);
    let mask = Mask::from_distance(width, size, |point| {
        let x = match side {
            FlareSide::Left => point.x,
            FlareSide::Right => width as f32 - point.x,
        };
        // Negative inside the solid part, which is the outside of the circle.
        radius - x.hypot(point.y - radius)
    });
    mask.tinted(color.r, color.g, color.b)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::gfx::{
        Align, UI_FONT, Weight,
        test_support::{SURFACE_WIDTH, render},
    };

    /// Same face and size as the 13 px regular style the pages use; `gfx` cannot import `ui`.
    const REGULAR_13: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Leading);

    fn ink_columns(pixels: &[u8]) -> (usize, usize) {
        let row_pixels = SURFACE_WIDTH as usize;
        let inked: Vec<usize> = pixels
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(_, pixel)| pixel[3] > 24)
            .map(|(index, _)| index % row_pixels)
            .collect();
        (*inked.iter().min().unwrap(), *inked.iter().max().unwrap())
    }

    #[test]
    fn a_rect_drawn_in_units_covers_the_same_pixels_as_one_scaled_by_hand() {
        let in_units = render(1.5, |canvas| {
            canvas.in_units(Point::new(7.0, 3.0), |canvas| {
                canvas.fill_rounded_rect(Rect::new(10.0, 10.0, 30.0, 20.0), 0.0, Color::WHITE);
            });
        });
        let by_hand = render(1.5, |canvas| {
            canvas.fill_rounded_rect(Rect::new(22.0, 18.0, 52.0, 33.0), 0.0, Color::WHITE);
        });
        assert_eq!(in_units, by_hand);
    }

    #[test]
    fn zooming_inside_a_unit_space_keeps_the_unit_transform() {
        let in_units = render(1.5, |canvas| {
            canvas.in_units(Point::new(7.0, 3.0), |canvas| {
                canvas.scaled_about(Point::new(20.0, 16.0), 2.0, || {
                    canvas.fill_rounded_rect(Rect::new(10.0, 10.0, 30.0, 20.0), 0.0, Color::WHITE);
                });
                canvas.fill_rounded_rect(Rect::new(50.0, 2.0, 60.0, 8.0), 0.0, Color::WHITE);
            });
        });
        // The zoomed rect spans units (0,4)-(40,24); the one drawn after it sits in units unzoomed.
        let by_hand = render(1.5, |canvas| {
            canvas.fill_rounded_rect(Rect::new(7.0, 9.0, 67.0, 39.0), 0.0, Color::WHITE);
            canvas.fill_rounded_rect(Rect::new(82.0, 6.0, 97.0, 15.0), 0.0, Color::WHITE);
        });
        assert_eq!(in_units, by_hand);
    }

    #[test]
    fn text_measured_in_units_is_within_a_pixel_of_its_drawn_width() {
        let text = "Write the v0.5 release notes";
        let measured_units = Cell::new(0.0_f32);
        let pixels = render(1.25, |canvas| {
            canvas.in_units(Point::new(10.0, 10.0), |canvas| {
                measured_units.set(canvas.measure(&REGULAR_13, text));
                let area = Rect::new(0.0, 0.0, measured_units.get() + 20.0, 20.0);
                canvas.text(&REGULAR_13, text, area, Color::WHITE);
            });
        });
        let (first, last) = ink_columns(&pixels);
        let inked_width = (last - first + 1) as f32;
        let measured_pixels = measured_units.get() * 1.25;
        assert!((inked_width - measured_pixels).abs() <= 1.0, "inked {inked_width}, measured {measured_pixels}");
    }

    #[test]
    fn the_canvas_reports_the_device_scale_only_inside_a_unit_space() {
        let seen = Cell::new([0.0_f32; 4]);
        let origins = Cell::new([Point::new(-1.0, -1.0); 2]);
        render(1.5, |canvas| {
            let outside = [canvas.scale(), canvas.raster_scale()];
            let outside_origin = canvas.origin();
            canvas.in_units(Point::new(7.0, 3.0), |canvas| {
                seen.set([outside[0], outside[1], canvas.scale(), canvas.raster_scale()]);
                origins.set([outside_origin, canvas.origin()]);
            });
        });
        assert_eq!(seen.get(), [1.5, 1.0, 1.0, 1.5]);
        assert_eq!(origins.get(), [Point::new(0.0, 0.0), Point::new(7.0, 3.0)]);
    }

    fn coverage(image: &ImageData, x: u32, y: u32) -> u8 {
        image.bgra_premultiplied[((y * image.width + x) * 4 + 3) as usize]
    }

    #[test]
    fn a_left_flare_is_solid_at_the_top_edge_and_empty_where_the_curve_meets_the_side() {
        let image = flare_image(10, FlareSide::Left, Color::BLACK);
        assert_eq!((image.width, image.height), (11, 10));
        assert_eq!(coverage(&image, 9, 0), 255);
        assert_eq!(coverage(&image, 0, 9), 0);
        assert_eq!(coverage(&image, 10, 9), 255);
    }

    #[test]
    fn the_right_flare_mirrors_the_left_one() {
        let left = flare_image(10, FlareSide::Left, Color::BLACK);
        let right = flare_image(10, FlareSide::Right, Color::BLACK);
        for y in 0..10 {
            for x in 0..11 {
                assert_eq!(coverage(&left, x, y), coverage(&right, 10 - x, y), "pixel {x},{y}");
            }
        }
    }
}
