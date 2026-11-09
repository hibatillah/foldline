//! The window's colors: a dark or light set that follows the system theme.
//!
//! The frame is a neutral near-black (dark theme) or white (light theme) at partial opacity over a blurred backdrop, so
//! the desktop shows through; it carries the sidebar directly. The content card is the one solid surface. Blocks,
//! buttons and inputs sit on the card, and one blue accent marks icons, the active breadcrumb, switches, links, focus
//! rings and the selected option.

use std::sync::OnceLock;

use gpui::{App, Hsla, WindowBackgroundAppearance, rgba, transparent_black};
use gpui_component::{ActiveTheme as _, Theme};

use crate::FONT_FAMILY;

/// Every color is `0xRRGGBBAA`; only the frame colors are translucent.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Behind everything, translucent; the sidebar sits on it and the content card is inset into it.
    pub frame: u32,
    /// The frame in the `NOTCH_SETTINGS_SOLID=1` fallback: a neutral, fully opaque color.
    pub frame_solid: u32,
    pub card: u32,
    pub card_border: u32,
    /// The card's header strip: a little lighter than the card so it reads as its own band.
    pub header: u32,
    /// A section's bordered block, one step above the card.
    pub block: u32,
    /// A field or a button, darker than the block it sits in.
    pub input: u32,
    pub input_border: u32,
    pub divider: u32,
    pub option: u32,
    pub option_selected: u32,
    pub option_border: u32,
    pub title: u32,
    pub muted: u32,
    /// A switch that is off.
    pub switch_off: u32,
    /// The blue of icons, links, the page title, switches that are on and the selected option's border.
    pub accent: u32,
}

impl Palette {
    pub const DARK: Palette = Palette {
        frame: 0x101010a3,
        frame_solid: 0x141414ff,
        card: 0x161618ff,
        card_border: 0x2a2a2eff,
        header: 0x1b1c21ff,
        block: 0x202022ff,
        input: 0x151517ff,
        input_border: 0x303035ff,
        divider: 0x2c2c30ff,
        option: 0x151517ff,
        option_selected: 0x1b2535ff,
        option_border: 0x2e2e33ff,
        title: 0xf4f6faff,
        muted: 0xa3a3aaff,
        switch_off: 0x3a3a40ff,
        accent: 0x4c9affff,
    };

    pub const LIGHT: Palette = Palette {
        frame: 0xffffff59,
        frame_solid: 0xeeeeeeff,
        card: 0xffffffff,
        card_border: 0xd9dde6ff,
        header: 0xf5f7fbff,
        block: 0xf8f9fcff,
        input: 0xffffffff,
        input_border: 0xd0d5e0ff,
        divider: 0xe6e9f0ff,
        option: 0xffffffff,
        option_selected: 0xeaf1ffff,
        option_border: 0xd5dae4ff,
        title: 0x0b0d14ff,
        muted: 0x4b4d55ff,
        switch_off: 0xcdd2dcff,
        accent: 0x2563ebff,
    };

    pub fn for_theme(is_dark: bool) -> Palette {
        if is_dark { Self::DARK } else { Self::LIGHT }
    }
}

/// Whether the frame lets the backdrop through. `NOTCH_SETTINGS_SOLID=1` turns it off for a system where blur
/// misbehaves: the frame is then a neutral, fully opaque color.
pub fn translucent() -> bool {
    static TRANSLUCENT: OnceLock<bool> = OnceLock::new();
    *TRANSLUCENT.get_or_init(|| std::env::var("NOTCH_SETTINGS_SOLID").map(|value| value.trim() != "1").unwrap_or(true))
}

/// The window's backdrop: blurred (acrylic) behind the translucent frame, or opaque in the solid fallback.
pub fn window_background() -> WindowBackgroundAppearance {
    if translucent() { WindowBackgroundAppearance::Blurred } else { WindowBackgroundAppearance::Opaque }
}

fn current(cx: &App) -> Palette {
    Palette::for_theme(cx.theme().is_dark())
}

fn color(value: u32) -> Hsla {
    rgba(value).into()
}

/// The frame's color: translucent, or the neutral opaque one in the solid fallback.
fn frame_color(colors: Palette) -> Hsla {
    color(if translucent() { colors.frame } else { colors.frame_solid })
}

/// The frame around the content card, also the sidebar's surface: one flat neutral color, no gradient or glow.
pub fn frame_fill(cx: &App) -> Hsla {
    frame_color(current(cx))
}

/// The content card's thin outline, also the outline of a section's block.
pub fn card_border(cx: &App) -> Hsla {
    color(current(cx).card_border)
}

/// The fill of a section's block.
pub fn block_fill(cx: &App) -> Hsla {
    color(current(cx).block)
}

/// The field's and the button's own background, darker than the block it sits in on the dark theme.
pub fn input_background(cx: &App) -> Hsla {
    color(current(cx).input)
}

/// The thin outline of a field or a button.
pub fn input_border(cx: &App) -> Hsla {
    color(current(cx).input_border)
}

/// A faint wash over the translucent frame: the selected and hovered sidebar item and a hovered window button.
pub fn wash(cx: &App) -> Hsla {
    if cx.theme().is_dark() { gpui::white().opacity(0.08) } else { gpui::black().opacity(0.06) }
}

/// The line between two rows, and under the header.
pub fn row_divider(cx: &App) -> Hsla {
    color(current(cx).divider)
}

/// The fill of an option card: tinted blue when selected, darker than the block around it otherwise.
pub fn option_fill(selected: bool, cx: &App) -> Hsla {
    let colors = current(cx);
    color(if selected { colors.option_selected } else { colors.option })
}

/// The outline of an option card: the accent when selected, quiet otherwise.
pub fn option_border(selected: bool, cx: &App) -> Hsla {
    if selected { accent(cx) } else { color(current(cx).option_border) }
}

/// Muted text: descriptions, group labels and the breadcrumb's first part.
pub fn muted(cx: &App) -> Hsla {
    color(current(cx).muted)
}

/// The content card's solid fill; the only opaque surface besides the solid-fallback frame.
pub fn card(cx: &App) -> Hsla {
    color(current(cx).card)
}

/// The card header's fill.
pub fn header(cx: &App) -> Hsla {
    color(current(cx).header)
}

/// The page description and the section titles: the full title color.
pub fn heading(cx: &App) -> Hsla {
    color(current(cx).title)
}

/// The blue of icons, links and the page title.
pub fn accent(cx: &App) -> Hsla {
    color(current(cx).accent)
}

/// Gives the component library's theme this window's colors; the title bar stays clear so the card's header shows
/// through.
pub fn apply(cx: &mut App) {
    let colors = current(cx);
    let accent = color(colors.accent);
    let theme = Theme::global_mut(cx);
    theme.font_family = FONT_FAMILY.into();
    theme.font_size = gpui::px(crate::UI_FONT_SIZE);
    theme.sidebar = frame_color(colors);
    theme.sidebar_border = transparent_black();
    // gpui-component's Root paints `theme.background` over the whole window, so it must stay clear or the frame is opaque;
    // the content card paints `palette::card` itself.
    theme.background = transparent_black();
    theme.foreground = color(colors.title);
    theme.group_box = color(colors.card);
    theme.accordion = color(colors.card);
    theme.accordion_hover = color(colors.input);
    theme.border = color(colors.card_border);
    theme.muted_foreground = color(colors.muted);
    theme.title_bar = transparent_black();
    theme.title_bar_border = transparent_black();
    theme.primary = accent;
    theme.primary_hover = accent.opacity(0.9);
    theme.primary_active = accent.opacity(0.8);
    theme.primary_foreground = gpui::white();
    theme.ring = accent;
    theme.link = accent;
    theme.link_hover = accent.opacity(0.85);
    theme.link_active = accent.opacity(0.75);
    theme.selection = accent.opacity(0.3);
    theme.switch = color(colors.switch_off);
    theme.switch_thumb = gpui::white();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linear(channel: u32) -> f32 {
        let value = channel as f32 / 255.0;
        if value <= 0.03928 { value / 12.92 } else { ((value + 0.055) / 1.055).powf(2.4) }
    }

    fn luminance(value: u32) -> f32 {
        let (red, green, blue) = ((value >> 24) & 0xff, (value >> 16) & 0xff, (value >> 8) & 0xff);
        0.2126 * linear(red) + 0.7152 * linear(green) + 0.0722 * linear(blue)
    }

    fn contrast(foreground: u32, background: u32) -> f32 {
        let (a, b) = (luminance(foreground), luminance(background));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    /// The translucent `top` color laid over an opaque `backdrop` (both `0xRRGGBBAA`, the backdrop opaque).
    fn over(top: u32, backdrop: u32) -> u32 {
        let alpha = (top & 0xff) as f32 / 255.0;
        let mix = |shift: u32| {
            let (front, back) = (((top >> shift) & 0xff) as f32, ((backdrop >> shift) & 0xff) as f32);
            (front * alpha + back * (1.0 - alpha)).round() as u32
        };
        (mix(24) << 24) | (mix(16) << 16) | (mix(8) << 8) | 0xff
    }

    /// The frame over the brightest possible desktop (dark theme) or a mid-grey one (light theme, whose text is dark).
    fn worst_frame(palette: Palette) -> u32 {
        over(palette.frame, if palette.card == Palette::DARK.card { 0xffffffff } else { 0x808080ff })
    }

    /// The dark frame over a bright wallpaper (not pure white), where the muted group labels still have to read.
    fn bright_wallpaper_frame() -> u32 {
        over(Palette::DARK.frame, 0xc8c8c8ff)
    }

    #[test]
    fn titles_and_descriptions_read_on_the_card_in_both_themes() {
        for palette in [Palette::DARK, Palette::LIGHT] {
            assert!(contrast(palette.title, palette.card) >= 7.0, "{palette:?}");
            assert!(contrast(palette.muted, palette.card) >= 4.5, "{palette:?}");
            assert!(contrast(palette.muted, palette.block) >= 4.5, "{palette:?}");
        }
    }

    #[test]
    fn sidebar_text_reads_on_the_translucent_frame_over_a_worst_case_desktop() {
        for palette in [Palette::DARK, Palette::LIGHT] {
            assert!(contrast(palette.title, worst_frame(palette)) >= 4.5, "{palette:?}");
            assert!(contrast(palette.muted, worst_frame(palette)) >= 3.0 || palette.card == Palette::DARK.card);
        }
        assert!(contrast(Palette::DARK.muted, bright_wallpaper_frame()) >= 3.0);
    }

    #[test]
    fn the_light_title_reads_on_the_frame_over_a_mid_grey_desktop() {
        assert!(contrast(Palette::LIGHT.title, worst_frame(Palette::LIGHT)) >= 7.0);
    }

    #[test]
    fn the_accent_reads_as_text_on_the_card_and_as_an_icon_on_the_frame() {
        for palette in [Palette::DARK, Palette::LIGHT] {
            assert!(contrast(palette.accent, palette.card) >= 4.5, "{palette:?}");
            assert!(contrast(palette.accent, palette.header) >= 4.5, "{palette:?}");
            let frame =
                if palette.card == Palette::DARK.card { bright_wallpaper_frame() } else { worst_frame(palette) };
            assert!(contrast(palette.accent, frame) >= 2.0, "{palette:?}");
            assert!(contrast(palette.accent, palette.block) >= 3.0, "{palette:?}");
        }
    }

    #[test]
    fn the_accent_is_blue() {
        for palette in [Palette::DARK, Palette::LIGHT] {
            let (red, green, blue) =
                ((palette.accent >> 24) & 0xff, (palette.accent >> 16) & 0xff, (palette.accent >> 8) & 0xff);
            assert!(blue > green && green > red, "{palette:?}");
        }
    }

    #[test]
    fn only_the_frame_is_translucent() {
        for palette in [Palette::DARK, Palette::LIGHT] {
            assert!(palette.frame & 0xff < 0xff, "{palette:?}");
            for solid in [
                palette.frame_solid,
                palette.card,
                palette.card_border,
                palette.header,
                palette.block,
                palette.input,
                palette.divider,
            ] {
                assert_eq!(solid & 0xff, 0xff, "{palette:?}");
            }
        }
    }

    #[test]
    fn the_frame_opacity_is_in_the_chosen_range() {
        let opacity = |value: u32| (value & 0xff) as f32 / 255.0;
        assert!((0.55..=0.65).contains(&opacity(Palette::DARK.frame)));
        assert!((0.25..=0.40).contains(&opacity(Palette::LIGHT.frame)));
    }

    #[test]
    fn the_frames_are_neutral_not_blue_in_both_themes() {
        for value in [Palette::DARK.frame, Palette::DARK.frame_solid, Palette::LIGHT.frame, Palette::LIGHT.frame_solid]
        {
            let (red, green, blue) = ((value >> 24) & 0xff, (value >> 16) & 0xff, (value >> 8) & 0xff);
            assert!(red == green && green == blue, "{value:#010x}");
        }
    }

    #[test]
    fn the_dark_frame_is_near_black() {
        assert!(luminance(Palette::DARK.frame) < 0.01 && luminance(Palette::DARK.frame_solid) < 0.01);
    }

    #[test]
    fn the_solid_cards_are_lighter_than_the_dark_frame_and_the_solid_light_frame() {
        assert!(luminance(Palette::DARK.card) > luminance(Palette::DARK.frame_solid));
        assert!(luminance(Palette::LIGHT.card) > luminance(Palette::LIGHT.frame_solid));
    }

    #[test]
    fn a_dark_block_is_a_step_above_the_card_and_its_buttons_a_step_below_the_block() {
        let dark = Palette::DARK;
        assert!(luminance(dark.block) > luminance(dark.card));
        assert!(luminance(dark.input) < luminance(dark.block));
        assert!(contrast(dark.title, dark.input) >= 7.0);
    }

    #[test]
    fn the_header_sits_just_above_the_card_in_the_dark_theme_and_below_it_in_the_light_one() {
        let dark = Palette::DARK;
        assert!(luminance(dark.header) > luminance(dark.card) && luminance(dark.header) < luminance(dark.block));
        let light = Palette::LIGHT;
        assert!(luminance(light.header) < luminance(light.card));
    }

    #[test]
    fn titles_and_muted_text_read_on_the_header_and_on_a_block_in_both_themes() {
        for palette in [Palette::DARK, Palette::LIGHT] {
            assert!(contrast(palette.title, palette.header) >= 7.0, "{palette:?}");
            assert!(contrast(palette.muted, palette.header) >= 4.5, "{palette:?}");
            assert!(contrast(palette.title, palette.block) >= 7.0, "{palette:?}");
        }
    }

    #[test]
    fn the_theme_picks_its_palette() {
        assert_eq!(Palette::for_theme(true).frame, Palette::DARK.frame);
        assert_eq!(Palette::for_theme(false).frame, Palette::LIGHT.frame);
        assert_eq!(Palette::for_theme(true).accent, Palette::DARK.accent);
    }
}
