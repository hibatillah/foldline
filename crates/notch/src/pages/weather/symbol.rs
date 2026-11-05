//! What the sky looks like: MET Norway's symbol codes reduced to the few pictures the page draws.

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
    Thunderstorm,
    Snow,
}

impl Symbol {
    /// Reads a code like `partlycloudy_day`, `heavyrainandthunder` or `lightsleetshowers_night`; an unknown code
    /// shows as cloudy so the page still draws something.
    pub(super) fn from_code(code: &str) -> Self {
        let is_night = code.ends_with("_night");
        let name = code.split('_').next().unwrap_or_default();
        if name.contains("thunder") {
            Symbol::Thunderstorm
        } else if name.contains("snow") || name.contains("sleet") {
            Symbol::Snow
        } else if name.contains("rain") {
            if name.starts_with("light") { Symbol::Drizzle } else { Symbol::Rain }
        } else if name == "fog" {
            Symbol::Fog
        } else if name.starts_with("clearsky") || name == "fair" {
            if is_night { Symbol::ClearNight } else { Symbol::Clear }
        } else if name == "partlycloudy" {
            if is_night { Symbol::PartlyCloudyNight } else { Symbol::PartlyCloudy }
        } else {
            Symbol::Cloudy
        }
    }

    pub(super) fn label(self) -> &'static str {
        match self {
            Symbol::Clear => "Clear",
            Symbol::ClearNight => "Clear night",
            Symbol::PartlyCloudy | Symbol::PartlyCloudyNight => "Partly cloudy",
            Symbol::Cloudy => "Cloudy",
            Symbol::Fog => "Fog",
            Symbol::Drizzle => "Light rain",
            Symbol::Rain => "Rain",
            Symbol::Thunderstorm => "Thunderstorms",
            Symbol::Snow => "Snow",
        }
    }

    /// Whether this sky brings water: the page marks these hours.
    pub(super) fn is_wet(self) -> bool {
        matches!(self, Symbol::Drizzle | Symbol::Rain | Symbol::Thunderstorm | Symbol::Snow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_code_becomes_the_sky_it_describes() {
        assert_eq!(Symbol::from_code("clearsky_day"), Symbol::Clear);
        assert_eq!(Symbol::from_code("clearsky_night"), Symbol::ClearNight);
        assert_eq!(Symbol::from_code("fair_day"), Symbol::Clear);
        assert_eq!(Symbol::from_code("partlycloudy_night"), Symbol::PartlyCloudyNight);
        assert_eq!(Symbol::from_code("cloudy"), Symbol::Cloudy);
        assert_eq!(Symbol::from_code("fog"), Symbol::Fog);
        assert_eq!(Symbol::from_code("lightrain"), Symbol::Drizzle);
        assert_eq!(Symbol::from_code("heavyrain"), Symbol::Rain);
        assert_eq!(Symbol::from_code("rainshowers_day"), Symbol::Rain);
        assert_eq!(Symbol::from_code("heavyrainandthunder"), Symbol::Thunderstorm);
        assert_eq!(Symbol::from_code("lightsleetshowers_night"), Symbol::Snow);
        assert_eq!(Symbol::from_code("snow"), Symbol::Snow);
    }

    #[test]
    fn an_unknown_code_shows_as_cloudy() {
        assert_eq!(Symbol::from_code("something_new"), Symbol::Cloudy);
        assert_eq!(Symbol::from_code(""), Symbol::Cloudy);
    }

    #[test]
    fn only_skies_with_water_are_wet() {
        assert!(
            Symbol::Rain.is_wet() && Symbol::Drizzle.is_wet() && Symbol::Thunderstorm.is_wet() && Symbol::Snow.is_wet()
        );
        assert!(!Symbol::Clear.is_wet() && !Symbol::Cloudy.is_wet() && !Symbol::Fog.is_wet());
    }
}
