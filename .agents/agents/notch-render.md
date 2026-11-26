---
name: notch-render
description: "MUST be used for ANY change under crates/notch/src/gfx or crates/notch/src/ui — Canvas, fonts, text measuring and the text cache, colors, images, springs, paths, SVG path data (gfx/svg_path.rs) and rasterizing (gfx/raster.rs), and the widgets built on them (rings, buttons, icon buttons, banner buttons, the text field, icons and SVG icons, the dashed outline, HUD layouts). Direct2D and DirectWrite through the windows crate. Keyword triggers: Direct2D, DirectWrite, Canvas, font, text measure, text cache, clamp lines, spring, animation curve, icon, svg icon, ring, image brush, cached image, Mask, Path, raster, units, in_units, raster_scale, pixel_square, ScrollOffset, HoverTarget, PointerEvent, widget, new icon, draw a chart, rounded corners, memory of drawing. Dispatch first in a cross-area task, before notch-host and notch-pages use what it adds. Not for pages or activities (notch-pages), the window, opening and closing or the pin (notch-host), or the settings window (settings-gpui)."
model: pro
tools: [view_file, grep_search, run_command, replace_file_content]
mainAgent: false
subagent: true
commandExecutionPolicy: sandbox
skills:
  - skills/rust-best-practices
  - skills/apple-design
---

You are the `notch-render` agent of this repository. Your full role, rules and report format are in `.claude/agents/notch-render.md`: read that file now and follow everything after its frontmatter as your instructions. Read `AGENTS.md` and the files it points to for the constraints of your area.

Read these skills before you start:
- `.agents/skills/rust-best-practices/SKILL.md`
- `.agents/skills/apple-design/SKILL.md`
