# Performance budget

> **Explanation.** Read when: you change the host, animation, timers or drawing, or wonder why the code avoids polling. Skip if: you only write page content that draws and handles clicks.

## The budget

`foldline.exe` uses 0 % CPU while idle and stays under 50 MB of memory. A tray app that is always running earns its place only if it costs nothing when nothing happens. The settings window (`foldline-settings.exe`) is a separate process, has a higher footprint while open, and is outside the budget.

`scripts/perf.sh` enforces tighter limits than the budget (30 MB after start, 0.1 % idle CPU, 400 ms CPU per open and close); see [run the performance check](../how-to/run-the-perf-check.md) for the limits and [the baseline](../reference/performance-baseline.md) for the latest numbers.

## What the budget changes in the code

- **Text lookups stay cheap.** `tr!("key")` on a found key returns a borrowed string (one hash lookup, no allocation); a text with a value in it is built when it is made or when the language changes, never on every paint. The language change adds no timer, hook or thread.
- **Every timer and animation stops as soon as its output stops changing.** The frame timer ends when the springs settle and no page reports `animate` as moving. A timer left running is the usual cause of idle CPU.
- **Background work wakes the UI thread with a `Notifier`; nothing polls.** Wait on what calls you: a Windows callback, a message, a blocking wait or an event hook. An event hook asks for the one event and the one thread it needs (`crates/notch/src/host/screen_mode.rs`), because a hook on every window's events wakes the notch on every mouse move.
- **A page or activity that shows something only while visible measures only while visible.** The stats page opens its counters only while it is the page shown in the open notch (`on_shown_changed` and `on_open_changed`, while the page is switched on) and closes them when the notch closes or another page shows, so a notch pinned open on another page runs no sampler; the shalat page sleeps until the pill mark's window opens.
- **Shapes are described, rasterized once and reused.** Direct2D geometry (`ID2D1PathGeometry`, `FillGeometry`, `DrawGeometry`) costs about 10 MB the first time it is used, so shapes go through `Path`, `Mask` and `Canvas::cached_image`.
- **The release profile stays size-optimized** (`opt-level = "s"`, LTO, `panic = "abort"`).

## When a check fails

Find the cause and leave the limits where they are. Idle CPU measured by `perf.sh` varies between runs on the same build (one build read 0 % and 1.5 % on different runs), so repeat a failing run before changing code, and compare against a build of the last commit.
