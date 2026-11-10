//! Timer presentation rules, independent of drawing.

use super::session::{Phase, SESSIONS_PER_ROUND, State, Timer, compact_text, page_clock_text};
use crate::i18n::tr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Start,
    Shorter,
    Longer,
    Pause,
    Resume,
    Extend,
    Stop,
    StartNext,
    Skip,
}

pub struct ButtonSpec {
    pub action: Action,
    pub label: String,
    pub is_primary: bool,
    pub is_destructive: bool,
}

pub struct View {
    pub title: String,
    pub detail: Option<String>,
    pub time: Option<String>,
    pub buttons: Vec<ButtonSpec>,
}

fn button(action: Action, label: &str, is_primary: bool) -> ButtonSpec {
    ButtonSpec { action, label: label.to_owned(), is_primary, is_destructive: action == Action::Stop }
}

fn phase_name(phase: Phase) -> std::borrow::Cow<'static, str> {
    match phase {
        Phase::Focus => tr!("timer.focus"),
        Phase::ShortBreak => tr!("timer.short_break"),
        Phase::LongBreak => tr!("timer.long_break"),
    }
}

fn running_title(timer: &Timer, phase: Phase) -> String {
    match phase {
        Phase::Focus => tr!("timer.focus_session", n = timer.session_number(), total = SESSIONS_PER_ROUND).into_owned(),
        Phase::ShortBreak | Phase::LongBreak => phase_name(phase).into_owned(),
    }
}

fn start_button_label(next: Option<Phase>) -> std::borrow::Cow<'static, str> {
    match next {
        Some(Phase::ShortBreak) => tr!("timer.start_short_break"),
        Some(Phase::LongBreak) => tr!("timer.start_long_break"),
        Some(Phase::Focus) | None => tr!("timer.start_focus"),
    }
}

pub fn view(timer: &Timer, now: u64) -> View {
    match timer.state {
        State::Idle => View {
            title: tr!("timer.title").into_owned(),
            detail: Some(tr!("timer.session", n = timer.session_number(), total = SESSIONS_PER_ROUND).into_owned()),
            time: Some(page_clock_text(timer.lengths.focus * 60_000)),
            buttons: vec![
                button(Action::Shorter, "−5", false),
                button(Action::Longer, "+5", false),
                button(Action::Start, &tr!("timer.start"), true),
            ],
        },
        State::Running { phase, .. } => View {
            title: running_title(timer, phase),
            detail: None,
            time: timer.remaining(now).map(page_clock_text),
            buttons: vec![
                button(Action::Pause, &tr!("timer.pause"), false),
                button(Action::Extend, &tr!("timer.extend"), false),
                button(Action::Stop, &tr!("timer.stop"), false),
            ],
        },
        State::Paused { phase, .. } => View {
            title: tr!("timer.paused", title = running_title(timer, phase)).into_owned(),
            detail: None,
            time: timer.remaining(now).map(page_clock_text),
            buttons: vec![
                button(Action::Resume, &tr!("timer.resume"), true),
                button(Action::Extend, &tr!("timer.extend"), false),
                button(Action::Stop, &tr!("timer.stop"), false),
            ],
        },
        State::Finished { phase } => {
            let next = timer.next_phase();
            let detail = match (phase, next) {
                (Phase::Focus, Some(next)) => tr!(
                    "timer.next_up",
                    focus = timer.lengths.focus,
                    length = timer.lengths.minutes(next),
                    next = phase_name(next).to_lowercase()
                )
                .into_owned(),
                (_, Some(_)) => tr!("timer.ready", n = timer.session_number(), total = SESSIONS_PER_ROUND).into_owned(),
                _ => String::new(),
            };
            View {
                title: match phase {
                    Phase::Focus => tr!("timer.session_done").into_owned(),
                    Phase::ShortBreak | Phase::LongBreak => {
                        tr!("timer.break_over", phase = phase_name(phase)).into_owned()
                    }
                },
                detail: Some(detail),
                time: None,
                buttons: vec![
                    button(Action::StartNext, &start_button_label(next), true),
                    button(Action::Skip, &tr!("timer.skip"), false),
                ],
            }
        }
    }
}

pub struct Compact {
    pub text: String,
    pub fraction: f32,
    pub tone: Tone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Focus,
    Break,
    Paused,
}

pub fn compact(timer: &Timer, now: u64) -> Option<Compact> {
    let tone = match timer.state {
        State::Idle => return None,
        State::Paused { .. } => Tone::Paused,
        State::Running { phase: Phase::Focus, .. } => Tone::Focus,
        State::Running { .. } | State::Finished { .. } => Tone::Break,
    };
    let text = match timer.state {
        State::Finished { .. } => tr!("timer.done").into_owned(),
        _ => compact_text(timer.remaining(now)?),
    };
    Some(Compact { text, fraction: timer.elapsed_fraction(now), tone })
}

pub fn widest_form(text: &str) -> String {
    text.chars().map(|character| if character.is_ascii_digit() { '0' } else { character }).collect()
}

pub fn alert(timer: &Timer) -> Option<(String, String)> {
    let State::Finished { phase } = timer.state else { return None };
    Some(match phase {
        Phase::Focus => {
            (tr!("timer.session_done").into_owned(), tr!("timer.minutes", count = timer.lengths.focus).into_owned())
        }
        Phase::ShortBreak | Phase::LongBreak => {
            (tr!("timer.break_over", phase = phase_name(phase)).into_owned(), tr!("timer.back_to_focus").into_owned())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINUTE: u64 = 60_000;
    const NOW: u64 = 1_000_000_000_000;

    fn actions(view: &View) -> Vec<Action> {
        view.buttons.iter().map(|button| button.action).collect()
    }

    fn running() -> Timer {
        let mut timer = Timer::new();
        timer.start_focus(NOW);
        timer
    }

    #[test]
    fn idle_offers_a_length_and_a_start() {
        let view = view(&Timer::new(), NOW);
        assert_eq!(view.title, "Focus timer");
        assert_eq!(view.time.as_deref(), Some("25:00"));
        assert_eq!(actions(&view), vec![Action::Shorter, Action::Longer, Action::Start]);
        assert!(view.buttons[2].is_primary);
        assert!(!view.buttons[2].is_destructive);
    }

    #[test]
    fn a_running_session_shows_its_number_and_the_time_left() {
        let view = view(&running(), NOW + 60_000 + 13_000);
        assert_eq!(view.title, "Focus · 1 of 4");
        assert_eq!(view.time.as_deref(), Some("23:47"));
        assert_eq!(actions(&view), vec![Action::Pause, Action::Extend, Action::Stop]);
    }

    #[test]
    fn only_the_stop_button_is_red() {
        let view = view(&running(), NOW + 1_000);
        let red: Vec<Action> = view.buttons.iter().filter(|spec| spec.is_destructive).map(|spec| spec.action).collect();
        assert_eq!(red, vec![Action::Stop]);
    }

    #[test]
    fn a_paused_session_offers_resume_first() {
        let mut timer = running();
        timer.pause(NOW);
        let view = view(&timer, NOW + 5 * MINUTE);
        assert_eq!(view.title, "Focus · 1 of 4 · paused");
        assert_eq!(view.time.as_deref(), Some("25:00"));
        assert_eq!(actions(&view), vec![Action::Resume, Action::Extend, Action::Stop]);
    }

    #[test]
    fn a_finished_focus_session_offers_the_break() {
        let mut timer = running();
        timer.tick(NOW + 25 * MINUTE);
        let view = view(&timer, NOW + 26 * MINUTE);
        assert_eq!(view.title, "Focus session done");
        assert_eq!(view.detail.as_deref(), Some("25 min · 5 min short break next"));
        assert_eq!(view.buttons[0].label, "Start short break");
        assert_eq!(actions(&view), vec![Action::StartNext, Action::Skip]);
        assert!(view.time.is_none());
    }

    #[test]
    fn a_finished_break_offers_the_next_focus_session() {
        let mut timer = running();
        timer.tick(NOW + 25 * MINUTE);
        timer.start_next(NOW + 25 * MINUTE);
        timer.tick(NOW + 30 * MINUTE);
        let view = view(&timer, NOW + 31 * MINUTE);
        assert_eq!(view.title, "Short break over");
        assert_eq!(view.detail.as_deref(), Some("Ready for focus 2 of 4"));
        assert_eq!(view.buttons[0].label, "Start focus");
    }

    #[test]
    fn the_compact_view_counts_minutes_then_seconds() {
        let timer = running();
        assert_eq!(compact(&timer, NOW + MINUTE + 13_000).unwrap().text, "23m");
        assert_eq!(compact(&timer, NOW + 21 * MINUTE).unwrap().text, "4:00");
    }

    #[test]
    fn the_compact_view_is_toned_by_what_is_going_on() {
        let mut timer = running();
        assert_eq!(compact(&timer, NOW).unwrap().tone, Tone::Focus);
        timer.pause(NOW);
        assert_eq!(compact(&timer, NOW).unwrap().tone, Tone::Paused);
        assert!(compact(&Timer::new(), NOW).is_none());
    }

    #[test]
    fn a_finished_phase_shows_done_in_the_compact_view() {
        let mut timer = running();
        timer.tick(NOW + 25 * MINUTE);
        assert_eq!(compact(&timer, NOW + 26 * MINUTE).unwrap().text, "Done");
    }

    #[test]
    fn the_widest_form_keeps_the_shape_and_drops_the_digits() {
        assert_eq!(widest_form("24m"), "00m");
        assert_eq!(widest_form("4:12"), "0:00");
    }

    #[test]
    fn the_alert_says_what_ended() {
        let mut timer = running();
        assert!(alert(&timer).is_none());
        timer.tick(NOW + 25 * MINUTE);
        assert_eq!(alert(&timer), Some(("Focus session done".to_owned(), "25 min".to_owned())));
    }

    #[test]
    fn the_timer_phases_and_buttons_read_in_indonesian() {
        store::language::with_override(store::language::Language::Indonesian, || {
            let idle = view(&Timer::new(), NOW);
            assert_eq!(idle.title, "Timer fokus");
            assert_eq!(idle.detail.as_deref(), Some("Sesi 1 dari 4"));
            assert_eq!(idle.buttons[2].label, "Mulai");
            let running_view = view(&running(), NOW + 1_000);
            assert_eq!(running_view.title, "Fokus \u{b7} 1 dari 4");
            let labels: Vec<&str> = running_view.buttons.iter().map(|button| button.label.as_str()).collect();
            assert_eq!(labels, ["Jeda", "+5 mnt", "Berhenti"]);
            assert_eq!(start_button_label(Some(Phase::ShortBreak)), "Istirahat singkat");
            assert_eq!(phase_name(Phase::LongBreak), "Istirahat panjang");
        });
    }
}
