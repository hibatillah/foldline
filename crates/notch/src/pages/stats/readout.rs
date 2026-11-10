//! The words and rows of one reading, written when the reading or the saved choice changes so a paint only borrows.

use super::{
    MAX_DISKS, MAX_GPUS, Tile,
    format::{memory_pair_text, percent_text, rate_text},
    sampler::{Meter, Sample, chosen},
    tiles_for,
};
use crate::i18n::tr;

const MISSING: &str = "\u{2013}";

/// A disk or GPU with its percentage already written.
pub struct Row {
    pub meter: Meter,
    pub percent_text: String,
}

impl Row {
    fn new(meter: Meter) -> Self {
        let percent_text = meter.percent.map_or_else(|| MISSING.to_owned(), percent_text);
        Self { meter, percent_text }
    }

    /// The load from 0 to 1; a GPU still waiting for a reading counts as idle.
    pub fn load(&self) -> f32 {
        self.meter.percent.map_or(0.0, |percent| (percent / 100.0).clamp(0.0, 1.0))
    }
}

pub struct Readout {
    pub cpu_text: String,
    pub threads_text: String,
    pub memory_pair_text: String,
    pub memory_percent_text: String,
    pub download_text: String,
    pub upload_text: String,
    pub gpus: Vec<Row>,
    pub disks: Vec<Row>,
    pub tiles: Vec<Tile>,
    /// The current CPU and memory load from 0 to 1.
    pub cpu_load: f32,
    pub memory_load: f32,
}

impl Readout {
    pub fn new(sample: &Sample, disk_choice: Option<&[String]>, gpu_choice: Option<&[String]>) -> Self {
        let rate = |value: Option<f64>| value.map_or_else(|| MISSING.to_owned(), rate_text);
        let gpus: Vec<Row> = chosen(&sample.gpus, gpu_choice, MAX_GPUS).into_iter().map(Row::new).collect();
        let disks: Vec<Row> = chosen(&sample.disks, disk_choice, MAX_DISKS).into_iter().map(Row::new).collect();
        let tiles = tiles_for(!gpus.is_empty());
        Self {
            cpu_text: sample.cpu_percent.map_or_else(|| MISSING.to_owned(), percent_text),
            threads_text: tr!("stats.threads", count = sample.logical_processors).into_owned(),
            memory_pair_text: memory_pair_text(sample.memory_used_bytes, sample.memory_total_bytes),
            memory_percent_text: percent_text(sample.memory_percent),
            download_text: format!("\u{2193} {}", rate(sample.download_bytes_per_second)),
            upload_text: format!("\u{2191} {}", rate(sample.upload_bytes_per_second)),
            gpus,
            disks,
            tiles,
            cpu_load: sample.cpu_percent.map_or(0.0, |percent| (percent / 100.0).clamp(0.0, 1.0)),
            memory_load: (sample.memory_percent / 100.0).clamp(0.0, 1.0),
        }
    }
}
