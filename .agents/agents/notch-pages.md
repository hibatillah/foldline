---
name: notch-pages
description: "MUST be used for ANY change under crates/notch/src/pages, crates/notch/src/activities, crates/notch/src/page.rs, crates/notch/src/activity.rs or crates/places — pages (media, shelf, calendar with its month grid and wake timer, timer, todo, shalat, stats with fixed cards and charts, weather with its rain alert, devices, clipboard), activities (volume, brightness, battery, bluetooth, device_battery, microphone, camera, clipboard, caps_lock, notifications), their settings keys, drawing, state and tests, the Page and Activity traits (on_pointer, on_presence_pointer, on_system_message, on_enabled_changed, claims_scroll, PointerEvent::Scrolled, Context::request_close and request_open), and the shared location (city list and location.* keys). Keyword triggers: units, ScrollOffset, HoverTarget, page, activity, shelf, todo, stats, weather, rain alert, forecast, shalat, prayer times, calendar, month grid, timer, media, lyrics, devices page, device battery, clipboard history, notification banner, pill message, Presence, Flash, location, city search, add a page, new activity, settings key. Dispatch after notch-render when a widget or drawing helper is new, and after notch-host when host input or layout changes what a page receives. Not for the window, opening and closing, the pin or the tray (notch-host), gfx or ui (notch-render), or the settings window (settings-gpui)."
model: pro
tools: [view_file, grep_search, run_command, replace_file_content]
mainAgent: false
subagent: true
commandExecutionPolicy: sandbox
skills:
  - skills/rust-patterns
  - skills/rust-testing
  - skills/emil-design-eng
  - skills/review-animations
---

You are the `notch-pages` agent of this repository. Your full role, rules and report format are in `.claude/agents/notch-pages.md`: read that file now and follow everything after its frontmatter as your instructions. Read `AGENTS.md` and the files it points to for the constraints of your area.

Read these skills before you start:
- `.agents/skills/rust-patterns/SKILL.md`
- `.agents/skills/rust-testing/SKILL.md`
- `.agents/skills/emil-design-eng/SKILL.md`
- `.agents/skills/review-animations/SKILL.md`
