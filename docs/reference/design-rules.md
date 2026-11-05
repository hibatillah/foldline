# Design rules

> **Reference.** Read when: you draw or change anything the user sees, or must decide where something goes, how far
> apart, in which color or font. Skip if: the change has no visual effect.

The "Notch Design" canvas (artifact `6c345878-8861-46d2-bcfd-d3b82723bf1b`, page "board") shows these rules; the canvas
leads and the code follows. Sizes are in device-independent pixels; shapes and padding are in [layout.md](layout.md). A
new visual decision is added here with its reason, in the same change.

## Collapsed pill

Left to right: battery ring, clock, static activities, countdown activities. The pill is as wide as its content, never
narrower than ring and clock (150). The order is fixed and never re-sorts by time left.

- **Persistence rule:** within a group, the longest-lived item sits leftmost, next to the clock, and the most volatile
  at the outer end, so an item that appears or leaves moves only the ones outside it. Frequent and short-lived means
  volatile.
- **Static group, leftmost: music.** Its moving wave stays away from the ticking countdowns. Muted is on-demand, and Caps
  Lock toggles within seconds, so both follow, Caps Lock last.
- **Countdown group, leftmost: timer.** It lasts the whole session. A prayer shows only in its reminder window, and
  events are more frequent with a shorter window, so the event is outermost.
- **Countdowns after statics:** they change every minute, so they sit at the edge.
- **Music shows only its wave**, alone or with others; the wave already says something plays, and the title is one
  hover away.
- **Several at once:** event and prayer shrink to a short countdown ("12m", "Asr 24m"); alone, each shows its longer form.
- **Camera and microphone** are 6 px dots in one column 6 px right of the battery ring: camera on top, microphone below.
  A dot alone keeps its slot, since both dots share a shape and position carries half the meaning.

| Gap between | px |
| --- | --- |
| ring, clock, activity group | 24 |
| two activities | 8 |
| an activity's icon and its text | 4 |
| pill edge and first or last item | 12 |

## Lyrics on

- The artwork (20 px) replaces the battery ring and the current line replaces the clock, at the width the line needs,
  trimmed at 280 px, or 200 px when activities share the pill. The time stays in the header when the notch opens.
- Camera and microphone dots stay right of the artwork.
- Static and countdown activities follow the line in their usual order; music's wave steps aside, since the artwork and
  the line already say it.

## Short messages and notifications

- A short message (volume, Caps Lock, prayer starts) replaces the pill for a moment and follows no order.
- A message that reports a percentage (Bluetooth device, charging, on battery, low battery) shows a 32 px ring around
  its icon, like the battery; that pill grows to 48 px.
- A message's value takes its icon's color (the percentage, "25 min", "15:19"); the name stays white.
- A notification opens the pill into a 400 px card: app icon, sender, two lines of the message. The two lines are
  clamped with an ellipsis, hovered or not; hover adds buttons and never grows the text, so the buttons do not move
  under the pointer.
- The heading is the sender in white, then the app name in grey (`#8e8e99`, regular weight) with no separator, so a
  missing sender cannot leave a dangling mark. A sender that is empty or only whitespace counts as missing and the
  app name takes its place, in white.
- Hover shows two buttons: Dismiss (secondary: translucent white fill, white text) and Open (primary: white fill,
  black text). A click on the card does nothing, so it never opens an app by accident.
- Open focuses the sending app and removes the notification, as a click on the native banner does; Dismiss only
  removes it. With the text hidden (privacy mode) Open still names the app.

## Color and type

- An activity's icon and text share one color: timer `#ff9e0a`, prayer `#63e6be`, event `#4dabf7`. Static icons are white.
  Mixed colors read as two things.
- The idle battery ring is `#2f9e44`, darker than the prayer mint so the two never read alike; charging is `#30d158`,
  low battery `#ff453a`.
- Camera dot `#30d158`, microphone dot `#ff9e0a`.
- The collapsed pill is all Geist, clock and countdown numbers included; it is a label, not a readout.
- Geist Mono is for numbers in page content, where digits align or tick in large type. The header clock stays Geist.

## Open

- The reminder windows are fixed: 30 min for the prayer (`BADGE_MINUTES`), 15 min for an event
  (`REMINDER_LEAD_MINUTES`). A per-page number setting, default 10, is proposed and not decided.
- Between lyric lines (intro, instrumental) the slot's content is not decided; a "♪" placeholder that keeps lyric mode
  for the whole song is proposed.

- The idle pill's clock position and the case of a PC without a battery are under review on the "raw" page (board
  "Idle layout"); the order above stays until that is decided. The code has no idle battery ring that I found: the
  battery activity only reports plugging, unplugging and low charge.

## Code gaps

- The event countdown is drawn in Consolas (`BADGE_TIME`, `crates/notch/src/pages/calendar/mod.rs`); the rules say Geist.
- Camera and microphone are drawn as one dot badge at the pill's right end (`Presence::Badge` in
  `crates/notch/src/activities/privacy.rs`); the rules put two stacked dots beside the battery ring.
- The notification card shows the whole message on hover (`Words::shown` in `crates/notch/src/activities/notifications/mod.rs`) and
  has no buttons or click handling.
- The notification heading is built as "sender · app" (`heading` in `crates/notch/src/activities/notifications/items.rs`) and tests
  only an empty sender, so a sender of only whitespace likely shows "· Discord" (not reproduced); the rules drop
  the separator and style the app name grey.
- The battery ring, the percentage rings and the lyric layout have not been checked against the code.
