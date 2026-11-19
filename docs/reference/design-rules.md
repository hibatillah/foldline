# Design rules

> **Reference.** Read when: you draw or change anything the user sees, or must decide where something goes, how far
> apart, in which color or font. Skip if: the change has no visual effect.

The "Foldline Design" canvas (artifact `6c345878-8861-46d2-bcfd-d3b82723bf1b`, page "board") shows these rules; the canvas
leads and the code follows. Sizes are in device-independent pixels; shapes and padding are in [layout.md](layout.md). A
new visual decision is added here with its reason, in the same change.

## Collapsed pill

Left to right: clock, group. The clock is at the left edge; everything else is one group anchored to the right edge:
the marks of what is going on, then the battery ring, then the camera and microphone dots. Positions and gaps are in
[layout.md](layout.md).

- **The clock is at the left edge** whenever the group has anything in it, 14 pt Medium. It is the only thing the idle pill
  has to say, so it is set larger than a mark.
- **The group is right-anchored and its order is fixed, left to right:** music wave, Caps Lock, muted, timer, event, prayer,
  battery ring, dots. The pill and the open header use one order, `header_status_rank`: wave 0, Caps Lock 1, muted 2,
  timer 3, event 4, prayer 5, rain 6, browser downloads 7, with the lowest rank nearest the clock. The ring and the dots are not ranked; they stay at
  the right end. A fixed order puts each mark where the eye looks for it every time.
- **A new item appears between the clock and the ring, and nothing right of it moves.** The right end stays still, so the
  marks the user has already read do not shift.
- **Persistence rule:** among the ranked marks, the longest-lived sits nearest the clock and the most volatile nearest the
  ring, so a mark that appears or leaves moves only the ones outside it. Frequent and short-lived means volatile.
- **The dot column reserves no width.** With no dot showing the group fills its place; when a dot appears everything left
  of it slides by the dot and its gap. A reserved empty column reads as a hole in the pill.
- **The dots sit 4 px after the battery ring**, closer than the 8 px between other items, so the ring and the dots read
  as one cluster of device state at the right end. Dots without the ring before them keep the usual gap.
- **Everything slides, nothing snaps.** Every item has its own spring (`host/notch/row_motion.rs`), so an item that joins
  or leaves, or a dot that flickers, moves its neighbours visibly and the user can tell what changed. The springs settle
  and stop, so the pill costs no CPU at rest.
- **A window that comes back for a priority activity shows at its final size without animating.** Volume or brightness
  over a window that hides the notch brings the window back already at the level's size (`refresh_layout` in
  `host/notch/layout.rs` snaps the layout springs), so the user sees the level at once instead of the old pill morphing.
- **A page switch is a quick swap, never a slide of two pages.** The leaving page fades out in about 110 ms while it
  drifts 12 px away; the arriving one starts 80 ms in, fades in and shifts 24 px into place from the side it comes from
  (east for the next page), and is still about 420 ms after the switch (`host/page_swap.rs`, numbers in
  `host/motion.rs`). The two pages are almost never visible together, and a switch always runs to its end, so the open
  notch never rests between pages. The content finishes just after the pill's width and height are within 5 % of the
  new page's size, so the pill has its new shape before the page is still. Content moves only sideways, keeps its own
  height whatever the pill's height is, and is clipped below the header.
- **The empty pill centers the clock.** A PC with nothing active shows only the time, centered, in a pill that hugs it
  with 28 px on each side (narrower than the 150 px minimum, which holds everywhere else). The clock glides to the left
  edge and the pill grows when anything appears.
- **Idle shows icon and text for timer, event and prayer** ("12m", "Asr 24m"); the wave, muted and Caps Lock are icon only.
  Two or more texts at once use the short form; alone, each shows its longer form.
- **A badge reports exactly its content width** (icon, 4 px, text; the icon alone for the wave, muted and Caps Lock), with no padding before or after, so the 8 px between two items is the visible gap. A message or banner keeps its own padding.
- **Music while idle is only the wave**, as a group item: no artwork and no title. Playing music keeps the pill up under
  Hide when idle.
- **Muted and Caps Lock are icon-only group items** while their state is on, in the pill and in the open header. Their
  message (icon and text) still takes over the pill when it fires; the icon returns when the message ends.
- **Battery ring:** 14 px, laptop only. The arc is the charge. A PC shows nothing in its place, because a ring that never
  changes says nothing; there is no weather icon, prayer or day ring, and no other stand-in. While the battery charges,
  a solid green bolt (`Icon::Bolt`, `SUCCESS_GREEN`, the ring's opacity) sits in the middle of the ring, half its diameter
  tall (`Ring::draw_bolt`). On battery, or while the source is unknown, the ring is empty inside. The collapsed ring and
  the open header's ring are the same activity drawing.
- **Camera and microphone** are 6 px dots in one column, camera on top, microphone below, 3 px apart. A dot alone keeps its
  row, since both dots share a shape and position carries half the meaning. The microphone alone is the exception: a 5 px
  dot in a 15 px column (2 px padding above and below, no gap, the 6 px camera slot kept empty above it), 0.5 px left of
  the column's middle as the design draws it. The camera alone keeps its 6 px dot in the same 15 px column.

| Gap between | px |
| --- | --- |
| two items of the group | 8 |
| the battery ring and the dots after it | 4 |
| the clock and the first item of the group (at least) | 24 |
| an activity's icon and its text | 4 |
| pill edge and the clock or the last item | 12 |
| the empty pill's edge and its clock | 28 |

## Open header

- Left to right: the page dots at the left edge, the clock in the exact middle, then the group, the pin and the gear at the right
  edge, 8 px between the group's items, 4 px between the battery ring and the dots, 8 px between the group and the pin glyph and 4 px between the pin glyph and the gear glyph. **The pin and gear gaps are measured between the glyphs**; their 24 px buttons are hit areas only and draw nothing, so they may overlap without showing. The group is
  the pill's group (same items, same order) as icons only. The clock glides from its place in the pill; the ring and
  the dots travel with it.
- **The clock is alone and never moves.** It stays in the exact middle of the header whatever shows beside it, so it reads
  first. At least 40 px separate it from the page dots and at least 40 px from the nearest icon.
- **Icons with no room are dropped, nearest the clock first** (the wave, then Caps Lock, and so on by rank). The battery
  ring, the dots, the pin and the gear always show; the header's least width keeps room for them. Dropping keeps the clock still;
  the icons nearest the clock are the steadiest states and the least urgent to see when the notch opens.
- **The header's least width** is the clock plus twice the wider side: the page dots and 40 px, or 40 px, the ring, the
  dots, the pin and the gear with their gaps. With nine pages that is 341 px.
- **The open notch is never narrower than the timer page's open width** (or its header, when that needs more), so
  every page opens at one shared least width instead of one squeezed around the header. A page wider than that floor
  sets the width itself.
- **Icons only, never text** (no "12m" in the open header). The icons keep the size their activity or page gives them.
- **A click on an icon opens its page** (timer, event to Calendar, prayer to Shalat, wave to Media). Muted, Caps Lock, the
  battery ring and the dots take the click and do nothing, so a click near them never reaches the page below. The hit
  area is 24 px around the icon, without widening the gaps; where two areas meet, the nearer item wins.
- **The gear** is a hexagon around a ring (circumradius 9.5 and ring radius 3 in a 24 px box), drawn in a 16 px box
  so the hexagon (12.3 x 13.6 px) is a little smaller than the 14 px battery ring, `#8c8c94`, the
  last item, its box flush with the header's right edge. It shows only in the open notch, never in the idle pill, and opens the
  settings window. It is not an activity and has no rank. The page dots keep the left edge to themselves, so the page
  the user is on reads first; the gear sits at the right end, where controls usually are, and the group slides left by
  its width and the gaps as the notch opens.
- **The pin** (`Icon::Pin`) is the Lucide "pin" outline (head above a needle, stroke 2 in a 24 px box). It sits 4 px left of
  the gear in a 13 px box (`PIN_SIZE`), where it shows about 9 x 12 px, a little smaller than the battery ring and no taller than the
  gear's 13.6 px. The group ends 8 px before it. Unpinned it is the gear's
  `#8c8c94`; pinned it is white, so the state reads at a glance. A click toggles it;
  its hit area is 24 px like the gear's. It shows only in the open notch and is never dropped for room. It is not an
  activity and has no rank. A pinned notch takes no file drops (the drop overlay steps aside while the notch is
  open); the user unpins it and drags onto the collapsed pill.
- **Hover on the pin and the gear** shows no background: the 24 px square around each is a hit area only, so the two
  glyphs keep their close spacing. The hovered pin or gear draws its glyph white instead; a pinned pin is white with or
  without the pointer. The other header items do not react to hover.

## Lyrics on

- The pill reads artwork (20 px), lyric line, group, then the dots only while a dot shows, with 12 px at both edges,
  12 px between the artwork and the line, 24 px between the line and the group and 8 px before the dots. The dots end
  the row 12 px from the right edge, the same place as in the idle pill, so they never jump when lyrics start or stop.
  The artwork is the media page's `resting_lead_width()`; the host passes the gap after it in `Frame::lead_gap`
  (24 px unless the page answers `resting_lead_gap()`; the media page answers 12).
  The line is trimmed so both gaps hold. The time stays in the header when the notch opens.
- The line's width sets the pill's width, within the page's limits (`crates/notch/src/pages/media/resting.rs`).
- There is no clock, no battery ring and no wave while the line shows: the artwork and the line already say that music plays.
- The group follows the line: muted, then Caps Lock, then only one countdown, the one with the least time left
  (`Page::countdown_seconds`), drawn as its icon alone (the header's status mark) so the line keeps its room. The nearest
  is the one the user will act on.
- The dots slide in and out like everything else: the line slides right by the dot and its 10 px (its own spring) and
  the pill grows; the group stays at the right end.

## Short messages and notifications

- A short message (volume, Caps Lock, prayer starts) replaces the pill for a moment and follows no order.
- A message that reports a percentage (Bluetooth device, charging, on battery, low battery) shows a 32 px ring around
  its icon, like the battery; that pill grows to 48 px.
- A message's value takes its icon's color (the percentage, "25 min", "15:19"); the name stays white.
- A notification opens the pill into a 380 px card: app icon, sender, two lines of the message. The two lines are
  clamped with an ellipsis, hovered or not; hover adds buttons and never grows the text, so the buttons do not move
  under the pointer.
- The heading is the sender in white, then the app name in grey (`#8e8e99`, regular weight) with no separator, so a
  missing sender cannot leave a dangling mark. A sender that is empty or only whitespace counts as missing and the
  app name takes its place, in white.
- Hover shows two buttons: Dismiss (secondary: translucent white fill, white text) and Open (primary: white fill,
  black text). A click on the card does nothing, so it never opens an app by accident. The card shows for 5 s, stays
  while the pointer is on it and goes 1.5 s after the pointer leaves; the rain alert's banner follows the same rules
  with Dismiss and Open (it shows the weather page in the notch).
- Open focuses the sending app and removes the notification, as a click on the native banner does; Dismiss only
  removes it. With the text hidden (privacy mode) Open still names the app.

## Settings window

The picture of the redesigned window leads where it differs from earlier text. The colors are `Palette` in
`crates/settings/src/palette.rs`, which has tests for their contrast.

- **A translucent neutral sidebar and one solid, flush content pane (shadcn's "sidebar" variant).** The sidebar is one flat neutral color
  over a blurred backdrop (`WindowBackgroundAppearance::Blurred`, Windows acrylic), so the desktop shows through frosted;
  there is no tint, gradient or glow. Dark theme: `#101010` at
  64 %. Light theme: white at 35 %. `FOLDLINE_SETTINGS_SOLID=1` makes it opaque and neutral (`#141414` dark, `#eeeeee`
  light). The content is the only solid surface: a pane that fills the window to the top, right and bottom edges with no padding, margin or rounded corner, and a 1 px hairline in the card border color between it and the sidebar. Dark theme (the v7 board's surfaces): card `#1b1b1d`, card border `#2a2a2e`, header `#1e1e21`, a section's block
  `#2a2a2d` (its border is the card border, so it reads as a plain lighter block), inputs `#1d1d1f` with a `#3a3a3e`
  border (darker than the block), buttons `#38383b` with a `#444448` border (lighter than the block), row divider
  `#353538`, switch off `#47474f`, titles `#f4f6fa`, muted text `#a3a3aa` (the board's `#8e8e98` would fall under 4.5:1
  on the block, so the muted text stays where the contrast test holds it).
  Light theme: card `#ffffff`, border `#d9dde6`, block `#f8f9fc`, muted `#4b4d55`, titles `#0b0d14`. A translucent frame
  with one solid card keeps the content the brightest, steadiest thing in the window. The contrast tests keep titles at
  7:1 and muted text at 4.5:1 on the card and the blocks; on the frame they keep titles at 4.5:1 over a pure white desktop
  (dark theme) and 7:1 over a mid-grey one (light theme), and muted text at 3:1 over a bright wallpaper (`#c8c8c8`). Over a
  pure white desktop the dark theme's muted group labels fall below 3:1, because the frame stays within 55 to 65 %
  opacity.
- **One blue accent**, from the logo's blue: dark `#4c9aff`, light `#2563eb`. Every sidebar icon is blue (selected or
  not), the active part of the breadcrumb, a section's small icon, switches that are on, checkboxes, links, focus rings and
  the selected option card's border and radio. Text stays white or muted. The tests keep the accent at 4.5:1 as text on
  the card and 3:1 as an icon on a block; as an icon on the frame it is held at 2:1 (over a bright wallpaper in the dark
  theme, over a mid-grey desktop in the light theme).
- **The sidebar** is 216 px wide with 16 px of padding above, 10 px at the sides and 12 px below. It opens with the brand (the tile 16 px from the window top and 18 px from the left edge; the squared tile logo, 28 px, then "Foldline" in `text_lg`
  semibold and the version in `text_xs`, muted, 8 px apart, with 36 px below it before the first item), then the group "Foldline" (General, Pages, Behavior, Advanced) with no label and the labelled group "Page settings" (`text_sm`, muted, 12 px above the label and 5 px below: Calendar, Timer, Shalat, System stats, Weather), with **About** at the bottom, without a label. The empty sidebar (the strips between and below the items, at their sides, and the group labels) moves the window like the header; the items never do. Items are 36 px
  high and 1 px apart, 14 px text 9 px from a 17 px blue icon, 10 px of side padding, all in the foreground color; the selected one differs only by a rounded (8 px), faint fill: its text keeps the same color and weight. The tile is blue in a
  stable build and indigo in a pre-release build, in both themes. The icons are General the hex nut (the same drawing as
  the notch's gear), Pages Lucide "layers", Behavior "puzzle", Calendar "calendar-days", Timer "timer", Shalat "moon-star" (the shalat
  icon in the notch too), System stats "chart-column", Weather "cloud-drizzle", Advanced "wrench" and About "info". They
  are SVG files in `crates/settings/assets/icons`, served by `crates/settings/src/icons.rs`; the Lucide ISC notice is in
  `THIRD-PARTY-LICENSES.txt`. The numbers are constants in `crates/settings/src/sidebar.rs`.
- **The pane's header** is 44 px with its own fill, a little lighter than the pane, over a divider. It reads
  "Settings › <Page>" (`text_base`, 22 px from the pane's edge, 10 px around the chevron; Settings muted, the page in the accent blue and medium weight) with the window buttons (minimize, maximize, close) at
  the right end, and it is the drag area together with the brand strip. The window draws it itself and not the component
  library's title bar: the header has square corners because the pane is flush with the window, so the close button's red hover fills the window's top-right corner.
- **A redesigned page opens with its first section**, as the v7 boards do: General, Pages, Behavior, Advanced
  and About have no description sentence (`Page::plain`); the page settings keep theirs (one sentence in `text_base`
  and the full title color) until their boards are built. A section heading has no icon; a block that needs no name (the files on About) has no heading.
- **Pictures explain a setting where words alone would not**: the three Hide-when-idle pictures, the city map, the two
  notification banners, the screens of **Show the notch on**, the page dots on Pages, the log sample on Advanced. They
  sit in their own row of the block (12 px by 14 px padding), drawn with GPUI elements on a dark blue scene
  (`#1d2438` to `#12151f`, 9 px radius, a faint white border) and never as images, with a 12 px muted caption under
  each when they come as a strip. A picture that is also a choice is an option card; the others are not clickable.
- **Tags and hints.** A small rounded tag after a row title states a fact about the row: "After restart" in muted gray
  (only where it is true: the debug log), "Also key 0" in the accent. A hint that applies
  to a whole list sits under it in a blue-bordered box with a blue icon (the digit keys and switching a page off on
  Pages).
- **A section opens with a heading row** (`text_sm`, muted, inset 4 px): its title at the left, with a small blue icon
  where one fits, and an optional short description at the right. Its parts follow 7 px apart: bordered blocks (12 px
  radius, the card's border color, one step lighter than the card) with rows separated by dividers, where the border
  closes the last row, so only rows between two others carry a divider; and items drawn outside a block, like the
  Behavior cards. The page content is padded 16 px above, 22 px at the sides and 20 px below, and sections sit 14 px
  apart. Muted gray is for row descriptions (`text_sm`), section headings and group labels; row titles are medium
  weight (`text_base`) in the full foreground.
- **Rows** are at least 50 px high with 9 px by 14 px padding: the title and a muted description (17 px lines) at the
  left, 12 px from the control at the right. A switch is 42 x 25 (the window's own `switch.rs`). A button
  is rounded (30 px high, 8 px radius, 13 px medium text), a step lighter than the block with a thin border; every button does something, so none is dimmed or marked "Coming soon". A select (a dropdown button: the Snooze and Pause lengths) is an outline button 190 px wide with a chevron that opens a popup menu. A field is 8 px round, darker than the block, and its border turns to the accent while it has focus.
- **The font is Geist**, set through the component library's `theme.font_family`, so every control uses it.
- **One scale knob.** The window follows the Windows display scale (GPUI sizes are logical pixels times the monitor's scale factor). On top of that, `theme.font_size` (`UI_FONT_SIZE` in `main.rs`, 14.5 instead of the library's 16, set in `palette::apply`) is the window's rem size, so every rem-based size shrinks together: `text_*` (`text_xs` is about 10.9 px, `text_sm` 12.7 px, `text_base` 14.5 px, `text_lg` 16.3 px), `px_*`, `gap_*` and the library's controls. Sizes written in `px(..)` (sidebar, header, brand) do not follow it.
- **Option cards** are for picking among pictures (the calculation methods, the Asr choice, what the notch shows over a
  maximized or full-screen window). They follow shadcn's radio card: the radio ring sits inside the card at the top left,
  left of the title (semibold, full foreground) with the muted description under it; the picture follows. Selected:
  blue-tinted fill, accent border, accent ring with an accent dot. Unselected: darker fill, quiet border, empty ring. A
  multi-select option uses a checkbox in the same place. A card is rounded 10 px with a 1.5 px border and 8 px padding, 6 px between its parts; the title is 13.5 px and the
  description 12 px on 16 px lines. Unselected, its fill is the block's (`#2a2a2d`) with a faint border, so it reads
  as an outline. Where the picture already names the choice (the Behavior levels), the radio gives way to a filled
  accent check after the title of the selected card (`card::checked_header`). Cards sit in equal-width columns 10 px
  apart (`card::grid`): four for the calculation methods and the Behavior levels (the picture, 78 px high, above the
  title there), three for the language and the screen choice, two for the rest; a partial last row keeps the cell width
  and starts at the left.
- **The chosen screen is picked on a drawing of the desktop**: the connected screens where they stand, scaled to fit,
  each with its number and size; the chosen one is outlined in the accent with the notch on it. A list of names would
  not say which physical screen is which.

## Connected devices page

The page is off until added and has no collapsed state; it reads Windows only while it shows (`crates/notch/src/pages/devices/`).

- **One 88 px column per device, up to five; the page is as wide as its tiles** (at least 253 px of content, so one or two
  devices do not make it tiny) and the row is centered. A tile is a 58 px ring (4 px
  stroke, round ends, track `#2c2c30`), the device icon (22 px, `#c4c4cc`) and the percentage inside it, and the name
  under it (10 px, `#9a9aa6`, at most 76 px wide, cut with an ellipsis). A device is read at a glance by its icon, its
  ring and one number.
- **Ring colors follow the battery ring:** idle `#2f9e44`, charging `#30d158` with a bolt after the number, low
  `#ff453a` at 20 % or less (number red too). Charging wins over low. Windows reports no charging state for Bluetooth
  devices, so the bolt shows only when a source supplies one.
- **No reading is "On" in white on a full solid ring (`#3a3a44`, 4 px); a device that is off is a dashed ring of the same
  color (20 dashes), a dimmed icon (`#55555f`) and "Off" (`#6e6e7a`).** "On" says the device is connected and does not
  report a battery; "Off" says it is not connected. A device that is off shows only beside a connected one, so an old pairing never hides the empty message.
- **More than five devices show the five lowest batteries (then devices with no reading, then off devices) in Windows'
  order, with no "+N more".** Up to five keep Windows' order, so tiles do not jump between reads. The ones that need
  charging are the ones the user came to see.
- **Two messages replace the tiles, on a 318 px content width (350 px with the host padding):** "No wireless devices"
  (Bluetooth mark in `#63e6be`, with "Connect any device to see its battery") and "Bluetooth is off" (mark in `#8a8a96`,
  with "Turn it on in Windows to see battery levels"). One row, centered as a whole: the 22 px mark in a 36 px round badge
  (`#17171c`, 1 px `#26262d` border) at the left, 10 px, then the left-aligned text: a 15 px title (the design says
  semibold; only Medium is bundled), 3 px, a 12 px detail line in `#9a9aa6`. The width is fixed, not measured, because a
  page reports its size without fonts; it fits the longer of the two lines with room to spare.
- **Devices behind a 2.4 GHz USB receiver are tiles like Bluetooth ones**; built-in, wired and Bluetooth devices
  are not listed as receiver devices. Their battery comes over the receiver when the device answers (Logitech HID++);
  one that does not answer, or that is asleep, shows the dash, because a receiver cannot tell asleep from off. The name
  is the one the receiver or device gives ("B330/M330/M331"), else the brand and kind ("Logitech mouse"), because
  Windows only says "HID-compliant mouse". Bluetooth devices come first, then receiver devices; a name and kind
  listed twice counts once. "Bluetooth is off" shows only when Bluetooth is off and no receiver device is there; with
  receiver devices and Bluetooth off the page lists them. A name wider than 76 px is cut with an ellipsis.
- **Numbers (including "On") are Geist with tabular figures, not the Geist Mono the design file uses**, as the rule below sets for every
  number.

## Media page

- **The open player is the artwork (86 px, radius 12), 20 px, then a text column fitted to the text, 134 to 170 px wide**
  (the page is as wide as the artwork, the gap and the column, so at least 240 px; 170 px with lyrics on): the title
  (14 px Medium), the artist (12 px, `#9a9aa6`) 2 px under it, and 12 px below that the controls, previous, play or
  pause, next, then lyrics, left-aligned under the text. Transport icons are 16 px, 30 px from center to center (14 px
  between them), the lyrics icon 18 px; it is green when on, white when lyrics exist and gray when they do not. A
  control answers clicks in a 30 x 30 px area around its center, so neighbors meet without overlapping. The control row
  (108 px) is narrower than the 134 px least column, which keeps the page width a short title had with the larger
  controls. There is no remaining time and no progress bar.
- **The source app's icon is a 19 px tile (`#0c0c0c`) with a 3 px black rim, its right and bottom edges 1 px past the
  artwork's corner**, with the app icon at 15 px inside it.
- **Lyrics on widen the page to 614 px (582 px of content) and tint the backdrop from the artwork color**, from black at
  the top through 11 % at 35 % of the height to 45 % at the bottom. The lyric panel is the rest of the row after a 20 px
  gap: the current line 17 px Medium white, the lines before and after it 13 px at 20 % white, 10 px between lines.
- **Nothing playing:** a `#292929` artwork with a 16 px note, "Play a song", "Nothing is playing" and dimmed controls
  (`#6e6e7a`); the page is as wide as its text needs, within the same limits.
- **The collapsed lyric pill's artwork has radius 5;** the wave is 18 px wide with five 2 px bars.

## Focus timer page

- **Time at the left in 38 px Medium tabular figures (`mm:ss`), the label 12 px after it, the words ending on the bottom edge of
  the time row** (13 px Medium, with a 11 px `#9a9aa6` detail line under it while idle). A paused time is `#9a9aa6`.
- **The page is 86 px high: a 42 px time row, a 12 px gap, then the buttons.** A headline without a time (finished) is
  centered in the row, so the buttons are never closer than 12 px to the text.
- **Buttons are 30 px high pills in equal columns with 9 px between, along the bottom.** The main action (Start, Resume,
  Start next) is the timer orange `#ff9e0a` with dark text, or blue `#4dabf7` when it starts a break (the green is the battery ring's); the others are
  `#232323`, and Stop is the only red one (`#8c1a1f`). Orange as a main action belongs to this page only; the other pages'
  main action stays white.
- **A finished session or break has no time:** a 17 px Medium headline over a 12 px `#9a9aa6` detail line, then
  the next action and Skip. Breaks are "Short break" and "Long break"; the page is 322 px wide.

## Calendar page

- **The left column is 182 px wide and the events are 16 px after it; both end on the bottom of the page.** The page is
  168 px high (the events viewport), or 170 px for a month of six rows. With `calendar.view` set to `week` the left
  column holds only the current Monday-to-Sunday row (the same cells, a day of another month keeps its date) and the page
  is 86 px high like the other pages (`DEFAULT_CONTENT_HEIGHT`): the viewport shows two cards and part of a third, and the list scrolls for more.
- **The month is the month name (20 px Medium) and the year (12 px, `#9a9aa6`, on the month's baseline: its line ends 1 px
  above the month line's bottom, measured on a snapshot), one row of weekday letters (M T W T F S S,
  10 px, `#9a9aa6`) 8 px under it, and the dates 6 px under the letters:** seven 26 x 20 px cells, Monday first, up to six
  rows, the date in a 20 px circle (11 px, tabular). Days of other months stay empty. Sundays are `#ff6b6b`. Today is a
  solid `#2f81f7` circle with a white number (Medium; the design asks for 600, which is not bundled); a selected day
  is a `#4dabf7` circle at 28 %, and when it is today only the solid circle shows. There are no event dots, and the
  hovered day gets a white 6 % circle.
- **The events are a 168 px viewport of 34 px cards, 5 px apart,** radius 8, `#17171c` with a 1 px `#26262d` border, the
  start time (12 px, `#9a9aa6`, "All day" for an all-day event) in a 52 px column, then the title (13 px). Cards start at
  the top of the viewport. Four cards fit; with more, the fifth shows by 12 px, and the list scrolls with the
  wheel (one card a notch), a swipe or a drag, with no scrollbar and no "+N more". Picking a day shows its list from the top.
  A past event of today leaves the list; another day lists its events as planned. There is no "Join" button.
- **Done for today** is a check icon, "Done for today" (13 px Medium) and "No more events" in one centered row; **an empty
  day** is a 22 px calendar icon (`#9a9aa6`), "No events on Thursday 13" (14 px Medium) and "Events from your calendar show
  here." (12 px, `#9a9aa6`), 6 px apart and centered in the viewport.

## Stats page

- **Every card is exactly 132 x 86 px** (radius 12, `#141418`, 8 px apart) with 12 px of padding at both sides
  (`TILE_PADDING_X`). The page is five cards wide with a GPU (692 px content, 724 px with the host padding) and four
  without one (552 px content, 584 px with the padding; the board says 592, which does not follow from its own formula, so the
  formula wins). Order: CPU, memory, GPU, network, disks.
- **A chart is a dotted baseline and two bars**, not a history: the baseline runs from x 12 to 118 (1.4 px dots 4 px apart,
  90 % opacity) and two 2.6 px bars with a 1 px radius stand on it at x 111.5 and 115.5, 1 px high at rest up to 12 px at
  full load. The older bar is the lighter shade, and at no load only the newer bar shows. CPU `#4dabf7` / `#74c0fc`, memory
  `#cc5de8` / `#e599f7`, GPU `#f08c00` / `#ffa94d`. CPU and memory: label at 8, 20 px value at 19, baseline at 60, line
  under it at 64. One GPU: "GPU · name" label, 20 px value, baseline at 66. Two GPUs: "GPU" label and two rows, name and
  percentage at 22 and 53 with a baseline at 47 and 79 each.
- **Network** is the download (14 px Medium, white) at 20, the upload (`#8c8c94`) at 43 and the kind (11 px) at 66.
  **Disks** are rows of a name (`#9a9aa6`), a 56 x 4 px bar at x 34 (track `#26262d`, fill `#30d158`) and the percentage
  at the right (11 px; the design says 600, only Medium is bundled): one row at 32, two at 27 and 46, three at 24, 40 and 56.

## Clipboard page

Board `d2a7dd31` (`slim.html`). The page is opt-in, 508 x 179 px of content (540 x 229 px with the host padding and
header) once it holds at least one copy. With an empty history it reports the default 86 px of content height like the
other pages (508 x 86 px) and grows to 179 px as soon as one copy exists; it shrinks back when the last one is removed.
The bar stays at the bottom of the page in both heights.

- **Two columns 12 px apart.** The list is 214 px wide and shows the last 7 copies, newest on top, so it never scrolls:
  rows are 23 px high, 3 px apart, radius 8, 8 px of padding, a 14 px icon (`#8a8a96`, 1.8 px line) and 7 px before the
  text (11 px, `#e6e6eb`, cut with an ellipsis). A row is fill `#121216` with a 1 px `#1e1e24` border; the selected row is
  `#1a1a20` with `#3a3a44`. Hover is `#17171c` (not in the board; proposed). Code rows use 10.5 px Consolas, because
  Geist Mono is not bundled (the same fallback as the calendar). A color row shows a 15 px swatch of the color, an image
  row its own picture and "Image (360x70)", a link the link icon and text the document icon. The picture is the square
  middle of the copy (cropped to fill, never letterboxed), 15 px on whole device pixels, radius 4, with a 1 device pixel
  ring of white at 16 % inside its edge so a white, black or see-through picture keeps its outline on the row (not in
  the board; proposed). A picture that cannot be drawn shows the Lucide image icon instead.
- **The right column** is the preview panel (`#0b0b0e`, radius 8, 8 px above and below and 10 px at the sides) over one
  26 px action row 8 px below it. Text and code are 12 px on 17 px lines, at most 7 lines, the last ending with an
  ellipsis when more would not fit; a picture is drawn contained and centered, never upscaled beyond its stored size; a
  color is a 96 x 56 px swatch (radius 8) with its hex under it.
- **The action row.** Details at the left on two lines (10 px, `#6e6e7a`, 12 px apart): the type with its size or length
  ("Text · 24 characters", "Image · 360x70 · PNG", "Link · n characters", "Code · n characters", "Color · #4dabf7"), then
  how long ago ("just now", "30 sec ago", "12 min ago", "3 h ago", "2 d ago"). Then Delete and Copy, 26 px icon buttons
  (fill `#121216`, 1 px `#2c2c34` border, radius 8; trash `#ff6b6b`, copy `#9a9aa6`), and Paste, a white 76 px button
  (the board's is about 66 px; the width is fixed so "Pasted" fits), icon and label in black, 6 px apart.
- **Confirmations are green** (`#63e6be` on `#0d2b24`, 1 px border): after a paste the selected row reads "Pasted" with a
  check, the Paste button reads "Pasted" with a check, and the notch closes after about 600 ms; after Copy the row reads
  "Copied, moved to front" and the Copy button shows a check for 1.5 s.
- **Pins.** A row's pin is icon only (the header's pin outline, `Icon::Pin`, at 13 px) in a 23 px square at the row's right end, which the row's own
  click area leaves to it. An unpinned row shows it only while hovered (`#8a8a96`, `#e6e6eb` with the pointer on it); a
  pinned row looks like the others but its pin stays, in the accent blue `#74c0fc`. At most three copies are pinned;
  while three are, unpinned rows show no pin. Pinned copies sit on top in the order they were pinned, with no divider;
  a new or repeated copy goes on top of the unpinned ones. Clicking a pinned row's pin unpins it, and it goes back
  among the others by the time it was copied. Pins live in memory with the history and go with it.
- **The bar** is 26 px high over the bottom of the list (covering the last row), `#26262d`, radius 8, 11 px text with
  10 px of padding. "Removed" with a `#74c0fc` "Undo" at the right stays 5 s; "Skipped: password manager" stays 3 s
  (the board gives no time; proposed). One bar at a time: a new one replaces the old, which drops the undo.
  "Skipped: too large" is the same bar for text over 100 KB or a picture over 40 MB (not in the board; proposed).
- **The empty page** is a 36 px badge (circle, `#17171c`, 1 px `#26262d` border, a 20 px clipboard icon `#9a9aa6`), 16 px,
  then "Nothing copied yet" (15 px; the board says 600, only Medium is bundled) over "Copy text, a link or an image
  and it appears here" (12 px, `#9a9aa6`), the two centered as one row.
- **Behavior.** Click selects. A double-click (two presses within the system double-click time, 4 px apart) pastes into
  the window in front by writing the clipboard and sending Ctrl+V; with no window to paste into (the desktop, the
  taskbar, the notch itself) it copies the item again and moves it to the front. Right-click removes. Copying the same
  content again, even long after, moves the existing row to the front instead of adding a second one. Kinds: a color is
  `#rgb` or `#rrggbb`; a link starts with `http://`, `https://` or `www.`, or is a host name with a dot and a path
  (`docs.example.com/shortcuts`; a bare `readme.md` is text); code starts with a command (npm, git, cargo and similar)
  followed by an argument, has braces in a pair, a line ending in `;`, `=>`, or starts with a statement keyword
  (`fn`, `let`, `const`, `def`, `import`, `class`); everything else is text.
- **Limits and privacy** (decisions, not from the board): memory only, cleared on quit and when the page is switched off;
  capture happens only while the page is on; text over 100 KB is skipped; a picture over 40 MB as a bitmap is skipped and
  one over 786,432 pixels (about 3 MiB) is shrunk before it is kept, so pasting it back gives the smaller picture; all
  copies together stay under 12 MiB, oldest unpinned copy first out. The limits never drop a pinned copy or the newest
  one (pins count toward the 7; three pinned pictures and a new one fit in 12 MiB). Undo puts a removed pinned copy
  back pinned in its place, or unpinned on top of the others when three were pinned meanwhile. A copy marked `ExcludeClipboardContentFromMonitorProcessing` or
  `CanIncludeInClipboardHistory = 0` is never read.

## Shelf page

- Six files a row, two rows at most, so twelve; the page is 86 px high for one row and 159 px for two (10 px less the
  1.5 px border of padding, rows 6 px apart). Each file is a 78 px column: a 40 px tile with a 6 px radius, 3 px, then
  two lines of its name (10 px, `#d6d6de`).
- A picture shows its Explorer thumbnail; every other file is a small drawn document in the 40 px box. PDF, pictures and
  design files are a white 30 by 38 sheet (3 px radius) with a red `#e03131` "PDF" tag, a blue rounded square or a
  purple outlined square with a dot and a lilac bar. Text (and a file with no extension) is a sheet with four 1 px
  `#c4c4cc` lines. Archives are a yellow folder with a zipper (flat `#f8bd3b` where the design has a gradient).
  Spreadsheets are a sheet with a green strip and a `#107c41` "X" badge. Any other type is a sheet with its extension
  on a tag in the type's old tile color (documents `#3b5bdb`, slides `#c2255c`, unknown `#868e96`). The gradients of
  the design are flat (mid color), and the tag text is Medium, as the design's 600 is not bundled.
- The outline is dashed `#5a5a68`. Empty: a row in the middle of the outline, a 36 px round badge (`#17171c`, 1 px
  `#26262d`) with a download icon (stroke `#9a9aa6`), 16 px to its left of the text, "Drop files here" (15 px, Medium)
  and "They are held for a moment, then you drag them out" (12 px, `#9a9aa6`), left aligned. Dragging over: the outline turns `#4dabf7` and the area `#4dabf7` at 12 %, the empty line reads "Release to hold them"
  in `#74c0fc`, and files already there stay. Full and dragging over: outline `#d6453d`, area red at 8 % and the red
  line "The shelf is full. Remove a file to add more." (`#ff6b6b`); nothing is replaced.
- The remove button is a 16 px circle (`#2a2a30`, 2 px black ring) over the top right corner of the tile, 3 px outside
  it, with an 8 px `#c4c4cc` cross; it shows on hover.

## Todo page

- A task is a 32 px row (10 px radius, `#141418`, hover `#1c1c22`), 6 px apart, with a 14 px ring, the text (13 px, one
  line cut with an ellipsis, 120 characters at most) and, on hover, a 22 px delete button 6 px from the right edge with
  a 22 px copy button (a 12 px copy icon, the same circle) 6 px to its left; the gap is narrower than the 8 px between
  pills, and a press in it does nothing so it cannot land on the wrong button. Copy puts the task's text on the
  clipboard; the notch's own clipboard writes show no "Copied" message and are not added to the clipboard page.
  The copy button then shows a green check (the stroke of the clipboard page's Copy check, `#63e6be`, 12 px) in place of the icon for 1.5 s on that row only; a second copy moves it and restarts the 1.5 s, and a delete, Clear all, Undo or closing the page clears it. The button keeps taking presses while it shows the check.
  A done task has no fill, a green check (`#30d158`) and struck-through `#6e6e7a` text, and keeps its row. Done tasks
  follow open ones.
- No limit on tasks; the page shows five rows at most and the list scrolls under the wheel or a swipe (one row a notch),
  with no scrollbar and no hint of the rows beyond. The toolbar (28 px high, then 8 px) sits on top: "Todo" (20 px, Medium) and the count ("3 left",
  "3 left · 2 done", "0 left" in 12 px grey, 8 px after the title, on its baseline) at the left, and at the right, 8 px apart,
  **Undo** (while a removal can be undone), **Clear all** and **Add**. Rows run the full width below it.
- **Add** is a fixed white pill (72 px wide, widened from 60 so "Tambah" fits; 26 px high, 13 px radius, black "+ Add" in 12 px Medium, as the design's 600 is not
  bundled). It is never dimmed. Pressing it adds the
  input as the last row (scrolled into view), with the keyboard (1 px `#408fff` outline, "+" in the same blue, "What needs doing?", "Enter
  to add" at the right).
- **Clear all** ("Bersihkan" in Indonesian, 80 px wide) is a 26 px pill (`#1c1c22`, 1 px `#2c2c34` border, text `#e6e6eb`, text only). It
  removes every task, done or not, without asking, and is hidden when the list is empty.
- **Undo** (same pill as Clear all, 80 px, "Urungkan") shows for 5 s after a delete or Clear all, always left of Clear
  all's place; that place stays empty while the list is, so a second press on Clear all never lands on Undo. It puts the
  removed tasks back where they were. Only the last removal can be undone: a new removal replaces it; the removed tasks
  stay out of `todo.txt` unless undone, and the undo is not saved.
- In the add field (`ui/text_field.rs`), Ctrl+Backspace removes the spaces before the caret and the word before them, and Ctrl+Delete the spaces after it and the word after them.
- The page is 508 px wide and 28 + 8 + the rows high (3 tasks 144 px, five or more rows 220 px), at least 86 px.
- Empty: the same 508 px width as a list, 86 px high, with the title and Add on top and "All clear" centered below;
  pressing Add shows the input as the only row.

## Shalat page

- The left column (124 px): "Next" (11 px grey; "Starts soon" in the last ten minutes, "Tomorrow" after Isha), the
  prayer's name in 26 px, the countdown in green `#63e6be` (12 px) and the city (10 px dim). The name leads, not the time.
- The right side is one sun arc through Fajr, Dhuhr, Asr, Maghrib and Isha (Sunrise is not on it) with the name and
  time under each marker. The curve is a sketch, not the sun's height. A passed marker is a green dot, the next one a
  larger dot with a halo and a green name, a coming one a hollow dot. The arc and its fill are green up to now and grey
  after; the sun (yellow, with a glow) sits on the arc at the present time and is hidden before Fajr and after Isha,
  when the whole arc is green and tomorrow's Fajr is the next marker.

## Weather page

- The left column (146 px), always in this order: the place (10 px eyebrow), the temperature (28 px), the condition
  with "feels" (11 px) and the rain line in blue (12 px). Six cells fill the rest, 4 px apart: a label, a 20 px sky icon,
  the temperature and, for hours, the rain under it in millimetres. The first cell ("Now", "Today") has a `#2b2312`
  fill and an amber label. A pill ("Today" or "Week", 22 px high, `#1c1c22` with a 1 px `#2c2c34` border, text `#e6e6eb`)
  at the right end of the left column's first line, level with the place, names what is shown and switches between the
  hours and the days when pressed; a day shows its high above its low. The choice is kept while the notch runs, not
  across restarts.
- Sky icons are the Lucide icons of the sky, one color each, chosen by `Symbol::icon` (the one mapping, from the MET Norway code): thunder cloud-lightning, snow and sleet cloud-snow, showers cloud-sun-rain by day and twilight and cloud-moon-rain by night (any intensity), heavy rain cloud-rain-wind, rain and light rain cloud-rain, cloudy cloudy, fair and partly cloudy cloud-sun by day and cloud-moon by night, clear sky sun by day and moon by night, fog cloud-fog, an unknown code cloudy. `_polartwilight` counts as day. Colors: sun and partly cloudy by day `#ffc078`, rain and showers `#74c0fc`, thunder `#ffd43b`, snow `#e7f0ff`, cloud, night and fog `#c4c4cc` (`#e6e6eb` in the first cell). The same icon draws in the rain alert (see below).
- Offline and no location are rows: a 36 px round badge (`#17171c`, 1 px `#26262d`) on the left, 16 px, then a
  left-aligned title (15 px, Medium) and a line (12 px, `#9a9aa6`). Offline: a crossed-out cloud, "Weather is offline"
  and "Last updated 13:40 · retrying on its own". No location: a `#4dabf7` pin, "Choose a city" and "Pick a location in
  Settings to see the weather". Both hug their content: the page is as wide as the row plus the notch's padding, never
  below the notch's minimum width (the timer page's). The design draws them in a 392 px notch; that is not a rule.
- The rain alert starts 30 minutes before rain: a 340 px banner (the width of a download card, a little narrower than a notification card; a blue 30 px tile with the white icon of the coming weather, "Rain in 30 min" with "Weather" after it in grey, two lines of text) for 9 s, held while the pointer is on it, with Dismiss and Open under the text. The banner is the only moment the alert takes the pill; after it the alert stays as a blue badge in the group (the 14 px icon of the coming weather, 4 px, "Rain in 25 min"; "25m" when several badges share the pill) and as that icon in the open header, rank 6 (after the prayer), until the rain starts, so the clock and the battery ring stay in view. A press on the badge in the collapsed pill opens the notch on the weather page. Dismiss removes the badge too. Open opens the notch on the weather page (no browser); a click on the header icon does the same. Only data clicked on the weather page opens weather.com: an hour of the Today view opens `https://weather.com/weather/hourbyhour/l/<lat>,<lon>` and a day of the Week view opens `.../tenday/l/<lat>,<lon>` (coordinates trimmed to 4 decimals, numbers only; the site has no address for one hour or day, so the view lists them). The data still comes from MET Norway: the page asks for the forecast of the shared place (coordinates to three decimals) when it starts and every 30 minutes, every 10 minutes while offline, and only while it is switched on.
- Each hour (Today) and each day (Week) is a button: a click opens the same table for that hour's or that day's date, and the cell under the pointer takes a faint white tint (`#ffffff` at 7 %), as the Today/Week pill takes its lighter fill. The notch sets no pointer cursor anywhere, so none is set here. The Today/Week pill is unchanged.

## Color and type

- An activity's icon and text share one color: timer `#ff9e0a`, prayer `#63e6be`, event `#4dabf7`. Static icons are white.
  Mixed colors read as two things.
- The idle battery ring is `#2f9e44`, darker than the prayer mint so the two never read alike; charging is `#30d158`,
  low battery `#ff453a`.
- Camera dot `#30d158`, microphone dot `#ff9e0a`.
- The collapsed pill is all Geist, clock and countdown numbers included; it is a label, not a readout.
- Numbers are Geist everywhere, in the pill, the header and the pages. There is no monospace font. A number that changes
  while it is shown (a countdown, a percentage, a temperature) uses `FontStyle::tabular()`, so every digit has the same width
  and the neighbouring text stays put.

## Opening settings

- The gear in the open header and the tray's **Settings** item open the settings window (`foldline-settings.exe`); settings
  are edited only there, and the notch has no settings panel and no right-click menu. **Snooze notifications**, **Pause Foldline**,
  **Settings** and **Quit Foldline** are in the tray menu. A page that uses right-click for its own content (a file on the shelf) keeps it.

## Open

- The reminder windows are fixed: 30 min for the prayer (`BADGE_MINUTES`), 15 min for an event
  (`REMINDER_LEAD_SECONDS`, 15 minutes). A per-page number setting, default 10, is proposed and not decided.
- Between lyric lines (intro, instrumental) what the pill shows is not decided; a "♪" placeholder that keeps lyric mode
  for the whole song is proposed.

## Code gaps

- The weather page has no chance of rain (MET Norway's `complete` answer carries no probability for most places), so
  the cells show millimetres where the design shows percentages; the alert's pill message shows its time in grey and a
  plain cloud, not blue and a rain cloud (`ui::draw_message`).
- Only Regular and Medium Geist are bundled, so the design's 600 weight draws as Medium (the prayer name, the todo
  title, the offline title). Colors and text styles that several pages share are `ui/palette.rs` and `ui/type_styles.rs`; a page keeps only its own.
- The percentage rings have not been checked against the code.
