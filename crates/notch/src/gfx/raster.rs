//! Rasterize small shapes on the CPU to avoid the initial 10 MB cost of Direct2D geometry.

use super::{color::Color, geometry::Point, image_data::ImageData, path::Figure};

pub struct Mask {
    width: u32,
    height: u32,
    coverage: Vec<f32>,
}

impl Mask {
    /// Signed distance in pixels: negative inside, positive outside, half coverage on the edge.
    pub fn from_distance(width: u32, height: u32, distance: impl Fn(Point) -> f32) -> Self {
        let mut coverage = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                let center = Point::new(x as f32 + 0.5, y as f32 + 0.5);
                coverage.push((0.5 - distance(center)).clamp(0.0, 1.0));
            }
        }
        Self { width, height, coverage }
    }

    /// The inside of `figures`; open figures count as closed.
    pub fn fill(width: u32, height: u32, figures: &[Figure]) -> Self {
        let nearest = nearest_edge_distances(width, height, figures, true, EDGE_REACH);
        let mut coverage = Vec::with_capacity(nearest.len());
        let mut crossings = Vec::new();
        for y in 0..height {
            row_crossings(figures, y as f32 + 0.5, &mut crossings);
            for x in 0..width {
                let center_x = x as f32 + 0.5;
                let to_the_right = crossings.len() - crossings.partition_point(|&crossing| crossing <= center_x);
                let distance = nearest[(y * width + x) as usize];
                let signed = if to_the_right % 2 == 1 { -distance } else { distance };
                coverage.push((0.5 - signed).clamp(0.0, 1.0));
            }
        }
        Self { width, height, coverage }
    }

    pub fn stroke(width: u32, height: u32, figures: &[Figure], half_width: f32) -> Self {
        let nearest = nearest_edge_distances(width, height, figures, false, half_width + EDGE_REACH);
        let coverage = nearest.into_iter().map(|distance| (0.5 - (distance - half_width)).clamp(0.0, 1.0)).collect();
        Self { width, height, coverage }
    }

    #[cfg(test)]
    fn coverage_at(&self, x: u32, y: u32) -> f32 {
        self.coverage[(y * self.width + x) as usize]
    }

    /// Colors each pixel with `paint(x, y)`, scaled by the pixel's coverage; for gradients.
    pub fn painted(&self, paint: impl Fn(u32, u32) -> Color) -> ImageData {
        let mut pixels = Vec::with_capacity(self.coverage.len() * 4);
        for (index, &coverage) in self.coverage.iter().enumerate() {
            let color = paint(index as u32 % self.width, index as u32 / self.width);
            let weight = coverage * color.a;
            for channel in [color.b, color.g, color.r, 1.0] {
                pixels.push((channel * weight * 255.0).round() as u8);
            }
        }
        ImageData::new(self.width, self.height, pixels)
    }

    pub fn tinted(&self, red: f32, green: f32, blue: f32) -> ImageData {
        let mut pixels = Vec::with_capacity(self.coverage.len() * 4);
        for &coverage in &self.coverage {
            for channel in [blue, green, red, 1.0] {
                pixels.push((channel * coverage * 255.0).round() as u8);
            }
        }
        ImageData::new(self.width, self.height, pixels)
    }
}

/// Pixels farther than this from every edge are fully in or out, so distances beyond it need not be measured.
const EDGE_REACH: f32 = 1.0;

/// Distance from each pixel center to the nearest edge, measured only within `reach` of an edge; `f32::MAX` elsewhere.
fn nearest_edge_distances(
    width: u32,
    height: u32,
    figures: &[Figure],
    closes_open_figures: bool,
    reach: f32,
) -> Vec<f32> {
    let mut nearest = vec![f32::MAX; (width * height) as usize];
    let (last_column, last_row) = (width.saturating_sub(1), height.saturating_sub(1));
    for (start, end) in edges(figures, closes_open_figures) {
        let first_column = (start.x.min(end.x) - reach - 0.5).floor().max(0.0) as u32;
        let first_row = (start.y.min(end.y) - reach - 0.5).floor().max(0.0) as u32;
        let end_column = ((start.x.max(end.x) + reach).ceil().max(0.0) as u32).min(last_column);
        let end_row = ((start.y.max(end.y) + reach).ceil().max(0.0) as u32).min(last_row);
        for y in first_row..=end_row {
            for x in first_column..=end_column {
                let distance = distance_to_segment(Point::new(x as f32 + 0.5, y as f32 + 0.5), start, end);
                let slot = &mut nearest[(y * width + x) as usize];
                *slot = slot.min(distance);
            }
        }
    }
    nearest
}

/// The sorted x positions where the closed edges cross the horizontal line at `line_y`.
fn row_crossings(figures: &[Figure], line_y: f32, crossings: &mut Vec<f32>) {
    crossings.clear();
    for (start, end) in edges(figures, true) {
        if (start.y > line_y) != (end.y > line_y) {
            crossings.push(start.x + (line_y - start.y) / (end.y - start.y) * (end.x - start.x));
        }
    }
    crossings.sort_by(f32::total_cmp);
}

/// Fills close open figures; strokes must leave them open.
fn edges(figures: &[Figure], closes_open_figures: bool) -> impl Iterator<Item = (Point, Point)> + '_ {
    figures.iter().flat_map(move |figure| {
        let count = figure.points.len();
        let closes = figure.is_closed || closes_open_figures;
        let edge_count = if closes { count } else { count.saturating_sub(1) };
        (0..edge_count).map(move |index| {
            let end_index = if closes { (index + 1) % count } else { index + 1 };
            (figure.points[index], figure.points[end_index])
        })
    })
}

fn distance_to_segment(point: Point, start: Point, end: Point) -> f32 {
    let (edge_x, edge_y) = (end.x - start.x, end.y - start.y);
    let length_squared = edge_x * edge_x + edge_y * edge_y;
    let along = if length_squared == 0.0 {
        0.0
    } else {
        (((point.x - start.x) * edge_x + (point.y - start.y) * edge_y) / length_squared).clamp(0.0, 1.0)
    };
    (point.x - (start.x + edge_x * along)).hypot(point.y - (start.y + edge_y * along))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(left: f32, top: f32, right: f32, bottom: f32) -> Figure {
        let points =
            vec![Point::new(left, top), Point::new(right, top), Point::new(right, bottom), Point::new(left, bottom)];
        Figure { points, is_closed: true }
    }

    #[test]
    fn a_filled_square_covers_its_inside_and_nothing_outside() {
        let mask = Mask::fill(10, 10, &[square(2.0, 2.0, 8.0, 8.0)]);
        assert_eq!(mask.coverage_at(5, 5), 1.0);
        assert_eq!(mask.coverage_at(0, 0), 0.0);
        assert_eq!(mask.coverage_at(9, 5), 0.0);
    }

    #[test]
    fn an_edge_through_a_pixel_covers_it_partly() {
        let mask = Mask::fill(10, 10, &[square(0.0, 0.0, 5.25, 10.0)]);
        let coverage = mask.coverage_at(5, 5);
        assert!((coverage - 0.25).abs() < 0.01, "{coverage}");
    }

    #[test]
    fn a_hole_inside_a_figure_stays_empty() {
        let mask = Mask::fill(12, 12, &[square(0.0, 0.0, 12.0, 12.0), square(4.0, 4.0, 8.0, 8.0)]);
        assert_eq!(mask.coverage_at(1, 1), 1.0);
        assert_eq!(mask.coverage_at(6, 6), 0.0);
    }

    #[test]
    fn a_stroked_line_is_as_wide_as_asked_with_round_ends() {
        let line = Figure { points: vec![Point::new(4.0, 5.0), Point::new(16.0, 5.0)], is_closed: false };
        let mask = Mask::stroke(20, 10, &[line], 1.0);
        assert_eq!(mask.coverage_at(10, 4), 1.0);
        assert_eq!(mask.coverage_at(10, 5), 1.0);
        assert_eq!(mask.coverage_at(10, 2), 0.0);
        assert!(mask.coverage_at(16, 4) > 0.5);
        assert_eq!(mask.coverage_at(18, 4), 0.0);
    }

    fn brute_force_distance(figures: &[Figure], point: Point, closes_open_figures: bool) -> f32 {
        edges(figures, closes_open_figures)
            .map(|(start, end)| distance_to_segment(point, start, end))
            .fold(f32::MAX, f32::min)
    }

    fn brute_force_inside(figures: &[Figure], point: Point) -> bool {
        edges(figures, true)
            .filter(|(start, end)| (start.y > point.y) != (end.y > point.y))
            .filter(|(start, end)| start.x + (point.y - start.y) / (end.y - start.y) * (end.x - start.x) > point.x)
            .count()
            % 2
            == 1
    }

    fn slanted_shape() -> [Figure; 2] {
        let outline = [(2.3, 3.7), (27.1, 6.2), (30.4, 25.9), (14.2, 18.4), (3.6, 27.3)];
        let hole = [(10.5, 9.5), (17.0, 10.5), (14.0, 14.0)];
        let figure = |points: &[(f32, f32)]| Figure {
            points: points.iter().map(|&(x, y)| Point::new(x, y)).collect(),
            is_closed: true,
        };
        [figure(&outline), figure(&hole)]
    }

    #[test]
    fn the_fast_fill_matches_measuring_every_pixel_against_every_edge() {
        let figures = slanted_shape();
        let mask = Mask::fill(34, 30, &figures);
        let reference = Mask::from_distance(34, 30, |point| {
            let distance = brute_force_distance(&figures, point, true);
            if brute_force_inside(&figures, point) { -distance } else { distance }
        });
        for (fast, slow) in mask.coverage.iter().zip(&reference.coverage) {
            assert!((fast - slow).abs() < 1e-6, "{fast} {slow}");
        }
    }

    #[test]
    fn the_fast_stroke_matches_measuring_every_pixel_against_every_edge() {
        let figures = slanted_shape().map(|figure| Figure { is_closed: false, ..figure });
        let mask = Mask::stroke(34, 30, &figures, 1.25);
        let reference = Mask::from_distance(34, 30, |point| brute_force_distance(&figures, point, false) - 1.25);
        for (fast, slow) in mask.coverage.iter().zip(&reference.coverage) {
            assert!((fast - slow).abs() < 1e-6, "{fast} {slow}");
        }
    }

    #[test]
    fn a_tinted_mask_is_premultiplied() {
        let mask = Mask::from_distance(1, 1, |_| 0.0);
        let image = mask.tinted(1.0, 0.0, 0.0);
        assert_eq!(image.bgra_premultiplied, vec![0, 0, 128, 128]);
    }
}
