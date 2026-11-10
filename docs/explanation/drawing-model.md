# Drawing model

> **Explanation.** Read when: you change how something is drawn, add a widget, or want to know why the notch uses no UI framework. Skip if: you are writing a page; see [add a page](../how-to/add-a-page.md) and the [page API](../reference/page-api.md).

`notch.exe` draws every frame itself with Direct2D and DirectWrite through the `windows` crate. Pages, activities and
widgets draw in device-independent units, and only the host converts those units to device pixels. This document gives
the reasons for that structure; where each file lives is in the [source map](../reference/source-map.md).

## Why Direct2D by hand and not GPUI

The notch idles at 0 % CPU and stays under 50 MB of memory ([performance budget](performance-budget.md)). A UI
framework is only worth taking if a window built with it fits inside that budget before any page exists.

A spike tested this with GPUI 0.2.2, the framework the settings window uses. It built one borderless, transparent,
always-on-top GPUI pop-up window with a row of hoverable squares, in a release build with the notch's profile
(`opt-level = "s"`, LTO, `panic = "abort"`). The window worked. Its idle footprint was:

| Measure | Value |
| --- | --- |
| working set | 82.8 MB |
| private bytes | 102.8 MB |
| handles | 1121 |
| threads | 22 |
| idle CPU | 1.15 %, then 4.22 % |

This was a single run on one machine (Windows 11 build 10.0.26300). The two CPU readings disagree, so they are
inconclusive and no conclusion about CPU is drawn from them. The memory reading alone decides the question: a window
of about 83 MB is above the 50 MB budget with nothing in it. The conclusion is limited to `notch.exe`. The settings
window (`notch-settings.exe`) is built with GPUI because it is a separate process that runs only while open and is
outside the budget ([performance baseline](../reference/performance-baseline.md#settings-window)). A later GPUI
version or a second measurement could give a different number; the spike did not test that.

## The three layers

The drawing code has three layers, and each imports only from the ones below it.

- **`gfx`**, the backend: `Canvas` (what everything draws with), `Fonts` and text measuring, `Surface` (the bitmap the
  layered window is drawn into), colors, geometry, springs, `Path` and SVG path data, `Mask` rasterizing, and the image and
  text caches. It is the only layer that talks to Direct2D and DirectWrite.
- **`ui`**, the widgets built on `gfx`: `PillButton`, `IconButton`, `Ring`, `TextField`, `BannerButtons`, the icons,
  the HUD layouts and the dashed outline. It also holds what every page needs for input: `ScrollOffset` (a list's
  scroll position and its limits), `HoverTarget` (the one hovered thing on a page) and the input types `PointerEvent`
  and `KeyInput` (re-exported from `page`).
- **Pages and activities**, which use both and are driven by the host.

`gfx` and `ui` import nothing from `page`, `i18n`, `host`, `pages`, `activity` or `platform`. The test
`the_drawing_layers_import_nothing_above_them` in `crates/notch/src/ui/mod.rs` reads their source files and fails on
such an import. A widget therefore gets its texts as arguments: `BannerButtons` takes `BannerLabels` instead of
looking the words up itself. The reason is reuse: with no import from above, `gfx` and `ui` can move into a crate of
their own, shared with other programs, as a move plus import changes.

## Device pixels and units, and where the border is

A display at 125 % has 1.25 device pixels per unit. Pages, activities and widgets write every size in units
(a 26-unit button is 26 units at any display scale) and never multiply by a scale.

The border is one file, `crates/notch/src/host/notch/units.rs`, the only place that converts between the two:

- Around each page and activity draw hook it calls `Canvas::in_units(origin, draw)`, which sets a transform that scales
  by the display scale and moves the zero point to `origin`, the top-left of the page's frame or area in device pixels.
- It gives the hook a frame or area that starts at zero and is measured in units (`unit_frame`, `unit_area`).
- It converts every pointer position the page or activity receives to the same units, from the same top-left
  (`unit_point`, `unit_event`). A hit test against a unit rectangle therefore matches what was drawn in it; the tests in
  `units.rs` check the last pixel of a rectangle and the pixel past it.

Inside a unit space, `Canvas::scale()` is 1.0, `Canvas::raster_scale()` is the display scale, and `Canvas::origin()`
is the zero point in device pixels. Text is laid out with `Fonts::unit()`, a second set of text formats at scale 1.0
that is built once on first use and kept (at a display scale of 1.0 it is the device set itself). `in_units` cannot
be nested.

Two kinds of drawing stay in device pixels: the host's own (the shadow, the pill shape, the page dots and the clock) and
`Page::draw_backdrop`, which draws around the whole pill rather than inside the page's frame. There
`Canvas::scale()` is the display scale. With one conversion point, a page cannot mix pixels and units, and a change to
how the display scale is applied touches one file.

## How cached images stay sharp

Shapes are not drawn with Direct2D geometry, which is expensive in memory the first time it is used (see the
[performance budget](performance-budget.md)). They are described with `Path`, rasterized with `Mask` into a small
bitmap, and reused through `Canvas::cached_image`.

A bitmap is sharp only when it is built at its device size and drawn on whole device pixels; otherwise Direct2D
resamples it and the edges blur. A unit position times a scale such as 1.25 rarely lands on a whole pixel, so
`Canvas::pixel_square(center, size)` (`crates/notch/src/gfx/pixel_square.rs`) does the placement: it returns the
bitmap's side in device pixels, with one pixel of margin on each side for antialiased edges, and the area to draw it
in, in the canvas's current space but snapped to the device grid. The code that builds the bitmap uses
`Canvas::raster_scale()` to turn its unit sizes into pixels.

The cache key holds the device side that `pixel_square` returns, not the unit size. The same 18-unit ring is a
different bitmap at 1.0 and at 1.5, and a key without the device side would reuse the wrong one after the display
scale changes.

## What is deliberately missing

There is no retained widget tree and no layout engine. Each page writes its layout as functions over unit rectangles
(`Rect` and its methods) and draws immediately on every paint.

The reasons come from the budget. A frame is drawn only when something changes, and the frame timer stops as soon as
the springs settle, so an idle notch draws nothing. When a frame is drawn, it is a short list of immediate drawing
calls over a few cached bitmaps and text layouts. A retained tree would add work that this model does not need: there
is nothing to diff between frames, and no tree of nodes to hold in memory for as long as the notch runs. Layout
functions that take and return plain rectangles are also tested without a window.

A shared layout helper is added when two pages need the same one, not before.

## Where to go next

- Writing a page in units: [add a page](../how-to/add-a-page.md).
- Checking a drawing change at several display scales: [test and snapshot](../how-to/test-and-snapshot.md#pictures-of-the-pages).
- The host's paint order and the rest of the structure: [architecture](architecture.md).
