//! Where the user is: the city list behind the location search (GeoNames cities of 15,000 people or more, CC BY 4.0, see
//! the README) and the `location.*` settings keys the prayer times and the weather share.

/// `name<TAB>country<TAB>latitude<TAB>longitude`, largest cities first; made by `scripts/build-cities.sh`.
static CITIES: &str = include_str!("../../../assets/cities.tsv");

#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub name: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl City {
    /// How the city is named in the settings and on the page: "Pekanbaru, ID".
    pub fn label(&self) -> String {
        format!("{}, {}", self.name, self.country)
    }
}

pub const CITY_KEY: &str = "location.city";
pub const LATITUDE_KEY: &str = "location.lat";
pub const LONGITUDE_KEY: &str = "location.lon";
pub const COUNTRY_KEY: &str = "location.country";

/// The keys the shalat page saved the city under before the location was shared; they are read when the shared ones
/// are missing.
const LEGACY_KEYS: [(&str, &str); 4] = [
    (CITY_KEY, "shalat.city"),
    (LATITUDE_KEY, "shalat.lat"),
    (LONGITUDE_KEY, "shalat.lon"),
    (COUNTRY_KEY, "shalat.country"),
];

/// The one place the prayer times and the weather are for.
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    /// "Pekanbaru, ID"; empty when only coordinates were saved.
    pub label: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    /// Reads the saved location; `None` until a city with valid coordinates was picked.
    pub fn read(get: &dyn Fn(&str) -> Option<String>) -> Option<Self> {
        let value = |key: &str| {
            get(key)
                .or_else(|| LEGACY_KEYS.iter().find(|(shared, _)| *shared == key).and_then(|(_, legacy)| get(legacy)))
        };
        let number = |key: &str| value(key).and_then(|text| text.trim().parse::<f64>().ok());
        let (latitude, longitude) = (number(LATITUDE_KEY)?, number(LONGITUDE_KEY)?);
        (latitude.abs() <= 90.0 && longitude.abs() <= 180.0).then(|| Self {
            label: value(CITY_KEY).unwrap_or_default(),
            country: value(COUNTRY_KEY).unwrap_or_default(),
            latitude,
            longitude,
        })
    }
}

/// The settings values that choosing `city` saves.
pub fn values_of(city: &City) -> Vec<(&'static str, String)> {
    vec![
        (CITY_KEY, city.label()),
        (LATITUDE_KEY, format!("{:.3}", city.latitude)),
        (LONGITUDE_KEY, format!("{:.3}", city.longitude)),
        (COUNTRY_KEY, city.country.clone()),
    ]
}

fn parse(line: &str) -> Option<City> {
    let mut fields = line.split('\t');
    let name = fields.next()?.to_owned();
    let country = fields.next()?.to_owned();
    let latitude = fields.next()?.parse().ok()?;
    let longitude = fields.next()?.parse().ok()?;
    Some(City { name, country, latitude, longitude })
}

/// Lowercase with the common accented Latin letters flattened, so "Sao Paulo" finds "São Paulo".
fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .map(|character| match character {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'ē' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ı' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ō' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'ū' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

/// Cities whose name starts with `query` come first, then names that contain it; each group is largest first.
pub fn search(query: &str, limit: usize) -> Vec<City> {
    search_in(CITIES, query, limit)
}

fn search_in(data: &str, query: &str, limit: usize) -> Vec<City> {
    let wanted = fold(query.trim());
    if wanted.is_empty() || limit == 0 {
        return Vec::new();
    }
    let mut starting = Vec::new();
    let mut containing = Vec::new();
    for line in data.lines() {
        let name = fold(line.split('\t').next().unwrap_or_default());
        if name.starts_with(&wanted) {
            starting.push(line);
            if starting.len() == limit {
                break;
            }
        } else if containing.len() < limit && name.contains(&wanted) {
            containing.push(line);
        }
    }
    starting.into_iter().chain(containing).take(limit).filter_map(parse).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Shanghai\tCN\t31.222\t121.458\nPekanbaru\tID\t0.517\t101.442\nPekan\tMY\t3.5\t103.4\nSão Paulo\tBR\t-23.55\t-46.63\nNew Pekan\tXX\t1.0\t2.0\n";

    #[test]
    fn names_that_start_with_the_query_come_before_names_that_contain_it() {
        let found = search_in(SAMPLE, "pekan", 5);
        let names: Vec<&str> = found.iter().map(|city| city.name.as_str()).collect();
        assert_eq!(names, ["Pekanbaru", "Pekan", "New Pekan"]);
    }

    #[test]
    fn accents_and_case_do_not_matter() {
        assert_eq!(search_in(SAMPLE, "SAO paulo", 3)[0].country, "BR");
        assert_eq!(search_in(SAMPLE, "são", 3)[0].name, "São Paulo");
    }

    #[test]
    fn the_result_count_is_capped_and_a_blank_query_finds_nothing() {
        assert_eq!(search_in(SAMPLE, "pe", 1).len(), 1);
        assert!(search_in(SAMPLE, "  ", 3).is_empty());
        assert!(search_in(SAMPLE, "zzzz", 3).is_empty());
    }

    #[test]
    fn a_city_label_is_its_name_and_country() {
        assert_eq!(search_in(SAMPLE, "pekanbaru", 1)[0].label(), "Pekanbaru, ID");
    }

    #[test]
    fn the_bundled_list_has_the_big_cities_with_real_coordinates() {
        let jakarta = &search("jakarta", 1)[0];
        assert_eq!(jakarta.country, "ID");
        assert!((jakarta.latitude + 6.2).abs() < 0.3 && (jakarta.longitude - 106.8).abs() < 0.3);
        assert!(CITIES.lines().count() > 20_000);
    }

    fn saved<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| pairs.iter().find(|(name, _)| *name == key).map(|(_, value)| (*value).to_owned())
    }

    #[test]
    fn a_saved_location_reads_back_with_its_label_and_country() {
        let pairs =
            [(CITY_KEY, "Pekanbaru, ID"), (LATITUDE_KEY, "0.517"), (LONGITUDE_KEY, "101.442"), (COUNTRY_KEY, "ID")];
        let location = Location::read(&saved(&pairs)).unwrap();
        assert_eq!(location.label, "Pekanbaru, ID");
        assert_eq!((location.latitude, location.longitude, location.country.as_str()), (0.517, 101.442, "ID"));
    }

    #[test]
    fn a_location_saved_by_an_older_version_still_reads() {
        let pairs =
            [("shalat.city", "Jakarta, ID"), ("shalat.lat", "-6.2"), ("shalat.lon", "106.8"), ("shalat.country", "ID")];
        let location = Location::read(&saved(&pairs)).unwrap();
        assert_eq!(location.label, "Jakarta, ID");
        assert_eq!(location.country, "ID");
    }

    #[test]
    fn the_shared_keys_win_over_the_old_ones() {
        let pairs = [(LATITUDE_KEY, "1.0"), (LONGITUDE_KEY, "2.0"), ("shalat.lat", "9.0"), ("shalat.lon", "9.0")];
        assert_eq!(Location::read(&saved(&pairs)).unwrap().latitude, 1.0);
    }

    #[test]
    fn coordinates_outside_the_globe_or_missing_give_no_location() {
        assert!(Location::read(&saved(&[(LATITUDE_KEY, "0.5")])).is_none());
        assert!(Location::read(&saved(&[(LATITUDE_KEY, "95"), (LONGITUDE_KEY, "10")])).is_none());
        assert!(Location::read(&saved(&[])).is_none());
    }

    #[test]
    fn choosing_a_city_saves_its_label_coordinates_and_country() {
        let city =
            City { name: "Pekanbaru".to_owned(), country: "ID".to_owned(), latitude: 0.5171, longitude: 101.4424 };
        let keys: Vec<_> = values_of(&city).iter().map(|(key, _)| *key).collect();
        assert_eq!(keys, [CITY_KEY, LATITUDE_KEY, LONGITUDE_KEY, COUNTRY_KEY]);
        assert_eq!(values_of(&city)[1].1, "0.517");
    }
}
