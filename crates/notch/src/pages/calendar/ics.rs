//! Recurrences use wall time. Zone names without VTIMEZONE fall back to local time.
//! Zone offsets ignore DST; BYWEEKNO, BYYEARDAY and BYSETPOS are unsupported.

use fields::{Property, parse_duration, parse_moment, parse_offset, parse_property, unescape, unfold};
#[cfg(test)]
pub use model::FixedClock;
use model::WEEKDAY_CODES;
pub use model::{Clock, Event, Frequency, Occurrence, Rule, Zone};
pub use occurrences::occurrences_on;
pub use parse::parse;

mod fields;
mod model;
mod occurrences;
mod parse;
#[cfg(test)]
mod tests;
