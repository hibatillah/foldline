//! The addresses the window opens in the browser.

/// The Foldline repository; the About and Pages buttons both open it.
pub const REPOSITORY_URL: &str = "https://github.com/hibatillah/foldline";
/// The release notes.
pub const RELEASES_URL: &str = "https://fold.hibatillah.com/releases";
/// The page of the latest release, where an update is downloaded.
pub const LATEST_RELEASE_URL: &str = "https://github.com/hibatillah/foldline/releases/latest";
/// The GitHub API address the update check reads.
pub const LATEST_RELEASE_API_URL: &str = "https://api.github.com/repos/hibatillah/foldline/releases/latest";

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
