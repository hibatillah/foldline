//! What the microphone and camera activities share: a message when the device turns on or off, then a coloured dot
//! for as long as it is in use.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::{
    activity::{Flash, Presence, Slot},
    gfx::{Canvas, Color, Point, Rect},
    page::{Context, TimerId},
    ui::{self, Icon},
};

const SHOW_MILLISECONDS: u32 = 2500;
const DOT_RADIUS: f32 = 3.0;
const DOT_SLOT_WIDTH: f32 = DOT_RADIUS * 2.0;
/// Distance of each row's center from the slot's center: two dots with a 3 DIP gap.
const ROW_OFFSET: f32 = DOT_RADIUS + 1.5;
/// A microphone alone is a 5 DIP dot in the 15 DIP column (2 DIP padding, the 6 DIP camera slot kept empty above it).
const LONE_DOT_RADIUS: f32 = 2.5;
const LONE_DOT_OFFSET_X: f32 = -0.5;
const LONE_DOT_OFFSET_Y: f32 = 3.0;

/// Set by the device on the top row, so the one below it knows whether it shares the column; both draw on the UI thread.
static TOP_ROW_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Which of the two stacked dots a device owns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DotRow {
    Top,
    Bottom,
}

impl DotRow {
    /// The dot's radius and its center's offset from the slot's center in DIP (negative is up and left); the bottom
    /// row is smaller and closer to the middle while the top row is empty.
    fn dot(self, is_top_row_active: bool) -> (f32, f32, f32) {
        match self {
            DotRow::Top => (DOT_RADIUS, 0.0, -ROW_OFFSET),
            DotRow::Bottom if is_top_row_active => (DOT_RADIUS, 0.0, ROW_OFFSET),
            DotRow::Bottom => (LONE_DOT_RADIUS, LONE_DOT_OFFSET_X, LONE_DOT_OFFSET_Y),
        }
    }
}

pub struct Device {
    pub name: &'static str,
    pub icon: Icon,
    pub color: Color,
    pub row: DotRow,
}

pub struct Privacy {
    device: Device,
    is_active: bool,
    flash: Flash,
    message: String,
    message_width: f32,
}

impl Privacy {
    pub fn new(device: Device) -> Self {
        Self { device, is_active: false, flash: Flash::default(), message: String::new(), message_width: 0.0 }
    }

    /// Takes the state found at start without announcing it; a device already in use just shows its dot.
    pub fn initialize(&mut self, is_active: bool) {
        self.is_active = is_active;
        self.publish();
    }

    pub fn update(&mut self, is_active: bool, context: &mut Context) {
        if is_active == self.is_active {
            return;
        }
        self.is_active = is_active;
        self.publish();
        self.message = message_for(self.device.name, is_active);
        self.message_width = ui::message_width(context.fonts(), &self.message, None);
        self.flash.show(context, SHOW_MILLISECONDS);
    }

    fn publish(&self) {
        if self.device.row == DotRow::Top {
            TOP_ROW_ACTIVE.store(self.is_active, Ordering::Relaxed);
        }
    }

    pub fn on_timer(&mut self, timer: TimerId, context: &mut Context) {
        self.flash.on_timer(timer, context);
    }

    pub fn presence(&self) -> Option<Presence> {
        if self.flash.is_visible() {
            Some(Presence::Transient { width: self.message_width })
        } else if self.is_active {
            Some(Presence::Companion { slot: Slot::Dots, width: DOT_SLOT_WIDTH })
        } else {
            None
        }
    }

    pub fn draw(&self, canvas: &Canvas, area: Rect, opacity: f32) {
        if self.flash.is_visible() {
            let tint = if self.is_active { self.device.color } else { Color::gray(0.7) };
            ui::draw_message(canvas, area, opacity, self.device.icon, tint, &self.message, None);
        } else {
            let scale = canvas.scale();
            let (radius, offset_x, offset_y) = self.device.row.dot(TOP_ROW_ACTIVE.load(Ordering::Relaxed));
            let center = Point::new(area.center_x() + offset_x * scale, area.center_y() + offset_y * scale);
            canvas.fill_circle(center, radius * scale, self.device.color.faded(opacity));
        }
    }
}

fn message_for(name: &str, is_active: bool) -> String {
    format!("{name} {}", if is_active { "on" } else { "off" })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn microphone() -> Privacy {
        Privacy::new(Device {
            name: "Microphone",
            icon: Icon::Microphone,
            color: Color::rgb(1.0, 0.62, 0.04),
            row: DotRow::Bottom,
        })
    }

    #[test]
    fn messages_name_the_device_and_its_state() {
        assert_eq!(message_for("Microphone", true), "Microphone on");
        assert_eq!(message_for("Camera", false), "Camera off");
    }

    #[test]
    fn a_device_in_use_at_start_shows_only_its_dot() {
        let mut privacy = microphone();
        privacy.initialize(true);
        assert_eq!(privacy.presence(), Some(Presence::Companion { slot: Slot::Dots, width: DOT_SLOT_WIDTH }));
    }

    #[test]
    fn a_lone_microphone_is_a_5_px_dot_in_a_15_px_column_with_2_px_padding() {
        let (radius, offset_x, offset_y) = DotRow::Bottom.dot(false);
        assert_eq!(radius * 2.0, 5.0);
        let column_top = -7.5;
        let camera_slot = 6.0;
        assert_eq!(
            offset_y - radius,
            column_top + 2.0 + camera_slot,
            "the dot sits right under the empty 6 px camera slot"
        );
        assert_eq!(offset_y + radius + 2.0, 7.5, "2 px of padding stay below it");
        assert_eq!(offset_x, -0.5, "left-aligned in the 6 px column");
    }

    #[test]
    fn the_camera_alone_and_both_devices_keep_6_px_dots_3_px_apart() {
        let (top_radius, _, top_y) = DotRow::Top.dot(false);
        let (bottom_radius, bottom_x, bottom_y) = DotRow::Bottom.dot(true);
        assert_eq!((top_radius, bottom_radius, bottom_x), (3.0, 3.0, 0.0));
        assert_eq!(bottom_y - top_y - 2.0 * top_radius, 3.0);
        assert_eq!(DotRow::Top.dot(true), DotRow::Top.dot(false));
    }

    #[test]
    fn a_device_not_in_use_shows_nothing() {
        assert_eq!(microphone().presence(), None);
    }
}
