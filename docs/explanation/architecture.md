# Architecture

> **Explanation.** Read when: you want to know how the notch fits together and why. Skip if: you need a file's location or a step to follow; see the [source map](../reference/source-map.md) or the how-to guides.

The notch is a small window pinned to the top of the screen. It shows one **page** at a time, either collapsed
(a slim pill) or open (while the pointer is over it). While it is collapsed, **activities** (a volume change, a
charger being plugged in, a notification) take over the pill for a moment or add a small badge to it. The code is split
so that adding or removing a page or an activity never touches the host.

The app is two processes: `notch.exe` (this document's subject, drawn with Direct2D, no UI framework) and the settings
window, `notch-settings.exe`, started from the gear in the open header or from the tray (see [Two processes](#two-processes)).

## The host

`host/` owns everything that is not specific to a page: the window and its message loop, hover, the springs that open
and close the pill, which page shows, input, the tray and the launcher of the settings window. It names no page. `window.rs` turns
Windows messages into calls on the `Notch`, and the `Notch` decides what to show and draws it.

`Notch` is one struct whose methods are split one job per file (`host/notch/`), and a job that keeps its own state
owns a small struct for it (`Springs`, `RowMotion`, `HoverState`, `HidingState`, `KeyboardState`, `ShownPresences`). The reason is
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

The two modes are independent. The media page, for example, is shown collapsed only while a lyric line plays, but stays up
when open while paused so playback can be resumed. When no page answers for a mode, the host fills the gap: the
collapsed pill shows the time, and the open notch shows its header alone. The host asks again after every event, so a
page changes what it shows by changing its own state.

The media page claims the collapsed pill only while a lyric line shows: its pill is the artwork, the dots (only while a dot shows), the line and the group. While music plays without a line, the page answers `Presence::Badge` for the animated wave and the idle row shows (`host/pill_row.rs`; the claimed row is `host/claimed_row.rs`). The page's `resting_width` is the artwork plus the line, between 240 and 520 DIP; the host adds the edges, the dots and the group.

### The open notch

Every open page sits inside the same frame, drawn by `chrome.rs`: 10 px of padding, a 16 px header, a 10 px gap, then
the page's content area, with 16 px of padding at the sides and 14 below. The content area is 86 px high unless the page asks for more (`DEFAULT_CONTENT_HEIGHT` and `MAX_CONTENT_HEIGHT` in `page.rs`; the shelf, calendar, clipboard and todo pages do); its width is the page's own. The
header holds the page dots on the left, the time in the middle, and at the right the same group as the idle pill, as icons only, then the settings gear at the right edge (laid out by `host/pill_row.rs`). A page draws in
`Frame::bounds`, which is the content area, and cannot draw into the header or the padding. The layout with its
numbers is in [the layout reference](../reference/layout.md#the-notch-layout).

The settings window's Pages section (drag a page to reorder it, switch one on or off; saved as
`navigation.order` and `navigation.disabled`) lists the pages. A page can be opt-in (`Page::is_opt_in`), which starts it switched off.

### Opening and closing

Pointing at the notch starts a timer; if the pointer stays, the notch opens in one move, and it closes in one move
after a short delay once the pointer leaves. One spring, `expansion`, goes from 0 (collapsed) to 1 (open); width and
height both interpolate from the collapsed to the open size through that one value, so they grow in parallel and arrive
together, and the fades and the time's glide follow it too (`HEADER_STAGE` is only the point where the header content
is fully shown and the page content has not started). Delays and fades are constants in the file that uses them
(`notch/hover.rs`, `notch/hiding.rs`, `notch/layout.rs`).

Every size spring (`expansion`, the collapsed and open width and height, the badge room) uses the one response
`SIZE_RESPONSE` in `host/motion.rs`: stiffness 215, damping 16.7, a damping ratio of 0.57 and an overshoot of about 10 %
of a move, for open, close, page switches and the media page's lyrics width. Every size spring also has a lead-in (`SIZE_LEAD_IN`, 90 ms): when a move starts at rest, the pull toward the
target grows along a smoothstep from nothing, so the move begins gently (an ease-in) and the bounce stays at the end;
a retarget while moving keeps the pull it had. Springs with the same response and lead-in settle in the same time
whatever the distance, so width and height start and stop together; the frame timer still stops once they settle. `frame()` clamps the overshooting size to the
window (`size_room`) and keeps it above 75 % of the collapsed size, and the pill's radius never exceeds half its height,
so the shape stays a pill while it over- or undershoots. The page dots' long mark has its own spring
(`PAGE_SLIDE_RESPONSE`), critically damped so it never passes the open page. All the tuning numbers of this, of the
page swap and of page switching input are in `host/motion.rs`.
While files are dragged over the notch it opens at once on the page that takes drops and grows a little, as if ready.

### Hiding

The notch can step out of the way in two ways, both in `notch/hiding.rs` (`is_hidden`, `is_suppressed`), with the rules in
`visibility.rs`:

- **Hide when idle** (a setting, over regular windows): with nothing to show it shrinks to a nearly invisible
  240 x 6 px strip that still receives the pointer, shows the small pill after the pointer rests on it for 150 ms, and
  opens if the pointer stays.
- **Over a maximized or full-screen window** one `Showing` each (`notch.maximized`, default `lyrics`;
  `notch.fullscreen`, default `hidden`) replaces Hide when idle. The words are listed in
  [settings keys](../reference/settings-keys.md#what-the-notch-shows-over-another-window). `visibility::allows` decides what a word shows, with
  `Page::shows_live_text` marking active lyrics. `hidden` hides the window and draws nothing, so no hover, banner or
  message shows until the window in front changes; a suppressed notch keeps its full-size layout so no strip grows back
  when it returns. Whatever a word leaves out hides to the strip, 3 px high with a 350 ms wait unless Hide when idle is on, so the tabs and title bar of the window
  are not caught. `visibility::migrate` writes the new keys from the retired `notch.hide_for_fullscreen`,
  `notch.show_over_maximized` and `notch.lyrics_over_maximized` at start and on a reload, never over a saved new key.
  A reload that changes a choice while such a window is in front applies at once: it hides or shows the window and
  re-lays out the pill.

`screen_mode.rs` decides which case applies. It hooks Windows' events for the window in front changing, and, for as
long as one window is in front, for the moves and resizes of that window's thread (`SetWinEventHook`, one thread at a
time, so mouse moves never wake the notch). The caret and the pointer report moves on that thread too, so when two
moves of something other than the window come less than 100 ms apart, `screen_mode/location_hook.rs` unhooks for 250 ms
with a one-shot timer and reads the mode again when the hook returns; no timer is armed while moves are rare or have
stopped. A window covering the whole primary monitor without a title bar is full
screen; a zoomed one is maximized; the desktop, the taskbar and the notch's own windows never count. A screenshot
overlay is `Capture`: any window of `ScreenClippingHost.exe` (Win+Shift+S, Print Screen), or a window of
`SnippingTool.exe` or `GameBar.exe` that covers the monitor. The process name is read once per foreground change
(`OpenProcess` and `QueryFullProcessImageNameW`) and kept for that window's moves. `Capture` always means `hidden` and no
hover opening, whatever the settings say.

Every foreground change reaches the host, even when the screen mode is unchanged, so a demoted Notch window can
regain topmost status without taking focus. Hover restores topmost before the drop-target overlay steps aside.
Session lock/unlock and automatic resume reset hover, capture, drag and reveal state, then restore the overlay's
placement. Session notifications use `WTSRegisterSessionNotification` for the current session; if registration fails,
the error is logged and foreground recovery remains available. Leave tracking is armed only while
`WindowFromPoint` identifies the Notch window, preventing repeated leave messages from an occluded window.

### Navigation

With two or more pages that answer `Some` for `Mode::Expanded`, the page dots in the header show where the user is.
The user moves between pages by scrolling the mouse wheel, swiping two fingers on a touchpad (sent as scrolling), or
clicking a dot. A wheel notch moves one page; touchpad and precision-wheel input accumulates until 120 units cross
the threshold, moves one page and is then ignored until the gesture ends (150 ms without an event). After any
switch, scroll input is ignored for 300 ms (`PAGE_SWITCH_COOLDOWN`), so one scroll never runs through the row. The
selected page is remembered as the logical selection.

A switch is a swap (`host/page_swap.rs`, a pure `PageSwap` the notch holds): it starts when the open
page changes while the pointer is on the notch, and it runs on a fixed time curve to its end, so no input can leave the
notch between two pages. The leaving page fades out first; the arriving page starts a moment later and finishes just
after the size springs of the pill are within 5 % of its size. A time curve is used instead of a spring because its
end is known, which lets the content finish right after the pill instead of trailing it on a spring's long tail. A
swap started during another hands the half-arrived page's look to the leaving one, so nothing jumps. Each page's frame
has its own width and height from the top of the content area, independent of the pill's size spring, so content moves
only sideways; the host clips it below the header. The swap counts as motion for the frame timer, which stops once the
swap ends and the springs settle.

Windows input-source metadata identifies touchpads; drivers without it fall back to small deltas and the active
gesture, so a large delta inside a gesture stays part of it. `host/scroll_gate.rs` holds that logic as a pure type
(`ScrollGate`). Scrolling loops: next from the last page goes to the first and previous from the first goes to the last, one page per
step; a dot click still jumps to any page, taking the shorter way round the row. The page left open is remembered as `navigation.page` in
the settings. `host/navigation.rs` holds the page rules as pure functions; `notch/pages.rs` and `notch/layout.rs` apply them.

### Sizes

A page reports `sizes()`: its collapsed width and its open size, in device-independent pixels. The host animates
the pill toward them with springs, so a page that changes its mind (the media page widens for lyrics) gets a smooth
transition for free. The host uses the requested width and height, with the height capped at `MAX_CONTENT_HEIGHT`;
content that exceeds it is clipped. The collapsed height is `RESTING_HEIGHT` (33) plus 3 px of bottom padding.

### Painting

The notch is a layered window with per-pixel alpha. Each frame the host draws a soft shadow and the black pill, then
the page's backdrop (the media page's lyric glow), `draw_resting` on the page chosen for the collapsed mode,
`draw_expanded` on the one chosen for the open mode, and the header, with opacities that cross-fade as the notch opens.
Only the rectangle around the pill is redrawn and shown, and the window shrinks and grows with the pill, so the screen
outside the pill is never covered and never blocks clicks.

The host's own drawing (the shadow, the pill, the page dots and the clock) and `Page::draw_backdrop` are in device
pixels. Every other page and activity draw hook runs inside a unit space that `host/notch/units.rs` opens, so pages and activities write sizes in
device-independent units starting at zero and do not multiply by a scale; widgets read `canvas.scale()`. The reasons, the three drawing layers (`gfx`,
`ui`, pages) and how cached images stay sharp are in [the drawing model](drawing-model.md).

### Input and timers

Windows messages reach `window.rs`, which calls the notch. The notch routes pointer events to the page chosen for the
open mode, and `host/notch/units.rs` converts each position to units from the top-left of that page's frame (or of the
banner's or badge's area), the same space the page draws in. A right click reaches only a page that uses it for its own content (the shelf's files); the notch has no menu of its own. The tray icon has the three items Start with Windows, Settings and Quit Notch
(`host/tray.rs`), and the gear in the open header starts the settings window the way the tray's Settings item does (`host/settings_launcher.rs`). A page's timers (`Context::set_timer`) are Windows timers whose ids encode the page and the
timer number, so each page has its own.

### Activities

The host asks every running activity for its `presence()` after each event.

- A **transient** (up to two, newest win) replaces the page's collapsed content: the pill resizes to the transient
  widths and a blend spring cross-fades the two.
- A **badge** joins the group at the right edge of the idle pill (icon and text; icon only for the wave, muted and Caps Lock), ordered by `header_status_rank`, between the clock and the battery ring. While a page claims the pill, the group (muted, Caps Lock, one countdown) follows the page's line with 24 px between them.
- A **banner** opens the pill into a card taller than the idle pill and smaller than the open notch. It has the pill to
  itself, transients wait behind it, and while it shows, pointing at the notch holds it instead of opening the notch.
  Pointer input on a collapsed notch goes to the banner (`Activity::on_pointer`, `Page::on_presence_pointer`, in
  the rect it draws into), never to the page that would open, so its buttons are real hit targets. Without a banner,
  a page's badge gets the pointer over its own rect (`Page::on_presence_pointer`, `host/notch/badge_pointer.rs`); the
  rain badge opens the weather page on a press.

Required activities always start and ignore saved disable settings. Only optional activities expose a saved
`activity.<id>` settings toggle; Notifications and the device battery warning are the optional built-ins, and both default to on. The host registers the
window for Windows power-setting and clipboard messages on their behalf (`Context::listen_to_power_setting`,
`listen_to_clipboard`) and hands them to every running activity, and to every page that is switched on (`Page::on_system_message`), as a `SystemMessage`. Activity timers use ids from
1000 up and their wake-ups use their own window message, so they never clash with a page's. See
[adding-an-activity.md](../how-to/add-an-activity.md).

### Status marks

The same marks form the group at the right edge of the collapsed pill and the icons at the right of the open header: the media page's wave, an activity's Caps Lock arrow or muted speaker, the timer, the next event and the next prayer. A page or an activity answers `header_status_width` and `draw_header_status` (icon only), and the host lays them out lowest `header_status_rank` first, nearest the time (wave 0, Caps Lock 1, muted 2, timer 3, event 4, prayer 5), pages before activities within a rank. The battery ring and the camera and microphone dots follow them at the fixed right end and have no rank. `header_status_page` names the page a click on the icon opens (`host/notch/header_row.rs`).

### The rows and their motion

The idle row and the open header are one group in one order, right-anchored, and the pure math for both is in `host/pill_row.rs` (and `host/claimed_row.rs` for the lyric pill). The host keeps one spring per item and per clock (`host/notch/row_motion.rs`), retargeted whenever an activity event changes the group, so an item that joins or leaves, or a dot that appears, slides its neighbours instead of snapping. Each item has its own spring so that the ones that stay can move while another arrives. The springs join the frame timer and settle with the others, so the row adds no timer of its own. The battery ring is shown on a machine with a battery only (`platform/power.rs`), and nothing takes its place on a PC.

### Threads

All drawing and page code runs on the single UI thread. Work that waits runs on threads of its own, which wake the UI
thread with a `Notifier` that posts a message; the page or activity then reads the new state in `on_notify`:

- the media page watches the Windows media session and looks up lyrics
- the notifications activity looks for the new toast by its id when the notification folder changes (tens of milliseconds), and reads the whole list (seconds) only when that finds nothing
- the shelf asks Explorer for file pictures and carries a file being dragged out (that needs a single-threaded
  apartment, which the UI thread is not); `drop_target.rs` has the thread that takes dragged files in
- the device battery activity runs a WinRT `DeviceWatcher` that Windows pushes changes into
- the two hide-Windows-popups options each run a thread of their own: a show hook on the volume flyout's thread that finds the flyout window and empties its region (`host/volume_flyout/hook_thread.rs`) and a wait on registry changes (`host/notification_banners.rs`)
- the saved city follows the device (`host/location_tracker.rs`): a callback on network interface changes, registered only while `location.track` is on, restarts a one-shot 10 s timer (host timer 11); when it fires, or at start and after sleep, a worker thread asks Windows for the position once and ends with its answer. A continuous location subscription is avoided because Windows then shows its "location in use" icon for as long as the notch runs
- the Caps Lock activity runs a keyboard hook, which only wakes the activity

### Animation

While anything moves the host runs a frame timer at the display's refresh rate; when the host's springs have
settled and no page reports `animate` as still moving, the timer stops. A collapsed notch with nothing happening
has no timers running except the host's own clock (once a minute) and a page's own, which is why it uses no CPU. The calendar page is the one page that has work while it is not shown: its reminder badge needs fresh events, so it keeps one timer for the next reminder window (every minute while shown or while a reminder counts down) and downloads again at its refresh interval (`pages/calendar/wake.rs`).

## Two processes

`crates/settings/` (GPUI) and `crates/notch/` depend on `crates/store/` and never on each other, so the notch links no UI framework. The notch
publishes `pages.known` when it starts so the window can list pages it does not link. The window writes keys through
`crates/store/` and tells the notch to re-read them; the message, keys and field behavior are in
[settings keys](../reference/settings-keys.md#settings-file) and the [settings window](../reference/settings-window.md). The idle budget applies to `notch.exe` only
([performance baseline](../reference/performance-baseline.md#settings-window)).

## Text in two languages

Both programs look texts up with `tr!("key")` (rust-i18n 3, compiled in from one `locales/app.yml` per crate, English and Indonesian side by side). The language is `general.language` resolved by `store::language` (`auto` reads the Windows display language); `store::language::current()` is what `tr!` reads, with a per-thread override that tests use so a test in Indonesian never changes the text under another. The notch re-reads the key on `WM_SETTINGS_CHANGED` and calls `Page::on_language_changed` and `Activity::on_language_changed` so cached text is rebuilt; the settings window builds its pages again when the language differs from the one they were built in. A key found in the table comes back borrowed (no allocation); a text with a value in it is built when it is made, not on every paint. A test per crate checks that every `tr!` key exists in both languages and that no key is unused; fit tests measure the Indonesian text of each fixed-size place with the real fonts.

| File in `crates/settings/src/` | Job |
| --- | --- |
| `main.rs` | the window: the frame, the brand strip, the sidebar and the content card, in the system's light or dark theme |
| `palette.rs` | the dark and light colors of the frame, card, inputs and text, with contrast tests |
| `sidebar.rs` | the sidebar: the "Notch" and "Page settings" groups, each item with an icon, and About pinned at the bottom |
| `icons.rs` | the sidebar's SVG icons (`assets/icons`) and the asset source that serves them next to the component library's |
| `header.rs` | the card's 36 px header, drawn by the window: the breadcrumb "Settings › <Page>" and the window buttons, with corners that follow the card's |
| `layout.rs` | `Page`, `Group` and `Item`: the page title and description, each section as one bordered block under its title, and rows (title and description left, control right, dividers) |
| `sections/` | one file per section, each returning a `Page`; `mod.rs` lists them and writes every key through `apply`; `about/` holds the About page (it writes no key) |
| `field.rs` | the switch and the text and number fields bound to a key, and their save rules |
| `notify.rs` | posts `WM_SETTINGS_CHANGED` to the notch |
| `single_instance.rs` | allows one window and focuses the open one |
| `startup.rs` | the `Run` registry value behind **Start with Windows** |

To add a setting, see [add a setting](../how-to/add-a-setting.md).

## Pages

`pages/mod.rs` is the registry. See [add a page](../how-to/add-a-page.md) to write one. A page may also answer
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
- Why the notch draws by hand, in units: [drawing model](drawing-model.md).
