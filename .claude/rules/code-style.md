---
paths:
  - "crates/**/*.rs"
  - "scripts/**"
  - "crates/notch/build.rs"
  - "crates/settings/build.rs"
---

# Code style

- Name things in full: `present_track_ids`, not `ptids`.
- Comment only what the code cannot show: a reason, a unit, a Windows workaround or the failure it prevents. One line,
  two when the second carries a new fact, in sentence case. A comment that needs more is a docstring.
- Leave undocumented what the name and signature already say, and leave out step narration, history, issue numbers and
  banners. Begin a file with a one-line `//!` only when its purpose is not clear from its name.
- Write layout sizes in device-independent pixels and multiply by `frame.scale` (or `canvas.scale()`) when drawing.
- Wrap `unsafe` around the single Windows call that needs it, with a `// SAFETY:` line when the reason is not obvious.
- `cargo fmt` is the formatter; `rustfmt.toml` holds its settings.
- In the notch, log a failed cleanup or recovery with `log::debug(&format!("could not <action>: {error}"))` and keep running; reserve
  `expect` and `unwrap` for tests and values that cannot fail.
- Silence a lint with `#[expect(lint)]` and a reason, not `#[allow]`; the code has neither today, so fix the cause first.
- Borrow before you clone: take `&str` and `&[T]` in parameters, and clone only where the value must be owned.
- Do not `collect` into a `Vec` just to iterate again; keep the iterator.
- Errors are `windows::core::Result` at the Windows boundary; no
  error-derive crate is a dependency (`Cargo.toml` has no `thiserror` or `anyhow`), so ask before adding one.
- Clippy 1.97 flags `manual_checked_ops` (write `checked_div`, `checked_sub` and the like instead of an `if` that
  guards the operator) and `assertions_on_constants`: a test must not assert on a constant or a value the compiler
  knows. Assert on what a function returns, or use `const { assert!(..) }` for an invariant between constants.
