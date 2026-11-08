# Workflow

- Replace the running notch with `scripts/reload.sh`, or let the user's `scripts/watch.sh` rebuild it; confirm a
  rebuild by reading the tail of its output. The user closes an old copy themselves while developing.
- A change is done when `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass (add `--workspace`, with the MSVC toolchain, when `crates/settings/` or `crates/store/` changed).
  After touching the host, animation, timers or drawing, `scripts/perf.sh` must pass too; tell the user first, because
  it moves the mouse for about three minutes and replaces the running notch.
- Mouse-driven checks (`scripts/perf.sh`) and screenshots need an unlocked screen; while it is locked (a `LogonUI`
  process runs) screenshots come out black. Verify through the `NOTCH_DEBUG` log or unit tests then. A check that
  opens windows over the screen or moves the mouse is run on purpose, after telling the user.
- When the `Page` trait or anything pages use changes, update `docs/tutorials/add-a-page.md` in the same change; its example is
  a real page that compiled. Do the same for the `Activity` trait and `docs/how-to/add-an-activity.md`.
- When behavior a user can see changes, update `README.md` and the doc that covers it in the same change.
- Commit only when the user asks, and then with `/group-commit` (see `AGENTS.md`).
- Write brainstorming specs, plans and spikes to `.superpowers/` (`specs/`, `plans/`, `spikes/`), which git ignores; `docs/` holds only documentation.
- Build with a `CARGO_TARGET_DIR` of your own when you run beside the user's notch or another agent, so the running `notch.exe` is never overwritten. A target directory takes about 3 to 9 GB; delete the stale ones you made.
- `scripts/test-build.sh` builds the current tree, committed or not, into `dist/` as an installer (a zip when Inno Setup is missing) for a teammate.
- `scripts/perf.sh` failures are findings, not noise: see `.claude/rules/performance.md` before rerunning, and never raise a limit.
- After editing a file in `.claude/agents/`, run `claude plugin validate .claude/agents`.
- Start the notch from a tool shell detached (`cmd start` with the output redirected), or the shell can end with it.
