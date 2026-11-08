# Performance

Budget for `notch.exe` (not the settings window; see `AGENTS.md`): 0 % CPU while idle, under 50 MB of memory. The limits
`scripts/perf.sh` enforces are in `crates/notch/examples/perf.rs` (scenarios in `examples/perf/`); never raise one.

- Stop every timer and animation as soon as its output stops changing; the frame timer ends when the springs settle
  and no page reports `animate` as moving. A page's timer runs only while its work does (the stats sampler and the
  devices refresh run while the page is shown; the calendar's wake timer is armed for the next reminder window).
- Wake the UI thread from background work with a `Notifier`. Wait on what calls you: a Windows callback, a message,
  a blocking wait, a watcher or an event hook. An event hook asks for the one event and the one thread it needs
  (`host/screen_mode.rs`, `host/volume_flyout/hook_thread.rs`), because a hook on every window's events wakes the notch
  on every mouse move. `host/screen_mode/location_hook.rs` pauses itself when the pointer or a caret floods it.
- A read that must not run at start-up asks for the least it needs: the notification listener reads only ids at start,
  never the texts and logos of notifications already waiting (reading them all grew the handle count). Keep the
  handle growth limit (20) as it is.
- Draw shapes into small cached images: describe them with `Path` (or `gfx::svg_path` for SVG path data), rasterize
  with `Mask`, reuse through `Canvas::cached_image`. Direct2D geometry (`ID2D1PathGeometry`, `FillGeometry`,
  `DrawGeometry`) costs about 10 MB the first time it is used. Build text that changes rarely once, not on every paint.
- When `scripts/perf.sh` fails, find the cause and leave the limits where they are. An idle CPU reading just above the
  limit can come from something else on the machine (speculation, not a measured cause): run it again with the pointer
  parked and no other app switched, compare with `docs/reference/performance-baseline.md`, and look in the diff for a
  timer or hook that outlives its work before blaming the machine. A repeat failure is a real finding.
- Keep the release profile (`opt-level = "s"`, LTO, `panic = "abort"`).
