//! The CPU, memory and network cards.

use super::{
    CPU_ACCENT, GRAY, LABEL, MEMORY_ACCENT, RATE, UPLOAD_GRAY, VALUE,
    charts::{Accent, draw_load},
    label_area, network_kind_area, network_row_area,
    readout::Readout,
    sampler::NetworkKind,
    sub_area, text, value_area,
};
use crate::gfx::{Canvas, Color, Rect};

const BASELINE: f32 = 60.0;

/// A name, a big value, a dotted baseline with the load bars and a line under it.
pub(super) struct ChartCard<'a> {
    pub name: &'a str,
    pub value: &'a str,
    pub sub: &'a str,
    /// The current load from 0 to 1.
    pub load: f32,
    pub accent: Accent,
}

impl<'a> ChartCard<'a> {
    pub fn cpu(readout: &'a Readout) -> Self {
        Self {
            name: "CPU",
            value: &readout.cpu_text,
            sub: &readout.threads_text,
            load: readout.cpu_load,
            accent: CPU_ACCENT,
        }
    }

    pub fn memory(readout: &'a Readout) -> Self {
        Self {
            name: "Memory",
            value: &readout.memory_percent_text,
            sub: &readout.memory_pair_text,
            load: readout.memory_load,
            accent: MEMORY_ACCENT,
        }
    }

    pub fn draw(&self, canvas: &Canvas, card: Rect, scale: f32, opacity: f32) {
        text(canvas, &LABEL, self.name, label_area(card, scale), GRAY, opacity);
        text(canvas, &VALUE, self.value, value_area(card, scale), Color::WHITE, opacity);
        draw_load(canvas, card, scale, BASELINE, self.load, self.accent, opacity);
        text(canvas, &LABEL, self.sub, sub_area(card, scale), GRAY, opacity);
    }
}

/// Download in white, upload in gray, and the kind of network at the bottom.
pub(super) fn draw_network(
    canvas: &Canvas,
    card: Rect,
    scale: f32,
    readout: &Readout,
    kind: NetworkKind,
    opacity: f32,
) {
    text(canvas, &LABEL, "Network", label_area(card, scale), GRAY, opacity);
    text(canvas, &RATE, &readout.download_text, network_row_area(card, scale, 0), Color::WHITE, opacity);
    text(canvas, &RATE, &readout.upload_text, network_row_area(card, scale, 1), UPLOAD_GRAY, opacity);
    text(canvas, &LABEL, kind.label(), network_kind_area(card, scale), GRAY, opacity);
}
