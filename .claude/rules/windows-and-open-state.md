---
paths:
  - "crates/notch/src/host/**"
  - "crates/notch/src/gfx/**"
  - "crates/notch/src/platform/**"
  - "Cargo.toml"
---

# Windows specifics and the open state

- `windows = 0.62.2` and `windows-numerics = 0.3` move together; `Matrix3x2` and `Vector2` come from
  `windows_numerics`. `.cargo/config.toml` sets the `x86_64-pc-windows-msvc` target because the GNU toolchain has no
  linker here.
- The window is layered with per-pixel alpha and resizes with the pill, which can lose mouse tracking. The code that
  closes a notch opened by the pointer keeps three safeguards: `TrackMouseEvent` re-armed on every move,
  `WM_MOUSELEAVE` checked against the cursor and window rect, and a 120 ms hover-watch timer while open and not
  pinned. `WM_MOUSELEAVE` is defined as `0x02A3` in `host` because the `windows` crate does not export it.
- The pill is black and flush with the top edge. Concave "flare" fillets join its sides to the top of the screen, like
  a MacBook notch, and the window region includes their width.
- Enable `windows` crate features one API at a time in `Cargo.toml`.
- An undocumented Windows entry point (the dark tray menu in `host/tray.rs`) is looked up by `GetProcAddress` at run
  time and skipped when missing, so a Windows version without it keeps working.
- `track_mouse_leave` re-arms only while the pointer is over the notch window; re-arming under another window posts an
  endless stream of `WM_MOUSELEAVE`.
- Session lock, unlock and resume from sleep clear the pointer, drag and scroll-gesture state
  (`reset_session_interaction`) and refresh the foreground window mode; a pinned notch keeps its pin and stays open,
  and the drop overlay is told again that it is open. The window registers for `WM_WTSSESSION_CHANGE` for that.
- With `notch.monitor=focused` the window moves between monitors (`host/notch/relocate.rs`, rules in
  `host/monitor/choose.rs`): the move waits while the notch is open or pinned, `WM_DPICHANGED` is not applied (the
  notch sets its own size and `Surface::resize` keeps the render target), and the judged full-screen monitor is the
  notch's current one. No timer, thread or hook serves it; it rides the foreground hook and `WM_DISPLAYCHANGE`.
- `restore_topmost` re-asserts `WS_EX_TOPMOST` before the notch opens for the pointer or takes drop input, because a
  hidden-over-maximized notch drops it.
- `GetCurrentInputMessageSource` tells a touchpad scroll from a wheel; when a driver gives no source, `scrolled()` falls
  back to the delta size and the active gesture.
- Hiding Windows' own popups is reversible by design: the volume and brightness flyout gets an empty window region, found by a hook on Explorer's show event
  (`host/volume_flyout.rs`), which is put back when the notch exits or the switch goes off (a forced end leaves it until the next start (a new copy puts it back)); the banners are turned off per app with the previous
  values saved to `%APPDATA%\Foldline\notification-banners.txt` (`host/notification_banners/passes.rs`) and restored on
  switch-off, on quit and at the next start after a crash. Both settings are on by default (an unset key counts as on,
  through `wants_hidden`), so the first start already saves the previous values. Keep both undoable before changing either.
- The screen-mode hook follows the foreground window only (`host/screen_mode/location_hook.rs`), and `ScreenMode::Capture`
  hides the notch over the Win+Shift+S overlay (not yet verified in the real app).
- A pin (`host/notch/pin.rs`) keeps the notch open until unpinned and overrides `notch.maximized`, `notch.fullscreen`
  and `notch.hide_when_idle` (`visibility::pin_shows`); only `ScreenMode::Capture` hides it, and it returns pinned.
  Every close path (pointer leave, the close timer, `request_close`, a drag that ends away) leaves a pinned notch open.
  A pinned notch takes no file drops: the drop overlay, the only drop target, steps aside while the notch is open.
