# Notch

<img src="assets/branding/tile-stable.png" alt="Notch logo" width="96" />

Notch puts a slim black pill at the top of your Windows screen. Point at it and it opens into a media player with synced lyrics, a file shelf, a calendar, a focus timer, prayer times, a todo list and system stats. Short messages (volume, notifications, Caps Lock) appear in the pill while they happen.

It is written in Rust and draws straight to the screen with Direct2D: no UI framework, no web view. It idles at 0 % CPU and stays under 50 MB of memory.

![The open notch on the media page, playing a song with its synced lyrics beside the controls](assets/images/notch-lyrics.png)

## Install

Run `notch-<version>-windows-x64-setup.exe`. The installer is not code-signed, so Windows may show a SmartScreen warning. A stable installer offers **Start Notch when Windows starts** (on by default); you can also switch it later in the settings window (General page).

Requirements: Windows 10 or 11, 64-bit (tested on Windows 11). The media page needs a player that appears in the Windows media controls.

**Updating.** Notch does not check for new versions. Run the newer `-setup.exe`: a stable installer sees the installed stable copy, says which version it has and which it would put in its place, and asks whether to continue. If you say yes, it closes the running notch, replaces it and starts it again. Your settings stay.

**Pre-releases.** A version with a suffix (`0.5.0-alpha.1`, beta, rc) installs to its own folder as `Notch <version>`, with its own Start menu and Add or remove programs entry. It never asks and never replaces another copy, so several can sit side by side. It has no start-with-Windows option. All copies share `%APPDATA%\notch\settings.txt`, and the settings window opens once across copies.

## What it includes

Pages are what you see when you open the notch; the time sits in the header of every one.

- **Media:** the current track with synced lyrics, line by line. While music plays without a lyric line, the collapsed pill shows an animated wave at its right end. With a lyric line, the pill shows the artwork and the line.
- **Shelf:** drop files to keep them for a while: six per row, two rows at most, so twelve. A full shelf refuses a drop and says so in red.
- **Calendar:** the whole month with Monday first (today a solid blue circle, the day you pick a light blue one, Sundays red), and all the events of the day you pick, each with its start time ("All day" for an all-day event). Five cards fit; with more, the sixth shows by half and the list scrolls with the wheel, a swipe or a drag, with no scrollbar. A past event leaves today's list and "Done for today" shows when all are over.
- **Timer:** a focus timer with short breaks and a long break after four sessions; the main button is orange (blue for a break) and Stop is red.
- **Shalat** (off until you add it): prayer times for any city, worked out on your PC with no internet. It shows the next prayer in large type with a countdown, and the day as a sun arc through the five prayers, filled up to now. It puts a mark in the collapsed pill 30 minutes before, and shows a short message when a prayer starts. You choose the city, the calculation method and a few minutes of adjustment per time.
- **Todo:** a list with no limit that shows five tasks and scrolls for the rest (no scrollbar). Add is a button at the top right: press it, type and press Enter. Clear all removes every task. Tick a task off, or point at it to copy its text or delete it. After a copy the copy icon turns into a green check for 1.5 seconds. After a delete or Clear all, Undo brings the tasks back for 5 seconds; it undoes only the last removal, so a later delete replaces it. In the text field, Ctrl+Backspace and Ctrl+Delete remove a whole word. The list is the file `%APPDATA%\notch\todo.txt`.
- **System stats** (off until you add it): five fixed-size cards: CPU, memory and up to 2 GPUs with a load chart (a dotted line and two bars), the network rates and up to 3 disks.
- **Weather** (off until you add it): the temperature now and the next hours for your location, or the next days (the "Today" / "Week" pill at the top of the left column switches), and an optional alert 30 minutes before rain: a banner that stays while the pointer is on it, then a countdown badge beside the clock until the rain starts; click the badge to open the weather page. Click an hour (hour by hour view) or a day (ten day view) to open that forecast on weather.com in your browser; the forecast data itself comes from MET Norway. Under each hour the page shows the rain in millimetres when there is rain. The location is the one you pick in the settings window (General), shared with the prayer times.
- **Clipboard** (off until you add it): your last 7 copies (text, links, code, colors and pictures) on the left, each picture shown as a small thumbnail of itself, and a preview of the selected one on the right. Click selects, double-click pastes into the window you were working in (or copies it again and moves it to the front when there is nowhere to paste), right-click removes it with a 5 second Undo, and the Delete, Copy and Paste buttons do the same. Point at a row and click its pin to keep it on top: up to 3 copies stay pinned, in the order you pinned them, and are never pushed out by new copies. Nothing is saved: the history lives in memory only and is gone when Notch quits or when you switch the page off. A copy from a password manager (one that asks not to be recorded) is skipped, text over 100 KB is skipped, and a large picture is kept at a reduced size.
- **Connected devices** (off until you add it): the battery of the Bluetooth mouse, keyboard, headphones, controller or pen connected to Windows, one ring per device (red at 20 % or less, "On" on a full gray ring when Windows does not report a battery, dimmed with "Off" for a paired device that is switched off). The page is as wide as its devices; with more than five, the five lowest batteries show. It reads Windows when you open the page and once a minute while it shows, and does nothing otherwise. A mouse, keyboard or controller on a 2.4 GHz USB receiver shows too; a Logitech receiver's device shows its battery while it is awake, any other one shows "On". "Bluetooth is off" shows only when no receiver device is there to list.

Activities are what the collapsed pill shows while something happens: volume, brightness, charger and battery, Bluetooth devices (connecting, and a low battery at 20 % and 10 % on a connected mouse, keyboard or headphones), microphone in use, camera in use, copied to the clipboard, Caps Lock, Windows notifications, and browser downloads. A notification opens the pill into a card with the app's logo and two lines of the message for five seconds. Point at it to keep it on screen and get two buttons: Dismiss clears it from the Windows notification center, and Open brings up the app, in apps such as WhatsApp at the conversation the notification came from (an app that is already open comes forward instead of starting a second window). When the notification offers a quick reply, as WhatsApp's does, Reply is the main button: it turns the buttons into a text field, Enter sends the reply to the app and Escape cancels. The rain alert of the weather page offers Dismiss and Open (Open shows the weather page in the notch) the same way.

A browser download shows a small card when it starts (the file's name in Firefox, "Preparing download…" in Chrome, Edge and Brave, which name the file only at the end) and a download icon in the group at the right while any runs. When it finishes, a card shows the file's icon, name and size for five seconds; point at it to get **Open** and **Show in folder**. A cancelled download shows no finish card. Notch watches the file names in your Downloads folder only: a browser that saves elsewhere shows nothing, there is no percentage, and a file already half-downloaded when Notch starts is ignored. Switch it off with **Browser downloads** in Settings > General.

The collapsed pill reads, left to right: the time at the left edge, then one group at the right edge. The group is the same in the open notch, always in this order: a wave while music plays, an arrow while Caps Lock is on, a crossed speaker while the sound is muted, a timer, the next calendar event, the next prayer (the last three with their countdown, for example `12m`), then, on a laptop, a ring for the battery charge, then a small dot for the camera (above) and the microphone when one is in use. A new mark appears between the time and the ones already there, and nothing to its right moves; marks slide when they come or go. A PC with nothing active shows only the time, centered in a small pill. While the notch is open the page dots sit at the left edge, the time stays in the exact middle, and the same group shows as icons only, followed by the pin and the settings gear at the right edge; when the notch is too narrow for every icon, the ones nearest the time are left out, and a click on the timer, event, prayer or wave icon opens its page. While a lyric line shows, the pill is the artwork, the line and a shorter group: muted, Caps Lock and the one countdown with the least time left, shown as its icon alone; the camera and microphone dot stays at the right end, where it sits without lyrics.

## Use it

- **Open and close.** Point at the pill to open it. Move away to close it.
- **Pin it open.** Click the pin left of the gear in the open notch and it turns blue (the pin and the gear light up under the pointer): the notch stays open with the pointer away, and over maximized and full-screen windows whatever their level or Hide when idle say, until you click the pin again. A screenshot overlay still hides it, and it comes back pinned. Notch always starts unpinned. Files cannot be dropped on a pinned notch: unpin it and drag them onto the collapsed pill.
- **Switch pages.** Scroll the wheel, swipe two fingers on a touchpad, or click a dot. One wheel notch or one swipe moves one page, and scrolling loops from the last page to the first. The page switches in a quick fade with a short shift from the side it comes from; it never stops between two pages. The notch opens on the page you left. While the open notch is under the pointer, the number keys jump to a page in the order set in Settings: 1 to 9 for the first nine, 0 for the last (those keys go to the notch, not to the app in front, only for that time, and not while you type in a page's field).
- **Media.** Click the buttons. The speech-bubble button turns lyrics on and off.
- **Shelf.** Drag files onto the notch (not while it is pinned open); it turns blue when it is ready. It shows up to twelve files, six to a row. Pictures show a thumbnail and other files a small drawn document for their type (PDF, text, archive, spreadsheet and so on). Double-click a file to open it, drag it out to copy it into another app, and right-click it or press the cross on its corner to remove it from the shelf.
- **Calendar.** Click a day of the month to list its events; **Settings > Calendar** can show only the current week instead, on a shorter page (the list scrolls past two events). Paste your calendar's iCal address into **Settings > Calendar**, which lists the Google Calendar steps for finding it. Treat the address like a password.
- **Settings gear.** The gear at the right edge of the open header opens the settings window. On the shelf, right-clicking a file removes it; the notch has no other right-click menu.
- **Tray menu.** Right-click the tray icon (on Windows 11 it may sit under the `^` menu): **Pause Notch**, **Settings** (the settings window) and **Quit Notch**. **Pause Notch** opens a list: For 10 minutes, For 30 minutes, For 1 hour, Until I resume. While paused, the menu shows "Paused until 14:30" (or "Paused") and **Resume Notch** in its place.
- **Settings window.** The gear and **Settings** in the tray menu open it. Settings are edited only there, not inside the notch. The sidebar lists General, Pages, Behavior and Advanced under "Notch", Calendar, Shalat, System stats and Weather under "Page settings", and About at the bottom; each page opens with a sentence on what it is for, then its sections as bordered blocks of rows, each with a description and a control. The sidebar sits on a translucent neutral frame with the app's tile logo on top (blue in a stable build, orange in a pre-release), and the page is one solid card in front of it with its own header ("Settings › page") and the minimize, maximize and close buttons; icons and the page title are blue. On a small display it opens smaller than its default size, never below 720 x 480. **About** shows the version, links to the licenses, the lyrics source and the release notes (not available yet), and opens the settings file and the data folder; its **Check now** button is not available yet. The city search for the prayer times and the weather is in General, under the **Track location** switch (on by default): while it is on the city changes by itself when the device moves to another one, and a city you pick stays until then; switch it off to keep a city for good. Toggles apply at once; type a value and press Enter, or click away, to save it. Which disks and GPUs the stats page shows are chosen under System stats (the lists appear after the stats page has been shown once); the message before rain is the **Rain alert** switch under Weather (see [docs/reference/settings-keys.md](docs/reference/settings-keys.md)). The **Pause Notch** group on General pauses the notch for 10 minutes, 30 minutes, an hour or until you resume it; while paused it shows when the pause ends and a **Resume** button.

[docs/reference/settings-keys.md](docs/reference/settings-keys.md) lists every setting and its key.

| Option | Default |
| --- | --- |
| Language | Automatic (follows the Windows display language: Indonesian when it is Indonesian, English otherwise) |
| Notifications | On |
| Device battery warnings | On |
| Browser downloads | On |
| Hide when idle | Off |
| Hide the Windows volume and brightness popups | On |
| Hide Windows notification banners | On |
| Over a maximized window | 2 · Lyrics |
| Over a full-screen app | 0 · Hidden |
| Open by pointing over a maximized window | On |
| Open by pointing over a full-screen app | On |
| Show the notch on | Main screen |
| Start with Windows | On in the installer |

**Show the notch on** (Settings > General, group Screen) puts the notch on the main screen (the primary display) or on the focused screen, the one with the window in front. It applies at once. The distance from the top (Settings > Advanced) is measured from the top of the screen the notch is on.

**Language** (Settings > General) switches the notch and the settings window between English and Bahasa Indonesia while both run; Automatic follows the Windows display language. Notification titles and texts, track names and event titles stay as the sending app wrote them.

**Hide when idle** keeps the notch hidden over regular windows until a short message or a banner arrives (a volume change, "camera active"); a lasting mark (Caps Lock, a running countdown, the camera or microphone dot) does not bring it back, in any mode, so while the notch is hidden the privacy dot is not visible. Playing music keeps the pill up. The muted speaker is a lasting mark like the others: it does not keep the pill up.

Defaults apply when no choice has been saved. Choices saved with older versions carry over.

### Over maximized and full-screen windows

Over a **maximized** window and over a **full-screen** app (a game, a video, an F11 window), the notch follows its own choice for each, as a level: 0 Hidden, 1 Activities (messages only, such as a volume change), 2 Lyrics (lyrics and messages) or 3 Always visible (the idle pill as well). Every level also shows everything below it. Set them in **Settings > Behavior**. They take precedence over **Hide when idle**. The same section can stop the pointer from opening the notch over either kind of window. A pinned notch stays open over both, whatever the level. The notch hides at once and comes back half a second after such a window leaves the front, so switching between two maximized windows does not flash it. While a screenshot overlay is up (Win+Shift+S, Print Screen, the Snipping Tool, the Xbox Game Bar), the notch is hidden and pointing at the top edge does not open it, whatever these settings say.

Volume and brightness are the exception while **Hide the Windows volume and brightness popups** is on: the notch is then their only feedback, so a volume or brightness change, and a mute or unmute, shows its level message over every level, Hidden included, and with Hide when idle, then the pill hides again; the muted speaker that stays afterwards follows the word like any lasting mark. Pointing at the top edge still follows the level. Muting shows the same bar as a volume change, with a crossed speaker and the level the sound returns to. With the switch off, Windows shows its own box and the levels apply to volume and brightness as well. A screenshot overlay hides them too.

### Windows' own popups

While the notch is paused (tray or Settings > General), it is hidden completely and Windows' own volume and brightness box and notification banners show as usual; both are hidden again when the pause ends. A pause wins over the pin, the maximized and full-screen levels and the priority exception above. Notch always starts unpaused. See [docs/reference/settings-keys.md](docs/reference/settings-keys.md#pausing-the-notch).

**Hide the Windows volume and brightness popups** (Settings > General, on by default) keeps away the box Windows draws above the taskbar for both, so only the notch shows them. The box's window gets an empty shape, so Windows can show it as often as it likes and nothing appears; the shape is put back when the notch closes or the switch goes off. If the notch is ended by force, the box stays away until you start the notch and close it again, or restart Explorer.

**Hide Windows notification banners** (Settings > General, on by default) turns off the banner Windows shows at the bottom right for every app while the notch runs, so only the notch shows notifications. Notifications stay in Windows' notification center, which is where the notch reads them. Each app's previous choice is saved first (also at the first start, when the switch is on without you having set it) and put back when you turn the switch off, when the notch quits, or at the next start after a crash. An app that has never shown a notification gets its first Windows banner before the notch can turn it off. [docs/how-to/hide-windows-popups.md](docs/how-to/hide-windows-popups.md) has the details and what is not yet confirmed, such as the brightness popup.

After you unlock Windows or resume from sleep, Notch moves back above app windows and forgets where the pointer was, so the hidden strip can open again; a pinned notch stays open. It does not take focus from the app you are using.

## Limitations

- Windows only, centered at the top of one screen: the main screen, the focused one, or a chosen one (`notch.monitor=chosen` with `notch.monitor_id`) with **Show the notch on**; **A chosen screen** adds a list of the connected screens. Not yet tested on more than one monitor (a two-monitor check is pending). The animation frame rate stays that of the main display. The focused screen follows changes of the window in front only: a window dragged to another screen without becoming the front window does not move the notch. A change of resolution or scale while running places the notch again, centered.
- Microphone and camera show that an app has the device open, not that it is recording. An app that keeps the microphone open while you are muted (Google Meet in a browser) keeps the orange dot. The camera is found through the Windows Frame Server log, so it misses some older DirectShow apps and a camera that was already on at start-up.
- Notifications show about half a second after Windows shows its own banner (a few seconds when Windows numbers a toast in an unexpected way), and only as a card: there is no list of past ones. Open reaches the conversation only when the app put it in the notification (WhatsApp does; Discord does not, so it opens the app). `notifications.show_text=off` in the settings file leaves the text out.
- The calendar reads an iCal address, not a Google sign-in. An event that crosses a daylight-saving change can show an hour off.
- The shelf keeps the paths of files, not copies. Dragging a file out copies it and never moves or deletes the original.
- Brightness follows the laptop panel only.
- Prayer times are within a few minutes of published tables. Check them against your local authority before relying on them.
- Notch is at version 0.5: expect rough edges.

## Resource use

The numbers below are the latest recorded run of `scripts/perf.sh`, on 2026-11-03: Windows 11 Home build 26300, Ryzen 5 8645HS, 15.3 GiB RAM, Rust 1.97.1, release build. The test used the shelf page, 20-second idle samples and 10 open/close cycles. CPU is a percentage of one core; RAM is private working set.

| Measurement | Result | Limit |
| --- | ---: | ---: |
| Initial RAM | 24.3 MiB | 30 MiB |
| Collapsed idle CPU | 0 % | 0.1 % |
| CPU time per open/close cycle | 131 ms | 400 ms |
| RAM growth over 10 cycles | 0.6 MiB | 3 MiB |
| Handle growth over 10 cycles | -9 | 20 |
| Open events | 20 of 20 | exactly 20 |
| Idle CPU after the cycles | 0 % | 0.1 % |

Every gate passed in this run. The run did not measure music playback, live lyrics, the notch held open by the pointer or swiping, and one passing run does not guarantee the same result on every machine. The settings window is a separate program with a higher footprint while it is open (about 45 MiB private working set); see [docs/reference/performance-baseline.md](docs/reference/performance-baseline.md).

## Privacy

Notch runs on your computer, with no account and no analytics. Everything below stays on your machine unless it is listed under "Leaves your computer".

**What it reads, locally**

- What is playing, from the Windows media controls.
- The text and app of your Windows notifications.
- Whether the clipboard changed. It never reads what you copied, unless you switch the **Clipboard** page on: then it reads copied text and pictures (see below).
- The names of the files in your Downloads folder, to tell when a browser download starts and finishes; their contents are never read.
- The paths of the files on your shelf, and the todo list in `%APPDATA%\notch\todo.txt`.
- Which window is in front, only to tell whether it is maximized or full screen.
- Caps Lock, through a keyboard hook that looks at that one key and keeps nothing it sees.
- CPU, memory, GPU, disk and network readings for the stats page.
- What you copy (text and pictures), only while the Clipboard page is on: the last 7 copies are kept in memory, never written to disk, never sent anywhere, and dropped when Notch quits or the page is switched off.
- The location: a city you pick, or, while **Track location** is on, the city nearest to the position Windows reports (asked once at start, after sleep and after a network change; the notch itself sends nothing for this, and Windows' own location service may use Wi-Fi or the internet). The city list ships with the app and prayer times are calculated on your PC from it; only the Weather page sends the place's coordinates anywhere (see below).

Settings are in `%APPDATA%\notch\settings.txt`. A stable installer adds a "Start with Windows" entry under your user's `Run` key unless you untick it; a pre-release installer adds none.

**Leaves your computer**

- **Lyrics.** When a track starts, Notch asks [LRCLIB](https://lrclib.net) for synced lyrics and sends the artist, title, album and length. The player itself (Spotify, a browser) is only read through Windows, never contacted. Set `NOTCH_LYRICS=off` to stop lookups; the lyrics button only hides them.
- **Calendar.** If you set a calendar link, Notch downloads that file every 15 minutes. The link sits in the settings file, so treat that file like a password.

- **Weather.** Only if you add the Weather page and choose a location: Notch asks the [MET Norway Weather API](https://api.met.no) for the forecast at the coordinates of that place, at start and every half hour, and sends the name "Notch" with its version. Your IP address reaches MET Norway as with any web request; no account and no other data is sent. Clicking an hour or a day on the Weather page opens weather.com in your default browser, with the place's coordinates (numbers only, no name) in the address; the data still comes from MET Norway, and nothing is opened otherwise.

## For developers

New to the project? Read [ONBOARDING.md](ONBOARDING.md) for the setup, the structure and the rules the code follows. [AGENTS.md](AGENTS.md) and `.claude/rules/` hold the rules for AI coding agents, [ARCHITECTURE.md](ARCHITECTURE.md) gives the overview of the code, and [docs/README.md](docs/README.md) indexes the documentation.

```
cargo build --release
target\x86_64-pc-windows-msvc\release\notch.exe
```

## Acknowledgements

Notch is inspired by Apple's Dynamic Island. Apple and Dynamic Island are trademarks of Apple Inc.; Notch is not affiliated with or endorsed by Apple.

It builds on public APIs and resources:

- **Windows APIs** for media controls, notifications, window events and the tray, through the [`windows`](https://crates.io/crates/windows) crate.
- **[LRCLIB](https://lrclib.net)** for synced lyrics.
- **[GeoNames](https://www.geonames.org)** for the prayer-time city list: cities of 15,000 people or more, reduced to names, countries and coordinates (`scripts/build-cities.sh`), under [Creative Commons Attribution 4.0](https://creativecommons.org/licenses/by/4.0/).
- **[MET Norway](https://api.met.no)** (CC BY 4.0) for the weather forecast: data from the Norwegian Meteorological Institute.
- **[salah](https://crates.io/crates/salah)** (MIT) for the prayer-time calculation, kept in `vendor/salah` with one crash fix (see `vendor/salah/VENDORED.md`). A Jakarta check against Aladhan's Kemenag method was 0 to 2 minutes later.
- **[Geist](https://vercel.com/font)** for the font and **[Lucide](https://lucide.dev)** (ISC) for some icons.
- **[GPUI](https://www.gpui.rs)** and gpui-component for the settings window.
- **iCal** (RFC 5545) calendar addresses, such as the one Google Calendar gives you.

## License

Proprietary. Copyright © 2026 hibatillah. All rights reserved; see [LICENSE](LICENSE). Third-party components keep their own licenses, listed in `THIRD-PARTY-LICENSES.txt` and under **Licenses** on the settings window's About page.
