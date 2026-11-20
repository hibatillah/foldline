//! Test helpers that read the crate's own source to enforce which module may import which.

use std::path::{Path, PathBuf};

pub fn rust_files(directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(directory).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn leading_identifier(text: &str) -> &str {
    let text = text.trim_start();
    let end = text.find(|character: char| !character.is_alphanumeric() && character != '_').unwrap_or(text.len());
    &text[..end]
}

/// The first name after every `prefix` in `text`, including each item of a `prefix{..}` group.
pub fn names_after<'a>(text: &'a str, prefix: &str) -> Vec<&'a str> {
    let mut names = Vec::new();
    for (start, _) in text.match_indices(prefix) {
        let rest = &text[start + prefix.len()..];
        let Some(group) = rest.strip_prefix('{') else {
            names.push(leading_identifier(rest));
            continue;
        };
        let mut depth = 0;
        let mut item_start = 0;
        for (offset, character) in group.char_indices() {
            match character {
                '{' => depth += 1,
                '}' if depth == 0 => {
                    names.push(leading_identifier(&group[item_start..]));
                    break;
                }
                '}' => depth -= 1,
                ',' if depth == 0 => {
                    names.push(leading_identifier(&group[item_start..]));
                    item_start = offset + 1;
                }
                _ => {}
            }
        }
    }
    names.retain(|name| !name.is_empty());
    names
}

/// Every `super::super::..::name` path as `(number of super, name)`.
pub fn super_paths(text: &str) -> Vec<(usize, &str)> {
    const SUPER: &str = "super::";
    let mut paths = Vec::new();
    for (start, _) in text.match_indices(SUPER) {
        let before = &text[..start];
        let continues_a_path = before.ends_with(SUPER) || before.ends_with(|c: char| c.is_alphanumeric() || c == '_');
        if continues_a_path {
            continue;
        }
        let mut rest = &text[start..];
        let mut count = 0;
        while let Some(next) = rest.strip_prefix(SUPER) {
            count += 1;
            rest = next;
        }
        paths.push((count, leading_identifier(rest)));
    }
    paths
}

/// The other units (pages or activities) that `text` imports. `unit` is the one the file belongs to, `depth` the number
/// of modules between `crate::<root_module>` and the file (a `mod.rs` counts as its folder), and `units` every unit.
/// Inline `mod` blocks add depth the text cannot show, so a `super` chain counts when it is at least `depth` long.
pub fn other_units_imported<'a>(
    text: &'a str,
    root_module: &str,
    unit: &str,
    depth: usize,
    units: &[String],
) -> Vec<&'a str> {
    let mut found = names_after(text, &format!("crate::{root_module}::"));
    found.extend(
        super_paths(text)
            .into_iter()
            .filter(|(count, name)| *count >= depth && units.iter().any(|candidate| candidate == name))
            .map(|(_, name)| name),
    );
    found.retain(|name| *name != unit);
    found
}

/// One line per import of another unit under `root` (`src/pages`, `src/activities`); each names the file.
/// `shared_helpers` are modules in `root` that are no unit of their own and may be imported by any unit.
pub fn imports_between_units(root: &Path, root_module: &str, shared_helpers: &[&str]) -> Vec<String> {
    let mut units: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(root).unwrap().flatten() {
        let name = entry.path().file_stem().unwrap().to_string_lossy().into_owned();
        if name != "mod" {
            units.push(name);
        }
    }
    let mut offenders = Vec::new();
    for file in rust_files(root) {
        let mut segments: Vec<String> = file
            .strip_prefix(root)
            .unwrap()
            .with_extension("")
            .iter()
            .map(|part| part.to_string_lossy().into_owned())
            .collect();
        if segments.len() == 1 && segments[0] == "mod" {
            continue;
        }
        if segments.last().is_some_and(|last| last == "mod") {
            segments.pop();
        }
        let text = std::fs::read_to_string(&file).unwrap();
        for name in other_units_imported(&text, root_module, &segments[0], segments.len(), &units) {
            if shared_helpers.contains(&name) {
                continue;
            }
            offenders.push(format!("{} imports {root_module}::{name}", file.display()));
        }
    }
    offenders
}

/// One line per file under `root` that imports one of the crate modules in `forbidden` (`crate::pages`).
pub fn imports_of_modules(root: &Path, forbidden: &[&str]) -> Vec<String> {
    let mut offenders = Vec::new();
    for file in rust_files(root) {
        let text = std::fs::read_to_string(&file).unwrap();
        for module in names_after(&text, "crate::") {
            if forbidden.contains(&module) {
                offenders.push(format!("{} imports crate::{module}", file.display()));
            }
        }
    }
    offenders
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units() -> Vec<String> {
        ["media", "shelf", "todo"].map(String::from).to_vec()
    }

    #[test]
    fn a_use_group_is_read_item_by_item() {
        let text = "use crate::pages::{media::Track, shelf::{A, B}, todo};";
        assert_eq!(names_after(text, "crate::pages::"), ["media", "shelf", "todo"]);
    }

    #[test]
    fn a_super_chain_counts_its_steps_and_ignores_names_that_end_in_super() {
        let text = "use super::super::media::A; use super::x; use mysuper::y;";
        assert_eq!(super_paths(text), [(2, "media"), (1, "x")]);
    }

    #[test]
    fn another_page_is_found_by_crate_path_group_or_super_chain() {
        let text = "use crate::pages::shelf::A;\nuse crate::pages::{todo::B, media::C};\nuse super::super::shelf::D;";
        assert_eq!(other_units_imported(text, "pages", "media", 2, &units()), ["shelf", "todo", "shelf"]);
    }

    #[test]
    fn a_page_may_name_itself_and_a_short_super_chain_stays_inside_it() {
        let text = "use crate::pages::media::A;\nuse super::shelf::B;\nuse super::super::EXPANDED_WIDTH;";
        assert!(other_units_imported(text, "pages", "media", 2, &units()).is_empty());
    }
}
