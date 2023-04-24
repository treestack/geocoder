use std::fmt::{Display, Formatter};
use std::fs::File;

use csv::ReaderBuilder;
use kiddo::float::neighbour::Neighbour;
use kiddo::float::{distance::squared_euclidean, kdtree::KdTree};
use serde::Deserialize;

const EARTH_RADIUS_IN_KM: f32 = 6371.0;

/// City structure, as defined in the http://www.geonames.org export.
///
/// # Examples
/// ```rust
/// let city = geocoder::City {
///     id: 1,
///     name: String::from("Nikolassee"),
///     asciiname: String::from("Nikolassee"),
///     alternatenames: String::from("Berlin-Nikolassee,Nicolassee,Nikolassee,Nikolasze,Николасзе"),
///     latitude: 52.4344,
///     longitude: 13.20095,
///     feature_class: String::from("P"),
///     feature_code: String::from("PPLX"),
///     country_code: String::from("DE"),
///     cc2: String::from("16"),
///     admin1_code: String::from("00"),
///     admin2_code: String::from("11000"),
///     admin3_code: String::from("11000000"),
///     admin4_code: String::from("11642"),
///     population: None,
///     elevation: Some(42),
///     dem: String::from(""),
///     timezone: String::from("Europe/Berlin"),
///     modification_date: String::from("2022-08-04"),
/// };
/// ```
#[rustfmt::skip]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct City {
    pub id: u32,                   // integer id of record in geonames database
    pub name: String,              // name of geographical point (utf8) varchar(200)
    pub asciiname: String,         // name of geographical point in plain ascii characters, varchar(200)
    pub alternatenames: String,    // alternatenames, comma separated, ascii names automatically transliterated, convenience attribute from alternatename table, varchar(10000)
    pub latitude: f32,             // latitude in decimal degrees (wgs84)
    pub longitude: f32,            // longitude in decimal degrees (wgs84)
    pub feature_class: String,     // see http://www.geonames.org/export/codes.html, char(1)
    pub feature_code: String,      // see http://www.geonames.org/export/codes.html, varchar(10)
    pub country_code: String,      // ISO-3166 2-letter country code, 2 characters
    pub cc2: String,               // alternate country codes, comma separated, ISO-3166 2-letter country code, 200 characters
    pub admin1_code: String,       // fipscode (subject to change to iso code), see exceptions below, see file admin1Codes.txt for display names of this code; varchar(20)
    pub admin2_code: String,       // code for the second administrative division, a county in the US, see file admin2Codes.txt; varchar(80)
    pub admin3_code: String,       // code for third level administrative division, varchar(20)
    pub admin4_code: String,       // code for fourth level administrative division, varchar(20)
    pub population: Option<u32>,   // bigint (8 byte int)
    pub elevation: Option<i16>,    // in metres, integer
    pub dem: String,               // digital elevation model, srtm3 or gtopo30, average elevation of 3''x3'' (ca 90mx90m) or 30''x30'' (ca 900mx900m) area in metres, integer. srtm processed by cgiar/ciat.
    pub timezone: String,          // the IANA timezone id (see file timeZone.txt) varchar(40)
    pub modification_date: String, // date of last modification in yyyy-MM-dd format
}

impl City {
    /// Get coordinates as ECEF (x;y;z)-coordinates.
    pub fn as_xyz(&self) -> [f32; 3] {
        degrees_lat_lng_to_unit_sphere(self.latitude, self.longitude)
    }
}

impl Display for City {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.name, self.country_code)
    }
}

#[derive(Debug)]
pub struct ReverseGeocoder {
    cities: Vec<City>,
    tree: KdTree<f32, usize, 3, 32, u16>,
}

impl Display for ReverseGeocoder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ReverseGeocoder<cities={}, tree={}>",
            self.cities.len(),
            self.tree.size()
        )
    }
}

impl Default for ReverseGeocoder {
    fn default() -> Self {
        Self {
            tree: KdTree::with_capacity(0),
            cities: vec![],
        }
    }
}

impl ReverseGeocoder {
    /// Initialize ReverseGeocoder with a list of cities.
    ///
    /// This will fill the tree with the [ECEF coordinates](https://en.wikipedia.org/wiki/Earth-centered,_Earth-fixed_coordinate_system)
    /// and the city's index in the given vector, so the tree's payload doesn't contain the
    /// whole City struct.
    ///
    /// # Examples
    /// ```rust
    /// let cities: Vec<geocoder::City> = vec![ /* ... */];
    /// let gc = geocoder::ReverseGeocoder::new(cities);
    /// ```
    pub fn new(cities: Vec<City>) -> ReverseGeocoder {
        let mut tree: KdTree<f32, usize, 3, 32, u16> = KdTree::with_capacity(cities.len());
        cities.iter().enumerate().for_each(|(idx, city)| {
            tree.add(&city.as_xyz(), idx);
        });
        tracing::info!("Populated tree with {} cities", cities.len());

        Self { cities, tree }
    }

    /// Initialize ReverseGeocoder from a CSV file.
    ///
    /// Loads a CSV file into memory and initializes the GeoCoder with the contents.
    /// Currently only supported format is http://www.geonames.org citiesX.txt
    ///
    /// # Example
    /// ```rust
    /// let gc = geocoder::ReverseGeocoder::from_file("../cities.txt");
    /// ```
    pub fn from_file(csv_path: &str) -> ReverseGeocoder {
        let cities: Vec<City> = parse_csv_file(csv_path).expect("panic!");
        Self::new(cities)
    }

    /// Finds the `results` cities nearest to the given coordinates (WGS84, decimal format).
    ///
    /// Returns a Vec of tuples consisting of the distance in kilometres to the given coordinates and
    /// the found `City`..
    ///
    /// # Arguments
    /// * `lat` - latitude
    /// * `lng` - longitude
    /// * `results` - number of results
    ///
    /// # Example
    /// ```rust
    /// # let gc = geocoder::ReverseGeocoder::from_file("../cities.txt");
    /// let results = gc.search(47.11, 8.15, 10);
    /// ```
    pub fn search(&self, lat: f32, lng: f32, results: usize) -> Vec<(u32, &City)> {
        tracing::debug!(
            "Searching for {} cities closest to {};{}",
            results,
            lat,
            lng
        );

        let query = degrees_lat_lng_to_unit_sphere(lat, lng);
        let results = self.tree.nearest_n(&query, results, &squared_euclidean);
        tracing::debug!("Found: {:?}", results);
        results
            .iter()
            .map(|Neighbour { distance, item }| {
                (
                    unit_sphere_squared_euclidean_to_kilometres(*distance) as u32,
                    self.cities.get(*item).unwrap(),
                )
            })
            .collect()
    }
}

/// Parse CSV file into Vec of `R`.
fn parse_csv_file<R: for<'de> serde::Deserialize<'de> + Display>(
    filename: &str,
) -> Result<Vec<R>, std::io::Error> {
    tracing::debug!("Loading from file {}", filename);
    let file = File::open(filename)?;
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .delimiter(b'\t')
        .from_reader(file);

    let cities: Vec<R> = reader.deserialize().filter_map(Result::ok).collect();

    Ok(cities)
}

/// Convert geodetic coordinates to ECEF coordinates
fn degrees_lat_lng_to_unit_sphere(lat: f32, lng: f32) -> [f32; 3] {
    let lat = lat.to_radians();
    let lng = lng.to_radians();
    [lat.cos() * lng.cos(), lat.cos() * lng.sin(), lat.sin()]
}

/// Convert distance between two ECEF coordinates to kilometres
pub fn unit_sphere_squared_euclidean_to_kilometres(sq_euc_dist: f32) -> f32 {
    sq_euc_dist.sqrt() * EARTH_RADIUS_IN_KM
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn doesnt_find_anything_if_list_is_empty() {
        let gc = ReverseGeocoder::new(vec![]);
        let result = gc.search(50.93, 6.95, 1);
        assert!(result.is_empty());
    }

    #[test]
    #[traced_test]
    fn finds_test_city() {
        let gc = ReverseGeocoder::from_file("../cities.txt");
        let (d, city) = gc.search(50.88, 6.92, 1).first().unwrap().clone();
        assert_eq!(city.id, 2929622);
        assert_eq!(d, 47);
        assert_eq!(format!("{}", city), "Erkelenz, DE")
    }
}
