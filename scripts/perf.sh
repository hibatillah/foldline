#!/usr/bin/env bash
# Measures memory and CPU; moves the pointer while checking the limits in crates/notch/examples/perf.rs.
#
#   scripts/perf.sh                 build and check (about three minutes): shelf, devices, stats, then pinned
#     devices: CPU per open+close cycle 400 ms, idle after cycles 0.1 %, handle growth 20
#     stats: CPU idle after closing the page 0.1 % (its 1 Hz timer and counters stop), handle growth 20
#     pinned: shelf and stats on, pinned on the shelf, pointer parked; stays open, idle 0.1 %, handle growth 20
#   scripts/perf.sh --no-build      reuse the release build
#   scripts/perf.sh --with-media    also sample normal pages with music playing
#   scripts/perf.sh --extended      also sample hover, swipes and recovery
#   scripts/perf.sh --lyrics-only   compare live synced lyrics on/off
#   scripts/perf.sh --screen-mode   also maximize and restore a test window it opens: CPU per toggle 100 ms,
#                                   idle after 0.1 %, handle growth 20
#
# An idle sample disturbed by an outside event in the debug log (Windows wrote its notification database, or an
# activity changed the pill) is taken again, three samples at most.
# Hands off the mouse while it runs: it moves the pointer to open and close the notch.
# The test starts its own notch, which asks a running notch to close; start yours again with scripts/reload.sh.
set -euo pipefail
cd "$(dirname "$0")/.."

build=1
extra_arguments=()
for argument in "$@"; do
    case "$argument" in
        --no-build) build=0 ;;
        --with-media) extra_arguments+=(--with-media) ;;
        --extended) extra_arguments+=(--extended) ;;
        --lyrics-only) extra_arguments+=(--lyrics-only) ;;
        --screen-mode) extra_arguments+=(--screen-mode) ;;
        -h | --help) sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown option: $argument" >&2; exit 2 ;;
    esac
done

[ "$build" = 1 ] && cargo +stable-x86_64-pc-windows-msvc build --release

mkdir -p run
cp target/x86_64-pc-windows-msvc/release/foldline.exe run/foldline-perf.exe

cargo +stable-x86_64-pc-windows-msvc run --release --quiet -p notch --example perf -- \
    --exe "$(cygpath -w "$PWD/run/foldline-perf.exe")" \
    --log "$(cygpath -w "$PWD/run/perf.log")" \
    "${extra_arguments[@]}"
