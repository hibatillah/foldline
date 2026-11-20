//! The settings of the {{Title}} page. Keys start with the page id, and the notch reads them (see `docs/how-to/add-a-setting.md`).

use crate::{
    field,
    i18n::tr,
    layout::{Group, Item, Page},
};

const EXAMPLE_KEY: &str = "{{id}}.example";

pub fn page() -> Page {
    Page::plain(
        tr!("{{id}}.title"),
        vec![Group::titled(
            tr!("{{id}}.group_main"),
            vec![
                Item::new(tr!("{{id}}.example"), field::toggle(EXAMPLE_KEY, true))
                    .description(tr!("{{id}}.example_note")),
            ],
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_section_has_one_group() {
        assert_eq!(page().group_titles(), [tr!("{{id}}.group_main")]);
    }
}
