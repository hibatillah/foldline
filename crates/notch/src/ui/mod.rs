//! Reusable widgets for pages, so every page looks and behaves the same way.

mod banner_buttons;
mod hud;
mod icon_button;
mod icons;
mod input;
mod outline;
pub mod palette;
mod pill_button;
mod ring;
mod scroll;
mod svg_icon;
mod text_field;
pub mod type_styles;

pub use banner_buttons::{
    BANNER_BUTTON_HEIGHT, BANNER_BUTTONS_GAP_ABOVE, BANNER_BUTTONS_HEIGHT, BannerButton, BannerButtons, BannerLabels,
};
pub use hud::{LEVEL_WIDTH, draw_level, draw_message, message_width};
pub use icon_button::IconButton;
pub use icons::Icon;
pub use input::{KeyInput, PointerEvent};
pub use outline::draw_dashed_outline;
pub use pill_button::PillButton;
pub use ring::Ring;
pub use scroll::ScrollOffset;
pub use svg_icon::{Stroke, draw_strokes};
pub use text_field::{FieldLook, FieldOutcome, TextField};

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    const FORBIDDEN_MODULES: [&str; 6] = ["page", "i18n", "host", "pages", "activity", "platform"];

    fn source_files(directory: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(directory).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                source_files(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    fn leading_identifier(text: &str) -> &str {
        let text = text.trim_start();
        let end = text.find(|character: char| !character.is_alphanumeric() && character != '_').unwrap_or(text.len());
        &text[..end]
    }

    /// The first module of every `crate::` path, including those inside a `use crate::{..}` group.
    fn crate_modules_used(text: &str) -> Vec<&str> {
        let mut modules = Vec::new();
        for (start, _) in text.match_indices("crate::") {
            let rest = &text[start + "crate::".len()..];
            let Some(group) = rest.strip_prefix('{') else {
                modules.push(leading_identifier(rest));
                continue;
            };
            let mut depth = 0;
            let mut item_start = 0;
            for (offset, character) in group.char_indices() {
                match character {
                    '{' => depth += 1,
                    '}' if depth == 0 => {
                        modules.push(leading_identifier(&group[item_start..]));
                        break;
                    }
                    '}' => depth -= 1,
                    ',' if depth == 0 => {
                        modules.push(leading_identifier(&group[item_start..]));
                        item_start = offset + 1;
                    }
                    _ => {}
                }
            }
        }
        modules.retain(|module| !module.is_empty());
        modules
    }

    #[test]
    fn the_drawing_layers_import_nothing_above_them() {
        let mut files = Vec::new();
        for layer in ["gfx", "ui"] {
            source_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(layer), &mut files);
        }
        let mut offenders = Vec::new();
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            for module in crate_modules_used(&text) {
                if FORBIDDEN_MODULES.contains(&module) {
                    offenders.push(format!("{} imports crate::{module}", file.display()));
                }
            }
        }
        assert!(offenders.is_empty(), "{offenders:#?}");
    }

    #[test]
    fn a_use_group_is_read_item_by_item() {
        let text = [
            "use crate",
            "::{\n    gfx::Canvas,\n    page::{KeyInput, Other},\n    ui::{A, B},\n};\nuse crate",
            "::i18n::tr;",
        ]
        .concat();
        assert_eq!(crate_modules_used(&text), ["gfx", "page", "ui", "i18n"]);
    }
}
