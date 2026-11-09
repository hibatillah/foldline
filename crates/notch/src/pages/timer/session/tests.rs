use crate::host::Settings;

use super::{
    storage::{FOCUS_MINUTES_KEY, LONG_BREAK_MINUTES_KEY, SHORT_BREAK_MINUTES_KEY, format_state, parse_state},
    text::clock_text,
    *,
};

const MINUTE: u64 = MILLISECONDS_PER_MINUTE;
const NOW: u64 = 1_000_000_000_000;

fn running() -> Timer {
    let mut timer = Timer::new();
    timer.start_focus(NOW);
    timer
}

#[test]
fn a_focus_session_runs_for_its_length() {
    let timer = running();
    assert_eq!(timer.state, State::Running { phase: Phase::Focus, total: 25 * MINUTE, ends_at: NOW + 25 * MINUTE });
    assert_eq!(timer.remaining(NOW + 5 * MINUTE), Some(20 * MINUTE));
}

#[test]
fn starting_only_works_from_idle() {
    let mut timer = running();
    timer.start_focus(NOW + MINUTE);
    assert_eq!(timer.remaining(NOW + MINUTE), Some(24 * MINUTE));
}

#[test]
fn pausing_freezes_the_time_and_resuming_continues_from_it() {
    let mut timer = running();
    timer.pause(NOW + 10 * MINUTE);
    assert_eq!(timer.remaining(NOW + 99 * MINUTE), Some(15 * MINUTE));
    timer.resume(NOW + 30 * MINUTE);
    assert_eq!(timer.remaining(NOW + 30 * MINUTE), Some(15 * MINUTE));
    assert!(!timer.tick(NOW + 44 * MINUTE));
    assert!(timer.tick(NOW + 45 * MINUTE));
}

#[test]
fn extending_adds_five_minutes_to_a_running_or_paused_phase() {
    let mut timer = running();
    timer.extend();
    assert_eq!(timer.remaining(NOW), Some(30 * MINUTE));
    timer.pause(NOW);
    timer.extend();
    assert_eq!(timer.remaining(NOW), Some(35 * MINUTE));
}

#[test]
fn extending_keeps_the_ring_honest() {
    let mut timer = running();
    timer.extend();
    let fraction = timer.elapsed_fraction(NOW + 15 * MINUTE);
    assert!((fraction - 0.5).abs() < 0.001, "{fraction}");
}

#[test]
fn stopping_returns_to_idle_without_counting_the_session() {
    let mut timer = running();
    timer.stop();
    assert_eq!(timer.state, State::Idle);
    assert_eq!(timer.completed, 0);
}

#[test]
fn running_out_finishes_the_phase_and_counts_a_focus_session() {
    let mut timer = running();
    assert!(!timer.tick(NOW + 25 * MINUTE - 1));
    assert!(timer.tick(NOW + 25 * MINUTE));
    assert_eq!(timer.state, State::Finished { phase: Phase::Focus });
    assert_eq!(timer.completed, 1);
    assert!(!timer.tick(NOW + 26 * MINUTE));
}

#[test]
fn a_short_break_follows_a_focus_session_and_focus_follows_the_break() {
    let mut timer = running();
    timer.tick(NOW + 25 * MINUTE);
    assert_eq!(timer.next_phase(), Some(Phase::ShortBreak));
    timer.start_next(NOW + 26 * MINUTE);
    assert_eq!(timer.phase(), Some(Phase::ShortBreak));
    assert_eq!(timer.remaining(NOW + 26 * MINUTE), Some(5 * MINUTE));
    timer.tick(NOW + 31 * MINUTE);
    assert_eq!(timer.completed, 1);
    assert_eq!(timer.next_phase(), Some(Phase::Focus));
}

#[test]
fn the_fourth_focus_session_earns_the_long_break_and_then_a_new_round_starts() {
    let mut timer = Timer::new();
    let mut now = NOW;
    for session in 1..=SESSIONS_PER_ROUND {
        assert_eq!(timer.session_number(), session);
        timer.start_focus(now);
        now += 25 * MINUTE;
        timer.tick(now);
        if session < SESSIONS_PER_ROUND {
            timer.start_next(now);
            now += 5 * MINUTE;
            timer.tick(now);
            timer.start_next(now);
            timer.stop();
        }
    }
    assert_eq!(timer.next_phase(), Some(Phase::LongBreak));
    timer.start_next(now);
    assert_eq!(timer.remaining(now), Some(15 * MINUTE));
    now += 15 * MINUTE;
    timer.tick(now);
    timer.start_next(now);
    assert_eq!(timer.completed, 0);
    assert_eq!(timer.session_number(), 1);
}

#[test]
fn skipping_a_finished_phase_goes_back_to_idle() {
    let mut timer = running();
    timer.tick(NOW + 25 * MINUTE);
    timer.skip();
    assert_eq!(timer.state, State::Idle);
    assert_eq!(timer.completed, 1);
}

#[test]
fn the_length_changes_in_steps_inside_its_limits_and_only_while_idle() {
    let mut timer = Timer::new();
    timer.adjust_focus_length(1);
    assert_eq!(timer.lengths.focus, 30);
    timer.adjust_focus_length(-100);
    assert_eq!(timer.lengths.focus, MIN_FOCUS_MINUTES);
    timer.adjust_focus_length(1000);
    assert_eq!(timer.lengths.focus, MAX_FOCUS_MINUTES);
    timer.adjust_focus_length(-1);
    timer.start_focus(NOW);
    timer.adjust_focus_length(1);
    assert_eq!(timer.lengths.focus, MAX_FOCUS_MINUTES - 5);
}

#[test]
fn the_ring_fills_as_time_passes() {
    let timer = running();
    assert_eq!(timer.elapsed_fraction(NOW), 0.0);
    assert!((timer.elapsed_fraction(NOW + 5 * MINUTE) - 0.2).abs() < 0.001);
    assert_eq!(Timer::new().elapsed_fraction(NOW), 0.0);
}

#[test]
fn compact_text_shows_minutes_above_ten_and_seconds_below() {
    assert_eq!(compact_text(24 * MINUTE + 13_000), "24m");
    assert_eq!(compact_text(25 * MINUTE), "25m");
    assert_eq!(compact_text(10 * MINUTE + 1), "10m");
    assert_eq!(compact_text(10 * MINUTE), "10:00");
    assert_eq!(compact_text(4 * MINUTE + 12_000), "4:12");
}

#[test]
fn the_page_clock_pads_the_minutes_to_two_digits() {
    assert_eq!(page_clock_text(4 * MINUTE + 12_000), "04:12");
    assert_eq!(page_clock_text(25 * MINUTE), "25:00");
    assert_eq!(page_clock_text(1), "00:01");
}

#[test]
fn a_started_second_counts_as_a_whole_one() {
    assert_eq!(clock_text(1), "0:01");
    assert_eq!(clock_text(59_001), "1:00");
    assert_eq!(clock_text(0), "0:00");
}

#[test]
fn the_wake_comes_when_the_text_changes() {
    assert_eq!(next_wake(24 * MINUTE + 13_000, false), 13_000 + WAKE_SLACK_MILLISECONDS);
    assert_eq!(next_wake(25 * MINUTE, false), MINUTE + WAKE_SLACK_MILLISECONDS);
    assert_eq!(next_wake(4 * MINUTE + 12_300, false), 300 + WAKE_SLACK_MILLISECONDS);
    assert_eq!(next_wake(24 * MINUTE + 13_300, true), 300 + WAKE_SLACK_MILLISECONDS);
}

#[test]
fn the_last_wake_is_when_the_phase_ends() {
    assert_eq!(next_wake(400, false), 400 + WAKE_SLACK_MILLISECONDS);
    assert_eq!(next_wake(1000, false), 1000 + WAKE_SLACK_MILLISECONDS);
}

#[test]
fn states_survive_saving_and_loading() {
    let states = [
        State::Idle,
        State::Running { phase: Phase::Focus, total: 1_500_000, ends_at: NOW + 700_000 },
        State::Paused { phase: Phase::ShortBreak, total: 300_000, remaining: 120_000 },
        State::Finished { phase: Phase::LongBreak },
    ];
    for state in states {
        assert_eq!(parse_state(&format_state(state)), Some(state), "{state:?}");
    }
}

#[test]
fn unreadable_saved_states_are_ignored() {
    for text in ["", "running", "running focus x 1", "paused nap 1 2", "flying focus 1 2", "finished"] {
        assert_eq!(parse_state(text), None, "{text:?}");
    }
}

#[test]
fn a_saved_timer_comes_back_and_one_that_ran_out_meanwhile_comes_back_finished() {
    let path = std::env::temp_dir().join(format!("notch-test-{}-timer", std::process::id())).join("settings.txt");
    let mut settings = Settings::at(Some(path.clone()));
    let mut timer = running();
    timer.completed = 2;
    timer.save(&mut settings);

    let restored = Timer::load(&Settings::at(Some(path.clone())), NOW + 10 * MINUTE);
    assert_eq!(restored.state, timer.state);
    assert_eq!(restored.completed, 2);

    let late = Timer::load(&Settings::at(Some(path.clone())), NOW + 40 * MINUTE);
    assert_eq!(late.state, State::Finished { phase: Phase::Focus });
    assert_eq!(late.completed, 3);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn the_focus_length_is_saved_and_other_lengths_can_be_set_by_hand() {
    let path = std::env::temp_dir().join(format!("notch-test-{}-lengths", std::process::id())).join("settings.txt");
    let mut settings = Settings::at(Some(path.clone()));
    settings.set_string(SHORT_BREAK_MINUTES_KEY, "7");
    let mut timer = Timer::new();
    timer.adjust_focus_length(1);
    timer.save(&mut settings);
    let restored = Timer::load(&Settings::at(Some(path.clone())), NOW);
    assert_eq!(restored.lengths, Lengths { focus: 30, short_break: 7, long_break: 15 });
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn reloaded_lengths_take_new_values_keep_unreadable_ones_and_leave_a_running_phase_alone() {
    let path = std::env::temp_dir().join(format!("notch-test-{}-reload-lengths", std::process::id()));
    let path = path.join("settings.txt");
    let mut settings = Settings::at(Some(path.clone()));
    let mut timer = running();
    timer.lengths = Lengths { focus: 30, short_break: 7, long_break: 20 };
    let running_state = timer.state;
    settings.set_string(FOCUS_MINUTES_KEY, "45");
    settings.set_string(SHORT_BREAK_MINUTES_KEY, "soon");
    settings.set_string(LONG_BREAK_MINUTES_KEY, "0");
    timer.reload_lengths(&settings);
    assert_eq!(timer.lengths, Lengths { focus: 45, short_break: 7, long_break: 20 });
    assert_eq!(timer.state, running_state);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
