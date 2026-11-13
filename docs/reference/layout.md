# The notch layout

> **Reference.** Read when: you change padding, the header, or the open and collapsed shapes. Skip if: you only add a page; the host sizes it for you.

The host owns the shape, the padding and the header. A page draws only the content area inside them, and all sizes
are in device-independent pixels. A page draws in these units from zero and does not multiply by the display scale itself;
the host converts them (`crates/notch/src/host/notch/units.rs`, see [the drawing model](../explanation/drawing-model.md)). The numbers are constants in
`crates/notch/src/host/chrome.rs`; the time is drawn by `crates/notch/src/host/clock.rs`.

**Open (pointer over the notch)**, here a page whose content is 300 px wide (the height is always 86):

```
      flare                                                flare
         \                                                  /
          \________________________________________________/     top edge of the screen
          |                                                |
          |  10 px padding top                             |
          |  [==ooo]    18:41    [~][t][O][.] [pin][gear] |     header, 16 px high
          |  page dots  clock    group (icons), pin, gear  |     page dots left, clock in the middle, group, pin and gear at the right edge
          |  10 px gap                                     |
          |  +------------------------------------------+  |
          |  |                                          |  |
    16 px |  |  THE PAGE: only this area is its own     |  | 16 px
          |  |  width  = PageSizes.expanded.width       |  |
          |  |  height = 86 (DEFAULT_CONTENT_HEIGHT)    |  |
          |  |  drawn in Frame::bounds                  |  |
          |  +------------------------------------------+  |
          |  14 px padding bottom                          |
          \________________________________________________/

    pill width  = page width + 32     (16 + 16)
    pill height = 86 + 50 = 136       (10 + 16 + 10 + 86 + 14)
```

- The header belongs to the host, on every page: the page dots (the long one is the open page) at the left, the clock in
  the middle, and at the right the group as icons only (the same items and order as the idle pill), the pin and the gear at the
  right edge.
  A click on a status icon opens its page, a click on the pin pins the notch open and a click on the gear opens the settings window; both are described under
  [the open header](#the-open-header).
- A page content is 86 px high unless the page asks for more (`PageSizes::expanded.height`, at most 220 px, `MAX_CONTENT_HEIGHT` in `page.rs`: the shelf and
  the calendar do). Content that is taller than what the page asked for is clipped.
- A page cannot draw into the header or the padding. Its frame always has its own requested size, from the top of the
  content area, even while the pill is still resizing; the host clips page content to the pill below the header
  (`chrome::page_clip_rect`), so a page entering a pill that is still growing is cut at the pill's bottom edge and never
  covers the header. During a page switch the frames shift sideways by a few pixels (see the swap rule in
  [design rules](design-rules.md)).

**Collapsed**, the pill is 33 px high (`RESTING_HEIGHT`, plus 3 px of bottom padding) and at least 150 px wide
(`RESTING_MIN_WIDTH`), except the empty idle pill below, and one of four things fills it:

```
  nothing claims the pill   lyrics on                transient              notification banner
  (idle row)                (media claims the pill)  (volume, Caps Lock msg)
   ____________________      ____________________     __________________    ____________________
  / 18:41  [12m][ring][.] \  / [A] lyric line [t] . \  / (activity draws) \  | [D] Rafi - Discord  |
  \____________________/    \_____________________/  \__________________/  |  Deploy is green... |
  clock, group              artwork, line, group,    replaces the content   \____________________/
                            dots
```

The idle row, left to right (`idle_row` in `crates/notch/src/host/pill_row.rs`):

```
  12   clock   24 or more   group, right-anchored, 8 between items (4 ring to dots)  12
  edge 14 pt                [wave][Caps][muted] [timer][event][prayer] [ring][dots] edge
```

- **Clock:** at the left edge, 12 px from it, 14 pt Medium Geist (`clock.rs`).
- **Group:** one run anchored to the right edge, left to right: the badges by `header_status_rank` (lowest first: music
  wave 0, Caps Lock 1, muted 2, timer 3, event 4, prayer 5, rain 6), then the battery ring, then the dot column. The ring and
  the dots are not ranked; they are always the right end. A new item joins between the clock and the items already
  there, so nothing right of it moves (`pill_row.rs`). The rank list is in [design-rules.md](design-rules.md#collapsed-pill).
- **Badges:** timer, event and prayer draw an icon and text; the wave, muted and Caps Lock draw an icon only. With two or
  more badges that have a short form (`Page::short_presence_width`), each takes it ("12m", "Asr 24m"); alone, a badge
  draws its longer form.
- **Battery ring:** 14 px, laptop only; a PC has no ring, no weather icon and no other item in its place
  (`activities/battery.rs`).
- **Dot column:** 6 px wide, camera over microphone; a microphone alone is a 5 px dot (see [activities.md](activities.md)). It reserves no width: with no dot
  showing the rest of the group fills its place, and when a dot appears everything left of it moves by its width and
  its gap (4 px after the battery ring, 8 px after anything else).
- **Gaps:** 8 px between two items of the group, 4 px from the battery ring to the dot column right after it
  (`RING_DOTS_GAP`), at least 24 px between the clock and the first item, 12 px from the
  pill's edge to the clock and to the last item. The pill is as wide as its row, at least 150 px; a wider minimum leaves
  the extra room between the clock and the group.
- **The empty pill:** with no item at all (a PC with nothing active) the clock is centered and the pill hugs it with
  `EMPTY_SIDE` (28 px) on each side, narrower than the 150 px minimum. When anything appears the clock glides to the left
  edge and the pill grows.
- **Every item has its own spring** (`host/notch/row_motion.rs`: one for the idle clock, one for the header clock, one
  per item of the pill's row and of the header's group). An item joining or leaving, or a dot appearing, slides its
  neighbours and never snaps; a new item starts at its place and only the items already shown move. The springs settle,
  so no timer keeps running.
- **A page claims the pill** (`priority(Mode::Resting)`): the media page does it only while a lyric line shows. With a
  lead (`Page::resting_lead_width() > 0`, the media page's 20 px artwork) the row reads
  (`host/claimed_row.rs`):

  ```
  12   lead   24   page content   24   group                             8   dots   12
  edge artwork     lyric line          muted, Caps Lock, one countdown        6 px   edge
  ```

  The group and its 24 px gap exist only with a group; the dots and their 8 px gap only while a dot shows, and they
  take no width otherwise. The dot column ends the row 12 px from the edge, where it ends the idle row; when a dot comes
  or goes the pill widens or narrows and the group slides by the dot and 8 px. `Frame::lead_gap` is the room the page
  keeps free after its lead: 24 px (moved on the `Springs::claim_lead_gap` spring when a page starts or stops
  claiming). A page's
  `sizes().resting_width` is the lead plus the content, without the lead gap, the group, the dots and the pill's edges;
  the host adds them. The line is trimmed so both 24 px gaps hold. The
  group is muted, then Caps Lock, then at most one countdown (the one with the least time left,
  `Page::countdown_seconds`), each in its long form (icon and text). There is no clock, no battery ring and no music
  wave; the artwork and the line already say that music plays. A claiming page without a lead keeps its content at the
  left edge; its right end is the same.
- **The dot column** is the right end of the idle row and of a lyric pill, 12 px from the edge in both, so when the pill
  changes between the two it only follows the pill's right edge.
- With no page claiming, the host shows the idle row.
- An activity or a page can add a **badge** to the group, a **transient** that replaces the pill's content for a moment,
  or, for an activity, a **banner** that opens the pill into a card, or a **companion** (the battery ring and the
  camera and microphone dots; `presence()`, see [add-an-activity.md](../how-to/add-an-activity.md)).
  Only activities place companions. The music wave is a `Presence::Badge` from the media page; the host treats a badge
  of a page with `resting_lead_width() > 0` as the wave.
- The time is one element: it glides from its place in the collapsed row to the middle of the header as the notch opens,
  and stays visible the whole way. The ring and the dot column travel with it to their places in the header's group.

### The open header

```
  page dots   12+   clock (exact middle)   14+   group, icons only, 10 apart (4 ring to dots)         6    pin    4    gear
  at the left                                    [wave][Caps][muted] [timer][event][prayer] [ring][dots]    13 px        16 px
```

(`header_row` in `crates/notch/src/host/pill_row.rs`; the items and their clicks are in `host/notch/header_row.rs`.)

- **Left:** the page dots start at the header's left edge.
- **Right:** the gear (16 px box, its hexagon a little narrower than the 14 px ring) is the last item, at the header's right
  edge, and the pin (a 13 px box, its pin outline showing about 9 x 12 px) sits 4 px before it. The group ends 6 px before the pin (about 8 px of air to the glyph), in the idle row's order, 10 px between items and 4 px from the battery ring to
  the dot column: the icons by rank (the activities' and pages' header status marks,
  `header_status_width` and `draw_header_status`; icons only, never text), then the battery ring, then the dot column.
  The dot column reserves no width here either. Every group item's header place is the pin's and the gear's widths and the 6 and 4 px gaps left of
  the edge, so the group slides there as the notch opens; the pin and the gear have their own springs in the header row.
- **Middle:** the clock is in the exact middle of the header and never moves. An icon that would come nearer than 40 px
  to it is not shown: the icons are dropped from the clock side (lowest rank first) until the rest fit
  (`HeaderRow::dropped_icons`). The battery ring, the dot column, the pin and the gear are never dropped.
- **Width floor:** the open notch is never narrower than the timer page's open width: its 290 px plus the 32 px of side
  padding, 322 px (`open_min_width` in `host/notch/layout.rs`). When the header needs more, that is the floor instead:
  the clock plus twice the wider of its two sides (the page dots and 40 px, or 40 px, the ring, the dot column, the
  pin and the gear with their gaps) plus the side padding (`header_min_width`; 341 px with nine pages). With the timer page
  switched off only the header counts. A page narrower than the floor opens at it; a wider one (the 507 px shelf at
  539) sets the width itself.
- **Clicks** land in a 24 px square around each shown icon, without moving the gaps; where two squares meet, the nearer icon
  wins (`hit_item`). A click on the timer, event, prayer or wave icon opens its page (`header_status_page`: Timer,
  Calendar, Shalat, Media). Muted, Caps Lock, the ring and the dots take the click and do nothing. A click that lands on
  an item never reaches the page below it.
- **The gear** is drawn as a hexagon around a ring (`Icon::Gear`), `#8c8c94`, and shows only in the open notch, never in the idle
  pill. It is not an activity and has no rank. A click on it starts the settings window (`notch-settings.exe`) through
  `host/settings_launcher.rs`, the launcher the tray's **Settings** item uses.
- **The pin** is drawn with `Icon::Pin`, `#8c8c94` while unpinned and `#74c0fc` while pinned (`pin_color` in
  `host/notch/header_row.rs`). A click toggles it (`Notch::toggle_pin`, `host/notch/pin.rs`). A pinned notch stays open
  with the pointer away and is never hidden by the words below or by Hide when idle; only a screenshot overlay hides it.
- **Hover:** the pointer over the pin or the gear draws a 24 px wash, the item's hit square, behind it
  (`HeaderMarks::hovered`, set on pointer moves and cleared when the pointer leaves or the notch closes; one repaint per
  change, no timer). The layout and the hit areas stay as they are.
- By default, over a maximized window the pill hides to a strip unless lyrics play, and over a full-screen window the
  notch is off the screen; see
  [host and pages](../explanation/host-and-pages.md#hiding). Hiding is immediate, but the notch comes back only after the
  window in front has stopped covering the screen for 500 ms (`WINDOW_REVEAL_DELAY_MILLISECONDS` in
  `host/notch/hiding.rs`), so switching between two maximized windows does not flash it. A screenshot overlay in front
  (`ScreenMode::Capture`) hides the notch entirely, banners included, and the pointer does not open it, whatever the
  settings say; it comes back through the same 500 ms wait. In every mode only a transient or a banner counts as an activity: other badges and dots (Caps Lock, a countdown, the camera and microphone dots) never keep a hiding pill up, so the privacy dot is not visible while the notch is hidden. The muted speaker is no exception; only a volume, brightness or mute message shows the pill over a word while `notch.hide_windows_volume_popup` is on ([priority activities](settings-keys.md#what-the-notch-shows-over-another-window)). Music (a claiming page or the music wave) keeps the pill up under Hide when idle (`Notch::is_hidden` in `host/notch/hiding.rs`).

### Where the notch sits on several monitors

`notch.monitor` ([settings keys](settings-keys.md)) picks the monitor: `main` is the primary one, `focused` is the one
with the window in front, `chosen` is the one whose identifier is stored in `notch.monitor_id` (the main screen while that
screen is not connected; it moves back when the screen returns, on the next display change). The window is centered on that monitor, its top is `notch.top` below the top edge of that
monitor (not of the virtual desktop), and its size and every pixel value follow that monitor's scale
(`placement` in `host/monitor/choose.rs`).

- **One move.** `Notch::relocate_if_needed` (`host/notch/relocate.rs`) makes the scale change in one message: it builds
  the fonts and resizes the window's bitmap (`Surface::resize`, which keeps uploaded images valid), swaps them in, tells
  every page and started activity to rebuild cached text (`on_language_changed`), moves the window with `SetWindowPos` and
  tells the drop overlay. A failure keeps the old scale whole. Windows' `WM_DPICHANGED` suggested rectangle is never applied.
- **When it runs:** at start, after `notch.monitor` changes, on a display or scale change (`WM_DISPLAYCHANGE`, `WM_DPICHANGED`), after unlock or resume from sleep, and
  in `focused` mode on every change of the window in front. The foreground window's monitor is read only then; a
  hidden, minimized or desktop window, a taskbar and the notch itself leave the notch where it is. When the monitor it
  sits on is unplugged it moves to a remaining one.
- **Open or pinned:** a move to another monitor waits until the notch is closed and not pinned, then reads the monitors
  again (`release_waiting_move`). A new resolution or scale on the monitor the notch is already on applies at once.
- **Screen mode:** maximized and full-screen are judged against the monitor the notch is on, so a full-screen window on
  another monitor does not hide it (`host/screen_mode.rs`).
- **One monitor:** a change of resolution or scale re-centers the notch.

Limits are in [the README](../../README.md#limitations).

**Where to change what:**

| To change | Edit |
| --- | --- |
| padding, header height, the gap, the dots, the content height | `crates/notch/src/host/chrome.rs` |
| the time (14 pt Medium) and the collapsed pill's minimum width | `crates/notch/src/host/clock.rs` |
| the idle row's and the open header's positions and gaps (pure math) | `crates/notch/src/host/pill_row.rs` |
| the claimed pill's row (lead, content, group, dot column; pure math) | `crates/notch/src/host/claimed_row.rs` |
| the idle row's state and drawing: the group, the ring, the dot column | `crates/notch/src/host/notch/resting_row.rs` |
| the springs of every row item and of the clocks | `crates/notch/src/host/notch/row_motion.rs` |
| the open header's items: the pin, the gear, the icons and what a click on them does | `crates/notch/src/host/notch/header_row.rs` |
| which page shows, swiping, opening and closing, the pin, the springs, hiding | `crates/notch/src/host/notch/` (`animation.rs`, `open_state.rs`, `pin.rs`, `hiding.rs`, `layout.rs`) |
| the monitor the notch sits on and the placement there | `crates/notch/src/host/monitor.rs`, `crates/notch/src/host/monitor/choose.rs` (the rules), `crates/notch/src/host/notch/relocate.rs` (the move) |
| maximized, full-screen and screenshot-overlay detection | `crates/notch/src/host/screen_mode.rs` (the hooks), `crates/notch/src/host/screen_mode/classify.rs` (the rules) |
| the tray menu | `crates/notch/src/host/tray.rs` (the items), `crates/notch/src/host/window.rs` (what they do) |
| the settings window | `crates/settings/src/sections/` (one file per section), `crates/settings/src/field.rs` (toggles and text fields); [add a setting](../how-to/add-a-setting.md) |
| start with Windows | `crates/settings/src/startup.rs` (the window's switch), `installer/notch.iss` (the installer's choice) |
| a page's content and size | that page's folder in `crates/notch/src/pages/` |
| register a page | `crates/notch/src/pages/mod.rs` |
| register an activity | `crates/notch/src/activities/mod.rs` |
