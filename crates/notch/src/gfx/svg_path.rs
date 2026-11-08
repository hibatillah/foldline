//! Reads SVG path data ("M12 4 4 12h4v5z") into a `Path`, so a Lucide-style icon can be pasted as it is.

use super::{geometry::Point, path::Path};

/// An arc is cut into pieces of at most a quarter turn, each drawn as one cubic curve.
const QUARTER_TURN: f32 = std::f32::consts::FRAC_PI_2;

enum Token {
    Command(char),
    Number(f32),
}

fn tokens(data: &str) -> Vec<Token> {
    let characters: Vec<char> = data.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if character.is_ascii_alphabetic() {
            tokens.push(Token::Command(character));
            index += 1;
        } else if character.is_ascii_digit() || matches!(character, '-' | '+' | '.') {
            let start = index;
            index += 1;
            let mut has_dot = character == '.';
            while index < characters.len() {
                let next = characters[index];
                if next.is_ascii_digit() {
                    index += 1;
                } else if next == '.' && !has_dot {
                    has_dot = true;
                    index += 1;
                } else {
                    break;
                }
            }
            let text: String = characters[start..index].iter().collect();
            if let Ok(number) = text.parse::<f32>() {
                tokens.push(Token::Number(number));
            }
        } else {
            index += 1;
        }
    }
    tokens
}

struct Cursor {
    tokens: Vec<Token>,
    position: usize,
}

impl Cursor {
    fn next_command(&mut self) -> Option<char> {
        match self.tokens.get(self.position) {
            Some(&Token::Command(command)) => {
                self.position += 1;
                Some(command)
            }
            _ => None,
        }
    }

    fn has_number(&self) -> bool {
        matches!(self.tokens.get(self.position), Some(Token::Number(_)))
    }

    fn number(&mut self) -> Option<f32> {
        match self.tokens.get(self.position) {
            Some(&Token::Number(number)) => {
                self.position += 1;
                Some(number)
            }
            _ => None,
        }
    }

    fn numbers<const COUNT: usize>(&mut self) -> Option<[f32; COUNT]> {
        let mut values = [0.0; COUNT];
        for value in &mut values {
            *value = self.number()?;
        }
        Some(values)
    }
}

impl Path {
    /// Reads `data` (M, L, H, V, C, S, A and Z, absolute or relative) and maps each point `(x, y)` of it to
    /// `origin + (x, y) * unit`; an unreadable command ends the path where it is.
    pub fn from_svg(data: &str, origin: Point, unit: f32) -> Path {
        let at = |x: f32, y: f32| Point::new(origin.x + x * unit, origin.y + y * unit);
        let mut cursor = Cursor { tokens: tokens(data), position: 0 };
        let mut path = Path::new();
        let (mut x, mut y) = (0.0_f32, 0.0_f32);
        let (mut start_x, mut start_y) = (0.0_f32, 0.0_f32);
        let mut last_control: Option<(f32, f32)> = None;
        let mut command = ' ';
        let mut is_closed = false;
        loop {
            if !cursor.has_number() {
                match cursor.next_command() {
                    Some(next) => command = next,
                    None => break,
                }
            } else if command == ' ' {
                break;
            }
            let is_relative = command.is_ascii_lowercase();
            let (offset_x, offset_y) = if is_relative { (x, y) } else { (0.0, 0.0) };
            let mut control = None;
            if is_closed && !matches!(command.to_ascii_uppercase(), 'M' | 'Z') {
                path = path.move_to(at(x, y));
            }
            is_closed = false;
            match command.to_ascii_uppercase() {
                'M' => {
                    let Some([next_x, next_y]) = cursor.numbers() else { break };
                    (x, y) = (offset_x + next_x, offset_y + next_y);
                    (start_x, start_y) = (x, y);
                    path = path.move_to(at(x, y));
                    // More pairs after a move are lines.
                    command = if is_relative { 'l' } else { 'L' };
                }
                'L' => {
                    let Some([next_x, next_y]) = cursor.numbers() else { break };
                    (x, y) = (offset_x + next_x, offset_y + next_y);
                    path = path.line_to(at(x, y));
                }
                'H' => {
                    let Some(next_x) = cursor.number() else { break };
                    x = offset_x + next_x;
                    path = path.line_to(at(x, y));
                }
                'V' => {
                    let Some(next_y) = cursor.number() else { break };
                    y = offset_y + next_y;
                    path = path.line_to(at(x, y));
                }
                'C' => {
                    let Some([x1, y1, x2, y2, next_x, next_y]) = cursor.numbers() else { break };
                    let (first, second) = ((offset_x + x1, offset_y + y1), (offset_x + x2, offset_y + y2));
                    path = path.curve_to(
                        at(first.0, first.1),
                        at(second.0, second.1),
                        at(offset_x + next_x, offset_y + next_y),
                    );
                    (x, y) = (offset_x + next_x, offset_y + next_y);
                    control = Some(second);
                }
                'S' => {
                    let Some([x2, y2, next_x, next_y]) = cursor.numbers() else { break };
                    let first = last_control
                        .map_or((x, y), |(control_x, control_y)| (2.0 * x - control_x, 2.0 * y - control_y));
                    let second = (offset_x + x2, offset_y + y2);
                    path = path.curve_to(
                        at(first.0, first.1),
                        at(second.0, second.1),
                        at(offset_x + next_x, offset_y + next_y),
                    );
                    (x, y) = (offset_x + next_x, offset_y + next_y);
                    control = Some(second);
                }
                'A' => {
                    let Some([radius_x, radius_y, rotation, large, sweep, next_x, next_y]) = cursor.numbers() else {
                        break;
                    };
                    let end = (offset_x + next_x, offset_y + next_y);
                    let curves = arc_curves((x, y), (radius_x, radius_y), rotation, large != 0.0, sweep != 0.0, end);
                    if curves.is_empty() {
                        path = path.line_to(at(end.0, end.1));
                    }
                    for [first, second, last] in curves {
                        path = path.curve_to(at(first.0, first.1), at(second.0, second.1), at(last.0, last.1));
                    }
                    (x, y) = end;
                }
                'Z' => {
                    path = path.close();
                    (x, y) = (start_x, start_y);
                    is_closed = true;
                }
                _ => break,
            }
            last_control = control;
        }
        path
    }
}

type Pair = (f32, f32);

/// An elliptical arc from `from` to `to` as cubic curves (SVG implementation notes, F.6.5); empty when the arc has no
/// length, which the caller draws as a straight line.
fn arc_curves(
    from: Pair,
    radii: Pair,
    rotation_degrees: f32,
    is_large: bool,
    is_sweep: bool,
    to: Pair,
) -> Vec<[Pair; 3]> {
    let (mut radius_x, mut radius_y) = (radii.0.abs(), radii.1.abs());
    if from == to || radius_x == 0.0 || radius_y == 0.0 {
        return Vec::new();
    }
    let rotation = rotation_degrees.to_radians();
    let (sin, cos) = rotation.sin_cos();
    let half = ((from.0 - to.0) / 2.0, (from.1 - to.1) / 2.0);
    let rotated = (cos * half.0 + sin * half.1, -sin * half.0 + cos * half.1);
    let too_small = rotated.0 * rotated.0 / (radius_x * radius_x) + rotated.1 * rotated.1 / (radius_y * radius_y);
    if too_small > 1.0 {
        let grow = too_small.sqrt();
        radius_x *= grow;
        radius_y *= grow;
    }
    let (rx2, ry2) = (radius_x * radius_x, radius_y * radius_y);
    let (x2, y2) = (rotated.0 * rotated.0, rotated.1 * rotated.1);
    let numerator = (rx2 * ry2 - rx2 * y2 - ry2 * x2).max(0.0);
    let denominator = rx2 * y2 + ry2 * x2;
    let sign = if is_large == is_sweep { -1.0 } else { 1.0 };
    let factor = if denominator == 0.0 { 0.0 } else { sign * (numerator / denominator).sqrt() };
    let center_rotated = (factor * radius_x * rotated.1 / radius_y, -factor * radius_y * rotated.0 / radius_x);
    let center = (
        cos * center_rotated.0 - sin * center_rotated.1 + (from.0 + to.0) / 2.0,
        sin * center_rotated.0 + cos * center_rotated.1 + (from.1 + to.1) / 2.0,
    );
    let start_vector = ((rotated.0 - center_rotated.0) / radius_x, (rotated.1 - center_rotated.1) / radius_y);
    let end_vector = ((-rotated.0 - center_rotated.0) / radius_x, (-rotated.1 - center_rotated.1) / radius_y);
    let start_angle = start_vector.1.atan2(start_vector.0);
    let mut sweep = end_vector.1.atan2(end_vector.0) - start_angle;
    if !is_sweep && sweep > 0.0 {
        sweep -= std::f32::consts::TAU;
    } else if is_sweep && sweep < 0.0 {
        sweep += std::f32::consts::TAU;
    }
    let pieces = (sweep.abs() / QUARTER_TURN - 0.001).ceil().max(1.0) as usize;
    let step = sweep / pieces as f32;
    let handle = 4.0 / 3.0 * (step / 4.0).tan();
    let point = |angle: f32| {
        let (ellipse_x, ellipse_y) = (radius_x * angle.cos(), radius_y * angle.sin());
        (cos * ellipse_x - sin * ellipse_y + center.0, sin * ellipse_x + cos * ellipse_y + center.1)
    };
    let slope = |angle: f32| {
        let (ellipse_x, ellipse_y) = (-radius_x * angle.sin(), radius_y * angle.cos());
        (cos * ellipse_x - sin * ellipse_y, sin * ellipse_x + cos * ellipse_y)
    };
    (0..pieces)
        .map(|piece| {
            let (from_angle, to_angle) = (start_angle + step * piece as f32, start_angle + step * (piece + 1) as f32);
            let (begin, finish) = (point(from_angle), point(to_angle));
            let (begin_slope, finish_slope) = (slope(from_angle), slope(to_angle));
            let end = if piece + 1 == pieces { to } else { finish };
            [
                (begin.0 + handle * begin_slope.0, begin.1 + handle * begin_slope.1),
                (finish.0 - handle * finish_slope.0, finish.1 - handle * finish_slope.1),
                end,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(data: &str) -> Vec<Point> {
        Path::from_svg(data, Point::new(0.0, 0.0), 1.0).points().collect()
    }

    #[test]
    fn lines_read_absolute_and_relative_with_implicit_repeats() {
        assert_eq!(
            read("M1 2 3 4h2v1"),
            [Point::new(1.0, 2.0), Point::new(3.0, 4.0), Point::new(5.0, 4.0), Point::new(5.0, 5.0)]
        );
        assert_eq!(read("m1 1 2 2 2 2"), [Point::new(1.0, 1.0), Point::new(3.0, 3.0), Point::new(5.0, 5.0)]);
    }

    #[test]
    fn numbers_may_touch_signs_and_dots() {
        assert_eq!(read("M.5.5-1-2"), [Point::new(0.5, 0.5), Point::new(-1.0, -2.0)]);
        assert_eq!(read("M8 18l-1 3M12 18l-1 3").len(), 4);
    }

    #[test]
    fn a_unit_and_origin_map_every_point() {
        let points: Vec<Point> = Path::from_svg("M2 4L6 8", Point::new(10.0, 20.0), 2.0).points().collect();
        assert_eq!(points, [Point::new(14.0, 28.0), Point::new(22.0, 36.0)]);
    }

    #[test]
    fn a_half_circle_arc_stays_on_its_circle_and_bulges_the_sweep_way() {
        let figures = Path::from_svg("M0 0a5 5 0 0 1 10 0", Point::new(0.0, 0.0), 1.0).flatten();
        let points = &figures[0].points;
        assert!(points.iter().all(|point| ((point.x - 5.0).hypot(point.y) - 5.0).abs() < 0.05), "{points:?}");
        assert!(points.iter().any(|point| point.y < -4.9), "a sweep of 1 passes above in screen coordinates");
        let end = points.last().unwrap();
        assert!((end.x - 10.0).abs() < 0.001 && end.y.abs() < 0.001);
    }

    #[test]
    fn a_full_circle_from_two_arcs_is_round() {
        let figures = Path::from_svg("M8 12a4 4 0 1 0 8 0a4 4 0 1 0-8 0", Point::new(0.0, 0.0), 1.0).flatten();
        assert!(figures[0].points.iter().all(|point| ((point.x - 12.0).hypot(point.y - 12.0) - 4.0).abs() < 0.05));
    }

    #[test]
    fn a_close_returns_to_the_start_for_the_next_relative_command() {
        let figures = Path::from_svg("M0 0h4v4zh2", Point::new(0.0, 0.0), 1.0).flatten();
        assert!(figures[0].is_closed);
        assert_eq!(figures[1].points, [Point::new(0.0, 0.0), Point::new(2.0, 0.0)]);
    }

    #[test]
    fn an_unreadable_command_ends_the_path() {
        assert_eq!(read("M1 1L2 2X3 3L4 4"), [Point::new(1.0, 1.0), Point::new(2.0, 2.0)]);
        assert!(read("").is_empty());
    }
}
