# Run the performance check

> **How-to.** Read when: you changed the host, animation, timers or drawing, or you prepare a release. Skip if: you only need the budget; see [performance budget](../explanation/performance-budget.md).

## Run it

```
scripts/perf.sh               # builds, then runs the checks (about three minutes)
scripts/perf.sh --no-build    # reuse the existing release build
scripts/perf.sh --with-media  # also print numbers with every page on; play music first
scripts/perf.sh --extended    # also measure held open by the pointer, swipes, release snap, and recovery
scripts/perf.sh --lyrics-only # live lyrics on/off, collapsed and open, in isolated test profiles
scripts/perf.sh --screen-mode # also maximize and restore a test window it opens (opt-in: it shows a window)
```

The script starts a copy of the notch with only the shelf page (`NOTCH_PAGES=shelf`), so the result does not depend
on what you are listening to. It then starts two more copies: the shelf and devices pages opening on devices
(`NOTCH_PAGES=shelf,devices`), the stats page alone (`NOTCH_PAGES=stats`), and the shelf and stats pages (`NOTCH_PAGES=shelf,stats`, opening on the shelf), pinned open by
a click it sends to the header pin. It moves the mouse to open and close the notch, so leave the mouse alone while it
runs. It also asks a running notch to close; start yours again with `scripts/reload.sh`.

It checks, and fails when a limit is exceeded:

| Check | Limit |
| --- | --- |
| the notch settles after start-up and after each animation | CPU back to idle within 20 s |
| CPU while collapsed and idle | 0.1 % of one core, measured over 20 s |
| memory (private working set) after start | 30 MB |
| CPU for one open-and-close | 400 ms |
| memory growth over 10 open-and-closes | 3 MB |
| handle growth over 10 open-and-closes | 20 handles |
| open events logged (`open=` log lines) | exactly two per cycle, so none were missed or doubled |
| CPU while idle again after the animations | 0.1 %; the frame timer must have stopped |
| devices: CPU for one open-and-close of the devices page (5 cycles) | 400 ms |
| devices: CPU while idle after those cycles | 0.1 %; the open-time device reads must have stopped |
| devices: handle growth from start to the end of the idle sample | 20 handles |
| stats: CPU while idle after the page was open for 3 s and closed | 0.1 %; its 1 Hz timer and counters must have stopped |
| stats: handle growth from start to the end of the idle sample | 20 handles |
| pinned: a press on the header pin pins it (`pinned=true` log line), the notch is open (`open=true`), and it stays open with the pointer parked (no `open=false`) | required |
| pinned: CPU while idle pinned open, and handle growth | 0.1 %; no hover-watch timer runs while pinned, and the stats sampler stays off because the shelf shows; 20 handles |
| screen mode (`--screen-mode` only): CPU per maximize-and-restore of the test window | 100 ms |
| screen mode (`--screen-mode` only): CPU while idle after, and handle growth | 0.1 %, 20 handles |

Measure the idle windows with the pointer parked and no app switch. The isolated settings keep the pill up over
maximized and full-screen windows (`notch.maximized=music`, `notch.fullscreen=music`), so an app switch costs only the
topmost raise rather than a hide animation of about 1.5 s; `--screen-mode` sets `notch.maximized=lyrics` again to
measure that animation. Moving the pointer over the foreground app still costs CPU through the location hook on that
app's thread (`host/screen_mode/location_hook.rs` pauses the hook during a flood of moves, but each move still reaches
it first), so leave the mouse alone until the run ends.

Some outside events make the notch work during an idle sample, and that work is real. Windows writes its notification
database for every toast, badge, tile update and expiry, and the notifications listener answers each write with about
200 ms of CPU, most of it in lookups by id of about 8 ms each. An activity can also show a message, for example a
Bluetooth device that connects. When the debug log shows one of these during an idle sample (`notifications: read`,
`new by id`, `card for`, or `pill shows`), the sample is taken again, three samples at most, and the last one counts.
The run prints `disturbed (<percent> %) by: <log line>; measuring again` for each repeat.

The last check is the important one. A common mistake is leaving a timer or animation running, which makes the
notch use CPU forever; it shows up there as several percent of a core.

The latest recorded baseline and its workload are summarized in [README.md](../../README.md#resource-use).
Run the check for your current machine. The limits are set
in constants at the top of `crates/notch/examples/perf.rs`; they exist to catch regressions, so fix the cause rather than raising one.

`--with-media` prints memory and CPU with every page on and your music playing. It has no limits, because the result
depends on the track and on how often its lyrics change.

`--extended` adds 20-second samples while held open by the pointer, after the touchpad release snap, and collapsed after swiping. Its swipe
workload sends horizontal wheel messages to its own notch window and verifies that the window width changes.
`--screen-mode` opens a plain window of the measuring program, brings it to the front, and maximizes and restores it
six times, 1.5 s each, so the notch hides and comes back through its 500 ms reveal delay every time. It is off by
default because it shows a window over your screen; it reports NOT RUN when Windows keeps another window in front.
`--lyrics-only` requires a playing Windows media session. It uses separate temporary settings under `run/` to compare
the real display toggle on and off without editing your settings. It verifies that the synced-lyrics pane actually
opens before reporting active-lyrics results; missing lyrics or stopped playback leave this comparison incomplete.
These extra scenarios are informational, and CPU is reported as a percentage of one core. RAM means private working
set; the reported swipe peak is sampled at each end of a swipe, not a continuously traced maximum.

### When a check fails

A single failed idle-CPU check can be noise: rerun on a quiet machine, see the variation table in
[performance baseline](../reference/performance-baseline.md#run-to-run-variation-2026-11-06).
`perf.rs` deletes `run/perf.log` when it ends, so read the log through a hand run with `NOTCH_DEBUG=<file>` instead.
The script stops the notch it started and leaves none running; start yours with `scripts/reload.sh`.

### The two files

- `scripts/perf.sh` is only a runner: it parses the options, builds, copies the executable to `run/` and starts the
  measuring program.
- `crates/notch/examples/perf.rs` does the measuring, with the devices, stats, pinned and screen-mode scenarios in
  `crates/notch/examples/perf/scenarios.rs`. It is a Rust program (run with `cargo run --release -p notch --example perf`) that
  starts the notch, moves the pointer with `SetCursorPos`, and reads CPU time, handles and private memory from the
  process. A shell script cannot move the mouse or read those numbers, and an example is not part of the shipped
  `notch.exe`. The limits are constants at the top of the file.

Every scenario starts the notch with its own settings file under `run/` (`notch.fullscreen=music`, so a full-screen
window in front cannot hide the notch mid-check), and your `settings.txt` stays untouched. The check also writes a copy
of the executable under `run/` and a temporary log.
