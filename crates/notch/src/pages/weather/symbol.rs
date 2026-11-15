//! What the sky looks like: MET Norway symbol codes reduced to the few skies the page names and draws.
use crate::i18n::tr;
use crate::ui::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Symbol {
    Clear,
    ClearNight,
    PartlyCloudy,
    PartlyCloudyNight,
    Cloudy,
    Fog,
    Drizzle,
    Rain,
    HeavyRain,
    RainShowers,
    RainShowersNight,
    Thunderstorm,
    Snow,
}

impl Symbol {
    /// Reads a code like `partlycloudy_day`, `heavyrainandthunder` or `lightsleetshowers_night`; an unknown code
    /// shows as cloudy so the page still draws something. `_polartwilight` counts as day.
    pub(super) fn from_code(code: &str) -> Self {
        let is_night = code.ends_with("_night");
        let name = code.split('_').next().unwrap_or_default();
        if name.contains("thunder") {
            Symbol::Thunderstorm
        } else if name.contains("snow") || name.contains("sleet") {
            Symbol::Snow
        } else if name.contains("rainshowers") {
            if is_night { Symbol::RainShowersNight } else { Symbol::RainShowers }
        } else if name.contains("rain") {
            if name.starts_with("light") {
                Symbol::Drizzle
            } else if name.starts_with("heavy") {
                Symbol::HeavyRain
            } else {
                Symbol::Rain
            }
        } else if name == "fog" {
            Symbol::Fog
        } else if name.starts_with("clearsky") {
            if is_night { Symbol::ClearNight } else { Symbol::Clear }
        } else if name == "partlycloudy" || name == "fair" {
            if is_night { Symbol::PartlyCloudyNight } else { Symbol::PartlyCloudy }
        } else {
            Symbol::Cloudy
        }
    }

    pub(super) fn label(self) -> std::borrow::Cow<'static, str> {
        match self {
            Symbol::Clear => tr!("weather.sky_clear"),
            Symbol::ClearNight => tr!("weather.sky_clear_night"),
            Symbol::PartlyCloudy | Symbol::PartlyCloudyNight => tr!("weather.sky_partly_cloudy"),
            Symbol::Cloudy => tr!("weather.sky_cloudy"),
            Symbol::Fog => tr!("weather.sky_fog"),
            Symbol::Drizzle => tr!("weather.sky_light_rain"),
            Symbol::Rain | Symbol::HeavyRain | Symbol::RainShowers | Symbol::RainShowersNight => {
                tr!("weather.sky_rain")
            }
            Symbol::Thunderstorm => tr!("weather.sky_thunderstorms"),
            Symbol::Snow => tr!("weather.sky_snow"),
        }
    }

    /// The Lucide icon of this sky: the one place a sky becomes a picture, for the page, the banner and the badge.
    pub(super) fn icon(self) -> Icon {
        match self {
            Symbol::Clear => Icon::Sun,
            Symbol::ClearNight => Icon::Moon,
            Symbol::PartlyCloudy => Icon::CloudSun,
            Symbol::PartlyCloudyNight => Icon::CloudMoon,
            Symbol::Cloudy => Icon::Cloudy,
            Symbol::Fog => Icon::CloudFog,
            Symbol::Drizzle | Symbol::Rain => Icon::CloudRain,
            Symbol::HeavyRain => Icon::CloudRainWind,
            Symbol::RainShowers => Icon::CloudSunRain,
            Symbol::RainShowersNight => Icon::CloudMoonRain,
            Symbol::Thunderstorm => Icon::CloudLightning,
            Symbol::Snow => Icon::CloudSnow,
        }
    }

    /// Whether this sky brings water: the page marks these hours.
    pub(super) fn is_wet(self) -> bool {
        matches!(
            self,
            Symbol::Drizzle
                | Symbol::Rain
                | Symbol::HeavyRain
                | Symbol::RainShowers
                | Symbol::RainShowersNight
                | Symbol::Thunderstorm
                | Symbol::Snow
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_becomes_the_sky_it_describes() {
        assert_eq!(Symbol::from_code("clearsky_day"), Symbol::Clear);
        assert_eq!(Symbol::from_code("clearsky_night"), Symbol::ClearNight);
        assert_eq!(Symbol::from_code("fair_day"), Symbol::PartlyCloudy);
        assert_eq!(Symbol::from_code("fair_night"), Symbol::PartlyCloudyNight);
        assert_eq!(Symbol::from_code("partlycloudy_night"), Symbol::PartlyCloudyNight);
        assert_eq!(Symbol::from_code("cloudy"), Symbol::Cloudy);
        assert_eq!(Symbol::from_code("fog"), Symbol::Fog);
        assert_eq!(Symbol::from_code("lightrain"), Symbol::Drizzle);
        assert_eq!(Symbol::from_code("rain"), Symbol::Rain);
        assert_eq!(Symbol::from_code("heavyrain"), Symbol::HeavyRain);
        assert_eq!(Symbol::from_code("rainshowers_day"), Symbol::RainShowers);
        assert_eq!(Symbol::from_code("heavyrainshowers_night"), Symbol::RainShowersNight);
        assert_eq!(Symbol::from_code("heavyrainandthunder"), Symbol::Thunderstorm);
        assert_eq!(Symbol::from_code("lightsleetshowers_night"), Symbol::Snow);
        assert_eq!(Symbol::from_code("snow"), Symbol::Snow);
    }

    #[test]
    fn a_code_becomes_the_icon_of_its_sky() {
        let icon = |code: &str| Symbol::from_code(code).icon();
        assert!(icon("lightrainshowersandthunder_night") == Icon::CloudLightning);
        assert!(icon("heavyrainandthunder") == Icon::CloudLightning);
        assert!(icon("snow") == Icon::CloudSnow);
        assert!(icon("heavysleetshowers_day") == Icon::CloudSnow);
        assert!(icon("lightsnowshowers_polartwilight") == Icon::CloudSnow);
        assert!(icon("rainshowers_day") == Icon::CloudSunRain);
        assert!(icon("lightrainshowers_polartwilight") == Icon::CloudSunRain);
        assert!(icon("heavyrainshowers_day") == Icon::CloudSunRain);
        assert!(icon("rainshowers_night") == Icon::CloudMoonRain);
        assert!(icon("lightrainshowers_night") == Icon::CloudMoonRain);
        assert!(icon("heavyrain") == Icon::CloudRainWind);
        assert!(icon("rain") == Icon::CloudRain);
        assert!(icon("lightrain") == Icon::CloudRain);
        assert!(icon("cloudy") == Icon::Cloudy);
        assert!(icon("partlycloudy_day") == Icon::CloudSun);
        assert!(icon("partlycloudy_polartwilight") == Icon::CloudSun);
        assert!(icon("fair_day") == Icon::CloudSun);
        assert!(icon("partlycloudy_night") == Icon::CloudMoon);
        assert!(icon("fair_night") == Icon::CloudMoon);
        assert!(icon("clearsky_day") == Icon::Sun);
        assert!(icon("clearsky_polartwilight") == Icon::Sun);
        assert!(icon("clearsky_night") == Icon::Moon);
        assert!(icon("fog") == Icon::CloudFog);
        assert!(icon("something_new") == Icon::Cloudy);
        assert!(icon("") == Icon::Cloudy);
    }

    #[test]
    fn an_unknown_code_shows_as_cloudy() {
        assert_eq!(Symbol::from_code("something_new"), Symbol::Cloudy);
        assert_eq!(Symbol::from_code(""), Symbol::Cloudy);
    }

    #[test]
    fn only_skies_with_water_are_wet() {
        for code in [
            "lightrain",
            "rain",
            "heavyrain",
            "rainshowers_day",
            "rainshowers_night",
            "sleet",
            "snow",
            "heavyrainandthunder",
        ] {
            assert!(Symbol::from_code(code).is_wet(), "{code}");
        }
        assert!(!Symbol::Clear.is_wet() && !Symbol::Cloudy.is_wet() && !Symbol::Fog.is_wet());
    }

    #[test]
    fn heavy_rain_and_showers_read_as_rain_and_fair_as_partly_cloudy() {
        assert_eq!(Symbol::HeavyRain.label(), Symbol::Rain.label());
        assert_eq!(Symbol::RainShowersNight.label(), Symbol::Rain.label());
        assert_eq!(Symbol::from_code("fair_day").label(), Symbol::PartlyCloudy.label());
    }

    #[test]
    fn the_sky_reads_in_indonesian() {
        assert_eq!(Symbol::Thunderstorm.label(), "Thunderstorms");
        store::language::with_override(store::language::Language::Indonesian, || {
            assert_eq!(Symbol::Clear.label(), "Cerah");
            assert_eq!(Symbol::PartlyCloudy.label(), "Berawan sebagian");
            assert_eq!(Symbol::Thunderstorm.label(), "Badai petir");
        });
    }
}
