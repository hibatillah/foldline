use super::{MAX_FILES, PREVIEW_PIXELS, Preview, ShelfPage, items};
use crate::{gfx::Fonts, platform};
use std::path::{Path, PathBuf};

impl ShelfPage {
    #[cfg(test)]
    pub fn sample(directory: &Path, fonts: &Fonts, file_count: usize, is_drag_over: bool) -> Self {
        let mut page = Self::new();
        page.storage = None;
        page.is_drag_over = is_drag_over;
        const NAMES: [&str; 12] = [
            "Report Q3.pdf",
            "Hero shot.png",
            "Invoice 0412.pdf",
            "Notes.txt",
            "release.zip",
            "Review.fig",
            "Budget 27.xlsx",
            "Contract.docx",
            "wallpaper-1440p.jpg",
            "Deck.pptx",
            "tasks.md",
            "Backup of a very long file name.zip",
        ];
        let files: Vec<PathBuf> = NAMES
            .iter()
            .take(file_count)
            .map(|name| {
                let path = directory.join(name);
                let _ = std::fs::write(&path, "x");
                path
            })
            .collect();
        page.shelf.add(files);
        page.refresh_names(fonts);
        for path in page.shelf.items().iter().take(MAX_FILES) {
            if !items::has_thumbnail(path) {
                continue;
            }
            if let Some(image) = platform::file_preview(path, PREVIEW_PIXELS) {
                page.previews.get_mut().insert(path.clone(), Preview::new(image));
            }
        }
        page
    }
}
