# Add a page

> **How-to.** Read when: you are writing your first page and want one working example to copy. Skip if: you already know the `Page` trait; use [the page API reference](../reference/page-api.md) instead.

You build a small greeting page that follows the time of day and switches between casual and formal wording on a click. Each step ends with something you can check.

Each page sets its own open size in `PageSizes::expanded`. Most pages use `DEFAULT_CONTENT_HEIGHT` (86
device-independent pixels); the notch adds 50 for the header and padding (136 total). A page may ask for more, up to
`MAX_CONTENT_HEIGHT` (220), and the notch grows to fit; taller requests are capped there.

## Steps

### 1. Create the folder

Make `crates/notch/src/pages/<name>/mod.rs`. A page may be one file or many; keep everything it needs inside its folder. A page
can use `page`, `activity`, `gfx`, `ui` and `platform`, but never another page.

### 2. Implement `Page`

A page needs five methods; the rest are optional hooks. This is a complete page, a greeting that follows the time
of day and switches between casual and formal wording when you click it while the notch is open. Copy it to
`crates/notch/src/pages/greeting/mod.rs`:

```rust
//! A greeting that follows the time of day. Click it while open to switch between casual and formal wording.

use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::{
    gfx::{Align, Canvas, Color, FontStyle, Rect, UI_FONT, Size, Weight},
    page::{Context, Frame, Mode, Page, PageSizes, PointerEvent, Priority, RESTING_HEIGHT, TimerId},
};

const RESTING_STYLE: FontStyle = FontStyle::new(UI_FONT, 13.0, Weight::Medium, Align::Center);
const OPEN_STYLE: FontStyle = FontStyle::new(UI_FONT, 22.0, Weight::Medium, Align::Center);
/// Settings keys start with the page id, so pages never collide.
const SETTING_FORMAL: &str = "greeting.formal";
/// The greeting changes at most once an hour, so a timer once a minute keeps it right without waking the CPU often.
const REFRESH_TIMER: TimerId = TimerId(0);
const REFRESH_INTERVAL_MILLISECONDS: u32 = 60_000;

pub struct GreetingPage {
    is_formal: bool,
}

impl GreetingPage {
    pub fn new() -> Self {
        Self { is_formal: false }
    }

    fn text(&self) -> &'static str {
        let hour = unsafe { GetLocalTime() }.wHour;
        match (hour, self.is_formal) {
            (5..=11, false) => "Morning!",
            (5..=11, true) => "Good morning",
            (12..=17, false) => "Afternoon!",
            (12..=17, true) => "Good afternoon",
            (_, false) => "Evening!",
            (_, true) => "Good evening",
        }
    }
}

impl Page for GreetingPage {
    fn id(&self) -> &'static str {
        "greeting"
    }

    /// `NORMAL` beats the `BACKGROUND` priority the other pages answer open, so this page opens first.
    fn priority(&self, _mode: Mode) -> Option<Priority> {
        Some(Priority::NORMAL)
    }

    fn sizes(&self) -> PageSizes {
        PageSizes { resting_width: 150.0, expanded: Size::new(320.0, 86.0) }
    }

    fn draw_resting(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        let bounds = frame.bounds;
        let row = Rect::new(bounds.left, bounds.top, bounds.right, bounds.top + RESTING_HEIGHT);
        canvas.text(&RESTING_STYLE, self.text(), row, Color::WHITE.faded(opacity));
    }

    fn draw_expanded(&self, canvas: &Canvas, frame: &Frame, opacity: f32) {
        canvas.text(&OPEN_STYLE, self.text(), frame.bounds, Color::WHITE.faded(opacity));
    }

    fn on_start(&mut self, context: &mut Context) {
        self.is_formal = context.settings.get_bool(SETTING_FORMAL, false);
        context.set_timer(REFRESH_TIMER, REFRESH_INTERVAL_MILLISECONDS);
    }

    fn on_timer(&mut self, _timer: TimerId, context: &mut Context) {
        context.request_repaint();
    }

    fn on_settings_changed(&mut self, context: &mut Context) {
        self.is_formal = context.settings.get_bool(SETTING_FORMAL, self.is_formal);
        context.request_repaint();
    }

    fn on_pointer(&mut self, event: PointerEvent, frame: &Frame, context: &mut Context) {
        if let PointerEvent::Released(point) = event
            && frame.is_open()
            && frame.bounds.contains(point)
        {
            self.is_formal = !self.is_formal;
            context.settings.set_bool(SETTING_FORMAL, self.is_formal);
            context.request_repaint();
        }
    }
}
```

What each part does:

- `priority` answers "do you want to be shown?" for the collapsed and the open notch. `None` means no. Collapsed, the
  highest priority wins; `Priority::NORMAL` beats `BACKGROUND`. Open, every page that answers `Some` can be
  reached by scrolling or by the page dots, and the highest priority is only the page shown first.
- Report the size of your content only; the notch adds 32 px to the width and 50 px to the height for its padding and
  header. Ask for `DEFAULT_CONTENT_HEIGHT` unless the content needs more; the cap is `MAX_CONTENT_HEIGHT`.
- `sizes` is the size you want, in device-independent pixels. Ask again whenever your content changes size; the host
  asks after every event and animates the change.
- Resting content uses `RESTING_HEIGHT` (33 DIP); the host adds 3 DIP below it and enforces a minimum collapsed width of 150 DIP. `resting_width` is your content only: when a page claims the pill, the host adds 12 DIP at each edge and room for the dot column and for any badges. A page that draws something at its left end can answer `resting_lead_width()` and read `Frame::lead_gap` for the gap the host keeps after it (24 DIP, or what `resting_lead_gap()` answers); the media page does this (see [page API](../reference/page-api.md)).
- `draw_resting` and `draw_expanded` draw your content in device-independent units: `frame.bounds` starts at zero,
  the host turns units into pixels, and the canvas's `scale()` is 1. Fade your colors with the `opacity` you are
  given so the content follows the open and close animation. Only `draw_backdrop` draws in device pixels.
  During navigation, each visible page keeps its requested width in `frame.bounds` and `frame.expanded.width`.
  The notch clips the visible portion; do not stretch your content to the neighbouring page's width. Pointer events
  use the same shifted frame as drawing, including while the user rests between pages.
- `on_start` runs once. Read settings and start timers or background work here.
- `on_settings_changed` runs after another program (the settings window) changed `settings.txt` and the notch re-read
  it. Read your keys again here; fall back to the current value, not the default, so a missing or unreadable line
  changes nothing. Never write settings from it.
- `on_language_changed` runs once when `general.language` changed (after `on_settings_changed`), and again after the notch moves to a monitor of another scale. Rebuild text you cached from
  `tr!` (a label, a count, a measured width) and request a repaint. `name()` returns a `String` read in the current
  language through `tr!("page.<id>")`; look texts up with `crate::i18n::tr!` when you create or rebuild them, never in
  `draw_*`, and add the English and Indonesian text to `crates/notch/locales/app.yml`.
- `on_timer` runs when a timer from `context.set_timer` fires. Request a repaint when something visible changed.
- `on_pointer` receives pointer events while your page is the open one, including while it is still fading in after a switch; a page fading out gets none.
  The collapsed pill sends none to pages: a page whose `presence()` is a banner or a badge gets the pointer over it in
  `on_presence_pointer` (the rain alert's Dismiss and Open buttons, and a press on its badge; both call `context.request_open()`, which
  opens the notch on the calling page).
- A page that uses right-click answers `uses_right_click` for those points and then gets `PointerEvent::SecondaryReleased`; the shelf does this for its files. A right-click elsewhere does nothing.
  Positions are units in the same space as its own `frame.bounds`, from the page's top-left.
- A page with a scrolling area answers `claims_scroll` for the points where the vertical wheel or a touchpad swipe scrolls it and then gets `PointerEvent::Scrolled { notches }` (positive moves toward later content, one wheel notch is 1.0); the calendar's event list does this. Elsewhere the wheel still switches pages, and a horizontal swipe always does.
- A page that must not hold anything while it is off (the clipboard page) answers `on_enabled_changed`, which runs at start and whenever the settings window switches the page. A page that wants a Windows message (the clipboard) calls `context.listen_to_clipboard()` in `on_start` and reads it in `on_system_message`; only a page that is on gets it. `context.request_close()` closes the notch, except while the user has pinned it open: then the request does nothing.
- A page that takes typing (the todo page's add field) calls `context.take_keyboard()` from a click, never on hover,
  and gets `on_key` with a `KeyInput` until it calls `context.release_keyboard()`. The notch stays open meanwhile. When
  the user clicks another window first, the page gets `on_keyboard_lost` and stops editing. `ui::TextField` handles
  the editing and draws the text with a steady caret, so editing runs no timer.

### 3. Register it

Open `crates/notch/src/pages/mod.rs`, declare the module and add the page to `all()`:

```rust
mod greeting;
mod media;

pub fn all() -> Vec<Box<dyn Page>> {
    vec![
        Box::new(media::MediaPage::new()),
        Box::new(greeting::GreetingPage::new()),
    ]
}
```

The order matters only for ties: when two pages have the same priority, the earlier one wins. Here the greeting and
the media page are both `NORMAL`, so the media page wins while music plays and the greeting shows the rest of the
time.

### 4. Run it

`scripts/reload.sh` builds and starts it. To see your page on its own, hide the others:

```
set NOTCH_PAGES=shelf,greeting
```

Nothing else in the project needs to change: the host, the drawing code and the other pages do not know your page
exists.

## Next

- Look up a hook, a drawing call or a convention: [page API](../reference/page-api.md).
- Give the page settings: [add a setting](../how-to/add-a-setting.md).
- A page that shows something in the collapsed pill: [activity presence](../reference/activities.md).
- Remove it, or switch it off: [switch off or remove](../how-to/switch-off-or-remove.md).
