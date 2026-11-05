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
states, the short messages, the notification banner and the shelf with files dragged over it. Look at the pictures
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
- play, pause, previous, next, shuffle and repeat work, and the hovered button gets a faint wash
- clicking and dragging the progress bar jumps to that point
- the lyric button hides and shows lyrics, and the choice is remembered after restarting
- the tray icon's Quit closes the notch and removes the icon

For a change to the notch's frame or hiding: open the notch and check the dots, the status marks and the time, tick "Hide when idle" in the tray menu and check that the notch
disappears, shows its small pill when you rest the pointer on the top of the screen, and opens if you stay. To check the "Edit pages" view, set `SETTINGS_BUTTON_SHOWN` to true in `chrome.rs` and
add, drag, double-click and remove a page. The mouse-driven checks need an unlocked screen.

For lock/unlock recovery, enable Hide when idle, focus a maximized app, lock Windows, then unlock. Rest the pointer
on the hidden strip and verify that it reveals and opens. Switch between two maximized apps, a restored window and
the desktop: Notch must remain above ordinary app windows and hover must still work. Fullscreen hiding must still
win. Automated regressions demote an owned test window and reset stale interaction state; simulated unlock messages
exercise the running app, but do not replace this physical lock/unlock check.

For a change to the timer, seed a short session by hand (stop the notch, put `timer.state=running focus 30000 <end>` in
the settings file, where `<end>` is the Unix time in milliseconds 30 seconds from now, and start it) and check: the ring
and the time show in the collapsed pill, the alert and sound arrive at the end, the open page then offers the break, and
pause, +5 min and stop work.

For a change to navigation, play something so the media page appears, then check: scrolling the wheel (and swiping a
touchpad, and scrolling sideways) moves one page per notch (several quick notches go several pages) and loops from the last page to the first, a
touchpad swipe drags the pages along and settles on the nearest one, clicking a dot opens that page, and the notch opens on the page you left.

For a change to the shelf: drop files on the notch (it turns blue and the files drop in), check the pictures match Explorer, double-click one to open it, drag one into a folder (it is copied, and leaves the shelf), right-click one and press its cross to remove it, and check the original files are untouched.

For a change to the calendar: with a calendar linked, click each day of the week (the events change, and today stays blue), and close and open the notch (today again).

For a change to activities, also check (each one is a real event, so run them on a laptop):

- maximize a browser: the notch shrinks to a strip, wakes at the very top edge after a short wait, and still shows the volume; press F11 (or play a video full screen): it disappears entirely until you leave full screen. `cargo test a_real_maximized -- --ignored` opens real windows to check the detection
- change the volume and press mute: a bar with the value, or "Muted", for about a second and a half
- change the brightness with the slider and with the brightness keys: a bar with the value
- copy something: "Copied"; copy a password from a password manager: nothing
- press Caps Lock: "Caps Lock on" and an arrow beside the time while the notch is open; press it again: "Caps Lock off"
- mute the speakers: "Muted", and a crossed speaker beside the time while the notch is open
- start recording in any app: "Microphone on", then an orange dot until it stops, then "Microphone off"
- open the camera in a browser or a video-call app: "Camera on", then a green dot, then "Camera off"
- plug and unplug the charger, and let the battery fall through 20 % and 10 %: the matching message
- receive a notification: a banner with the app's logo for about four seconds; point at the notch while it shows and it stays until you leave
- the activity section contains only Notifications; untick it and verify it stays off after restarting
- old disable settings for volume, brightness and other required activities do not suppress them
- `notch.maximized=music` keeps music without lyrics and the idle clock visible over a maximized window, even with Hide when idle on
- `notch.maximized=lyrics` (the default) keeps only active lyrics and activities; `activities` keeps only activities; `hidden` takes the notch off the screen
- `notch.fullscreen=activities` shows a notification banner over a full-screen video but hides the idle clock to the strip; `hidden` (the default) shows nothing
- a settings file with only the old keys (`notch.show_over_maximized=on`) behaves as before and gains the matching new key at start
- open the tray menu with Windows in dark mode: the menu is dark; tick "Start with Windows" and check the entry under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
