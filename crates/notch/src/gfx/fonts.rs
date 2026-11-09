use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
};

use windows::{
    Win32::Graphics::DirectWrite::{
        DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_FEATURE, DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES,
        DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_MEDIUM,
        DWRITE_FONT_WEIGHT_NORMAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT,
        DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING,
        DWRITE_TEXT_METRICS, DWRITE_TEXT_RANGE, DWRITE_TRIMMING, DWRITE_TRIMMING_GRANULARITY_CHARACTER,
        DWRITE_WORD_WRAPPING_NO_WRAP, DWRITE_WORD_WRAPPING_WRAP, DWriteCreateFactory, IDWriteFactory, IDWriteFactory5,
        IDWriteFontCollection, IDWriteFontCollection1, IDWriteInMemoryFontFileLoader, IDWriteTextFormat,
        IDWriteTextLayout, IDWriteTypography,
    },
    core::{HSTRING, Interface, Result, w},
};

use super::text_cache::{TextCache, TextKey};

/// Bundled under the SIL Open Font License; see assets/fonts/Geist-LICENSE.txt.
pub const UI_FONT: &str = "Geist";
/// Used when the bundled font cannot be loaded (Windows older than 10 version 1803).
const FALLBACK_FONT: &str = "Segoe UI";

const BUNDLED_FONT_FILES: [&[u8]; 2] = [
    include_bytes!("../../../../assets/fonts/Geist-Regular.ttf"),
    include_bytes!("../../../../assets/fonts/Geist-Medium.ttf"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weight {
    Regular,
    Medium,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Align {
    Leading,
    Center,
    Trailing,
}

/// A text style. Pages declare their styles as `const`s; the first use builds and caches the platform font.
#[derive(Clone, Copy, Debug)]
pub struct FontStyle {
    pub family: &'static str,
    /// Size in device-independent pixels.
    pub size: f32,
    pub weight: Weight,
    pub align: Align,
    /// Wrap long text onto more lines instead of cutting it with an ellipsis.
    pub wrap: bool,
    /// Give every digit the same width, so a number that changes keeps its width and neighbours stay put.
    pub tabular: bool,
}

impl FontStyle {
    pub const fn new(family: &'static str, size: f32, weight: Weight, align: Align) -> Self {
        Self { family, size, weight, align, wrap: false, tabular: false }
    }

    pub const fn wrapped(self) -> Self {
        Self { wrap: true, ..self }
    }

    pub const fn tabular(self) -> Self {
        Self { tabular: true, ..self }
    }

    fn key(&self) -> StyleKey {
        (self.family, self.size.to_bits(), self.weight, self.align, self.wrap)
    }
}

pub(super) type StyleKey = (&'static str, u32, Weight, Align, bool);

/// Builds text formats on demand and measures text. Single-threaded, like the rest of the drawing code.
pub struct Fonts {
    factory: IDWriteFactory,
    bundled: Option<IDWriteFontCollection1>,
    scale: f32,
    cache: RefCell<HashMap<StyleKey, IDWriteTextFormat>>,
    tabular_typography: OnceCell<IDWriteTypography>,
    wide_scratch: RefCell<Vec<u16>>,
    widths: RefCell<TextCache<f32>>,
    layouts: RefCell<TextCache<IDWriteTextLayout>>,
    #[cfg(test)]
    builds: BuildCounts,
}

#[cfg(test)]
#[derive(Default)]
struct BuildCounts {
    typographies: std::cell::Cell<u32>,
    layouts: std::cell::Cell<u32>,
}

/// A measuring layout is never wrapped or cut, so these only need to be larger than any single line.
const MEASURE_WIDTH: f32 = 4000.0;
const MEASURE_HEIGHT: f32 = 200.0;

impl Fonts {
    pub fn new(scale: f32) -> Result<Self> {
        let factory: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        let bundled = load_bundled_fonts(&factory).ok();
        Ok(Self {
            factory,
            bundled,
            scale,
            cache: RefCell::new(HashMap::new()),
            tabular_typography: OnceCell::new(),
            wide_scratch: RefCell::new(Vec::new()),
            widths: RefCell::new(TextCache::new()),
            layouts: RefCell::new(TextCache::new()),
            #[cfg(test)]
            builds: BuildCounts::default(),
        })
    }

    pub(super) fn format(&self, style: &FontStyle) -> Option<IDWriteTextFormat> {
        let key = style.key();
        if let Some(format) = self.cache.borrow().get(&key) {
            return Some(format.clone());
        }
        let format = self.build(style).ok()?;
        self.cache.borrow_mut().insert(key, format.clone());
        Some(format)
    }

    /// Single-line width in device-independent pixels.
    pub fn measure(&self, style: &FontStyle, text: &str) -> f32 {
        let key = text_key(style, MEASURE_WIDTH, MEASURE_HEIGHT);
        if let Some(width) = self.widths.borrow().get(&key, text) {
            return width;
        }
        let Some(layout) = self.build_layout(style, text, MEASURE_WIDTH, MEASURE_HEIGHT) else { return 0.0 };
        let mut metrics = DWRITE_TEXT_METRICS::default();
        if unsafe { layout.GetMetrics(&mut metrics) }.is_err() {
            return 0.0;
        }
        let width = metrics.widthIncludingTrailingWhitespace / self.scale;
        self.widths.borrow_mut().insert(key, text, width);
        width
    }

    /// Size is in device pixels. Recent layouts are reused, so redrawing the same text builds nothing.
    pub(super) fn layout(&self, style: &FontStyle, text: &str, width: f32, height: f32) -> Option<IDWriteTextLayout> {
        let key = text_key(style, width, height);
        if let Some(layout) = self.layouts.borrow().get(&key, text) {
            return Some(layout);
        }
        let layout = self.build_layout(style, text, width, height)?;
        self.layouts.borrow_mut().insert(key, text, layout.clone());
        Some(layout)
    }

    /// Hands out the UTF-16 form of `text` from one reused buffer.
    pub(super) fn with_wide_text<R>(&self, text: &str, use_text: impl FnOnce(&[u16]) -> R) -> R {
        let mut wide = self.wide_scratch.borrow_mut();
        wide.clear();
        wide.extend(text.encode_utf16());
        use_text(&wide)
    }

    fn build_layout(&self, style: &FontStyle, text: &str, width: f32, height: f32) -> Option<IDWriteTextLayout> {
        let format = self.format(style)?;
        let typography = if style.tabular { Some(self.tabular_typography()?) } else { None };
        self.with_wide_text(text, |wide_text| {
            let layout = unsafe { self.factory.CreateTextLayout(wide_text, &format, width, height) }.ok()?;
            #[cfg(test)]
            self.builds.layouts.set(self.builds.layouts.get() + 1);
            if let Some(typography) = typography {
                let range = DWRITE_TEXT_RANGE { startPosition: 0, length: wide_text.len() as u32 };
                unsafe { layout.SetTypography(typography, range) }.ok()?;
            }
            Some(layout)
        })
    }

    fn tabular_typography(&self) -> Option<&IDWriteTypography> {
        if let Some(typography) = self.tabular_typography.get() {
            return Some(typography);
        }
        let typography = unsafe { self.factory.CreateTypography() }.ok()?;
        let feature = DWRITE_FONT_FEATURE { nameTag: DWRITE_FONT_FEATURE_TAG_TABULAR_FIGURES, parameter: 1 };
        unsafe { typography.AddFontFeature(feature) }.ok()?;
        #[cfg(test)]
        self.builds.typographies.set(self.builds.typographies.get() + 1);
        Some(self.tabular_typography.get_or_init(|| typography))
    }

    fn build(&self, style: &FontStyle) -> Result<IDWriteTextFormat> {
        let weight: DWRITE_FONT_WEIGHT = match style.weight {
            Weight::Regular => DWRITE_FONT_WEIGHT_NORMAL,
            Weight::Medium => DWRITE_FONT_WEIGHT_MEDIUM,
        };
        let alignment: DWRITE_TEXT_ALIGNMENT = match style.align {
            Align::Leading => DWRITE_TEXT_ALIGNMENT_LEADING,
            Align::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
            Align::Trailing => DWRITE_TEXT_ALIGNMENT_TRAILING,
        };
        let (family, collection) = match (&self.bundled, style.family) {
            (Some(bundled), UI_FONT) => (UI_FONT, Some(bundled.cast::<IDWriteFontCollection>()?)),
            (None, UI_FONT) => (FALLBACK_FONT, None),
            (_, family) => (family, None),
        };
        let format = unsafe {
            self.factory.CreateTextFormat(
                &HSTRING::from(family),
                collection.as_ref(),
                weight,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                style.size * self.scale,
                w!("en-us"),
            )
        }?;
        let word_wrapping = if style.wrap { DWRITE_WORD_WRAPPING_WRAP } else { DWRITE_WORD_WRAPPING_NO_WRAP };
        let trimming =
            DWRITE_TRIMMING { granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER, delimiter: 0, delimiterCount: 0 };
        unsafe {
            format.SetTextAlignment(alignment)?;
            format.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
            format.SetWordWrapping(word_wrapping)?;
            let ellipsis = self.factory.CreateEllipsisTrimmingSign(&format)?;
            format.SetTrimming(&trimming, &ellipsis)?;
        }
        Ok(format)
    }
}

fn text_key(style: &FontStyle, width: f32, height: f32) -> TextKey {
    TextKey { style: style.key(), tabular: style.tabular, width: width.to_bits(), height: height.to_bits() }
}

fn load_bundled_fonts(factory: &IDWriteFactory) -> Result<IDWriteFontCollection1> {
    let factory: IDWriteFactory5 = factory.cast()?;
    unsafe {
        let loader: IDWriteInMemoryFontFileLoader = factory.CreateInMemoryFontFileLoader()?;
        factory.RegisterFontFileLoader(&loader)?;
        let builder = factory.CreateFontSetBuilder()?;
        for data in BUNDLED_FONT_FILES {
            // SAFETY: the bundled bytes are `static`, so they outlive every reference the loader keeps.
            let file = loader.CreateInMemoryFontFileReference(
                &factory,
                data.as_ptr().cast(),
                data.len() as u32,
                None::<&windows::core::IUnknown>,
            )?;
            builder.AddFontFile(&file)?;
        }
        factory.CreateFontCollectionFromFontSet(&builder.CreateFontSet()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_font_loads_and_is_what_text_is_measured_in() {
        let fonts = Fonts::new(1.0).unwrap();
        assert!(fonts.bundled.is_some(), "Geist should load from the executable");
        let bundled = FontStyle::new(UI_FONT, 14.0, Weight::Regular, Align::Leading);
        let system = FontStyle::new(FALLBACK_FONT, 14.0, Weight::Regular, Align::Leading);
        assert_ne!(fonts.measure(&bundled, "Notification"), fonts.measure(&system, "Notification"));
    }

    #[test]
    fn medium_text_is_wider_than_regular_text() {
        let fonts = Fonts::new(1.0).unwrap();
        let regular = FontStyle::new(UI_FONT, 14.0, Weight::Regular, Align::Leading);
        let medium = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading);
        assert!(fonts.measure(&medium, "Notification") > fonts.measure(&regular, "Notification"));
    }

    #[test]
    fn tabular_digits_share_one_width_and_proportional_digits_do_not() {
        let fonts = Fonts::new(1.0).unwrap();
        let proportional = FontStyle::new(UI_FONT, 28.0, Weight::Medium, Align::Leading);
        let tabular = proportional.tabular();
        assert!(fonts.measure(&proportional, "1111") < fonts.measure(&proportional, "0000"));
        assert_eq!(fonts.measure(&tabular, "1111"), fonts.measure(&tabular, "0000"));
    }

    #[test]
    fn repeated_tabular_measures_build_one_typography_and_one_layout_per_text() {
        let fonts = Fonts::new(1.0).unwrap();
        let tabular = FontStyle::new(UI_FONT, 28.0, Weight::Medium, Align::Leading).tabular();
        let first = fonts.measure(&tabular, "12:30");
        for _ in 0..5 {
            assert_eq!(fonts.measure(&tabular, "12:30"), first);
        }
        assert_eq!(fonts.builds.layouts.get(), 1);
        fonts.measure(&tabular, "12:31");
        assert_eq!(fonts.builds.layouts.get(), 2);
        assert_eq!(fonts.builds.typographies.get(), 1);
    }

    #[test]
    fn drawing_the_same_tabular_text_again_reuses_its_layout() {
        let fonts = Fonts::new(1.0).unwrap();
        let tabular = FontStyle::new(UI_FONT, 13.0, Weight::Regular, Align::Center).tabular();
        for _ in 0..3 {
            assert!(fonts.layout(&tabular, "24\u{b0}", 40.0, 16.0).is_some());
        }
        assert_eq!(fonts.builds.layouts.get(), 1);
        assert!(fonts.layout(&tabular, "24\u{b0}", 50.0, 16.0).is_some());
        assert_eq!(fonts.builds.layouts.get(), 2);
        assert_eq!(fonts.builds.typographies.get(), 1);
    }

    #[test]
    fn a_cached_measure_matches_the_first_one_for_text_of_different_widths() {
        let fonts = Fonts::new(1.0).unwrap();
        let style = FontStyle::new(UI_FONT, 14.0, Weight::Regular, Align::Leading);
        let narrow = fonts.measure(&style, "ii");
        let wide = fonts.measure(&style, "WW");
        assert!(narrow < wide);
        assert_eq!(fonts.measure(&style, "ii"), narrow);
        assert_eq!(fonts.measure(&style, "WW"), wide);
    }

    #[test]
    fn the_wide_text_buffer_holds_only_the_current_text() {
        let fonts = Fonts::new(1.0).unwrap();
        fonts.with_wide_text("longer text", |wide| assert_eq!(wide.len(), 11));
        fonts.with_wide_text("ab", |wide| assert_eq!(wide, [u16::from(b'a'), u16::from(b'b')]));
    }
}
