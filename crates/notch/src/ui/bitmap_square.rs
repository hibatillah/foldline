use crate::gfx::{Point, Rect};

/// One empty pixel on each side keeps antialiased edges from being cut.
const MARGIN: u32 = 1;

/// The pixel side of a cached shape bitmap for a shape `size` wide, and the area that places it centered on `center`
/// at whole pixels. Both are in device pixels.
pub(super) fn bitmap_square(center: Point, size: f32) -> (u32, Rect) {
    let side = size.ceil() as u32 + 2 * MARGIN;
    let left = (center.x - side as f32 / 2.0).round();
    let top = (center.y - side as f32 / 2.0).round();
    (side, Rect::new(left, top, left + side as f32, top + side as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
