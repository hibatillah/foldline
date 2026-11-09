---
paths:
  - "crates/notch/src/**/*.rs"
  - "crates/settings/src/**/*.rs"
  - "crates/store/src/**/*.rs"
---

# Module layout

`docs/reference/source-map.md` ("How a module is laid out") is the full convention; the rules below are the ones to
check before writing or moving code.

- A file holds one job and stays under about 400 lines (500 at most). Split by job into a child module before it grows;
  never add a job to a file that already has one.
- `mod.rs` (or `name.rs` beside `name/`) declares children, holds the main type and the trait impl, and keeps no logic a
  child could hold. `Notch` follows this: its methods live one job per file in `host/notch/`.
- State a job owns is a small struct defined beside that job's logic (`Springs` and `FrameTimer` in `animation.rs`, `ShownPresences` in `presences.rs`, `HoverState` in
  `hover.rs`) and held by the main type; a new job adds a field of its own struct, not loose fields.
- Pure logic (layout math, rules, wording, parsing) never imports `windows::`; only the host, `platform` and `ui` call
  Windows.
- A moved item becomes `pub(super)` before it becomes `pub`; re-export from the module root only what outside code uses.
- Move tests with the code they check; a module with many tests keeps them in `tests.rs` or `tests/`.
