//! Lucide weather icons (ISC license, https://lucide.dev, lucide-static 0.460.0) as SVG path data on the 24 grid.

use super::Icon;

const CLOUD_OPEN_BOTTOM: &str = "M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242";
const SUN_ARC_BEHIND_CLOUD: &str = "M15.947 12.65a4 4 0 0 0-5.925-4.128";
const MOON_ARC_BEHIND_CLOUD: &str = "M10.188 8.5A6 6 0 0 1 16 4a1 1 0 0 0 6 6 6 6 0 0 1-3 5.197";
const RAINING_CLOUD_BOTTOM_LEFT: &str = "M3 20a5 5 0 1 1 8.9-4H13a3 3 0 0 1 2 5.24";

/// The strokes of `icon`, each one SVG path; `None` for an icon that is not drawn from path data here.
pub(super) fn strokes(icon: Icon) -> Option<&'static [&'static str]> {
    Some(match icon {
        Icon::CloudLightning => &["M6 16.326A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 .5 8.973", "m13 12-3 5h4l-3 5"],
        Icon::Cloudy => &[
            "M17.5 21H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z",
            "M22 10a3 3 0 0 0-3-3h-2.207a5.502 5.502 0 0 0-10.702.5",
        ],
        Icon::CloudRain => &[CLOUD_OPEN_BOTTOM, "M16 14v6", "M8 14v6", "M12 16v6"],
        Icon::CloudRainWind => &[CLOUD_OPEN_BOTTOM, "m9.2 22 3-7", "m9 13-3 7", "m17 13-3 7"],
        Icon::CloudSunRain => &[
            "M12 2v2",
            "m4.93 4.93 1.41 1.41",
            "M20 12h2",
            "m19.07 4.93-1.41 1.41",
            SUN_ARC_BEHIND_CLOUD,
            RAINING_CLOUD_BOTTOM_LEFT,
            "M11 20v2",
            "M7 19v2",
        ],
        Icon::CloudSnow => {
            &[CLOUD_OPEN_BOTTOM, "M8 15h.01", "M8 19h.01", "M12 17h.01", "M12 21h.01", "M16 15h.01", "M16 19h.01"]
        }
        Icon::CloudMoon => &[MOON_ARC_BEHIND_CLOUD, "M13 16a3 3 0 1 1 0 6H7a5 5 0 1 1 4.9-6Z"],
        Icon::CloudMoonRain => &[MOON_ARC_BEHIND_CLOUD, "M11 20v2", RAINING_CLOUD_BOTTOM_LEFT, "M7 19v2"],
        Icon::CloudSun => &[
            "M12 2v2",
            "m4.93 4.93 1.41 1.41",
            "M20 12h2",
            "m19.07 4.93-1.41 1.41",
            SUN_ARC_BEHIND_CLOUD,
            "M13 22H7a5 5 0 1 1 4.9-6H13a3 3 0 0 1 0 6Z",
        ],
        Icon::CloudFog => &[CLOUD_OPEN_BOTTOM, "M16 17H7", "M17 21H9"],
        Icon::Sun => &[
            "M8 12a4 4 0 1 0 8 0a4 4 0 1 0-8 0",
            "M12 2v2",
            "M12 20v2",
            "m4.93 4.93 1.41 1.41",
            "m17.66 17.66 1.41 1.41",
            "M2 12h2",
            "M20 12h2",
            "m6.34 17.66-1.41 1.41",
            "m19.07 4.93-1.41 1.41",
        ],
        Icon::Moon => &["M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"],
        _ => return None,
    })
}
