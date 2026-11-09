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
    /// (`3`), four tasks and the input (`4`) or a list with done tasks (`5`).
    pub todo: u8,
    /// The shalat page shows its collapsed mark (`1`) or its prayer-time message (`2`); `0` shows neither.
    /// `3` is the open page under ten minutes before Asr and `4` the open page after Isha.
    pub shalat_mark: u8,
    /// The weather page shows its days (`1`), its offline message (`2`), its no-location message (`4`), the rain banner (`3`), the rain countdown badge (`5`) or its hours (`0`).
    pub weather: u8,
    /// The devices page shows scenario `n` of the design (see `devices/sample.rs`); `0` is five devices.
    pub devices: u8,
    /// The clipboard page shows scenario `n` of the design (see `clipboard/sample.rs`); `0` is the text selected.
    pub clipboard: u8,
}

#[cfg(test)]
pub fn samples(fonts: &crate::gfx::Fonts, directory: &std::path::Path, options: SampleOptions) -> Vec<Box<dyn Page>> {
    vec![
        Box::new(match options.media {
            Some(lyrics_on) => media::MediaPage::sample(fonts, lyrics_on),
            None => media::MediaPage::sample_idle(fonts),
        }),
        Box::new(shelf::ShelfPage::sample(
            directory,
            fonts,
            [6, 0, 7, 12][usize::from(options.shelf).min(3)],
            options.file_drag,
        )),
        Box::new(match (options.calendar, options.calendar_week) {
            (true, false) => calendar::CalendarPage::sample(fonts),
            (true, true) => calendar::CalendarPage::sample_week(fonts),
            (false, _) => calendar::CalendarPage::new(),
        }),
        Box::new(if options.timer { timer::TimerPage::sample(fonts) } else { timer::TimerPage::new() }),
        Box::new(todo::TodoPage::sample(options.todo)),
        Box::new(match options.shalat_mark {
            1 => shalat::ShalatPage::sample_badge(fonts),
            2 => shalat::ShalatPage::sample_alert(fonts),
            3 => shalat::ShalatPage::sample_soon(),
            4 => shalat::ShalatPage::sample_after_isha(),
            _ => shalat::ShalatPage::sample(),
        }),
        Box::new(stats::StatsPage::sample()),
        Box::new(match options.weather {
            1 => weather::WeatherPage::sample(true),
            2 => weather::WeatherPage::sample_offline(fonts),
            4 => weather::WeatherPage::sample_no_location(fonts),
            5 => weather::WeatherPage::sample_rain_badge(fonts),
            3 => weather::WeatherPage::sample_alert(fonts),
            _ => weather::WeatherPage::sample(false),
        }),
        Box::new(devices::DevicesPage::sample(options.devices)),
        Box::new(clipboard::ClipboardPage::sample(options.clipboard)),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn header_ranks_run_wave_caps_muted_timer_event_prayer() {
        let ranks: std::collections::HashMap<&str, u8> =
            super::all().iter().map(|page| (page.id(), page.header_status_rank())).collect();
        assert_eq!((ranks["media"], ranks["timer"], ranks["calendar"], ranks["shalat"]), (0, 3, 4, 5));
    }
}
