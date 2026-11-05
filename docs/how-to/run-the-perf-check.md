# Run the performance check

> **How-to.** Read when: you changed the host, animation, timers or drawing, or you prepare a release. Skip if: you only need the budget; see [performance budget](../explanation/performance-budget.md).

## Run it

```
scripts/perf.sh               # builds, then runs the checks (about two minutes)
scripts/perf.sh --no-build    # reuse the existing release build
scripts/perf.sh --with-media  # also print numbers with every page on; play music first
scripts/perf.sh --extended    # also measure held hover, swipes, release snap, and recovery
scripts/perf.sh --lyrics-only # live lyrics on/off, collapsed and open, in isolated test profiles
```

The script starts a copy of the notch with only the shelf page (`NOTCH_PAGES=shelf`), so the result does not depend
on what you are listening to. It moves the mouse to open and close the notch, so leave the mouse alone while it
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
| hover events logged | exactly two per cycle, so none were missed or doubled |
| CPU while idle again after the animations | 0.1 %; the frame timer must have stopped |

The last check is the important one. A common mistake is leaving a timer or animation running, which makes the
notch use CPU forever; it shows up there as several percent of a core.

The latest recorded baseline and its workload are summarized in [README.md](../../README.md#resource-use).
Run the check for your current machine. The limits are set
in constants at the top of `crates/notch/examples/perf.rs`; they exist to catch regressions, so fix the cause rather than raising one.

`--with-media` prints memory and CPU with every page on and your music playing. It has no limits, because the result
depends on the track and on how often its lyrics change.

`--extended` adds 20-second samples while hovered open, after the touchpad release snap, and collapsed after swiping. Its swipe
workload sends horizontal wheel messages to its own Notch window and verifies that the window width changes.
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
- `crates/notch/examples/perf.rs` does the measuring. It is a Rust program (run with `cargo run --release -p notch --example perf`) that
  starts the notch, moves the pointer with `SetCursorPos`, and reads CPU time, handles and private memory from the
  process. A shell script cannot move the mouse or read those numbers, and an example is not part of the shipped
  `notch.exe`. The limits are constants at the top of the file.

Every scenario starts the notch with its own settings file under `run/` (`notch.fullscreen=music`, so a full-screen
window in front cannot hide the notch mid-check), and your `settings.txt` stays untouched. The check also writes a copy
of the executable under `run/` and a temporary log.
