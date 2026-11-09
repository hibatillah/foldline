//! Opt-in system stats; sample and redraw once a second only while visible.

mod cards;
mod charts;
mod format;
mod gpu;
mod layout;
mod publish;
mod readout;
mod rows;
mod sampler;
#[cfg(test)]
mod tests;

#[cfg(test)]
use self::sampler::Meter;
use self::{
    cards::{ChartCard, draw_network},
    charts::Accent,
    layout::{card_rects, inside, label_area, network_kind_area, network_row_area, sub_area, text, value_area},
    publish::{GPU_SEPARATOR, publish_known},
    readout::Readout,
    rows::{draw_disks, draw_gpus},
    sampler::{Sample, Sampler},
};
use crate::ui::palette::SURFACE_ROW;
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Size, UI_FONT, Weight},
    page::{Context, Frame, Mode, Page, PageSizes, Priority, TimerId},
};

/// Every card is this size; the page is as wide as its cards, so a missing GPU card leaves the others as they are.
const CARD_WIDTH: f32 = 132.0;
const CARD_HEIGHT: f32 = 86.0;
const RESTING_WIDTH: f32 = 150.0;
const MAX_DISKS: usize = 3;
const MAX_GPUS: usize = 2;
const DISKS_SETTING: &str = "stats.disks";
const GPUS_SETTING: &str = "stats.gpus";
const TILE_GAP: f32 = 8.0;
const TILE_RADIUS: f32 = 12.0;
const TILE_PADDING_X: f32 = 12.0;

const SAMPLE_TIMER: TimerId = TimerId(0);
const FIRST_SAMPLE_MILLISECONDS: u32 = 600;
const SAMPLE_INTERVAL_MILLISECONDS: u32 = 1000;

const VALUE: FontStyle = FontStyle::new(UI_FONT, 20.0, Weight::Medium, Align::Leading).tabular();
const RATE: FontStyle = FontStyle::new(UI_FONT, 14.0, Weight::Medium, Align::Leading).tabular();
/// The board asks for 600; only Regular and Medium are bundled, so Medium stands in.
const LIST_PERCENT: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Medium, Align::Trailing).tabular();

const UPLOAD_GRAY: Color = Color::rgb(0.549, 0.549, 0.580);
const DISK_GREEN: Color = Color::rgb(0.188, 0.820, 0.345);
const CPU_ACCENT: Accent = Accent { base: Color::rgb(0.302, 0.671, 0.969), light: Color::rgb(0.455, 0.753, 0.988) };
const MEMORY_ACCENT: Accent = Accent { base: Color::rgb(0.800, 0.365, 0.910), light: Color::rgb(0.898, 0.600, 0.969) };
const GPU_ACCENT: Accent = Accent { base: Color::rgb(0.941, 0.549, 0.0), light: Color::rgb(1.0, 0.663, 0.302) };

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Cpu,
    Memory,
    Gpus,
    Network,
    Disks,
}

/// CPU, memory, GPU, network, then disks, left to right; the GPU card appears only when Windows
/// reports a hardware GPU.
fn tiles_for(has_gpus: bool) -> Vec<Tile> {
    let mut tiles = vec![Tile::Cpu, Tile::Memory];
    if has_gpus {
        tiles.push(Tile::Gpus);
    }
    tiles.extend([Tile::Network, Tile::Disks]);
    tiles
}

/// The page is as wide as its cards, with a gap between them.
fn expanded_width(tiles: &[Tile]) -> f32 {
    tiles.len() as f32 * CARD_WIDTH + TILE_GAP * tiles.len().saturating_sub(1) as f32
}

pub struct StatsPage {
    sampler: Sampler,
    latest: Option<Sample>,
    /// Words and rows of `latest`; rebuilt with it and when a saved choice changes.
    readout: Option<Readout>,
    /// Drive names to show, in order; `None` shows the first drives.
    disk_choice: Option<Vec<String>>,
    /// GPU names to show, in order; `None` shows the first GPUs.
    gpu_choice: Option<Vec<String>>,
}

impl StatsPage {
    pub fn new() -> Self {
        Self { sampler: Sampler::new(), latest: None, readout: None, disk_choice: None, gpu_choice: None }
    }

    #[cfg(test)]
    pub fn sample() -> Self {
        let mut page = Self::new();
        let reading = page.sampler.sample();
        page.set_latest(Sample {
            cpu_percent: Some(75.0),
            memory_percent: 51.0,
            memory_used_bytes: 8_697_308_160,
            memory_total_bytes: 17_179_869_184,
            download_bytes_per_second: Some(2_400_000.0),
            upload_bytes_per_second: Some(120_000.0),
            disks: vec![
                Meter {
                    name: "C:".to_owned(),
                    capacity: Some((442_000_000_000, 494_000_000_000)),
                    percent: Some(90.0),
                },
                Meter {
                    name: "G:".to_owned(),
                    capacity: Some((300_000_000_000, 380_000_000_000)),
                    percent: Some(79.0),
                },
            ],
            gpus: vec![
                Meter { name: "RTX 4050".to_owned(), capacity: None, percent: Some(65.0) },
                Meter { name: "Radeon 760M".to_owned(), capacity: None, percent: Some(0.0) },
            ],
            ..reading
        });
        page
    }

    /// A saved list picks which disks and GPUs show; no list shows the first ones.
    fn read_choices(&mut self, context: &Context) {
        let list =
            |key: &str, separators: &[char]| context.settings.get_string(key).map(|text| choice_list(text, separators));
        self.disk_choice = list(DISKS_SETTING, &[',']);
        self.gpu_choice = list(GPUS_SETTING, &[GPU_SEPARATOR, ',']);
        self.rebuild_readout();
    }

    fn set_latest(&mut self, sample: Sample) {
        self.latest = Some(sample);
        self.rebuild_readout();
    }

    fn rebuild_readout(&mut self) {
        self.readout = self
            .latest
            .as_ref()
            .map(|sample| Readout::new(sample, self.disk_choice.as_deref(), self.gpu_choice.as_deref()));
    }

    /// The cards follow the saved GPU choice, so the page width does too.
    fn tiles(&self) -> Vec<Tile> {
        self.readout.as_ref().map_or_else(|| tiles_for(false), |readout| readout.tiles.clone())
    }
}

/// Names split at any of `separators`; the GPU list accepts `|` (the published form) and a comma (the older form),
/// using `|` alone when the text has one.
fn choice_list(text: &str, separators: &[char]) -> Vec<String> {
    let separator = separators.iter().find(|separator| text.contains(**separator)).copied().unwrap_or(',');
    text.split(separator).map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

impl Page for StatsPage {
    fn id(&self) -> &'static str {
        "stats"
    }

    fn name(&self) -> &'static str {
        "System stats"
    }

    fn on_start(&mut self, context: &mut Context) {
        self.read_choices(context);
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        self.read_choices(context);
    }

    fn is_opt_in(&self) -> bool {
        true
    }

    fn priority(&self, mode: Mode) -> Option<Priority> {
        (mode == Mode::Expanded).then_some(Priority::BACKGROUND)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: RESTING_WIDTH, expanded: Size::new(expanded_width(&self.tiles()), CARD_HEIGHT) }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let scale = frame.scale;
        let (Some(sample), Some(readout)) = (&self.latest, &self.readout) else {
            for tile in card_rects(frame.bounds, scale, 4) {
                canvas.fill_rounded_rect(tile, TILE_RADIUS * scale, SURFACE_ROW.faded(opacity));
            }
            return;
        };
        for (kind, rect) in readout.tiles.iter().zip(card_rects(frame.bounds, scale, readout.tiles.len())) {
            canvas.fill_rounded_rect(rect, TILE_RADIUS * scale, SURFACE_ROW.faded(opacity));
            match kind {
                Tile::Cpu => ChartCard::cpu(readout).draw(canvas, rect, scale, opacity),
                Tile::Memory => ChartCard::memory(readout).draw(canvas, rect, scale, opacity),
                Tile::Gpus => draw_gpus(canvas, rect, scale, &readout.gpus, opacity),
                Tile::Network => draw_network(canvas, rect, scale, readout, sample.network_kind, opacity),
                Tile::Disks => draw_disks(canvas, rect, scale, &readout.disks, opacity),
            }
        }
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if is_shown {
            self.sampler.reset();
            let sample = self.sampler.sample();
            publish_known(context.settings, &sample.disks, &sample.gpus);
            self.set_latest(sample);
            context.set_timer(SAMPLE_TIMER, FIRST_SAMPLE_MILLISECONDS);
        } else {
            context.kill_timer(SAMPLE_TIMER);
            self.sampler.release();
        }
        context.request_repaint();
    }

    fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        if timer != SAMPLE_TIMER {
            return;
        }
        let sample = self.sampler.sample();
        self.set_latest(sample);
        context.set_timer(SAMPLE_TIMER, SAMPLE_INTERVAL_MILLISECONDS);
        context.request_repaint();
    }
}
