//! Opt-in system stats; sample and redraw once a second only while visible.

mod charts;
mod format;
mod gpu;
mod sampler;

use std::collections::{HashMap, VecDeque};

use self::{
    charts::{bar_track, draw_area, draw_bar},
    format::{memory_pair_text, percent_text, rate_text},
    sampler::{Meter, Sample, Sampler, chosen},
};
use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Rect, Size, UI_FONT, Weight},
    page::{Context, Control, Frame, Mode, Page, PageSizes, Priority, TimerId},
    ui::Icon,
};

const EXPANDED_HEIGHT: f32 = 86.0;
/// One card's width; the page is as wide as its cards, so a missing GPU card leaves the others as they are.
const UNIT_WIDTH: f32 = 104.0;
const RESTING_WIDTH: f32 = 150.0;
const MAX_DISKS: usize = 3;
const MAX_GPUS: usize = 2;
const DISKS_SETTING: &str = "stats.disks";
const GPUS_SETTING: &str = "stats.gpus";
/// The GPU, network and disk cards are wider than a single-value card so their names, rates and bars fit.
const WIDE_TILE_WEIGHT: f32 = 1.25;
const TILE_GAP: f32 = 8.0;
const TILE_RADIUS: f32 = 12.0;
const TILE_PADDING_X: f32 = 12.0;
const TILE_PADDING_Y: f32 = 7.0;
const SUB_TOP: f32 = 51.0;
const BAR_CENTER: f32 = 55.0;
const BAR_HEIGHT: f32 = 5.0;
const HISTORY_LENGTH: usize = 30;
/// The smallest rate the network chart is scaled to, so a quiet link does not look busy.
const NETWORK_CHART_FLOOR: f64 = 100_000.0;
const GPU_GAP: f32 = 6.0;
const DISK_ROWS_TOP: f32 = 9.0;
const DISK_ROW_HEIGHT: f32 = 24.0;
const DISK_BAR_HEIGHT: f32 = 4.0;
const DISK_BAR_OFFSET: f32 = 16.0;
const LIST_PERCENT_WIDTH: f32 = 28.0;
const LIST_GAP: f32 = 4.0;

const SAMPLE_TIMER: TimerId = TimerId(0);
const FIRST_SAMPLE_MILLISECONDS: u32 = 600;
const SAMPLE_INTERVAL_MILLISECONDS: u32 = 1000;

const MONO: &str = "Consolas";
const LABEL: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Leading);
const LABEL_END: FontStyle = FontStyle::new(UI_FONT, 11.0, Weight::Regular, Align::Trailing);
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
const NETWORK_TEAL: Color = Color::rgb(0.2, 0.78, 0.82);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Cpu,
    Memory,
    Gpus,
    Network,
    Disks,
}

impl Tile {
    fn weight(self) -> f32 {
        match self {
            Tile::Cpu | Tile::Memory => 1.0,
            Tile::Gpus | Tile::Network | Tile::Disks => WIDE_TILE_WEIGHT,
        }
    }
}

/// The four charts sit together and the disks stand alone at the right; the GPU card appears only when Windows
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
    let units: f32 = tiles.iter().map(|tile| tile.weight()).sum();
    units * UNIT_WIDTH + TILE_GAP * tiles.len().saturating_sub(1) as f32
}

pub struct StatsPage {
    sampler: Sampler,
    latest: Option<Sample>,
    cpu_history: VecDeque<f32>,
    memory_history: VecDeque<f32>,
    /// Download plus upload in bytes per second.
    network_history: VecDeque<f64>,
    gpu_history: HashMap<String, VecDeque<f32>>,
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
            memory_history: VecDeque::with_capacity(HISTORY_LENGTH),
            network_history: VecDeque::with_capacity(HISTORY_LENGTH),
            gpu_history: HashMap::new(),
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
                    Meter { name: "RTX 4050".to_owned(), capacity: None, percent: Some(23.0) },
                    Meter { name: "Radeon 760M".to_owned(), capacity: None, percent: Some(6.0) },
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
            push_reading(&mut self.cpu_history, cpu / 100.0);
        }
        push_reading(&mut self.memory_history, sample.memory_percent / 100.0);
        let traffic = sample.download_bytes_per_second.unwrap_or(0.0) + sample.upload_bytes_per_second.unwrap_or(0.0);
        if self.network_history.len() == HISTORY_LENGTH {
            self.network_history.pop_front();
        }
        self.network_history.push_back(traffic);
        for gpu in &sample.gpus {
            if let Some(percent) = gpu.percent {
                push_reading(self.gpu_history.entry(gpu.name.clone()).or_default(), percent / 100.0);
            }
        }
        self.latest = Some(sample);
    }

    /// The GPUs the page shows, so the card count and the page width follow the saved choice.
    fn shown_gpus(&self) -> Vec<Meter> {
        self.latest
            .as_ref()
            .map(|sample| chosen(&sample.gpus, self.gpu_choice.as_deref(), MAX_GPUS))
            .unwrap_or_default()
    }

    fn tiles(&self) -> Vec<Tile> {
        tiles_for(!self.shown_gpus().is_empty())
    }
}

/// Keeps the most recent readings; each is between 0 and 1.
fn push_reading(history: &mut VecDeque<f32>, reading: f32) {
    if history.len() == HISTORY_LENGTH {
        history.pop_front();
    }
    history.push_back(reading.clamp(0.0, 1.0));
}

/// The network history scaled to its busiest reading, but never below `NETWORK_CHART_FLOOR`.
fn network_chart(history: &VecDeque<f64>) -> VecDeque<f32> {
    let busiest = history.iter().copied().fold(NETWORK_CHART_FLOOR, f64::max);
    history.iter().map(|rate| (rate / busiest) as f32).collect()
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

fn text(canvas: &Canvas, style: &FontStyle, content: &str, area: Rect, color: Color, opacity: f32) {
    canvas.text(style, content, area, color.faded(opacity));
}

fn label_area(tile: Rect, scale: f32) -> Rect {
    inside(tile, scale, TILE_PADDING_X, TILE_PADDING_Y, TILE_PADDING_Y + 14.0)
}

fn value_area(tile: Rect, scale: f32) -> Rect {
    inside(tile, scale, TILE_PADDING_X, 19.0, 45.0)
}

fn sub_area(tile: Rect, scale: f32) -> Rect {
    inside(tile, scale, TILE_PADDING_X, SUB_TOP, SUB_TOP + 14.0)
}

/// A card with a name, a big value, a line under it and a chart along the bottom edge.
struct ChartCard<'a> {
    label: &'a str,
    value: &'a str,
    sub: &'a str,
    history: &'a VecDeque<f32>,
    color: Color,
}

fn draw_chart_card(canvas: &Canvas, tile: Rect, scale: f32, card: &ChartCard, opacity: f32) {
    text(canvas, &LABEL, card.label, label_area(tile, scale), GRAY, opacity);
    text(canvas, &VALUE, card.value, value_area(tile, scale), Color::WHITE, opacity);
    text(canvas, &LABEL, card.sub, sub_area(tile, scale), GRAY, opacity);
    draw_area(canvas, tile, scale, card.history, card.color, opacity);
}

/// One GPU gets a whole card; two GPUs each get their own card, stacked with a gap, as a name, a percentage and a chart.
fn draw_gpus(
    canvas: &Canvas,
    tile: Rect,
    scale: f32,
    gpus: &[Meter],
    histories: &HashMap<String, VecDeque<f32>>,
    opacity: f32,
) {
    let empty = VecDeque::new();
    let percent = |gpu: &Meter| gpu.percent.map_or_else(|| "\u{2013}".to_owned(), percent_text);
    if let [gpu] = gpus {
        canvas.fill_rounded_rect(tile, TILE_RADIUS * scale, TILE.faded(opacity));
        let label = format!("GPU \u{b7} {}", gpu.name);
        let card = ChartCard {
            label: &label,
            value: &percent(gpu),
            sub: "",
            history: histories.get(&gpu.name).unwrap_or(&empty),
            color: GPU_ORANGE,
        };
        draw_chart_card(canvas, tile, scale, &card, opacity);
        return;
    }
    let gap = GPU_GAP * scale;
    let height = (tile.height() - gap * gpus.len().saturating_sub(1) as f32) / gpus.len().max(1) as f32;
    for (row, gpu) in gpus.iter().enumerate() {
        let top = tile.top + row as f32 * (height + gap);
        let card = Rect::new(tile.left, top, tile.right, top + height);
        canvas.fill_rounded_rect(card, TILE_RADIUS * scale, TILE.faded(opacity));
        let line = inside(card, scale, TILE_PADDING_X, 6.0, 20.0);
        let percent_left = line.right - LIST_PERCENT_WIDTH * scale;
        text(
            canvas,
            &LIST_NAME,
            &gpu.name,
            Rect { right: percent_left - LIST_GAP * scale, ..line },
            NAME_GRAY,
            opacity,
        );
        text(canvas, &LIST_PERCENT, &percent(gpu), Rect { left: percent_left, ..line }, Color::WHITE, opacity);
        draw_area(canvas, card, scale, histories.get(&gpu.name).unwrap_or(&empty), GPU_ORANGE, opacity);
    }
}

/// One disk is a card like the others with a bar; several disks are rows of one card, each a name, a percentage and a
/// bar. A bar is a value and not a chart, so it keeps the card's padding.
fn draw_disks(canvas: &Canvas, tile: Rect, scale: f32, disks: &[Meter], opacity: f32) {
    if let [disk] = disks {
        let percent = disk.percent.unwrap_or(0.0);
        text(canvas, &LABEL, &format!("Disk {}", disk.name), label_area(tile, scale), GRAY, opacity);
        text(canvas, &VALUE, &percent_text(percent), value_area(tile, scale), Color::WHITE, opacity);
        let center_y = tile.top + BAR_CENTER * scale - 3.0 * scale;
        draw_bar(canvas, bar_track(tile, scale, center_y, BAR_HEIGHT), percent, DISK_GREEN, opacity);
        if let Some((used, total)) = disk.capacity {
            text(canvas, &LABEL, &memory_pair_text(used, total), sub_area(tile, scale), GRAY, opacity);
        }
        return;
    }
    for (row, disk) in disks.iter().enumerate() {
        let top = DISK_ROWS_TOP + row as f32 * DISK_ROW_HEIGHT;
        let line = inside(tile, scale, TILE_PADDING_X, top, top + 14.0);
        let percent_left = line.right - LIST_PERCENT_WIDTH * scale;
        let shown = disk.percent.map_or_else(|| "\u{2013}".to_owned(), percent_text);
        text(
            canvas,
            &LIST_NAME,
            &disk.name,
            Rect { right: percent_left - LIST_GAP * scale, ..line },
            NAME_GRAY,
            opacity,
        );
        text(canvas, &LIST_PERCENT, &shown, Rect { left: percent_left, ..line }, Color::WHITE, opacity);
        let center_y = tile.top + (top + DISK_BAR_OFFSET) * scale;
        let track = bar_track(tile, scale, center_y, DISK_BAR_HEIGHT);
        draw_bar(canvas, track, disk.percent.unwrap_or(0.0), DISK_GREEN, opacity);
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
        PageSizes { resting_width: RESTING_WIDTH, expanded: Size::new(expanded_width(&self.tiles()), EXPANDED_HEIGHT) }
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
        let gpus = self.shown_gpus();
        let disks = chosen(&sample.disks, self.disk_choice.as_deref(), MAX_DISKS);
        let tiles = tiles_for(!gpus.is_empty());
        let weights: Vec<f32> = tiles.iter().map(|tile| tile.weight()).collect();
        for (kind, rect) in tiles.iter().zip(tile_rects(frame.bounds, scale, &weights)) {
            if *kind != Tile::Gpus {
                canvas.fill_rounded_rect(rect, TILE_RADIUS * scale, TILE.faded(opacity));
            }
            match kind {
                Tile::Cpu => {
                    let cpu = sample.cpu_percent.map_or_else(|| "\u{2013}".to_owned(), percent_text);
                    let threads = format!("{} threads", sample.logical_processors);
                    let card = ChartCard {
                        label: "CPU",
                        value: &cpu,
                        sub: &threads,
                        history: &self.cpu_history,
                        color: CPU_BLUE,
                    };
                    draw_chart_card(canvas, rect, scale, &card, opacity);
                }
                Tile::Memory => {
                    let memory = memory_pair_text(sample.memory_used_bytes, sample.memory_total_bytes);
                    let value = percent_text(sample.memory_percent);
                    let card = ChartCard {
                        label: "Memory",
                        value: &value,
                        sub: &memory,
                        history: &self.memory_history,
                        color: MEMORY_PURPLE,
                    };
                    draw_chart_card(canvas, rect, scale, &card, opacity);
                }
                Tile::Gpus => draw_gpus(canvas, rect, scale, &gpus, &self.gpu_history, opacity),
                Tile::Network => {
                    text(canvas, &LABEL, "Network", label_area(rect, scale), GRAY, opacity);
                    text(canvas, &LABEL_END, sample.network_kind.label(), label_area(rect, scale), GRAY, opacity);
                    let rate = |value: Option<f64>| value.map_or_else(|| "\u{2013}".to_owned(), rate_text);
                    let download = format!("\u{2193} {}", rate(sample.download_bytes_per_second));
                    let upload = format!("\u{2191} {}", rate(sample.upload_bytes_per_second));
                    let download_area = inside(rect, scale, TILE_PADDING_X, 21.0, 37.0);
                    let upload_area = inside(rect, scale, TILE_PADDING_X, 37.0, 53.0);
                    text(canvas, &RATE, &download, download_area, Color::WHITE, opacity);
                    text(canvas, &RATE, &upload, upload_area, GRAY, opacity);
                    draw_area(canvas, rect, scale, &network_chart(&self.network_history), NETWORK_TEAL, opacity);
                }
                Tile::Disks => draw_disks(canvas, rect, scale, &disks, opacity),
            }
        }
    }

    fn on_shown_changed(&mut self, is_shown: bool, context: &mut Context) {
        if is_shown {
            self.sampler.reset();
            self.cpu_history.clear();
            self.memory_history.clear();
            self.network_history.clear();
            self.gpu_history.clear();
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
    fn the_page_samples_once_a_second() {
        assert_eq!(SAMPLE_INTERVAL_MILLISECONDS, 1000);
    }

    #[test]
    fn the_stats_page_is_opt_in_and_has_no_collapsed_state() {
        let page = StatsPage::new();
        assert!(page.is_opt_in());
        assert_eq!(page.priority(Mode::Resting), None);
        assert!(page.priority(Mode::Expanded).is_some());
    }

    #[test]
    fn tiles_share_the_width_by_weight_with_gaps_between() {
        let tiles = tiles_for(true);
        let weights: Vec<f32> = tiles.iter().map(|tile| tile.weight()).collect();
        let width = expanded_width(&tiles);
        let rects = tile_rects(Rect::new(0.0, 0.0, width, 86.0), 1.0, &weights);
        assert_eq!(rects.len(), 5);
        assert!((rects[0].width() - UNIT_WIDTH).abs() < 0.01);
        assert!((rects[2].width() - UNIT_WIDTH * WIDE_TILE_WEIGHT).abs() < 0.01);
        assert!((rects[1].left - rects[0].right - TILE_GAP).abs() < 0.01);
        assert!((rects[4].right - width).abs() < 0.01);
    }

    #[test]
    fn a_missing_gpu_leaves_every_other_card_its_width_and_the_page_narrower() {
        let with = expanded_width(&tiles_for(true));
        let without = expanded_width(&tiles_for(false));
        assert!((with - without - (UNIT_WIDTH * WIDE_TILE_WEIGHT + TILE_GAP)).abs() < 0.01);
    }

    #[test]
    fn a_saved_choice_reads_as_a_list_of_names() {
        assert_eq!(choice_list("G:, C:,,"), ["G:", "C:"]);
        assert!(choice_list("").is_empty());
    }

    #[test]
    fn the_gpu_tile_appears_only_when_there_is_a_gpu() {
        assert_eq!(tiles_for(true), [Tile::Cpu, Tile::Memory, Tile::Gpus, Tile::Network, Tile::Disks]);
        assert_eq!(tiles_for(false), [Tile::Cpu, Tile::Memory, Tile::Network, Tile::Disks]);
    }

    #[test]
    fn the_history_keeps_only_the_most_recent_readings() {
        let mut page = StatsPage::new();
        let sample = Sampler::new().sample();
        for index in 0..HISTORY_LENGTH + 5 {
            page.record(Sample { cpu_percent: Some(index as f32), ..sample.clone() });
        }
        assert_eq!(page.cpu_history.len(), HISTORY_LENGTH);
        assert_eq!(page.cpu_history.front().copied(), Some(0.05));
    }

    #[test]
    fn the_network_chart_is_scaled_to_its_busiest_reading_but_not_below_a_floor() {
        let quiet: VecDeque<f64> = [10_000.0, 20_000.0].into_iter().collect();
        assert_eq!(network_chart(&quiet), VecDeque::from([0.1, 0.2]));
        let busy: VecDeque<f64> = [1_000_000.0, 500_000.0].into_iter().collect();
        assert_eq!(network_chart(&busy), VecDeque::from([1.0, 0.5]));
    }

    #[test]
    fn memory_and_gpu_readings_are_kept_for_their_charts() {
        let mut page = StatsPage::sample();
        assert_eq!(page.memory_history.len(), HISTORY_LENGTH);
        assert!(page.gpu_history.contains_key("RTX 4050"));
        page.record(Sample { memory_percent: 120.0, ..Sampler::new().sample() });
        assert_eq!(page.memory_history.back().copied(), Some(1.0));
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
