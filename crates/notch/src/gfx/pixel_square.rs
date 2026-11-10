use super::{Canvas, Point, Rect};

/// One empty pixel on each side keeps antialiased edges from being cut.
const MARGIN: u32 = 1;

/// The pixel side of a cached shape bitmap for a shape `size` wide, and the area that places it centered on `center`
/// at whole pixels. Both are in device pixels.
fn bitmap_square(center: Point, size: f32) -> (u32, Rect) {
    let side = size.ceil() as u32 + 2 * MARGIN;
    let left = (center.x - side as f32 / 2.0).round();
    let top = (center.y - side as f32 / 2.0).round();
    (side, Rect::new(left, top, left + side as f32, top + side as f32))
}

impl Canvas<'_> {
    /// The pixel side of a cached shape bitmap for a shape `size` wide, and the area that draws it centered on `center`
    /// at whole device pixels. `center`, `size` and the area are in the canvas's current space; inside `in_units` the
    /// snapping still happens on the device grid, so a cache key holds the returned side, never `size`.
    pub fn pixel_square(&self, center: Point, size: f32) -> (u32, Rect) {
        let (scale, origin) = (self.raster_scale(), self.origin());
        let device_center = Point::new(origin.x + center.x * scale, origin.y + center.y * scale);
        let (side, area) = bitmap_square(device_center, size * scale);
        let to_units = |x: f32, y: f32| ((x - origin.x) / scale, (y - origin.y) / scale);
        let (left, top) = to_units(area.left, area.top);
        let (right, bottom) = to_units(area.right, area.bottom);
        (side, Rect::new(left, top, right, bottom))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gfx::test_support::render;
    use std::cell::Cell;

    #[test]
    fn the_bitmap_is_the_shape_plus_a_pixel_of_margin_on_each_side() {
        assert_eq!(bitmap_square(Point::new(50.0, 50.0), 18.0).0, 20);
        assert_eq!(bitmap_square(Point::new(50.0, 50.0), 17.2).0, 20);
    }

    #[test]
    fn the_area_sits_on_whole_pixels_around_the_center() {
        let (_, area) = bitmap_square(Point::new(50.3, 20.0), 18.0);
        assert_eq!(area, Rect::new(40.0, 10.0, 60.0, 30.0));
    }

    #[test]
    fn a_square_in_a_unit_space_lands_on_whole_device_pixels() {
        let placed = Cell::new((0, Rect::default()));
        render(1.25, |canvas| {
            canvas.in_units(Point::new(10.4, 0.0), |canvas| {
                placed.set(canvas.pixel_square(Point::new(50.3, 20.0), 18.0));
            });
        });
        let (side, area) = placed.get();
        assert_eq!(side, 25);
        let device_left = area.left * 1.25 + 10.4;
        let device_top = area.top * 1.25;
        assert!((device_left - device_left.round()).abs() < 0.001, "left {device_left}");
        assert!((device_top - device_top.round()).abs() < 0.001, "top {device_top}");
    }

    #[test]
    fn outside_a_unit_space_the_square_is_unchanged() {
        let placed = Cell::new((0, Rect::default()));
        render(1.25, |canvas| placed.set(canvas.pixel_square(Point::new(50.3, 20.0), 22.5)));
        assert_eq!(placed.get(), bitmap_square(Point::new(50.3, 20.0), 22.5));
    }
}
