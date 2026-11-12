use super::{GRID, Icon};
use crate::gfx::{Path, Point};

const CORNER_RADIUS: f32 = 1.8;
/// Lucide "pin" (ISC license, https://lucide.dev): the head, then the needle.
const PIN: &str = "M9 10.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V16a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1v-.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V7a1 1 0 0 1 1-1 2 2 0 0 0 0-4H8a2 2 0 0 0 0 4 1 1 0 0 1 1 1zM12 17v5";

#[derive(Debug, PartialEq)]
pub(super) enum Paint {
    Fill,
    Line,
}

impl Icon {
    /// One path per icon, so overlapping parts are painted once and fade evenly.
    pub(super) fn path(self, center: Point, size: f32) -> (Path, Paint) {
        let unit = size / GRID;
        let at = |x: f32, y: f32| Point::new(center.x + (x - GRID / 2.0) * unit, center.y + (y - GRID / 2.0) * unit);
        let line = |points: &[(f32, f32)]| Path::polyline(&points.iter().map(|&(x, y)| at(x, y)).collect::<Vec<_>>());
        let solid = |points: &[(f32, f32)], radius: f32| {
            Path::rounded_polygon(&points.iter().map(|&(x, y)| at(x, y)).collect::<Vec<_>>(), radius * unit)
        };
        match self {
            Icon::Play => (solid(&[(7.0, 4.5), (19.5, 12.0), (7.0, 19.5)], CORNER_RADIUS), Paint::Fill),
            Icon::Pause => (
                join(
                    solid(&[(6.5, 5.0), (10.5, 5.0), (10.5, 19.0), (6.5, 19.0)], CORNER_RADIUS * 0.8),
                    solid(&[(13.5, 5.0), (17.5, 5.0), (17.5, 19.0), (13.5, 19.0)], CORNER_RADIUS * 0.8),
                ),
                Paint::Fill,
            ),
            Icon::Previous => (
                join(
                    solid(&[(5.0, 5.0), (7.5, 5.0), (7.5, 19.0), (5.0, 19.0)], CORNER_RADIUS * 0.6),
                    solid(&[(19.5, 5.0), (19.5, 19.0), (9.0, 12.0)], CORNER_RADIUS),
                ),
                Paint::Fill,
            ),
            Icon::Next => (
                join(
                    solid(&[(19.0, 5.0), (16.5, 5.0), (16.5, 19.0), (19.0, 19.0)], CORNER_RADIUS * 0.6),
                    solid(&[(4.5, 5.0), (4.5, 19.0), (15.0, 12.0)], CORNER_RADIUS),
                ),
                Paint::Fill,
            ),
            Icon::Speaker | Icon::SpeakerMuted => {
                let body =
                    line(&[(3.5, 9.5), (7.5, 9.5), (12.5, 5.5), (12.5, 18.5), (7.5, 14.5), (3.5, 14.5), (3.5, 9.5)]);
                let marks = if self == Icon::Speaker {
                    join(
                        Path::new().move_to(at(15.5, 9.5)).curve_to(at(17.0, 10.8), at(17.0, 13.2), at(15.5, 14.5)),
                        Path::new().move_to(at(18.0, 7.0)).curve_to(at(21.2, 9.8), at(21.2, 14.2), at(18.0, 17.0)),
                    )
                } else {
                    join(line(&[(16.0, 9.5), (21.0, 14.5)]), line(&[(21.0, 9.5), (16.0, 14.5)]))
                };
                (join(body, marks), Paint::Line)
            }
            Icon::Sun => {
                let mut sun = circle(at(12.0, 12.0), 4.0 * unit);
                for step in 0..8 {
                    let angle = step as f32 * std::f32::consts::FRAC_PI_4;
                    let (sine, cosine) = angle.sin_cos();
                    sun.append(line(&[
                        (12.0 + 7.0 * cosine, 12.0 + 7.0 * sine),
                        (12.0 + 9.5 * cosine, 12.0 + 9.5 * sine),
                    ]));
                }
                (sun, Paint::Line)
            }
            Icon::Bolt => (
                solid(&[(13.5, 3.0), (6.0, 13.5), (11.0, 13.5), (10.0, 21.0), (18.0, 10.0), (13.0, 10.0)], 1.0),
                Paint::Fill,
            ),
            Icon::Battery | Icon::BatteryLow => {
                let body = solid(&[(3.0, 7.0), (19.0, 7.0), (19.0, 17.0), (3.0, 17.0)], 2.0);
                let terminal = line(&[(21.5, 10.5), (21.5, 13.5)]);
                let level = if self == Icon::Battery {
                    join(line(&[(6.5, 10.5), (6.5, 13.5)]), line(&[(10.0, 10.5), (10.0, 13.5)]))
                } else {
                    line(&[(6.5, 10.5), (6.5, 13.5)])
                };
                (join(join(body, terminal), level), Paint::Line)
            }
            Icon::Microphone => {
                let capsule = solid(&[(9.0, 3.0), (15.0, 3.0), (15.0, 12.0), (9.0, 12.0)], 3.0);
                let cradle = Path::new().move_to(at(5.5, 11.0)).curve_to(at(5.5, 17.0), at(18.5, 17.0), at(18.5, 11.0));
                let stand = join(line(&[(12.0, 16.5), (12.0, 21.0)]), line(&[(8.5, 21.0), (15.5, 21.0)]));
                (join(join(capsule, cradle), stand), Paint::Line)
            }
            Icon::Camera => {
                let body = solid(&[(3.0, 7.0), (15.0, 7.0), (15.0, 17.0), (3.0, 17.0)], 2.0);
                let lens = line(&[(15.5, 10.5), (21.0, 7.5), (21.0, 16.5), (15.5, 13.5)]);
                (join(body, lens), Paint::Line)
            }
            Icon::Bell => {
                let dome = Path::new()
                    .move_to(at(5.0, 17.0))
                    .line_to(at(7.0, 15.0))
                    .line_to(at(7.0, 10.5))
                    .curve_to(at(7.0, 7.5), at(9.2, 5.5), at(12.0, 5.5))
                    .curve_to(at(14.8, 5.5), at(17.0, 7.5), at(17.0, 10.5))
                    .line_to(at(17.0, 15.0))
                    .line_to(at(19.0, 17.0))
                    .line_to(at(5.0, 17.0));
                let clapper = Path::new()
                    .move_to(at(10.2, 19.5))
                    .curve_to(at(10.6, 20.5), at(11.2, 21.0), at(12.0, 21.0))
                    .curve_to(at(12.8, 21.0), at(13.4, 20.5), at(13.8, 19.5));
                (join(join(dome, clapper), line(&[(12.0, 3.5), (12.0, 5.5)])), Paint::Line)
            }
            Icon::Clock => {
                let face = circle(at(12.0, 12.0), 9.0 * unit);
                (join(face, line(&[(12.0, 7.0), (12.0, 12.0), (15.5, 14.0)])), Paint::Line)
            }
            Icon::Folder => (
                solid(&[(3.0, 6.0), (9.0, 6.0), (11.0, 9.0), (21.0, 9.0), (21.0, 19.0), (3.0, 19.0)], 2.0),
                Paint::Line,
            ),
            Icon::Calendar => {
                let body = solid(&[(4.0, 6.0), (20.0, 6.0), (20.0, 20.0), (4.0, 20.0)], 2.0);
                let rings = join(line(&[(8.5, 3.5), (8.5, 7.5)]), line(&[(15.5, 3.5), (15.5, 7.5)]));
                (join(join(body, rings), line(&[(4.0, 10.5), (20.0, 10.5)])), Paint::Line)
            }
            Icon::Bluetooth => {
                (line(&[(7.0, 8.0), (17.0, 16.0), (12.0, 20.5), (12.0, 3.5), (17.0, 8.0), (7.0, 16.0)]), Paint::Line)
            }
            Icon::Moon => {
                // Lucide "moon-star" (ISC license, https://lucide.dev).
                let outer = arc((12.0, 12.0), 9.0, 3.1, 266.9, 24);
                let inner = arc((17.0, 7.0), 6.0, 212.0, 58.0, 12);
                let mut crescent = Path::new().move_to(at(20.985, 12.486));
                for (x, y) in outer.skip(1) {
                    crescent = crescent.line_to(at(x, y));
                }
                crescent = crescent.curve_to(at(11.917, 2.992), at(12.129, 3.474), at(11.914, 3.817));
                for (x, y) in inner.skip(1) {
                    crescent = crescent.line_to(at(x, y));
                }
                crescent = crescent.curve_to(at(20.526, 11.870), at(21.007, 12.081), at(20.985, 12.486));
                let star = join(line(&[(19.0, 3.0), (19.0, 7.0)]), line(&[(17.0, 5.0), (21.0, 5.0)]));
                (join(crescent, star), Paint::Line)
            }
            Icon::Cloud => {
                // Lucide "cloud" (ISC license, https://lucide.dev).
                let points: Vec<(f32, f32)> = std::iter::once((17.5, 19.0))
                    .chain(arc((9.0, 12.0), 7.0, 90.0, 343.4, 24))
                    .chain(arc((17.5, 14.5), 4.5, 270.0, 450.0, 14))
                    .collect();
                (line(&points), Paint::Line)
            }
            // Lucide "copy" (ISC license, https://lucide.dev).
            Icon::Copy => {
                let front = solid(&[(8.0, 8.0), (22.0, 8.0), (22.0, 22.0), (8.0, 22.0)], 2.0);
                let back = Path::new()
                    .move_to(at(4.0, 16.0))
                    .curve_to(at(2.9, 16.0), at(2.0, 15.1), at(2.0, 14.0))
                    .line_to(at(2.0, 4.0))
                    .curve_to(at(2.0, 2.9), at(2.9, 2.0), at(4.0, 2.0))
                    .line_to(at(14.0, 2.0))
                    .curve_to(at(15.1, 2.0), at(16.0, 2.9), at(16.0, 4.0));
                (join(front, back), Paint::Line)
            }
            Icon::CapsLock => {
                let arrow = solid(
                    &[(12.0, 3.0), (20.5, 12.0), (15.5, 12.0), (15.5, 16.5), (8.5, 16.5), (8.5, 12.0), (3.5, 12.0)],
                    1.5,
                );
                (join(arrow, line(&[(8.5, 20.5), (15.5, 20.5)])), Paint::Line)
            }
            Icon::Gear => {
                const CIRCUMRADIUS: f32 = 9.5;
                const RING_RADIUS: f32 = 3.0;
                let corners: Vec<(f32, f32)> = (0..6)
                    .map(|corner| {
                        let angle = (corner as f32 * 60.0 - 90.0).to_radians();
                        (12.0 + CIRCUMRADIUS * angle.cos(), 12.0 + CIRCUMRADIUS * angle.sin())
                    })
                    .collect();
                (join(solid(&corners, CORNER_RADIUS), circle(at(12.0, 12.0), RING_RADIUS * unit)), Paint::Line)
            }
            Icon::Mouse => (
                join(
                    solid(&[(6.0, 3.0), (18.0, 3.0), (18.0, 21.0), (6.0, 21.0)], 6.0),
                    line(&[(12.0, 3.0), (12.0, 9.0)]),
                ),
                Paint::Line,
            ),
            Icon::Keyboard => {
                let body = solid(&[(2.0, 6.0), (22.0, 6.0), (22.0, 18.0), (2.0, 18.0)], 2.0);
                let mut keys = line(&[(7.0, 14.0), (17.0, 14.0)]);
                for x in [6.0, 10.0, 14.0, 18.0] {
                    keys.append(line(&[(x, 10.0), (x + 0.01, 10.0)]));
                }
                (join(body, keys), Paint::Line)
            }
            Icon::Headphones => {
                let band: Vec<(f32, f32)> = std::iter::once((4.0, 15.0))
                    .chain(arc((12.0, 12.0), 8.0, 180.0, 360.0, 16))
                    .chain(std::iter::once((20.0, 15.0)))
                    .collect();
                let left = solid(&[(3.0, 14.0), (7.0, 14.0), (7.0, 20.0), (3.0, 20.0)], 1.5);
                let right = solid(&[(17.0, 14.0), (21.0, 14.0), (21.0, 20.0), (17.0, 20.0)], 1.5);
                (join(join(line(&band), left), right), Paint::Line)
            }
            // Lucide "gamepad" (ISC license, https://lucide.dev).
            Icon::Gamepad => {
                let body = Path::new()
                    .move_to(at(6.0, 8.0))
                    .line_to(at(18.0, 8.0))
                    .curve_to(at(19.7, 8.0), at(21.0, 9.4), at(21.0, 11.2))
                    .line_to(at(20.4, 16.2))
                    .curve_to(at(20.2, 18.4), at(17.4, 19.2), at(16.6, 17.5))
                    .line_to(at(15.0, 16.0))
                    .line_to(at(9.0, 16.0))
                    .line_to(at(7.4, 17.5))
                    .curve_to(at(6.6, 19.2), at(3.8, 18.4), at(3.6, 16.2))
                    .line_to(at(3.0, 11.2))
                    .curve_to(at(3.0, 9.4), at(4.3, 8.0), at(6.0, 8.0))
                    .close();
                let plus = join(line(&[(8.0, 11.0), (8.0, 14.0)]), line(&[(6.5, 12.5), (9.5, 12.5)]));
                (join(body, plus), Paint::Line)
            }
            Icon::Pen => (
                Path::new()
                    .move_to(at(4.0, 20.0))
                    .line_to(at(5.0, 16.0))
                    .line_to(at(16.0, 5.0))
                    .curve_to(at(17.0, 3.9), at(19.2, 6.0), at(19.0, 8.0))
                    .line_to(at(8.0, 19.0))
                    .close(),
                Paint::Line,
            ),
            Icon::CircleCheck => {
                (join(circle(at(12.0, 12.0), 9.0 * unit), line(&[(8.0, 12.5), (10.8, 15.3), (16.0, 9.5)])), Paint::Line)
            }
            Icon::Pin => {
                let corner = Point::new(center.x - size / 2.0, center.y - size / 2.0);
                (Path::from_svg(PIN, corner, unit), Paint::Line)
            }
            Icon::Lyrics => {
                let bubble = solid(
                    &[(3.0, 4.0), (21.0, 4.0), (21.0, 16.0), (12.0, 16.0), (7.5, 20.5), (7.5, 16.0), (3.0, 16.0)],
                    2.5,
                );
                let text = join(line(&[(7.5, 8.5), (16.5, 8.5)]), line(&[(7.5, 11.5), (13.0, 11.5)]));
                (join(bubble, text), Paint::Line)
            }
        }
    }
}

/// Points along a circle's arc, angles in degrees (increasing runs clockwise on screen).
fn arc(center: (f32, f32), radius: f32, from: f32, to: f32, steps: u32) -> impl Iterator<Item = (f32, f32)> {
    (0..=steps).map(move |step| {
        let angle = (from + (to - from) * step as f32 / steps as f32).to_radians();
        (center.0 + radius * angle.cos(), center.1 + radius * angle.sin())
    })
}

fn circle(center: Point, radius: f32) -> Path {
    const KAPPA: f32 = 0.552_284_8;
    let (x, y, reach) = (center.x, center.y, radius * KAPPA);
    Path::new()
        .move_to(Point::new(x + radius, y))
        .curve_to(Point::new(x + radius, y + reach), Point::new(x + reach, y + radius), Point::new(x, y + radius))
        .curve_to(Point::new(x - reach, y + radius), Point::new(x - radius, y + reach), Point::new(x - radius, y))
        .curve_to(Point::new(x - radius, y - reach), Point::new(x - reach, y - radius), Point::new(x, y - radius))
        .curve_to(Point::new(x + reach, y - radius), Point::new(x + radius, y - reach), Point::new(x + radius, y))
        .close()
}

fn join(mut first: Path, second: Path) -> Path {
    first.append(second);
    first
}
