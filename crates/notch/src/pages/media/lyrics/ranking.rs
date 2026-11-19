//! Chooses among the differently timed lyric entries a database holds for one song.

use std::cmp::Reverse;

use super::Lyrics;

const DURATION_TOLERANCE_SECONDS: f64 = 4.0;
/// Entries whose length differs from the closest one by no more than this count as equally close.
const DURATION_TIE_SECONDS: f64 = 1.0;
/// Lyrics may start their last line this long after the track's end before they count as another recording's.
const OVERRUN_ALLOWANCE_SECONDS: f64 = 2.0;
const TICKS_PER_SECOND: i64 = 10_000_000;

pub(super) struct Candidate {
    pub id: i64,
    pub track_name: String,
    pub album_name: String,
    pub duration_seconds: f64,
    pub lyrics: Lyrics,
}

/// What the player reports; a zero `duration_seconds` means the length is unknown.
#[derive(Clone)]
pub(super) struct PlayerTrack {
    pub title: String,
    pub album: String,
    pub duration_seconds: i64,
}

/// Length in whole seconds, rounded to the nearest second because truncating makes 270.6 s match a 270 s entry worse.
pub fn seconds_from_ticks(ticks: i64) -> i64 {
    (ticks + TICKS_PER_SECOND / 2).div_euclid(TICKS_PER_SECOND)
}

/// True when the last sung line starts after the track ends, so the lyrics are timed for a longer recording.
pub(super) fn runs_past_track(lyrics: &Lyrics, duration_seconds: i64) -> bool {
    if duration_seconds <= 0 {
        return false;
    }
    let limit_ticks = (duration_seconds as f64 + OVERRUN_ALLOWANCE_SECONDS) * TICKS_PER_SECOND as f64;
    lyrics
        .lines
        .iter()
        .rev()
        .find(|line| !line.text.is_empty())
        .is_some_and(|line| line.start_ticks as f64 > limit_ticks)
}

/// Among the entries close enough in length, the closest wins; entries within a second of it are ordered by
/// title match, then album match, then their place in the list.
pub(super) fn pick_best(candidates: Vec<Candidate>, track: &PlayerTrack) -> Option<Candidate> {
    let length_known = track.duration_seconds > 0;
    let difference_of = |candidate: &Candidate| {
        if length_known { (candidate.duration_seconds - track.duration_seconds as f64).abs() } else { 0.0 }
    };
    let usable: Vec<(usize, f64, Candidate)> = candidates
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| (index, difference_of(&candidate), candidate))
        .filter(|(_, difference, candidate)| {
            *difference <= DURATION_TOLERANCE_SECONDS && !runs_past_track(&candidate.lyrics, track.duration_seconds)
        })
        .collect();
    let closest = usable.iter().map(|(_, difference, _)| *difference).min_by(f64::total_cmp)?;
    let original_title = normalize(&track.title);
    let original_album = normalize(&track.album);
    usable
        .into_iter()
        .filter(|(_, difference, _)| *difference <= closest + DURATION_TIE_SECONDS)
        .min_by_key(|(index, _, candidate)| {
            (
                Reverse(title_score(&original_title, &normalize(&candidate.track_name))),
                Reverse(!original_album.is_empty() && original_album == normalize(&candidate.album_name)),
                *index,
            )
        })
        .map(|(_, _, candidate)| candidate)
}

/// 2 for the same title, 1 when the entry's title contains the player's (it adds words), 0 otherwise.
fn title_score(original: &str, candidate: &str) -> u8 {
    if original.is_empty() {
        0
    } else if original == candidate {
        2
    } else {
        u8::from(candidate.contains(original))
    }
}

fn normalize(text: &str) -> String {
    text.trim().replace(['\u{2019}', '\u{2018}'], "'").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pages::media::lyrics::LyricLine;

    fn lyrics_ending_at(last_line_seconds: i64) -> Lyrics {
        let line =
            |seconds: i64, text: &str| LyricLine { start_ticks: seconds * TICKS_PER_SECOND, text: text.to_owned() };
        Lyrics { lines: vec![line(10, "first"), line(last_line_seconds, "last")] }
    }

    fn entry(id: i64, track_name: &str, album_name: &str, duration_seconds: f64) -> Candidate {
        Candidate {
            id,
            track_name: track_name.to_owned(),
            album_name: album_name.to_owned(),
            duration_seconds,
            lyrics: lyrics_ending_at(200),
        }
    }

    fn player(title: &str, album: &str, duration_seconds: i64) -> PlayerTrack {
        PlayerTrack { title: title.to_owned(), album: album.to_owned(), duration_seconds }
    }

    fn picked_id(candidates: Vec<Candidate>, track: &PlayerTrack) -> Option<i64> {
        pick_best(candidates, track).map(|candidate| candidate.id)
    }

    #[test]
    fn the_feat_entry_wins_a_270_second_tie_over_the_plain_entry() {
        let candidates = vec![
            entry(1, "It's You", "It's You", 270.0),
            entry(2, "It's You (feat. Kaleb J)", "It's You", 270.0),
            entry(3, "It's You", "It's You", 251.0),
        ];
        let track = player("It\u{2019}s You (feat. Kaleb J)", "It's You", 270);
        assert_eq!(picked_id(candidates, &track), Some(2));
    }

    #[test]
    fn a_tie_on_title_falls_back_to_the_album_match() {
        let candidates = vec![entry(1, "Song", "Greatest Hits", 200.0), entry(2, "Song", "Original Album", 200.5)];
        assert_eq!(picked_id(candidates, &player("Song", "original album", 200)), Some(2));
    }

    #[test]
    fn list_order_is_the_last_tiebreak() {
        let candidates = vec![entry(1, "Song", "A", 200.0), entry(2, "Song", "B", 200.0)];
        assert_eq!(picked_id(candidates, &player("Song", "C", 200)), Some(1));
    }

    #[test]
    fn a_clearly_closer_length_beats_a_better_title() {
        let candidates = vec![entry(1, "Song (feat. X)", "", 203.5), entry(2, "Song", "", 200.0)];
        assert_eq!(picked_id(candidates, &player("Song (feat. X)", "", 200)), Some(2));
    }

    #[test]
    fn an_entry_outside_the_length_tolerance_is_never_picked() {
        let candidates = vec![entry(1, "Song", "", 251.0)];
        assert_eq!(picked_id(candidates, &player("Song", "", 270)), None);
    }

    #[test]
    fn an_unknown_track_length_ranks_by_title_then_list_order() {
        let candidates = vec![entry(1, "Song", "", 100.0), entry(2, "Song (live)", "", 300.0)];
        assert_eq!(picked_id(candidates, &player("Song (live)", "", 0)), Some(2));
    }

    #[test]
    fn the_track_length_is_rounded_to_the_nearest_second() {
        assert_eq!(seconds_from_ticks(2_696_000_000), 270);
        assert_eq!(seconds_from_ticks(2_694_000_000), 269);
        assert_eq!(seconds_from_ticks(2_695_000_000), 270);
        assert_eq!(seconds_from_ticks(0), 0);
    }

    #[test]
    fn lyrics_that_run_past_the_track_are_rejected() {
        assert!(runs_past_track(&lyrics_ending_at(273), 270));
        assert!(!runs_past_track(&lyrics_ending_at(272), 270));
        assert!(!runs_past_track(&lyrics_ending_at(273), 0));
    }

    #[test]
    fn an_early_last_line_is_an_outro_and_not_a_reason_to_reject() {
        assert!(!runs_past_track(&lyrics_ending_at(180), 270));
    }

    #[test]
    fn a_trailing_empty_line_does_not_count_as_the_last_sung_line() {
        let mut lyrics = lyrics_ending_at(200);
        lyrics.lines.push(LyricLine { start_ticks: 290 * TICKS_PER_SECOND, text: String::new() });
        assert!(!runs_past_track(&lyrics, 270));
    }

    #[test]
    fn a_candidate_that_runs_past_the_track_loses_to_one_that_fits() {
        let mut too_long = entry(1, "Song", "", 270.0);
        too_long.lyrics = lyrics_ending_at(280);
        let candidates = vec![too_long, entry(2, "Song", "", 270.0)];
        assert_eq!(picked_id(candidates, &player("Song", "", 270)), Some(2));
    }
}
