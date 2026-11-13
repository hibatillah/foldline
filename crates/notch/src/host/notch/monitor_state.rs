use crate::host::{
    monitor::{self, MonitorId, MonitorInfo, MonitorMode, Placement},
    screen_mode::{self, ScreenMode},
};

/// Where the monitor facts come from; tests replace it because the rig has no real monitors.
#[derive(Clone, Copy)]
pub(in crate::host) struct MonitorSource {
    pub list: fn() -> Vec<MonitorInfo>,
    pub foreground: fn() -> Option<MonitorId>,
    /// Judges the window in front against this monitor from now on and returns the mode it is in now.
    pub judge: fn(Option<MonitorId>) -> ScreenMode,
}

impl MonitorSource {
    pub const WINDOWS: Self =
        Self { list: monitor::list, foreground: monitor::foreground, judge: screen_mode::judge_against };
}

/// The `notch.monitor` and `notch.monitor_id` choice, the monitor the notch is on, and a move that waits for the notch to close.
pub(in crate::host) struct MonitorState {
    pub mode: MonitorMode,
    /// `notch.monitor_id`; never overwritten when that screen is not connected.
    pub chosen_id: Option<String>,
    pub current: Option<MonitorId>,
    /// The placement last applied, so an unchanged one is not applied again.
    pub placed: Option<Placement>,
    /// `notch.top`, measured from the top edge of the monitor.
    pub top_offset: i32,
    pub source: MonitorSource,
    pub(super) waiting: Option<(MonitorId, Placement)>,
}

impl MonitorState {
    pub fn new(
        mode: MonitorMode,
        chosen_id: Option<String>,
        current: Option<MonitorId>,
        placed: Option<Placement>,
        top_offset: i32,
    ) -> Self {
        Self { mode, chosen_id, current, placed, top_offset, source: MonitorSource::WINDOWS, waiting: None }
    }

    pub(super) fn is_listed(monitors: &[MonitorInfo], id: Option<MonitorId>) -> bool {
        id.is_some_and(|id| monitors.iter().any(|monitor| monitor.id == id))
    }
}
