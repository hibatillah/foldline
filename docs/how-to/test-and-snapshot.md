# Test and snapshot

> **How-to.** Read when: you write a unit test, render the page pictures, or run the manual checklist. Skip if: you only need where a test lives; see [the test map](../reference/test-map.md).

## Unit tests

```
cargo test
```

They run in a fraction of a second and need no window. They cover the logic that can be checked without drawing:

- lyric parsing and timing (`.lrc` text, which line is being sung, when the next one starts)
- cleaning titles and artists for lyric lookups, and URL encoding
- springs, geometry, colors and image slots
- settings: parsing, defaults and saving
- page selection: priority rules, ties, switching pages off, timer keys
- the media page's rules: when it shows, how wide it asks to be
- layout of the open view and the hit areas of the buttons, the shelf's files and the calendar's days
- the lyric follower, which tracks the current line and arms timers

What they do not cover is the Windows integration (the window, the tray, the media session). Check that by running the
notch: see the manual checklist below.

### Pictures of the pages

```
cargo test snapshots -- --ignored
```

These tests are ignored by default. They draw every page off-screen with example content and write the pictures to
`target/snapshots` (BMP files; `host/notch/tests/images.rs` holds them): each open page, the collapsed pill in its
states (`resting-idle`, `resting-idle-with-group`, `resting-media-bars` for the idle wave, `resting-media-lyrics`, `resting-media-lyrics-with-timer`, and others), the short messages, the notification banner and the shelf with files dragged over it. `cargo test page_state -- --ignored` (`host/notch/tests/page_states.rs`) draws the states of the shelf, todo, shalat and weather pages (`state-*` files), and `cargo test clipboard_state -- --ignored` (`host/notch/tests/clipboard_states.rs`) those of the clipboard page. Look at the pictures
after a change to drawing or layout. Two more ignored tests need a real screen and open windows for a moment:

```
cargo test a_real_maximized -- --ignored      # a maximized and a full-screen window are told apart
cargo test this_windows_has -- --ignored      # the Windows entry points behind the dark tray menu exist
```

### Writing a test

Put a `#[cfg(test)] mod tests` at the bottom of the file you are changing and test through its normal functions.
Name tests as sentences about behavior, for example `a_paused_track_keeps_its_line_but_arms_no_timer`. Tests that
need text measurement create `Fonts::new(1.0)`; DirectWrite works in the test run without a window.

Keep logic testable by keeping it out of the Windows calls: a function that takes plain values and returns plain
values can be tested; one that talks to the window cannot.

## Manual checklist

For a change that affects drawing or input, run `scripts/reload.sh` and check:

- the collapsed pill is centered vertically and has a flat top and rounded bottom corners
- hovering opens it, and moving away closes it, every time
- previous, play or pause, and next work, and the hovered button gets a faint wash; there is no progress bar
- the lyric button hides and shows lyrics, and the choice is remembered after restarting
- the tray icon's Quit Notch closes the notch and removes the icon

For a change to the notch's frame or hiding: open the notch and check the dots, the status marks and the time, tick "Hide when idle" in the settings window and check that the notch
disappears, shows its small pill when you rest the pointer on the top of the screen, and opens if you stay. The mouse-driven checks need an unlocked screen.

For a change to the settings window (`crates/settings/`), run `cargo +stable-x86_64-pc-windows-msvc test --workspace`, then open the window from the gear in the open header (and once from the tray's **Settings** item) and check:

- the frame is a neutral translucent tint (dark `#101010` at 64 %, light white at 35 %; the desktop shows through, blurred, with no blue tint or glow) and the sidebar sits straight on it with no card: the squared tile logo (blue on a stable build, orange on a pre-release), "Notch" and the version at the top, then the groups "Notch" (General, Pages, Behavior, Advanced) and "Page settings" (Calendar, Shalat, System stats, Weather), the selected item with a rounded faint fill, and **About** at the bottom; every sidebar icon is blue; the window corners are rounded; with `NOTCH_SETTINGS_SOLID=1` the frame is a neutral opaque color
- the sidebar icons: a hex nut (General), layers (Pages), puzzle (Behavior), calendar-days (Calendar), moon-star (Shalat), chart-column (System stats), cloud-drizzle (Weather), wrench (Advanced) and info (About), none missing or blank, in both themes; the Weather page has one **Rain alert** switch (on by default) that writes `weather.rain_alert`
- the content is the only solid surface: a rounded (8 px), bordered card inset 8 px from the top, right and bottom edges; its 36 px header is a little lighter than the card and reads "Settings › <page>" (the page in blue) over a divider, with minimize, maximize and close at the right end; there is no separate title bar; sections are bordered blocks lighter than the card, buttons are dark bordered pills, and an inert button is dimmed with "Coming soon" to its left (Check now, What's new)
- the header's top corners follow the card's: hover the close button and check that its red fill follows the rounded top-right corner and does not poke out of it
- dragging the header or the brand strip moves the window and the three buttons work (maximize turns into restore)
- every page opens with a description in full white, bigger than the row text; each section is one bordered block with rows separated by dividers and its title above the block, 8 px above it and 22 px below the previous block
- the About page: the app card shows the tile of the build (orange for a pre-release version, blue for a stable one), "Notch" and "Version x", and **Check now** is dimmed with "Coming soon"; **What's new** is dimmed until `WHATS_NEW_URL` is set; **Licenses** opens `THIRD-PARTY-LICENSES.txt`; **Lyrics** opens lrclib.net; under "Files", **Settings file** opens `settings.txt` and **Data folder** opens the folder in Explorer
- the text is Geist in every control, including the fields and buttons
- switch Windows between dark and light: the palette follows at once (`palette.rs`) and the sidebar tile stays the same (the tile does not depend on the theme)
- the window fits the screen: it opens at 1040 x 660 on a large display and smaller on a small one, never below 720 x 480 (see [the settings window](../reference/settings-window.md#size)), so check it on the smallest display you have
- change a setting and watch the notch apply it without a restart; reorder a page in Pages and check the order in the open notch
- there is no right-click menu on the notch, and no settings inside it: the gear is the only way in besides the tray

For lock/unlock recovery, enable Hide when idle, focus a maximized app, lock Windows, then unlock. Rest the pointer
on the hidden strip and verify that it reveals and opens. Switch between two maximized apps, a restored window and
the desktop: Notch must remain above ordinary app windows and hover must still work. Fullscreen hiding must still
win. Automated regressions demote an owned test window and reset stale interaction state; simulated unlock messages
exercise the running app, but do not replace this physical lock/unlock check.

For a change to the timer, seed a short session by hand (stop the notch, put `timer.state=running focus 30000 <end>` in
the settings file, where `<end>` is the Unix time in milliseconds 30 seconds from now, and start it) and check: the timer's icon and its time
show in the collapsed pill, the alert and sound arrive at the end, the open page then offers the break, and
pause, +5 min and stop work.

For a change to navigation, play something so the media page appears, then check: scrolling the wheel (and swiping a
touchpad, and scrolling sideways) moves one page per notch or gesture (quick extra notches inside 300 ms are ignored) and loops from the last page to the first and back, a
touchpad swipe moves one page and nothing more until you lift off, clicking a dot opens that page, and the notch opens on the page you left. Also check that opening, closing, switching to a page of another width and toggling lyrics all start gently, overshoot a little and settle, with width and height arriving together.

For a change to the shelf: drop files on the notch (it turns blue and the files drop in), check the pictures match Explorer, double-click one to open it, drag one into a folder (it is copied, and leaves the shelf), right-click one and press its cross to remove it, and check the original files are untouched.

For a change to the calendar: with a calendar linked, click days in the month grid (the list shows that day's events from the top, and today keeps its solid circle), scroll a list of more than five events with the wheel (the sixth card shows by half) and check that the wheel switches pages anywhere else, and close and open the notch (today again). An event 15 minutes away shows its reminder in the pill even when the calendar page is not shown.

For a change to activities, also check (each one is a real event, so run them on a laptop):

- maximize a browser: the notch shrinks to a strip, wakes at the very top edge after a short wait, and still shows the volume; press F11 (or play a video full screen): it disappears entirely until you leave full screen. `cargo test a_real_maximized -- --ignored` opens real windows to check the detection
- with two maximized apps and the notch hidden over them, Alt+Tab and click between them: the notch never flashes; restore one window and it comes back after about half a second
- with the pointer resting at the top edge, press Win+Shift+S: the notch disappears, does not open while you point at the top edge over the dimmed overlay, and is not in the picture you select; press Esc and it comes back after about half a second. With `NOTCH_DEBUG` set, the log shows `foreground class=... process=ScreenClippingHost.exe mode=Capture` (or `process=SnippingTool.exe`)
- change the volume and press mute: a bar with the value, or "Muted", for about a second and a half
- change the brightness with the slider and with the brightness keys: a bar with the value
- copy something: "Copied"; copy a password from a password manager: nothing
- press Caps Lock: "Caps Lock on", then an arrow in the group at the right edge that stays in the pill and in the open header while the lock is on; press it again: "Caps Lock off" and the arrow goes
- mute the speakers: "Muted", then a crossed speaker in the group at the right edge that stays in the pill and in the open header until you unmute
- start recording in any app: "Microphone on", then an orange dot at the right end of the pill's group until it stops, then "Microphone off"
- open the camera in a browser or a video-call app: "Camera on", then a green dot at the right end of the group, above the microphone's dot, then "Camera off"
- with both on, the two dots form one column; turn one off and the other keeps its row
- the dot column reserves no width: when a dot appears, the whole group (and the pill, if it was at its minimum) slides left by the dot and 8 px, and slides back when it goes; nothing snaps and nothing is left running afterwards
- on a laptop: the battery ring is always the item before the dots, with the arc the charge, dark green on battery, bright green while charging, red at 20 % or less, and the clock sits at the left edge
- on a PC without a battery and with nothing active (no timer, no music, no Caps Lock, no muted speaker, no dot): the empty pill shows only the time, centered, with 28 px on each side and narrower than 150 px; turn something on (Caps Lock) and the time glides to the left edge while the pill grows, then back when it goes; there is no weather icon, slot or PC icon
- idle group order: with the wave, Caps Lock, muted, a timer, an event and a prayer all showing, they read left to right wave, Caps Lock, muted, timer, event, prayer, then the ring and the dots at the right end; start something new (say the timer) and it appears between the time and the items already there while nothing right of it moves; timer, event and prayer show icon and text (the short form such as `12m` when two or more show), the others an icon only
- connected devices page (switch it on in Settings, Pages, with a Bluetooth device connected): a ring per device with its battery and name; 20 % or less is red; a device with no battery reading shows "On" on a full gray ring (a device that is off, "Off" on a dashed one); with Bluetooth off the page says so, and with nothing connected it says that; switch the radio off and on in Windows, then close and reopen the page to see it follow; `cargo test snapshots_of_the_devices_page_states -- --ignored` writes `devices-*.bmp` for each state
- hover the notch: the page dots at the left edge, the time in the exact middle, and the same group at the right, icons only and never text, 10 px apart, then the gear (a hexagon around a ring, grey, 14 px) at the right edge; the gear never shows in the idle pill; with a crowded header the time stays in the middle and the icons nearest it (the wave first, then Caps Lock) are left out, while the ring, the dots and the gear always show
- click the timer, event, prayer or wave icon and that page opens (Timer, Calendar, Shalat, Media); click Caps Lock, muted, the ring or a dot and nothing happens and nothing opens underneath; click the gear and the settings window opens (a second click focuses the open one)
- play music without a lyric line (or with lyrics off): the idle pill shows only the animated wave in the group, with no artwork and no title; it stays up under Hide when idle
- play music with a lyric line: the pill reads artwork, the line and then the group (muted, then Caps Lock, then at most one countdown, in icon and text), with 24 px around the line, and no time, ring or wave; with several countdowns running only the one with the least time left shows; turn on the camera and the dot appears 10 px after the artwork and the line slides 16 px further right, and it slides back when the dot goes
- plug and unplug the charger, and let the battery fall through 20 % and 10 %: the matching message
- receive a notification: a card with the app's logo for five seconds; point at it and it stays, with Dismiss and Open
  under the two lines, and goes 1.5 s after you leave. Dismiss clears it from the notification center; Open brings the
  app forward (send yourself a WhatsApp message: Open goes to that chat). `NOTCH_DEBUG` logs which way opened it
- the optional activities are Notifications and the device battery warning (General > Notifications); untick one and verify it stays off after restarting
- clipboard page (switch it on in Pages): copy text, a link and an image and they appear newest first, at most seven; double-click a row to paste into the window in front (the notch closes after about 600 ms); right-click removes one and the bar offers **Undo** for 5 s; switch the page off and on and the history is empty
- weather rain alert (Settings > Weather): a 340 px banner with **Dismiss** and **Open** that stays while the pointer is on it, then a blue "Rain in N min" badge beside the clock and the battery ring (and a cloud in the open header) until the rain starts; **Open** (and a click on the header cloud) shows the Weather page; with the Weather page open, an hour opens weather.com hour by hour and a day opens its ten day view and takes a faint tint under the pointer
- Hide Windows popups (General > Windows popups): change the volume with the volume box switched off and check that only the notch shows; switch hiding notification banners on, receive a notification and check that Windows' banner stays away while the notch's card shows (see [hide Windows' own popups](hide-windows-popups.md))
- old disable settings for volume, brightness and other required activities do not suppress them
- `notch.maximized=music` keeps music without lyrics and the idle clock visible over a maximized window, even with Hide when idle on
- `notch.maximized=lyrics` (the default) keeps only active lyrics and activities; `activities` keeps only activities; `hidden` takes the notch off the screen
- `notch.fullscreen=activities` shows a notification banner over a full-screen video but hides the idle clock to the strip; `hidden` (the default) shows nothing
- a settings file with only the old keys (`notch.show_over_maximized=on`) behaves as before and gains the matching new key at start
- open the tray menu with Windows in dark mode: the menu is dark and lists Start with Windows, Settings and Quit Notch (a line above Quit); tick "Start with Windows" and check the entry under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
