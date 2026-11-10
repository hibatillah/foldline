---
paths:
  - "crates/notch/src/**/*.rs"
  - "scripts/**"
  - "crates/settings/src/**/*.rs"
  - "crates/store/src/**/*.rs"
---

# Testing

- Put tests in a `#[cfg(test)] mod tests` at the bottom of the file they test, or in `tests.rs` or `tests/` beside a module that has many; this binary crate has no crate-level `tests/`
  folder.
- Name a test as a sentence about behavior: `a_paused_track_keeps_its_line_but_arms_no_timer`.
- Keep logic in functions that take and return plain values so a test needs no window.
- Check drawing by rendering it: `cargo test snapshots -- --ignored` writes pictures to `target/snapshots` (see
  `docs/how-to/test-and-snapshot.md`); look at them. The Windows integration is checked by running the notch.
- Check a drawing change at `SCALE` 1.0, 1.25 and 1.5 (the const in `crates/notch/src/host/notch/tests/mod.rs`), with
  `.superpowers/spikes/canvas-units/compare_snapshots.py` (a local, git-ignored script) or by eye.
- Give a test that touches settings `Settings::at(Some(path))` with a temporary path, so the real
  `%APPDATA%\notch\settings.txt` stays untouched.
- Create `Fonts::new(1.0)` in a test that measures text; DirectWrite works without a window.
- Add or update unit tests for every piece of logic you add or change.
- The memory and CPU check is `crates/notch/examples/perf.rs`, started by `scripts/perf.sh`.
- Add a gate or flag to `crates/notch/examples/perf.rs` together with its line in the `scripts/perf.sh` header and its row in
  `docs/how-to/run-the-perf-check.md`.
- Name a regression test after the behavior that broke, as a sentence a user would recognize (`session_recovery_clears_stale_hover_and_allows_a_hidden_notch_to_open_again`).
- A test never asserts on a constant or on a value the compiler already knows (clippy `assertions_on_constants`); assert on what a function returns.
- A page's snapshot scenarios come from `SampleOptions` in `pages/mod.rs` (the devices and clipboard pages keep theirs in `sample.rs`); add a scenario there when a page gets a new visible state.
- The settings crate's tests cannot pass while a `notch-settings.exe` window is open (the `single_instance` test); close it before running them. Run them, and the store's, with the MSVC toolchain: `cargo +stable-x86_64-pc-windows-msvc test --workspace`.
- An incomplete `tree-sitter` build in a stale target directory: `cargo clean -p tree-sitter` there, then build again.
