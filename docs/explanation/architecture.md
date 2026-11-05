# Architecture

> **Explanation.** Read when: you want to know how the notch fits together and why. Skip if: you need a file's location or a step to follow; see the [source map](../reference/source-map.md) or the how-to guides.

The notch is a small window pinned to the top of the screen. It shows one **page** at a time, either collapsed
(a slim pill) or open (while the pointer is over it). While it is collapsed, **activities** (a volume change, a
charger being plugged in, a notification) take over the pill for a moment or add a small badge to it. The code is split
so that adding or removing a page or an activity never touches the host.

The app is two processes: `notch.exe` (this document's subject, drawn with Direct2D, no UI framework) and the settings
window, `notch-settings.exe`, started from the tray (see [Two processes](#two-processes)).

## The host

`host/` owns everything that is not specific to a page: the window and its message loop, hover, the springs that open
and close the pill, which page shows, input, the tray and the settings panel. It names no page. `window.rs` turns
Windows messages into calls on the `Notch`, and the `Notch` decides what to show and draws it.

`Notch` is one struct whose methods are split one job per file (`host/notch/`), and a job that keeps its own state
owns a small struct for it (`Springs`, `HoverState`, `HidingState`, `KeyboardState`, `PanelState`). The reason is
reach: with one 2,000-line file every change could touch every field, and a reader had to hold the whole notch in
mind. With a file per job, the springs can only be changed through `animation.rs`, hover only through `hover.rs`, and
a job's logic that takes and returns plain values can be tested without a window. [The source
map](../reference/source-map.md#host-files) lists every host file and what it does.

### Choosing a page

Each page answers `priority(mode)` for `Mode::Resting` (collapsed) and `Mode::Expanded` (open). `None` means
"nothing to show". The host shows the highest priority in each mode, and the earlier registered page wins ties. A page
that answers `Priority::URGENT` while open comes before the page the user chose, for as long as it does.

While collapsed the highest priority is always the page shown. While open it is only the starting point: with two or
more pages that have something to show, the user moves between them (see Navigation), and the page they moved to stays
open until they move again.

The two modes are independent. The media page, for example, is shown collapsed only while music plays, but stays up
when open while paused so playback can be resumed. When no page answers for a mode, the host fills the gap: the
collapsed pill shows the time, and the open notch shows its header alone. The host asks again after every event, so a
page changes what it shows by changing its own state.

With lyrics off, the collapsed media player uses the measured song-title width, artwork and wave spacing, plus 2 DIP
of text slack. Its total width stays between 96 and 400 DIP; titles that exceed this limit are ellipsized.

### The open notch

Every open page sits inside the same frame, drawn by `chrome.rs`: 10 px of padding, a 16 px header, a 10 px gap, then
the page's content area, with 16 px of padding at the sides and 14 below. The content area is **86 px high for every
page** (`chrome::CONTENT_HEIGHT`), so a page's content is laid out for that height; its width is the page's own. The
header holds the page dots on the left, and at the right the status marks and the time. A page draws in
`Frame::bounds`, which is the content area, and cannot draw into the header or the padding. The layout with its
numbers is in [development.md](../reference/layout.md#the-notch-layout).

The header's settings button, which opens the "Edit pages" view (drag a chip to reorder, double-click to remove, click
an available one to add; saved as `navigation.order` and `navigation.disabled`), is kept in the code and switched off
by `chrome::SETTINGS_BUTTON_SHOWN`. A page can be opt-in (`Page::is_opt_in`), which starts it switched off.

### Opening and closing

Pointing at the notch starts a timer; if the pointer stays, the notch opens in two legs, a header-only pill first and
then the whole page, and it closes in two legs the other way, after a short delay once the pointer leaves. One spring,
`expansion`, goes from 0 (collapsed) through `HEADER_STAGE` (the header-only pill) to 1 (open), and the sizes, the
fades and the time's glide all follow it. The springs, delays and fades are constants in `notch.rs` and the file that uses them (`notch/animation.rs`, `notch/hover.rs`, `notch/hiding.rs`).
The closing spring uses stiffness 330 and damping 22 for a small bounce into idle; the opening response is separate.
While files are dragged over the notch it opens at once on the page that takes drops and grows a little, as if ready.

### Hiding

The notch can step out of the way in two ways, both in `notch/hiding.rs` (`is_hidden`, `is_suppressed`), with the rules in
`visibility.rs`:

- **Hide when idle** (a tray option, over regular windows): with nothing to show it shrinks to a nearly invisible
  240 x 6 px strip that still receives the pointer, shows the small pill after the pointer rests on it for 150 ms, and
  opens if the pointer stays.
- **Over a maximized or full-screen window** one `Showing` each (`notch.maximized`, default `lyrics`;
  `notch.fullscreen`, default `hidden`) replaces Hide when idle. The words are listed in
  [configuration.md](../reference/settings-keys.md#settings-file). `visibility::allows` decides what a word shows, with
  `Page::shows_live_text` marking active lyrics. `hidden` hides the window and draws nothing, so no hover, banner or
  message shows until the window in front changes; a suppressed notch keeps its full-size layout so no strip grows back
  when it returns. Whatever a word leaves out hides to the strip, 3 px high with a 350 ms wait unless Hide when idle is on, so the tabs and title bar of the window
  are not caught. `visibility::migrate` writes the new keys from the retired `notch.hide_for_fullscreen`,
  `notch.show_over_maximized` and `notch.lyrics_over_maximized` at start and on a reload, never over a saved new key.
  A reload that changes a choice while such a window is in front applies at once: it hides or shows the window and
  re-lays out the pill.

`screen_mode.rs` decides which case applies. It hooks Windows' events for the window in front changing, and, for as
long as one window is in front, for the moves and resizes of that window's thread (`SetWinEventHook`, one thread at a
time, so mouse moves never wake the notch). A window covering the whole primary monitor without a title bar is full
screen; a zoomed one is maximized; the desktop, the taskbar and the notch's own windows never count.

Every foreground change reaches the host, even when the screen mode is unchanged, so a demoted Notch window can
regain topmost status without taking focus. Hover restores topmost before the drop-target overlay steps aside.
Session lock/unlock and automatic resume reset hover, capture, drag and reveal state, then restore the overlay's
placement. Session notifications use `WTSRegisterSessionNotification` for the current session; if registration fails,
the error is logged and foreground recovery remains available. Leave tracking is armed only while
`WindowFromPoint` identifies the Notch window, preventing repeated leave messages from an occluded window.

### Navigation

With two or more pages that answer `Some` for `Mode::Expanded`, the page dots in the header show where the user is.
The user moves between pages by scrolling the mouse wheel, swiping two fingers on a touchpad (sent as scrolling), or
clicking a dot. Touchpad scrolling moves continuously through the row with a fast smoothing spring. After a 200 ms
quiet interval, a softer, critically damped spring centers the nearest page, preserving the visible position and
velocity at release. New input can interrupt it immediately. Layout updates do not center the row while fingers
are scrolling. The nearest page is remembered as the logical selection.

Each visible page gets its own requested content width for drawing and pointer input. Their center spacing is half
of each width plus the page gap. The notch's width interpolates between these natural widths using the rendered
scroll progress, so it progressively shrinks or grows instead of stretching the incoming page to the outgoing one.
No per-frame allocations are needed for this pair of page frames, and the animation timer stops once motion settles.

Windows input-source metadata identifies touchpads; drivers without it fall back to small deltas and the active
gesture. Large deltas stay in touchpad mode once a swipe starts. Input-end timers ignore stale queued messages.
Smooth scroll sensitivity is 400 units per page. Wheel input still moves one page per notch and can queue up to two
pages ahead; wheel and dot navigation use a critically damped transition to a page center.
The pages form a loop (past the last comes the first), and the page left open is remembered as `navigation.page` in
the settings. `host/navigation.rs` holds the rules as pure functions; `notch/pages.rs` and `notch/layout.rs` apply them.

### Sizes

A page reports `sizes()`: its collapsed width and its open size, in device-independent pixels. The host animates
the pill toward them with springs, so a page that changes its mind (the media page widens for lyrics) gets a smooth
transition for free. The host uses the requested widths and ignores the open height, which is fixed at 86 (see above);
content that exceeds it is clipped. The collapsed height is `RESTING_HEIGHT` (30) plus 3 px of bottom padding.

### Painting

The notch is a layered window with per-pixel alpha. Each frame the host draws a soft shadow and the black pill, then
the page's backdrop (the media page's lyric glow), `draw_resting` on the page chosen for the collapsed mode,
`draw_expanded` on the one chosen for the open mode, and the header, with opacities that cross-fade as the notch opens.
Only the rectangle around the pill is redrawn and shown, and the window shrinks and grows with the pill, so the screen
outside the pill is never covered and never blocks clicks.

### Input and timers

Windows messages reach `window.rs`, which calls the notch. The notch converts positions to a common space and routes
pointer events to the page chosen for the open mode. A right click on the notch outside the open page does nothing: the
menu is the tray icon's. A page's timers (`Context::set_timer`) are Windows timers whose ids encode the page and the
timer number, so each page has its own.

### Activities

The host asks every running activity for its `presence()` after each event.

- A **transient** (up to two, newest win) replaces the page's collapsed content: the pill resizes to the transient
  widths and a blend spring cross-fades the two.
- A **badge** stays beside the page's content at the right end of the pill, which the page is told to leave free.
- A **banner** opens the pill into a card taller than the idle pill and smaller than the open notch. It has the pill to
  itself, transients wait behind it, and while it shows, pointing at the notch holds it instead of opening the notch
  (`Activity::on_hover_changed`).

Required activities always start and ignore saved disable settings. Only optional activities expose a saved
`activity.<id>` tray toggle; Notifications is the only optional built-in and defaults to on. The host registers the
window for Windows power-setting and clipboard messages on their behalf (`Context::listen_to_power_setting`,
`listen_to_clipboard`) and hands them to every running activity as a `SystemMessage`. Activity timers use ids from
1000 up and their wake-ups use their own window message, so they never clash with a page's. See
[adding-an-activity.md](../how-to/add-an-activity.md).

### Status marks

While the notch is open, small marks sit left of the time: the media page's wave, the timer's ring, an activity's
muted speaker or Caps Lock arrow. A page or an activity answers `header_status_width` and `draw_header_status`, and the
host lays them out from the time toward the left, lowest `header_status_rank` first (the timer's ring is last, at the
far left), pages before activities within a rank.

### Threads

All drawing and page code runs on the single UI thread. Work that waits runs on threads of its own, which wake the UI
thread with a `Notifier` that posts a message; the page or activity then reads the new state in `on_notify`:

- the media page watches the Windows media session and looks up lyrics
- the notifications activity reads the notification list when its folder changes
- the shelf asks Explorer for file pictures and carries a file being dragged out (that needs a single-threaded
  apartment, which the UI thread is not); `drop_target.rs` has the thread that takes dragged files in
- the Caps Lock activity runs a keyboard hook, which only wakes the activity

### Animation

While anything moves the host runs a frame timer at the display's refresh rate; when the host's springs have
settled and no page reports `animate` as still moving, the timer stops. A collapsed notch with nothing happening
has no timers running except the host's own clock (once a minute) and a page's own, which is why it uses no CPU.

## Two processes

`crates/settings/` (GPUI) and `crates/notch/` depend on `crates/store/` and never on each other, so the notch links no UI framework. The notch
publishes `pages.known` when it starts so the window can list pages it does not link. The window writes keys through
`crates/store/` and tells the notch to re-read them; the message, keys and field behavior are in
[configuration.md](../reference/performance-baseline.md#settings-window). The idle budget applies to `notch.exe` only
([performance.md](../reference/performance-baseline.md#settings-window)).

| File in `crates/settings/src/` | Job |
| --- | --- |
| `main.rs` | the window: the logo and page list in the sidebar, the selected page, a clear title bar over both, in the system's light or dark theme |
| `layout.rs` | `Page`, `Group` and `Item`: the page title and description, the bordered groups and their rows |
| `sections/` | one file per section, each returning a `SettingPage`; `mod.rs` lists them and writes every key through `apply` |
| `field.rs` | the switch and the text and number fields bound to a key, and their save rules |
| `notify.rs` | posts `WM_SETTINGS_CHANGED` to the notch |
| `single_instance.rs` | allows one window and focuses the open one |
| `startup.rs` | the `Run` registry value behind **Start with Windows** |

To add a setting, see [adding-a-setting.md](../how-to/add-a-setting.md).

## Pages

`pages/mod.rs` is the registry. See [adding-a-page.md](../tutorials/add-a-page.md) to write one. A page may also answer
`presence()` to show something in the collapsed pill beside or instead of the page on show (the timer does); the host
treats it like an activity's presence.

The media page is the largest example of how to split a page:

| File | Job |
| --- | --- |
| `mod.rs` | the `MediaPage` type: state, and the `Page` implementation |
| `session.rs` | mirrors the Windows media session into plain data, on its own threads |
| `artwork.rs`, `app_icon.rs` | decode the album art and find the player's icon |
| `lyrics/` | the lyric data model and `.lrc` parser, and the lookup client (`client.rs`) |
| `lyric_follower.rs` | follows which lyric line is being sung and animates the change |
| `layout.rs` | where everything in the open view goes |
| `resting.rs`, `expanded.rs` | draw the collapsed and open views |
| `controls.rs`, `lyric_panel.rs` | the buttons and the scrolling lyrics |
| `theme.rs`, `backdrop.rs` | the color of the cover, and the glow behind the lyrics |
| `wave.rs` | the moving bars shown while music plays |
| `styles.rs` | the page's text styles |

The timer page is the example of a page that also has a collapsed presence and a status mark:

| File | Job |
| --- | --- |
| `mod.rs` | the `TimerPage` type: input, timers, the alert, the `Page` implementation |
| `session.rs` | the rules: phases, the session counter, pause, extend, when to wake next, saving; pure |
| `view.rs` | what is said and offered in each state, and the compact text; pure |
| `layout.rs` | where the time, the words and the buttons go |

## Where to go next

- Where a file lives: [source map](../reference/source-map.md).
- How to test it: [test and snapshot](../how-to/test-and-snapshot.md).
- Why the budget is what it is: [performance budget](performance-budget.md).
