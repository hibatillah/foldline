# Performance baseline

> **Reference.** Read when: you need the latest measured CPU and memory, or the settings window's footprint. Skip if: you need the limits or how to measure; see [run the performance check](../how-to/run-the-perf-check.md).

These are local Windows x64 release measurements, not guarantees for every machine or media session.
Raw measurement archives are not kept under `docs/`.

## Test setup

- Date: 2026-11-22, Asia/Jakarta.
- Command: `scripts/perf.sh`. The recorded run is a complete `scripts/perf.sh` run that built the release first; every gate passed.
- Windows 11 Home build 26300.9974; AMD Ryzen 5 8645HS, 6 cores / 12 logical processors; 15.3 GiB usable RAM.
- Rust 1.99.0, MSVC x64 target, the repository's size-optimized release profile.
- CPU is process CPU time divided by wall time, expressed as a percentage of one core.
- RAM is private working set in MiB. The runner labels these binary units as MB.
- Workload: 20-second idle samples and 10 open/close cycles on the shelf page (5 on the devices scenario), then the stats and pinned scenarios. The limits are in [run the performance check](../how-to/run-the-perf-check.md).

## Latest recorded baseline

### Shelf page only

After start the notch held 23.8 MiB private working set and 977 handles.

| Measurement | Result | Limit |
| --- | ---: | ---: |
| Initial private working set | 23.8 MiB | 30 MiB |
| Collapsed idle CPU | 0% | 0.1% |
| CPU time per open/close cycle | 116 ms | 400 ms |
| RAM growth over 10 cycles | 0.3 MiB | 3 MiB |
| Handle growth over 10 cycles | -7 | 20 |
| Open events | 20/20 | exactly 20 |
| Idle CPU after cycles | 0% | 0.1% |

### Other scenarios

| Scenario | Measurement | Result | Limit |
| --- | --- | ---: | ---: |
| Devices page (`shelf,devices`, 5 cycles) | CPU time per open/close cycle | 169 ms | 400 ms |
| | Idle CPU after cycles | 0% | 0.1% |
| | Handle growth | -6 | 20 |
| Stats page only (open 3 s, then closed) | Idle CPU after closing | 0% | 0.1% |
| | Handle growth | 0 | 20 |
| Pinned (`shelf,stats`, pinned open on the shelf, pointer away) | Idle CPU | 0% | 0.1% |
| | Handle growth | 0 | 20 |
| | Stays open with the pointer away | yes | required |

### Not run

`--with-media`, `--extended`, `--lyrics-only` and `--screen-mode` were not run. This baseline did not measure music playback, live lyrics, the notch held open by the pointer, swiping, release snap, or a maximized window. Lock/unlock recovery is checked by hand; see [test and snapshot](../how-to/test-and-snapshot.md).

## Reading the numbers

- CPU comes from `GetProcessTimes`, which ticks in about 15.6 ms steps; 0.078 % over the 20 s window is one tick.
- The same build can pass and fail an idle-CPU check between runs, so one failed idle check does not show a stuck timer.
- To judge a change, close other apps, stop other `foldline.exe` copies and rerun `scripts/perf.sh` several times; the limits stay.

## Settings window

The 0 % idle and 50 MB budget applies to `foldline.exe` and is what the baseline above measures. The settings window is a
separate process (`foldline-settings.exe`, GPUI) with its own, higher footprint while it is open, and `scripts/perf.sh`
does not run it.

Source: one launch of the release `run/foldline-settings.exe` on 2026-10-31, same machine as above, with the General page
open and no input. Private working set is `\Process(foldline-settings)\Working Set - Private`, read 10 s after the
window opened; CPU is process CPU time over the following 10 s.

| Measurement | Result |
| --- | ---: |
| Release executable size | 9.1 MB |
| Private working set at 10 s | 44.6 MB |
| Working set at 10 s | 76.3 MB |
| Private bytes at 10 s | 86.3 MB |
| CPU while open and idle | 0.94 % of one core |

## Repeat the checks

Run `scripts/perf.sh` from Git Bash for the baseline, or add `--extended` for the notch held open by the pointer, swipes and recovery.
For live lyrics, play a track with synced lyrics and use `--lyrics-only`.

The check moves the pointer and replaces the running Foldline. Leave the mouse alone during measurement, then run
`scripts/reload.sh` to restore normal pages. See [run the performance check](../how-to/run-the-perf-check.md) for the gates and workload details.
