//! The only page registry; registration order breaks priority ties.

mod calendar;
mod clipboard;
mod devices;
mod media;
mod shalat;
mod shelf;
mod stats;
mod timer;
mod todo;
mod weather;

use crate::page::Page;

pub fn all() -> Vec<Box<dyn Page>> {
    // Add a new page as the last line below (`Box::new(<name>::<Name>Page::new())`); an earlier line wins a priority tie.
    vec![
        Box::new(media::MediaPage::new()),
        Box::new(shelf::ShelfPage::new()),
        Box::new(calendar::CalendarPage::new()),
        Box::new(timer::TimerPage::new()),
        Box::new(todo::TodoPage::new()),
        Box::new(shalat::ShalatPage::new()),
        Box::new(stats::StatsPage::new()),
        Box::new(weather::WeatherPage::new()),
        Box::new(devices::DevicesPage::new()),
        Box::new(clipboard::ClipboardPage::new()),
    ]
}

#[cfg(test)]
#[derive(Clone, Copy, Default)]
pub struct SampleOptions {
    /// The media page plays a track: whether lyrics are on; `None` leaves it empty.
    pub media: Option<bool>,
    pub timer: bool,
    pub calendar: bool,
    /// The calendar page shows its week view instead of the month.
    pub calendar_week: bool,
    pub file_drag: bool,
    /// The shelf holds six files (`0`), none (`1`), seven (`2`) or twelve (`3`).
    pub shelf: u8,
    /// The todo page shows its sample list (`0`), the empty page (`1`), the empty page being added to (`2`), five tasks
    /// (`3`), four tasks and the input (`4`), a list with done tasks (`5`) or a task just deleted with Undo and a hovered
    /// row (`6`) or the list just cleared with Undo (`7`).
    pub todo: u8,
    /// The shalat page shows its collapsed mark (`1`) or its prayer-time message (`2`); `0` shows neither.
    /// `3` is the open page under ten minutes before Asr and `4` the open page after Isha.
    pub shalat_mark: u8,
    /// The weather page shows its days (`1`), its offline message (`2`), its no-location message (`4`), the rain banner (`3` thunder, `6` snow, `8` showers), the rain countdown badge (`5` rain, `7` snow, `9` showers) or its hours (`0`).
    pub weather: u8,
    /// The devices page shows scenario `n` of the design (see `devices/sample.rs`); `0` is five devices.
    pub devices: u8,
    /// The clipboard page shows scenario `n` of the design (see `clipboard/sample.rs`); `0` is the text selected, `7` two
    /// pins and a hovered row, `8` three pins.
    pub clipboard: u8,
}

/// The registered pages that have a sample, in registration order. A page without one is left out on purpose: the host
/// tests build their rigs from this list, and a page a contributor adds must not change tests it did not write.
#[cfg(test)]
pub fn samples(fonts: &crate::gfx::Fonts, directory: &std::path::Path, options: SampleOptions) -> Vec<Box<dyn Page>> {
    all().iter().filter_map(|page| sample_of(page.id(), fonts, directory, options)).collect()
}

#[cfg(test)]
fn sample_of(
    page_id: &str,
    fonts: &crate::gfx::Fonts,
    directory: &std::path::Path,
    options: SampleOptions,
) -> Option<Box<dyn Page>> {
    Some(match page_id {
        "media" => Box::new(match options.media {
            Some(lyrics_on) => media::MediaPage::sample(fonts, lyrics_on),
            None => media::MediaPage::sample_idle(fonts),
        }),
        "shelf" => Box::new(shelf::ShelfPage::sample(
            directory,
            fonts,
            [6, 0, 7, 12][usize::from(options.shelf).min(3)],
            options.file_drag,
        )),
        "calendar" => Box::new(match (options.calendar, options.calendar_week) {
            (true, false) => calendar::CalendarPage::sample(fonts),
            (true, true) => calendar::CalendarPage::sample_week(fonts),
            (false, _) => calendar::CalendarPage::new(),
        }),
        "timer" => Box::new(if options.timer { timer::TimerPage::sample(fonts) } else { timer::TimerPage::new() }),
        "todo" => Box::new(todo::TodoPage::sample(options.todo)),
        "shalat" => Box::new(match options.shalat_mark {
            1 => shalat::ShalatPage::sample_badge(fonts),
            2 => shalat::ShalatPage::sample_alert(fonts),
            3 => shalat::ShalatPage::sample_soon(),
            4 => shalat::ShalatPage::sample_after_isha(),
            _ => shalat::ShalatPage::sample(),
        }),
        "stats" => Box::new(stats::StatsPage::sample()),
        "weather" => Box::new(match options.weather {
            1 => weather::WeatherPage::sample(true),
            2 => weather::WeatherPage::sample_offline(fonts),
            4 => weather::WeatherPage::sample_no_location(fonts),
            5 => weather::WeatherPage::sample_rain_badge(fonts, "rain"),
            7 => weather::WeatherPage::sample_rain_badge(fonts, "snow"),
            9 => weather::WeatherPage::sample_rain_badge(fonts, "rainshowers_day"),
            3 => weather::WeatherPage::sample_alert(fonts, "heavyrainandthunder"),
            6 => weather::WeatherPage::sample_alert(fonts, "snow"),
            8 => weather::WeatherPage::sample_alert(fonts, "rainshowers_day"),
            _ => weather::WeatherPage::sample(false),
        }),
        "devices" => Box::new(devices::DevicesPage::sample(options.devices)),
        "clipboard" => Box::new(clipboard::ClipboardPage::sample(options.clipboard)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::source_scan::imports_between_units;

    #[test]
    fn a_page_imports_no_other_page() {
        let pages = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("pages");
        let offenders = imports_between_units(&pages, "pages", &[]);
        assert!(offenders.is_empty(), "a page may not import another page: {offenders:#?}");
    }

    #[test]
    fn samples_keep_the_registration_order_of_the_pages_that_have_one() {
        let fonts = crate::gfx::Fonts::new(1.0).unwrap();
        let directory = std::env::temp_dir().join("foldline-samples-order");
        let sampled: Vec<&str> =
            super::samples(&fonts, &directory, super::SampleOptions::default()).iter().map(|page| page.id()).collect();
        let registered: Vec<&str> = super::all().iter().map(|page| page.id()).collect();
        let mut remaining = registered.iter();
        let keeps_order = sampled.iter().all(|id| remaining.any(|registered_id| registered_id == id));
        assert!(keeps_order, "samples {sampled:?} are not in the order of all() {registered:?}");
        assert!(!sampled.is_empty());
    }

    #[test]
    fn header_ranks_run_wave_caps_muted_timer_event_prayer() {
        let ranks: std::collections::HashMap<&str, u8> =
            super::all().iter().map(|page| (page.id(), page.header_status_rank())).collect();
        assert_eq!((ranks["media"], ranks["timer"], ranks["calendar"], ranks["shalat"]), (0, 3, 4, 5));
    }
}
