# The notch layout

> **Reference.** Read when: you change padding, the header, or the open and collapsed shapes. Skip if: you only add a page; the host sizes it for you.

The host owns the shape, the padding and the header. A page draws only the content area inside them, and all sizes
are in device-independent pixels (multiply by the display scale when drawing). The numbers are constants in
`crates/notch/src/host/chrome.rs`; the time is drawn by `crates/notch/src/host/clock.rs`.

**Open (pointer over the notch)**, here a page whose content is 300 px wide (the height is always 86):

```
      flare                                                flare
         \                                                  /
          \________________________________________________/     top edge of the screen
          |                                                |
          |  10 px padding top                             |
          |  [== o o o]               (o) |||  18:41       |     header, 16 px high
          |   page dots (left)     status marks  time      |     marks and time sit at the right edge
          |  10 px gap                                     |
          |  +------------------------------------------+  |
          |  |                                          |  |
    16 px |  |  THE PAGE: only this area is its own     |  | 16 px
          |  |  width  = PageSizes.expanded.width       |  |
          |  |  height = 86 (chrome::CONTENT_HEIGHT)    |  |
          |  |  drawn in Frame::bounds                  |  |
          |  +------------------------------------------+  |
          |  14 px padding bottom                          |
          \________________________________________________/

    pill width  = page width + 32     (16 + 16)
    pill height = 86 + 50 = 136       (10 + 16 + 10 + 86 + 14)
```

- The header belongs to the host, on every page: the page dots (the long one is the open page) on the left, and at the
  right the status marks (the music wave, the timer ring, muted speakers, Caps Lock) and the time. The header has no settings button; Settings is in the right-click menu.
- Every page has the same 86 px of content height, whatever it asks for. Content that is taller is clipped, so lay a
  page out for 86 px (the stats tiles and the calendar do).
- A page cannot draw into the header or the padding.

**Collapsed**, the pill is 33 px high (`RESTING_HEIGHT`, plus 3 px of bottom padding) and at least 150 px wide
(`RESTING_MIN_WIDTH`), and one of four things fills it:

```
  nothing happening     music playing        activity or timer      notification banner
     _________        ____________________    __________________    ____________________
    /  18:41  \      / (page draws it) [o] \  / (activity draws) \  | [D] Rafi - Discord  |
    \_________/      \____________________/  \__________________/  |  Deploy is green... |
    host: the time    the media page, plus    a transient takes      \____________________/
    in a 150 px pill  an optional badge       the pill for a moment  an activity opens the pill
                      at the right            (volume, Caps Lock)    into a card, 48 to 70 px high (400 px wide)
```

- A page answers `priority(Mode::Resting)` to claim the collapsed pill; with no answer the host shows the time.
- An activity or a page can add a **badge** at the right end (the page draws to the left of it), a **transient** that
  replaces the pill's content for a moment, or, for an activity, a **banner** that opens the pill into a card
  (`presence()`; see [adding-an-activity.md](../how-to/add-an-activity.md)).
- The time is one element: it glides from the middle of the collapsed pill to the right end of the header as the notch
  opens, and stays visible the whole way.
- By default, over a maximized window the pill hides to a strip unless lyrics play, and over a full-screen window the
  notch is off the screen; see
  [architecture.md](../explanation/architecture.md#hiding).

**Where to change what:**

| To change | Edit |
| --- | --- |
| padding, header height, the gap, the dots, the content height | `crates/notch/src/host/chrome.rs` |
| the time and the collapsed pill's minimum width | `crates/notch/src/host/clock.rs` |
| which page shows, swiping, hover, the springs, hiding | `crates/notch/src/host/notch/` (`animation.rs`, `hover.rs`, `hiding.rs`, `layout.rs`) |
| maximized and full-screen detection | `crates/notch/src/host/screen_mode.rs` |
| the tray menu | `crates/notch/src/host/tray.rs` (the items), `crates/notch/src/host/window.rs` (what they do) |
| the settings window | `crates/settings/src/sections/` (one file per section), `crates/settings/src/field.rs` (toggles and text fields); [adding-a-setting.md](../how-to/add-a-setting.md) |
| start with Windows | `crates/notch/src/host/startup.rs` (the tray item), `crates/settings/src/startup.rs` (the window's switch), `installer/notch.iss` (the installer's choice) |
| a page's content and size | that page's folder in `crates/notch/src/pages/` |
| register a page | `crates/notch/src/pages/mod.rs` |
| register an activity | `crates/notch/src/activities/mod.rs` |
