# Page API

> **Reference.** Read when: you need where a page hook, drawing call or widget is defined, or a rule the signatures do not show. Skip if: you are writing your first page; follow [add a page](../how-to/add-a-page.md).

## Where each part is defined

The source is the reference; each hook has a doc comment. `codegraph_explore` returns any of these with their callers.

| Part | Defined in |
| --- | --- |
| `Page` trait and every hook | `crates/notch/src/page.rs` |
| `Context` (repaint, timers, notifier, keyboard, settings, fonts), `Frame` (including `lead_gap`), `PointerEvent`, `KeyInput` | `crates/notch/src/page.rs` |
| `DEFAULT_CONTENT_HEIGHT` (86), `MAX_CONTENT_HEIGHT` (220), `RESTING_HEIGHT` (33), `RESTING_MIN_WIDTH` (150) | `crates/notch/src/page.rs` |
| `Presence`, `Flash` (what a page shows in the collapsed pill) | `crates/notch/src/activity.rs` |
| `Canvas` drawing calls | `crates/notch/src/gfx/canvas.rs` |
| `FontStyle`, `UI_FONT`, text measuring | `crates/notch/src/gfx/fonts.rs` |
| `Spring`, geometry, colors, `Path` (also read from SVG path data by `gfx/svg_path.rs`), `Mask` (`fill` and `stroke` build the coverage; `tinted` colors it with one color and `painted` per pixel, for gradients) | `crates/notch/src/gfx/` |
| shared colors and text styles (`palette.rs`, `type_styles.rs`; a color or style two or more pages share lives there, one page's own stays in the page) | `crates/notch/src/ui/` |
| widgets: `IconButton`, `PillButton`, `Ring`, `TextField`, `Icon`, `draw_strokes` (line icons from SVG path data in several colors, `svg_icon.rs`), `BannerButtons` (the Dismiss and Open pair of a banner, `banner_buttons.rs`), `draw_dashed_outline`, HUD layouts | `crates/notch/src/ui/` |

## Rules the signatures do not show

- A page must answer `id`, `priority`, `sizes`, `draw_resting` and `draw_expanded`; every other hook is optional.
- The host asks `priority` and `sizes` after every event. Keep both cheap and free of side effects.
- Report the size of the content only. The host adds 32 DIP to the width and 50 to the height for padding and the header. Ask for `DEFAULT_CONTENT_HEIGHT` unless the content needs more; requests above `MAX_CONTENT_HEIGHT` are capped.
- Drawing and pointer positions are device-independent units. Every draw hook except `draw_backdrop` receives a `Frame` or area that starts at zero and a canvas whose `scale()` is 1; `on_pointer`, `on_presence_pointer`, `claims_scroll` and `uses_right_click` receive points in the same units, from the top-left of that frame or area. `draw_backdrop` draws in device pixels: its frame, pill and radius are device pixels and `canvas.scale()` is the display scale. Multiply every color by the `opacity` you are given so the page follows the open and close animation.
- `frame.bounds` is the page's own content area, always at the page's requested width and height (the pill may still be resizing around it). During a page switch the leaving and the arriving page are drawn shifted a few pixels sideways and faded; pointer events go to the arriving page only, at its resting frame.
- `on_shown_changed(true)` means the open notch is showing this page. Start and stop measuring there, and stop every timer when it is `false`.
- Timers are `TimerId(n)` with `n` below 16. `Flash` uses `TimerId(0)`, so a page that uses `Flash` picks other numbers. Setting an existing timer re-arms it.
- Settings keys start with the page id (`media.lyrics`). `on_settings_changed` re-reads the page's keys, falls back to the current value (not the default) when one is missing, and never writes settings.
- `on_language_changed` also runs after the notch moves to a monitor of another scale, because `context.fonts()` is then a new set: measure cached widths again there.
- A page that holds personal data (the clipboard page) is told when the user switches it on or off: `on_enabled_changed(is_enabled)` runs once at start and after every change, and it drops its data when `is_enabled` is false. `on_system_message` delivers a Windows message the page asked for (`context.listen_to_clipboard()`, `SystemMessage::ClipboardChanged`) and only reaches pages that are switched on. `context.request_close()` closes the open notch (the clipboard page does it 600 ms after a paste), except while the user has pinned it open; then it does nothing. `Context::window()` is the notch window, for the few Windows calls that need an owner (writing the clipboard).
- `is_left_aligned` answers true for a page that sits at the left padding, under the header dots, when the open notch is wider than it (the media page); the default centers it.
- `on_pointer` receives `PointerEvent`s for the visible open page, in its own frame: `Moved`, `Pressed`, `Released`, `Left`, `CaptureLost`, `SecondaryReleased` (only where `uses_right_click` answers true) and `Scrolled { notches }`. The wheel switches pages unless `claims_scroll(point, frame)` answers true for the point under it; then the page gets `Scrolled` and the page does not switch (the calendar's event list does this). A banner or a badge a page answers in `presence()` gets its pointer through `on_presence_pointer(event, area, context)` while the notch is collapsed, with `PointerEvent::Left` when the pointer leaves it; `area` is the rect the host draws it into. A banner hears every event over the pill; a badge hears `Moved`, `Pressed` and `Released` only inside its own area, and only while no banner or transient takes the pill (the weather page opens the notch from a press on its rain badge). `context.request_open()` opens the notch from the page's code, `context.request_close()` closes it.
- Take the keyboard (`context.take_keyboard()`) only from a click, never on hover. The page then receives `on_key` and the notch stays open until the page calls `release_keyboard()`; `on_keyboard_lost` runs when the user clicks another window first.
- A page that has something to say while closed answers `presence()` with `Presence::Badge { width }` (a mark in the group at the right edge of the pill, after the page's line when it claims the pill) or `Presence::Transient { width }` (takes over the pill for a moment), and draws it in `draw_presence`. `Presence::Banner` is for activities, and a page's `Presence::Companion` is ignored. An activity banner is brought down by a priority activity's level (`Activity::dismiss_banner`); a page cannot have a banner dismissed that way. Answer `header_status_rank` to place the badge in the group (lowest nearest the clock; wave 0, Caps Lock 1, muted 2, timer 3, event 4, prayer 5; the battery ring and the dots are not ranked and always sit right of every badge). A page also answers `header_status_page` (the id of the page a click on its header icon opens, `None` for no click), `short_presence_width` and `draw_short_presence` (a shorter badge for when two or more share the pill; `draw_presence` is the longer form, drawn with its icon and text, while `draw_header_status` is the icon alone) and `countdown_seconds` (seconds left on the countdown the page shows as a badge, `None` when it shows none; the lyric line shows only the countdown with the least time left). [Activities](activities.md) describes how the host lays these out.
- A page claims the collapsed pill by answering `priority(Mode::Resting)`; the media page does it only while a lyric line shows. `sizes().resting_width` is the page's own content, without the pill's edges, the group and the dot column; the host adds them.
- A claiming page that draws something at its left end (the media page's artwork) answers `resting_lead_width()` with that width in DIP. The host passes the room it keeps after it in `Frame::lead_gap` (24 DIP unless the page answers `resting_lead_gap()` with its own width, which the host also reserves in the pill; the dot column sits at the pill's right end, so a dot never moves the line; `host/claimed_row.rs`); start the rest of the content at `frame.bounds.left + lead + lead_gap`. The same answer marks the page's `Presence::Badge` as the music wave: the host keeps the pill up for it under Hide when idle (unlike other badges, which never do), but it does not count as an activity over a maximized window.
- A page has no settings controls in the notch and the `Page` trait has none (no `Control`, `SettingsGroup`, `settings`, `search` or `icon` hook): the settings window edits a page's keys, and the page only reads them. See [add a setting](../how-to/add-a-setting.md).

## Drawing

- Text styles are `const FontStyle::new(family, size, weight, align)` declared in the page. `.wrapped()` wraps long text instead of cutting it with an ellipsis; `.tabular()` gives every digit the same width, for numbers that change while shown.
- Icons are shapes built from a `Path` and drawn into small cached images by `ui::Icon`, not font glyphs. Add one to the `Icon` enum in `crates/notch/src/ui/icons/mod.rs`, give it a shape in `icons/paths.rs`, and list it in the `ALL` test array in `icons/tests.rs`.
- Do not use Direct2D geometry (`ID2D1PathGeometry`, `FillGeometry`, `DrawGeometry`): its first use adds about 10 MB. Describe a shape with `Path`, rasterize it with `Mask`, and reuse it through `Canvas::cached_image`.
- A page places a cached image with `Canvas::pixel_square(center, size)` and keys the cache by the device side it returns (see `ui/ring.rs`).
- `Spring` eases a value toward a target. Call `step(elapsed_seconds)` from the page's `animate` and return `true` while `is_settled()` is `false`.

## Background work

For anything slow or blocking (network, files, media APIs), start a thread in `on_start`, give it `context.notifier()`, update shared state behind a lock, and call `notifier.notify()`. The host then calls `on_notify` on the UI thread, where you read the state and request a repaint. The media page does exactly this.

## Conventions

- Keep pure logic (parsing, layout rules, state machines) in functions that take and return plain values, so a test needs no window; see [test and snapshot](../how-to/test-and-snapshot.md).
- Keep timers and animations off while nothing changes. The notch uses no CPU when idle, and `scripts/perf.sh` checks that ([performance budget](../explanation/performance-budget.md)).
- Split a page into files by job once it grows. The media page is the model: see [host and pages](../explanation/host-and-pages.md#pages).
- A page uses `page`, `activity`, `gfx`, `ui` and `platform`, never another page.
