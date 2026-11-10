//! One remembered copy: what it holds, what kind it is and the words that describe it.

use crate::i18n::tr;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use crate::gfx::{Color, ImageData};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Text,
    Link,
    Code,
    Color,
    Image,
}

pub(super) struct StoredImage {
    pub pixels: ImageData,
    /// The size the picture had when it was copied; `pixels` may be smaller to keep memory in bounds.
    pub original_width: u32,
    pub original_height: u32,
    pub format: &'static str,
    fingerprint: u64,
}

impl StoredImage {
    pub fn new(pixels: ImageData, original_width: u32, original_height: u32, format: &'static str) -> Self {
        let mut hasher = DefaultHasher::new();
        (pixels.width, pixels.height, &pixels.bgra_premultiplied).hash(&mut hasher);
        Self { fingerprint: hasher.finish(), pixels, original_width, original_height, format }
    }
}

pub(super) enum Content {
    Text(String),
    Image(StoredImage),
}

impl Content {
    pub fn is_same_as(&self, other: &Content) -> bool {
        match (self, other) {
            (Content::Text(first), Content::Text(second)) => first == second,
            (Content::Image(first), Content::Image(second)) => first.fingerprint == second.fingerprint,
            _ => false,
        }
    }

    /// Memory the copy holds, for the history's total limit.
    pub fn byte_size(&self) -> usize {
        match self {
            Content::Text(text) => text.len(),
            Content::Image(image) => image.pixels.bgra_premultiplied.len(),
        }
    }
}

pub(super) struct Item {
    pub id: u64,
    pub kind: Kind,
    pub content: Content,
    pub copied_at: Instant,
}

pub(super) fn kind_of(content: &Content) -> Kind {
    match content {
        Content::Text(text) => classify(text),
        Content::Image(_) => Kind::Image,
    }
}

const COMMANDS: [&str; 24] = [
    "npm", "npx", "pnpm", "yarn", "git", "cargo", "rustup", "docker", "kubectl", "pip", "uv", "python", "node", "cd",
    "ls", "curl", "sudo", "winget", "dotnet", "go", "make", "brew", "apt", "deno",
];
const CODE_STARTS: [&str; 11] =
    ["fn ", "let ", "const ", "def ", "import ", "class ", "#include", "function ", "SELECT ", "<?", "#!/"];

/// Colors win over links, links over code, and anything else is text.
pub(super) fn classify(text: &str) -> Kind {
    let trimmed = text.trim();
    if parse_color(trimmed).is_some() {
        Kind::Color
    } else if is_link(trimmed) {
        Kind::Link
    } else if is_code(trimmed) {
        Kind::Code
    } else {
        Kind::Text
    }
}

/// `#rgb` or `#rrggbb`.
pub(super) fn parse_color(text: &str) -> Option<Color> {
    let digits = text.strip_prefix('#')?;
    if !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
        return None;
    }
    let value = |hex: &str| u8::from_str_radix(hex, 16).ok().map(|byte| f32::from(byte) / 255.0);
    match digits.len() {
        3 => {
            let channel = |index: usize| value(&digits[index..=index].repeat(2));
            Some(Color::rgb(channel(0)?, channel(1)?, channel(2)?))
        }
        6 => Some(Color::rgb(value(&digits[0..2])?, value(&digits[2..4])?, value(&digits[4..6])?)),
        _ => None,
    }
}

/// Starts with `http://`, `https://` or `www.`, or is a host name followed by a path (`docs.example.com/shortcuts`).
/// A bare name with a dot (`readme.md`) stays text.
fn is_link(text: &str) -> bool {
    if text.is_empty() || text.chars().any(char::is_whitespace) {
        return false;
    }
    let lower = text.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("https://").or_else(|| lower.strip_prefix("http://")) {
        return !rest.is_empty();
    }
    if lower.starts_with("www.") {
        return lower.len() > 4;
    }
    let Some((host, _path)) = text.split_once('/') else { return false };
    let host = host
        .rsplit_once(':')
        .map_or(host, |(name, port)| if port.chars().all(|digit| digit.is_ascii_digit()) { name } else { host });
    let labels: Vec<&str> = host.split('.').collect();
    let is_label = |label: &&str| !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    let top = labels.last().copied().unwrap_or("");
    labels.len() >= 2
        && labels.iter().all(is_label)
        && (2..=24).contains(&top.len())
        && top.chars().all(|character| character.is_ascii_alphabetic())
}

/// A command line, a line ending in a semicolon, braces in a pair, `=>`, or a keyword that starts a statement.
fn is_code(text: &str) -> bool {
    let first_word = text.split_whitespace().next().unwrap_or("");
    let has_arguments = text.split_whitespace().nth(1).is_some();
    if has_arguments && COMMANDS.contains(&first_word) {
        return true;
    }
    if CODE_STARTS.iter().any(|start| text.starts_with(start)) && text.contains(['(', '{', '=', ';']) {
        return true;
    }
    (text.contains('{') && text.contains('}'))
        || text.lines().any(|line| line.trim_end().ends_with(';') && line.trim().len() > 1)
        || text.contains("=>")
}

/// The one line the list shows: the first line that has text, or the picture size.
pub(super) fn row_label(item: &Item) -> String {
    match &item.content {
        Content::Text(text) => text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("").to_owned(),
        Content::Image(image) => {
            tr!("clipboard.image_size", width = image.original_width, height = image.original_height).into_owned()
        }
    }
}

/// The type and size line under the preview, for example `Text · 24 characters`.
pub(super) fn details(item: &Item) -> String {
    match (&item.content, item.kind) {
        (Content::Image(image), _) => tr!(
            "clipboard.image_details",
            width = image.original_width,
            height = image.original_height,
            format = image.format
        )
        .into_owned(),
        (Content::Text(text), Kind::Color) => tr!("clipboard.color_details", value = text.trim()).into_owned(),
        (Content::Text(text), kind) => {
            let count = text.chars().count();
            let name = match kind {
                Kind::Link => tr!("clipboard.kind_link"),
                Kind::Code => tr!("clipboard.kind_code"),
                _ => tr!("clipboard.kind_text"),
            };
            let unit = if count == 1 { tr!("clipboard.character") } else { tr!("clipboard.characters") };
            format!("{name} \u{b7} {count} {unit}")
        }
    }
}

/// How long ago, in the short words the details line uses.
pub(super) fn age_text(age: Duration) -> String {
    let seconds = age.as_secs();
    match seconds {
        0..=4 => tr!("clipboard.age_now").into_owned(),
        5..=59 => tr!("clipboard.age_seconds", count = seconds).into_owned(),
        60..=3599 => tr!("clipboard.age_minutes", count = seconds / 60).into_owned(),
        3600..=86_399 => tr!("clipboard.age_hours", count = seconds / 3600).into_owned(),
        _ => tr!("clipboard.age_days", count = seconds / 86_400).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_item(text: &str) -> Item {
        let content = Content::Text(text.to_owned());
        Item { id: 1, kind: kind_of(&content), content, copied_at: Instant::now() }
    }

    #[test]
    fn colors_in_three_or_six_digits_are_colors() {
        assert_eq!(classify("#4dabf7"), Kind::Color);
        assert_eq!(classify(" #fa0 "), Kind::Color);
        assert_eq!(classify("#4dabf"), Kind::Text);
        assert_eq!(classify("#ggg"), Kind::Text);
        let color = parse_color("#fa0").unwrap();
        assert_eq!(color.rgb_bytes(), [0xff, 0xaa, 0x00]);
    }

    #[test]
    fn web_addresses_and_host_paths_are_links() {
        for link in [
            "https://example.com",
            "HTTP://example.com/a?b=1",
            "www.example.com",
            "github.com/example/notch/issues/42",
            "docs.example.com/shortcuts",
            "localhost.dev:3000/app",
        ] {
            assert_eq!(classify(link), Kind::Link, "{link}");
        }
    }

    #[test]
    fn names_paths_and_sentences_with_dots_are_not_links() {
        for text in ["readme.md", "src/main.rs", "v1.2/notes", "see https://example.com now", "https://", "a.b/c"] {
            assert_ne!(classify(text), Kind::Link, "{text}");
        }
    }

    #[test]
    fn commands_braces_and_statements_are_code() {
        for code in [
            "npm run build -- --watch",
            "git status",
            "cargo test",
            "let value = 3;",
            "fn main() { }",
            "if (a) {\n  b();\n}",
            "items.map(x => x + 1)",
        ] {
            assert_eq!(classify(code), Kind::Code, "{code}");
        }
    }

    #[test]
    fn ordinary_sentences_and_lone_command_words_are_text() {
        for text in ["Meeting moved to Friday 14:00, room B", "git", "Call back Sam after lunch", "Hello; world"] {
            assert_eq!(classify(text), Kind::Text, "{text}");
        }
    }

    #[test]
    fn the_list_shows_the_first_line_with_text() {
        assert_eq!(row_label(&text_item("\n  first line  \nsecond")), "first line");
    }

    #[test]
    fn details_name_the_type_and_the_size() {
        assert_eq!(details(&text_item("Call back Sam after lunch")), "Text · 25 characters");
        assert_eq!(details(&text_item("a")), "Text · 1 character");
        assert_eq!(details(&text_item("#4dabf7")), "Color · #4dabf7");
        let image = StoredImage::new(ImageData::new(1, 1, vec![0; 4]), 360, 70, "PNG");
        let item = Item { id: 2, kind: Kind::Image, content: Content::Image(image), copied_at: Instant::now() };
        assert_eq!(details(&item), "Image · 360×70 · PNG");
        assert_eq!(row_label(&item), "Image (360×70)");
    }

    #[test]
    fn age_is_worded_in_the_largest_whole_unit() {
        let word = |seconds| age_text(Duration::from_secs(seconds));
        assert_eq!(word(0), "just now");
        assert_eq!(word(4), "just now");
        assert_eq!(word(30), "30 sec ago");
        assert_eq!(word(12 * 60 + 5), "12 min ago");
        assert_eq!(word(3 * 3600), "3 h ago");
        assert_eq!(word(2 * 86_400), "2 d ago");
    }

    #[test]
    fn two_pictures_are_the_same_when_their_pixels_are() {
        let make = |value| Content::Image(StoredImage::new(ImageData::new(1, 1, vec![value; 4]), 1, 1, "PNG"));
        assert!(make(1).is_same_as(&make(1)));
        assert!(!make(1).is_same_as(&make(2)));
        assert!(!make(1).is_same_as(&Content::Text("x".into())));
    }

    #[test]
    fn the_ages_read_in_indonesian() {
        store::language::with_override(store::language::Language::Indonesian, || {
            assert_eq!(age_text(Duration::from_secs(2)), "baru saja");
            assert_eq!(age_text(Duration::from_secs(5)), "5 dtk lalu");
            assert_eq!(age_text(Duration::from_secs(180)), "3 mnt lalu");
            assert_eq!(age_text(Duration::from_secs(2 * 3600)), "2 j lalu");
            assert_eq!(age_text(Duration::from_secs(4 * 86_400)), "4 hr lalu");
        });
    }
}
