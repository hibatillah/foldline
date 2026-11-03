# Page API

> **Reference.** Read when: you need where a page hook, drawing call or widget is defined, or a rule the signatures do not show. Skip if: you are writing your first page; follow [the tutorial](../tutorials/add-a-page.md).

## Where each part is defined

The source is the reference; each hook has a doc comment. `codegraph_explore` returns any of these with their callers.

| Part | Defined in |
| --- | --- |
| `Page` trait and every hook | `src/page.rs` |
| `Context` (repaint, timers, notifier, keyboard, settings, fonts), `Frame`, `PointerEvent`, `KeyInput` | `src/page.rs` |
| `Control`, `SettingsGroup`, `SearchHit` (a page's settings in the notch's panel) | `src/page.rs` |
| `DEFAULT_CONTENT_HEIGHT` (86), `MAX_CONTENT_HEIGHT` (220), `RESTING_HEIGHT` (33), `RESTING_MIN_WIDTH` (150) | `src/page.rs` |
| `Presence`, `Flash` (what a page shows in the collapsed pill) | `src/activity.rs` |
| `Canvas` drawing calls | `src/gfx/canvas.rs` |
| `FontStyle`, `UI_FONT`, text measuring | `src/gfx/fonts.rs` |
| `Spring`, geometry, colors, `Path`, `Mask` | `src/gfx/` |
| widgets: `IconButton`, `PillButton`, `Ring`, `TextField`, `Icon`, `draw_dashed_outline`, HUD layouts | `src/ui/` |

## Rules the signatures do not show

- A page must answer `id`, `priority`, `sizes`, `draw_resting` and `draw_expanded`; every other hook is optional.
- The host asks `priority` and `sizes` after every event. Keep both cheap and free of side effects.
- Report the size of the content only. The host adds 32 DIP to the width and 50 to the height for padding and the header. Ask for `DEFAULT_CONTENT_HEIGHT` unless the content needs more; requests above `MAX_CONTENT_HEIGHT` are capped.
- Drawing and pointer positions are device pixels. Multiply device-independent sizes by `frame.scale`, and multiply every color by the `opacity` you are given so the page follows the open and close animation.
- `frame.bounds` is the page's own content area. During a swipe each visible page keeps its requested width, and pointer events use the same shifted frame as drawing.
- `on_shown_changed(true)` means the open notch is showing this page. Start and stop measuring there, and stop every timer when it is `false`. While the settings panel is open no page is shown.
- Timers are `TimerId(n)` with `n` below 16. `Flash` uses `TimerId(0)`, so a page that uses `Flash` picks other numbers. Setting an existing timer re-arms it.
- Settings keys start with the page id (`media.lyrics`). `on_settings_changed` re-reads the page's keys, falls back to the current value (not the default) when one is missing, and never writes settings.
- Take the keyboard (`context.take_keyboard()`) only from a click, never on hover. The page then receives `on_key` and the notch stays open until the page calls `release_keyboard()`; `on_keyboard_lost` runs when the user clicks another window first.
- A page that has something to say while closed answers `presence()` with `Presence::Badge { width }` (a mark beside the page on show) or `Presence::Transient { width }` (takes over the pill for a moment), and draws it in `draw_presence`. `Presence::Banner` is for activities. [Activities](activities.md) describes how the host lays these out.
- To show settings in the notch's panel, return `Control`s from `settings()`, or several tabs from `settings_groups()`; see [add a setting](../how-to/add-a-setting.md).

## Drawing

- Text styles are `const FontStyle::new(family, size, weight, align)` declared in the page. `.wrapped()` wraps long text instead of cutting it with an ellipsis.
- Icons are shapes built from a `Path` and drawn into small cached images by `ui::Icon`, not font glyphs. Add one in `src/ui/icons.rs` and list it in that file's `ALL` test array.
- Do not use Direct2D geometry (`ID2D1PathGeometry`, `FillGeometry`, `DrawGeometry`): its first use adds about 10 MB. Describe a shape with `Path`, rasterize it with `Mask`, and reuse it through `Canvas::cached_image`.
- `Spring` eases a value toward a target. Call `step(elapsed_seconds)` from the page's `animate` and return `true` while `is_settled()` is `false`.

## Background work

For anything slow or blocking (network, files, media APIs), start a thread in `on_start`, give it `context.notifier()`, update shared state behind a lock, and call `notifier.notify()`. The host then calls `on_notify` on the UI thread, where you read the state and request a repaint. The media page does exactly this.

## Conventions

- Keep pure logic (parsing, layout rules, state machines) in functions that take and return plain values, so a test needs no window; see [test and snapshot](../how-to/test-and-snapshot.md).
- Keep timers and animations off while nothing changes. The notch uses no CPU when idle, and `scripts/perf.sh` checks that ([performance budget](../explanation/performance-budget.md)).
- Split a page into files by job once it grows. The media page is the model: see [architecture](../explanation/architecture.md#pages).
- A page uses `page`, `activity`, `gfx`, `ui` and `platform`, never another page.
