//! The lines of text the preview shows.

use crate::gfx::clamp_lines;

const ELLIPSIS: char = '\u{2026}';
/// Only the start of a long copy can show, so wrapping looks at no more than this many characters.
const MAX_CHARACTERS: usize = 4000;

/// Wraps `text` into at most `max_lines` lines that fit `max_width`, keeping its line breaks and blank lines. When more
/// would not fit, the last line ends with an ellipsis. `measure` gives the width of one line.
pub(super) fn preview_lines(
    text: &str,
    max_lines: usize,
    max_width: f32,
    measure: impl Fn(&str) -> f32,
) -> Vec<String> {
    let shown: String = text.chars().take(MAX_CHARACTERS).collect();
    let mut is_cut = shown.len() < text.len();
    let mut lines: Vec<String> = Vec::new();
    for source in shown.lines() {
        if lines.len() > max_lines {
            break;
        }
        if source.trim().is_empty() {
            lines.push(String::new());
            continue;
        }
        let room = max_lines + 1 - lines.len();
        lines.extend(clamp_lines(source.trim_end(), room, max_width, &measure));
    }
    while lines.last().is_some_and(String::is_empty) && lines.len() > 1 {
        lines.pop();
    }
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        is_cut = true;
    }
    if is_cut && let Some(last) = lines.last_mut() {
        let cut = clamp_lines(&format!("{last}{ELLIPSIS}"), 1, max_width, &measure);
        if let Some(line) = cut.into_iter().next() {
            *last = line;
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn width(text: &str) -> f32 {
        text.chars().count() as f32
    }

    #[test]
    fn short_text_stays_as_it_is() {
        assert_eq!(preview_lines("npm run build", 7, 40.0, width), vec!["npm run build"]);
    }

    #[test]
    fn line_breaks_and_blank_lines_are_kept() {
        assert_eq!(preview_lines("one\n\ntwo", 7, 40.0, width), vec!["one", "", "two"]);
    }

    #[test]
    fn long_lines_wrap_onto_more_lines() {
        assert_eq!(preview_lines("aaa bbb ccc", 7, 7.0, width), vec!["aaa bbb", "ccc"]);
    }

    #[test]
    fn text_longer_than_the_lines_allowed_ends_with_an_ellipsis() {
        let text = (1..=12).map(|number| format!("line {number}")).collect::<Vec<_>>().join("\n");
        let lines = preview_lines(&text, 7, 40.0, width);
        assert_eq!(lines.len(), 7);
        assert_eq!(lines[0], "line 1");
        assert!(lines[6].ends_with(ELLIPSIS));
    }

    #[test]
    fn text_that_fills_the_lines_exactly_gets_no_ellipsis() {
        let lines = preview_lines("a\nb\nc", 3, 40.0, width);
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    #[test]
    fn trailing_blank_lines_are_not_shown() {
        assert_eq!(preview_lines("a\n\n\n", 7, 40.0, width), vec!["a"]);
    }

    #[test]
    fn a_very_long_copy_is_cut_and_marked() {
        let lines = preview_lines(&"word ".repeat(5000), 2, 20.0, width);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].ends_with(ELLIPSIS));
    }
}
