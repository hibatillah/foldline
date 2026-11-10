//! The GPU card (one GPU with a large value, or two rows with a chart each) and the disk card (one row per disk).

use super::{
    GPU_ACCENT, LIST_PERCENT, MAX_GPUS, TILE_PADDING_X, VALUE,
    charts::{disk_bar_track, draw_disk_bar, draw_load},
    inside, label_area,
    readout::Row,
    text, value_area,
};
use crate::gfx::{Canvas, Color, Rect};
use crate::i18n::tr;
use crate::ui::palette::TEXT_MUTED;
use crate::ui::type_styles::REGULAR_11;

const LONE_GPU_BASELINE: f32 = 66.0;
/// Top of each GPU row's name line and the baseline of its chart, for two GPUs.
const GPU_ROW_TOPS: [f32; MAX_GPUS] = [22.0, 53.0];
const GPU_ROW_BASELINES: [f32; MAX_GPUS] = [47.0, 79.0];
const LINE_HEIGHT: f32 = 14.0;
/// The room kept for a percentage at the right of a name.
const PERCENT_WIDTH: f32 = 34.0;
/// A disk name has the room between the card padding and the bar.
const DISK_NAME_WIDTH: f32 = 20.0;

/// The top of each disk row's text for `count` disks.
pub(super) fn disk_row_tops(count: usize) -> &'static [f32] {
    match count {
        0 => &[],
        1 => &[32.0],
        2 => &[27.0, 46.0],
        _ => &[24.0, 40.0, 56.0],
    }
}

fn name_and_percent(canvas: &Canvas, card: Rect, top: f32, row: &Row, name_width: f32, opacity: f32) {
    let line = inside(card, TILE_PADDING_X, top, top + LINE_HEIGHT);
    let name_area = Rect { right: line.left + name_width, ..line };
    text(canvas, &REGULAR_11, &row.meter.name, name_area, TEXT_MUTED, opacity);
    text(canvas, &LIST_PERCENT, &row.percent_text, line, Color::WHITE, opacity);
}

/// One GPU puts its name in the label and shows a large value; two GPUs are two rows, each with its own chart.
pub(super) fn draw_gpus(canvas: &Canvas, card: Rect, gpus: &[Row], opacity: f32) {
    let card_inner = (card.width() - 2.0 * TILE_PADDING_X).max(0.0);
    match gpus {
        [] => {}
        [gpu] => {
            let label = format!("GPU \u{b7} {}", gpu.meter.name);
            text(canvas, &REGULAR_11, &label, label_area(card), TEXT_MUTED, opacity);
            text(canvas, &VALUE, &gpu.percent_text, value_area(card), Color::WHITE, opacity);
            draw_load(canvas, card, LONE_GPU_BASELINE, gpu.load(), GPU_ACCENT, opacity);
        }
        _ => {
            text(canvas, &REGULAR_11, "GPU", label_area(card), TEXT_MUTED, opacity);
            for (index, gpu) in gpus.iter().take(MAX_GPUS).enumerate() {
                let name_width = card_inner - PERCENT_WIDTH;
                name_and_percent(canvas, card, GPU_ROW_TOPS[index], gpu, name_width, opacity);
                draw_load(canvas, card, GPU_ROW_BASELINES[index], gpu.load(), GPU_ACCENT, opacity);
            }
        }
    }
}

/// Each disk is one line: its name, a 56 x 4 bar and a percentage at the right.
pub(super) fn draw_disks(canvas: &Canvas, card: Rect, disks: &[Row], opacity: f32) {
    text(canvas, &REGULAR_11, &tr!("stats.disks"), label_area(card), TEXT_MUTED, opacity);
    for (disk, &top) in disks.iter().zip(disk_row_tops(disks.len())) {
        name_and_percent(canvas, card, top, disk, DISK_NAME_WIDTH, opacity);
        draw_disk_bar(canvas, disk_bar_track(card, top), disk.meter.percent.unwrap_or(0.0), opacity);
    }
}
