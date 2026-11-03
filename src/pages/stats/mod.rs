//! Opt-in system stats; sample every two seconds only while visible.

mod format;
mod gpu;
mod sampler;

use std::collections::VecDeque;

use self::{
    format::{memory_pair_text, percent_text, rate_text},
    sampler::{Meter, Sample, Sampler, chosen},
};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Rect, Size, UI_FONT, Weight},
    page::{Context, Control, Frame, Mode, Page, PageSizes, Priority, TimerId},
    ui::Icon,
};

const EXPANDED_SIZE: Size = Size::new(608.0, 86.0);
const RESTING_WIDTH: f32 = 150.0;
const MAX_DISKS: usize = 3;
const MAX_GPUS: usize = 2;
const DISKS_SETTING: &str = "stats.disks";
const GPUS_SETTING: &str = "stats.gpus";
/// A list tile is wider than a single-value tile so its names and bars fit.
const LIST_TILE_WEIGHT: f32 = 1.25;
const TILE_GAP: f32 = 8.0;
const TILE_RADIUS: f32 = 12.0;
const TILE_PADDING_X: f32 = 12.0;
const TILE_PADDING_Y: f32 = 7.0;
const GRAPHIC_TOP: f32 = 47.0;
const GRAPHIC_HEIGHT: f32 = 16.0;
const SUB_TOP: f32 = 66.0;
const BAR_HEIGHT: f32 = 5.0;
const HISTORY_LENGTH: usize = 30;
const LIST_TOP: f32 = 25.0;
const LIST_ROW_HEIGHT: f32 = 18.0;
const LIST_BAR_HEIGHT: f32 = 4.0;
const LIST_PERCENT_WIDTH: f32 = 30.0;
const LIST_GAP: f32 = 6.0;
const DISK_NAME_WIDTH: f32 = 22.0;
const STACKED_ROW_HEIGHT: f32 = 27.0;
/// How far below its name line a stacked row draws its bar.
const STACKED_BAR_OFFSET: f32 = 2.0;

const SAMPLE_TIMER: TimerId = TimerId(0);
const FIRST_SAMPLE_MILLISECONDS: u32 = 600;
const SAMPLE_INTERVAL_MILLISECONDS: u32 = 2000;

const MONO: &str = "Consolas";
const LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
const VALUE: FontStyle = FontStyle::new(MONO, 20.0, Weight::Medium, Align::Leading);
const RATE: FontStyle = FontStyle::new(MONO, 14.0, Weight::Medium, Align::Leading);
const LIST_NAME: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
const LIST_PERCENT: FontStyle = FontStyle::new(MONO, 11.0, Weight::Medium, Align::Trailing);

const TILE: Color = Color::rgb(0.078, 0.078, 0.094);
const TRACK: Color = Color::rgb(0.149, 0.149, 0.176);
const GRAY: Color = Color::rgb(0.604, 0.604, 0.651);
const NAME_GRAY: Color = Color::rgb(0.769, 0.769, 0.8);
const CPU_BLUE: Color = Color::rgb(0.302, 0.671, 0.969);
const MEMORY_PURPLE: Color = Color::rgb(0.745, 0.294, 0.859);
const DISK_GREEN: Color = Color::rgb(0.251, 0.753, 0.341);
const GPU_ORANGE: Color = Color::rgb(1.0, 0.573, 0.169);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Cpu,
    Memory,
    Gpus,
    Disks,
    Network,
}

impl Tile {
    fn weight(self) -> f32 {
        match self {
            Tile::Gpus | Tile::Disks => LIST_TILE_WEIGHT,
            _ => 1.0,
        }
    }
}

/// The GPU tile appears only when Windows reports a hardware GPU.
fn tiles_for(has_gpus: bool) -> Vec<Tile> {
    let mut tiles = vec![Tile::Cpu, Tile::Memory];
    if has_gpus {
        tiles.push(Tile::Gpus);
    }
    tiles.extend([Tile::Disks, Tile::Network]);
    tiles
}

pub struct StatsPage {
    sampler: Sampler,
    latest: Option<Sample>,
    cpu_history: VecDeque<f32>,
    /// Drive names to show, in order; `None` shows the first drives. A later setting fills it.
    disk_choice: Option<Vec<String>>,
    /// GPU names to show, in order; `None` shows the first GPUs. A later setting fills it.
    gpu_choice: Option<Vec<String>>,
}

impl StatsPage {
    pub fn new() -> Self {
        Self {
            sampler: Sampler::new(),
            latest: None,
            cpu_history: VecDeque::with_capacity(HISTORY_LENGTH),
            disk_choice: None,
            gpu_choice: None,
        }
    }

    #[cfg(test)]
    pub fn sample() -> Self {
        let mut page = Self::new();
        let reading = page.sampler.sample();
        for index in 0..HISTORY_LENGTH {
            let wave = 20.0 + 18.0 * ((index as f32) * 0.6).sin() + (index % 5) as f32 * 3.0;
            page.record(Sample {
                cpu_percent: Some(wave),
                memory_percent: 51.0,
                memory_used_bytes: 8_697_308_160,
                memory_total_bytes: 17_179_869_184,
                download_bytes_per_second: Some(2_400_000.0),
                upload_bytes_per_second: Some(120_000.0),
                disks: vec![
                    Meter { name: "C:".to_owned(), percent: Some(90.0) },
                    Meter { name: "G:".to_owned(), percent: Some(79.0) },
                ],
                gpus: vec![
                    Meter { name: "RTX 4050".to_owned(), percent: Some(23.0) },
                    Meter { name: "Radeon 760M".to_owned(), percent: Some(6.0) },
                ],
                ..reading.clone()
            });
        }
        page
    }

    /// A saved list picks which disks and GPUs show; no list shows the first ones.
    fn read_choices(&mut self, context: &Context) {
        let list = |key: &str| context.settings.get_string(key).map(choice_list);
        self.disk_choice = list(DISKS_SETTING);
        self.gpu_choice = list(GPUS_SETTING);
    }

    fn record(&mut self, sample: Sample) {
        if let Some(cpu) = sample.cpu_percent {
            if self.cpu_history.len() == HISTORY_LENGTH {
                self.cpu_history.pop_front();
            }
            self.cpu_history.push_back(cpu);
        }
        self.latest = Some(sample);
    }
}

fn choice_list(text: &str) -> Vec<String> {
    text.split(',').map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned).collect()
}

/// Splits the width between the tiles by weight, with a fixed gap between them.
fn tile_rects(bounds: Rect, scale: f32, weights: &[f32]) -> Vec<Rect> {
    let gap = TILE_GAP * scale;
    let total_weight: f32 = weights.iter().sum();
    let unit = (bounds.width() - gap * weights.len().saturating_sub(1) as f32) / total_weight;
    let mut left = bounds.left;
    weights
        .iter()
        .map(|weight| {
            let rect = Rect::new(left, bounds.top, left + weight * unit, bounds.bottom);
            left = rect.right + gap;
            rect
        })
        .collect()
}

fn inside(tile: Rect, scale: f32, left: f32, top: f32, bottom: f32) -> Rect {
    Rect::new(
        tile.left + left * scale,
        tile.top + top * scale,
        tile.right - TILE_PADDING_X * scale,
        tile.top + bottom * scale,
    )
}

fn draw_bar(canvas: &Canvas, tile: Rect, scale: f32, percent: f32, color: Color, opacity: f32) {
    let area = inside(tile, scale, TILE_PADDING_X, 0.0, 0.0);
    let center_y = tile.top + (GRAPHIC_TOP + GRAPHIC_HEIGHT / 2.0) * scale;
    let half = BAR_HEIGHT * scale / 2.0;
    let track = Rect::new(area.left, center_y - half, area.right, center_y + half);
    canvas.fill_rounded_rect(track, half, TRACK.faded(opacity));
    let fill_right = track.left + track.width() * (percent / 100.0).clamp(0.0, 1.0);
    if fill_right - track.left > half {
        canvas.fill_rounded_rect(Rect { right: fill_right, ..track }, half, color.faded(opacity));
    }
}

fn draw_history(canvas: &Canvas, tile: Rect, scale: f32, history: &VecDeque<f32>, opacity: f32) {
    let area = inside(tile, scale, TILE_PADDING_X, 0.0, 0.0);
    let bottom = tile.top + (GRAPHIC_TOP + GRAPHIC_HEIGHT) * scale;
    let slot = (area.right - area.left) / HISTORY_LENGTH as f32;
    let first_slot = HISTORY_LENGTH - history.len();
    for (position, value) in history.iter().enumerate() {
        let left = area.left + (first_slot + position) as f32 * slot;
        let height = (value / 100.0).clamp(0.0, 1.0) * GRAPHIC_HEIGHT * scale;
        let rect = Rect::new(left + slot * 0.15, bottom - height.max(1.5 * scale), left + slot * 0.85, bottom);
        canvas.fill_rounded_rect(rect, slot * 0.3, CPU_BLUE.faded(opacity * 0.85));
    }
}

fn text(canvas: &Canvas, style: &FontStyle, content: &str, area: Rect, color: Color, opacity: f32) {
    canvas.text(style, content, area, color.faded(opacity));
}

#[derive(Clone, Copy)]
enum RowLayout {
    /// Name, bar and percentage on one line, for short names like "C:".
    Inline { name_width: f32 },
    /// Name and percentage on one line with the bar under them, for long names like "Radeon 760M".
    Stacked,
}

struct ListStyle {
    title: &'static str,
    layout: RowLayout,
    color: Color,
}

fn draw_list(canvas: &Canvas, tile: Rect, scale: f32, meters: &[Meter], style: &ListStyle, opacity: f32) {
    let title_area = inside(tile, scale, TILE_PADDING_X, TILE_PADDING_Y, TILE_PADDING_Y + 14.0);
    text(canvas, &LABEL, style.title, title_area, GRAY, opacity);
    let row_height = match style.layout {
        RowLayout::Inline { .. } => LIST_ROW_HEIGHT,
        RowLayout::Stacked => STACKED_ROW_HEIGHT,
    };
    for (row, meter) in meters.iter().enumerate() {
        let top = LIST_TOP + row as f32 * row_height;
        let line = inside(tile, scale, TILE_PADDING_X, top, top + LIST_ROW_HEIGHT);
        let percent_left = line.right - LIST_PERCENT_WIDTH * scale;
        let shown = meter.percent.map_or_else(|| "\u{2013}".to_owned(), percent_text);
        text(canvas, &LIST_PERCENT, &shown, Rect { left: percent_left, ..line }, Color::WHITE, opacity);
        let half = LIST_BAR_HEIGHT * scale / 2.0;
        let track = match style.layout {
            RowLayout::Inline { name_width } => {
                let name_right = line.left + name_width * scale;
                text(canvas, &LIST_NAME, &meter.name, Rect { right: name_right, ..line }, NAME_GRAY, opacity);
                let center_y = (line.top + line.bottom) / 2.0;
                Rect::new(
                    name_right + LIST_GAP * scale,
                    center_y - half,
                    percent_left - LIST_GAP * scale,
                    center_y + half,
                )
            }
            RowLayout::Stacked => {
                let name_area = Rect { right: percent_left - LIST_GAP * scale, ..line };
                text(canvas, &LIST_NAME, &meter.name, name_area, NAME_GRAY, opacity);
                let center_y = line.bottom + STACKED_BAR_OFFSET * scale;
                Rect::new(line.left, center_y - half, line.right, center_y + half)
            }
        };
        if track.width() <= 0.0 {
            continue;
        }
        canvas.fill_rounded_rect(track, half, TRACK.faded(opacity));
        let fill_right = track.left + track.width() * (meter.percent.unwrap_or(0.0) / 100.0).clamp(0.0, 1.0);
        if fill_right - track.left > half {
            canvas.fill_rounded_rect(Rect { right: fill_right, ..track }, half, style.color.faded(opacity));
        }
    }
}

impl Page for StatsPage {
    fn id(&self) -> &'static str {
        "stats"
    }

    fn name(&self) -> &'static str {
        "System stats"
    }

    fn icon(&self) -> Icon {
        Icon::Chart
    }

    fn settings(&self) -> Vec<Control> {
        let names = |meters: &[Meter]| meters.iter().map(|meter| meter.name.clone()).collect::<Vec<_>>();
        let (disks, gpus) = match &self.latest {
            Some(sample) => (names(&sample.disks), names(&sample.gpus)),
            None => (Vec::new(), Vec::new()),
        };
        let mut controls = vec![Control::Picks { key: DISKS_SETTING, label: "Disks", options: disks, max: MAX_DISKS }];
        if !gpus.is_empty() {
            controls.push(Control::Picks { key: GPUS_SETTING, label: "GPUs", options: gpus, max: MAX_GPUS });
        }
        controls
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
        PageSizes { resting_width: RESTING_WIDTH, expanded: EXPANDED_SIZE }
    }

    fn draw_resting(&self, _canvas: &Canvas, _frame: &Frame, _opacity: f32) {}

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let scale = frame.scale;
        let Some(sample) = &self.latest else {
            for tile in tile_rects(frame.bounds, scale, &[1.0; 4]) {
                canvas.fill_rounded_rect(tile, TILE_RADIUS * scale, TILE.faded(opacity));
            }
            return;
        };
        let gpus = chosen(&sample.gpus, self.gpu_choice.as_deref(), MAX_GPUS);
        let disks = chosen(&sample.disks, self.disk_choice.as_deref(), MAX_DISKS);
        let tiles = tiles_for(!gpus.is_empty());
        let weights: Vec<f32> = tiles.iter().map(|tile| tile.weight()).collect();
        let label_area = |tile: Rect| inside(tile, scale, TILE_PADDING_X, TILE_PADDING_Y, TILE_PADDING_Y + 14.0);
        let value_area = |tile: Rect| inside(tile, scale, TILE_PADDING_X, 19.0, 45.0);
        let sub_area = |tile: Rect| inside(tile, scale, TILE_PADDING_X, SUB_TOP, SUB_TOP + 14.0);

        for (kind, rect) in tiles.iter().zip(tile_rects(frame.bounds, scale, &weights)) {
            canvas.fill_rounded_rect(rect, TILE_RADIUS * scale, TILE.faded(opacity));
            match kind {
                Tile::Cpu => {
                    text(canvas, &LABEL, "CPU", label_area(rect), GRAY, opacity);
                    let cpu = sample.cpu_percent.map_or_else(|| "\u{2013}".to_owned(), percent_text);
                    text(canvas, &VALUE, &cpu, value_area(rect), Color::WHITE, opacity);
                    draw_history(canvas, rect, scale, &self.cpu_history, opacity);
                    let threads = format!("{} threads", sample.logical_processors);
                    text(canvas, &LABEL, &threads, sub_area(rect), GRAY, opacity);
                }
                Tile::Memory => {
                    text(canvas, &LABEL, "Memory", label_area(rect), GRAY, opacity);
                    text(canvas, &VALUE, &percent_text(sample.memory_percent), value_area(rect), Color::WHITE, opacity);
                    draw_bar(canvas, rect, scale, sample.memory_percent, MEMORY_PURPLE, opacity);
                    let memory = memory_pair_text(sample.memory_used_bytes, sample.memory_total_bytes);
                    text(canvas, &LABEL, &memory, sub_area(rect), GRAY, opacity);
                }
                Tile::Gpus => {
                    let style = ListStyle { title: "GPU", layout: RowLayout::Stacked, color: GPU_ORANGE };
                    draw_list(canvas, rect, scale, &gpus, &style, opacity);
                }
                Tile::Disks => {
                    let style = ListStyle {
                        title: "Disks",
                        layout: RowLayout::Inline { name_width: DISK_NAME_WIDTH },
                        color: DISK_GREEN,
                    };
                    draw_list(canvas, rect, scale, &disks, &style, opacity);
                }
                Tile::Network => {
                    text(canvas, &LABEL, "Network", label_area(rect), GRAY, opacity);
                    let rate = |value: Option<f64>| value.map_or_else(|| "\u{2013}".to_owned(), rate_text);
                    let download = format!("\u{2193} {}", rate(sample.download_bytes_per_second));
                    let upload = format!("\u{2191} {}", rate(sample.upload_bytes_per_second));
                    let download_area = inside(rect, scale, TILE_PADDING_X, 21.0, 37.0);
                    let upload_area = inside(rect, scale, TILE_PADDING_X, 37.0, 53.0);
                    text(canvas, &RATE, &download, download_area, Color::WHITE, opacity);
                    text(canvas, &RATE, &upload, upload_area, GRAY, opacity);
                    text(canvas, &LABEL, sample.network_kind.label(), sub_area(rect), GRAY, opacity);
                }
            }
        }
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if is_shown {
            self.sampler.reset();
            self.cpu_history.clear();
            self.latest = Some(self.sampler.sample());
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
        self.record(sample);
        context.set_timer(SAMPLE_TIMER, SAMPLE_INTERVAL_MILLISECONDS);
        context.request_repaint();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stats_page_is_opt_in_and_has_no_collapsed_state() {
        let page = StatsPage::new();
        assert!(page.is_opt_in());
        assert_eq!(page.priority(Mode::Resting), None);
        assert!(page.priority(Mode::Expanded).is_some());
    }

    #[test]
    fn tiles_share_the_width_by_weight_with_gaps_between() {
        let weights: Vec<f32> = tiles_for(true).iter().map(|tile| tile.weight()).collect();
        let tiles = tile_rects(Rect::new(0.0, 0.0, 608.0, 86.0), 1.0, &weights);
        assert_eq!(tiles.len(), 5);
        assert!((tiles[0].width() - 576.0 / 5.5).abs() < 0.01);
        assert!((tiles[2].width() - 576.0 / 5.5 * 1.25).abs() < 0.01);
        assert!((tiles[1].left - tiles[0].right - TILE_GAP).abs() < 0.01);
        assert!((tiles[4].right - 608.0).abs() < 0.01);
    }

    #[test]
    fn a_saved_choice_reads_as_a_list_of_names() {
        assert_eq!(choice_list("G:, C:,,"), ["G:", "C:"]);
        assert!(choice_list("").is_empty());
    }

    #[test]
    fn the_gpu_tile_appears_only_when_there_is_a_gpu() {
        assert_eq!(tiles_for(true), [Tile::Cpu, Tile::Memory, Tile::Gpus, Tile::Disks, Tile::Network]);
        assert_eq!(tiles_for(false), [Tile::Cpu, Tile::Memory, Tile::Disks, Tile::Network]);
    }

    #[test]
    fn the_history_keeps_only_the_most_recent_readings() {
        let mut page = StatsPage::new();
        let sample = Sampler::new().sample();
        for index in 0..HISTORY_LENGTH + 5 {
            page.record(Sample { cpu_percent: Some(index as f32), ..sample.clone() });
        }
        assert_eq!(page.cpu_history.len(), HISTORY_LENGTH);
        assert_eq!(page.cpu_history.front().copied(), Some(5.0));
    }

    #[test]
    fn a_reading_without_a_cpu_figure_adds_nothing_to_the_history() {
        let mut page = StatsPage::new();
        let sample = Sampler::new().sample();
        page.record(Sample { cpu_percent: None, ..sample });
        assert!(page.cpu_history.is_empty());
        assert!(page.latest.is_some());
    }
}
