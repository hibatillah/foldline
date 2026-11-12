//! The size of a finished download as text, in steps of 1024 like Explorer.

const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

pub fn size_text(bytes: u64) -> String {
    let mut value = bytes as f64;
    let mut unit = 0;
    while value.round() >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 || value.round() >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_like_explorer() {
        assert_eq!(size_text(0), "0 B");
        assert_eq!(size_text(999), "999 B");
        assert_eq!(size_text(512_000), "500 KB");
        assert_eq!(size_text(18_661_376), "17.8 MB");
        assert_eq!(size_text(113_678_991), "108 MB");
        assert_eq!(size_text(1_288_490_189), "1.2 GB");
    }

    #[test]
    fn a_size_just_under_a_step_moves_up_to_the_next_unit() {
        assert_eq!(size_text(1_048_575), "1.0 MB");
    }
}
