//! The addresses the window opens in the browser.

/// The Foldline repository; the About and Pages buttons both open it.
pub const REPOSITORY_URL: &str = "https://github.com/hibatillah/foldline";
/// The release notes.
pub const RELEASES_URL: &str = "https://fold.encure.id/releases";

/// The repository address as it reads on a button: without the scheme.
pub fn repository_label() -> &'static str {
    REPOSITORY_URL.trim_start_matches("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repository_button_reads_the_address_without_its_scheme() {
        assert_eq!(repository_label(), "github.com/hibatillah/foldline");
    }
}
