//! The only page registry; registration order breaks priority ties.

mod calendar;
mod media;
mod shalat;
mod shelf;
mod stats;
mod timer;
mod todo;

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
    ]
}

#[cfg(test)]
#[derive(Clone, Copy, Default)]
pub struct SampleOptions {
    /// The media page plays a track: `(lyrics on, title shown after a track change)`; `None` leaves it empty.
    pub media: Option<(bool, bool)>,
    pub timer: bool,
    pub calendar: bool,
    pub file_drag: bool,
    /// The shalat page shows its collapsed mark (`1`) or its prayer-time message (`2`); `0` shows neither.
    pub shalat_mark: u8,
}

#[cfg(test)]
pub fn samples(fonts: &crate::gfx::Fonts, directory: &std::path::Path, options: SampleOptions) -> Vec<Box<dyn Page>> {
    vec![
        Box::new(match options.media {
            Some((lyrics_on, show_title)) => media::MediaPage::sample(fonts, lyrics_on, show_title),
            None => media::MediaPage::sample_idle(fonts),
        }),
        Box::new(shelf::ShelfPage::sample(directory, fonts, options.file_drag)),
        Box::new(if options.calendar { calendar::CalendarPage::sample(fonts) } else { calendar::CalendarPage::new() }),
        Box::new(if options.timer { timer::TimerPage::sample(fonts) } else { timer::TimerPage::new() }),
        Box::new(todo::TodoPage::sample()),
        Box::new(match options.shalat_mark {
            1 => shalat::ShalatPage::sample_badge(fonts),
            2 => shalat::ShalatPage::sample_alert(fonts),
            _ => shalat::ShalatPage::sample(),
        }),
        Box::new(stats::StatsPage::sample()),
    ]
}
