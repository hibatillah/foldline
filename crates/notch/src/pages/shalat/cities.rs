//! The city list behind the settings search: GeoNames cities of 15,000 people or more (CC BY 4.0, see README).

/// `name<TAB>country<TAB>latitude<TAB>longitude`, largest cities first; made by `scripts/build-cities.sh`.
static CITIES: &str = include_str!("../../../../../assets/cities.tsv");

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
}
