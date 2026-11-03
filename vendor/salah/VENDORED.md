# Vendored salah 0.7.6

Copied from crates.io (MIT, see LICENSE). One change: `SolarTime::setting_hour` in `src/astronomy/solar.rs` adds whole
minutes to midnight instead of building a clock time by hand. The original produced hour 24 for a time in the last half
minute of a UTC day and panicked, which closes the notch (the release build aborts on panic).

Tests and benches of the original crate are not included.
