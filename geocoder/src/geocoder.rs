use std::fmt::{Display, Formatter};
use std::fs::File;
use std::sync::Arc;

use csv::Reader;
use kiddo::float::{distance::squared_euclidean, kdtree::KdTree};
use serde::Deserialize;

const EARTH_RADIUS_IN_M: f32 = 6371000.0;

#[derive(Debug)]
pub struct ReverseGeocoder {
    handle: Arc<Handle>,
}

impl Clone for ReverseGeocoder {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct City {
    pub lat: f32,
    pub lng: f32,
    pub name: String,
    pub admin1: String,
    pub admin2: String,
    pub country: String,
}

impl City {
    pub fn as_xyz(&self) -> [f32; 3] {
        degrees_lat_lng_to_unit_sphere(&self.lat, &self.lng)
    }
}

impl Display for City {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {}", self.name, self.country)
    }
}

#[derive(Debug)]
pub struct Handle {
    cities: Vec<City>,
    tree: KdTree<f32, usize, 3, 32, u16>,
}

impl Handle {
    pub(crate) fn new(csv_path: &str) -> Handle {
        let cities: Vec<City> = parse_csv_file(csv_path).expect("panic!");
        let mut tree: KdTree<f32, usize, 3, 32, u16> = KdTree::with_capacity(cities.len());

        cities.iter().enumerate().for_each(|(idx, city)| {
            tree.add(&city.as_xyz(), idx);
        });
        tracing::info!("Populated tree with {} cities", cities.len());

        Self { cities, tree }
    }
}

impl ReverseGeocoder {
    pub fn new(csv_path: &str) -> ReverseGeocoder {
        Self {
            handle: Arc::new(Handle::new(csv_path)),
        }
    }
}

impl ReverseGeocoder {
    pub fn search(&self, lat: &f32, lng: &f32) -> Option<(f32, usize)> {
        tracing::debug!("Searching for city closest to {};{}", lat, lng);

        let query = degrees_lat_lng_to_unit_sphere(lat, lng);
        let (d, nearest_idx) = self.handle.tree.nearest_one(&query, &squared_euclidean);
        tracing::debug!("idx {} with distance {}", nearest_idx, d);

        let result = match nearest_idx {
            0 => None,
            i => Some((unit_sphere_squared_euclidean_to_metres(d), i)),
        };
        tracing::debug!("Found: {:?}", result);

        result
    }

    pub fn get_city(&self, index: usize) -> Option<&City> {
        self.handle.cities.get(index)
    }
}

fn parse_csv_file<R: for<'de> serde::Deserialize<'de>>(
    filename: &str,
) -> Result<Vec<R>, std::io::Error> {
    tracing::debug!("Loading from file {}", filename);
    let file = File::open(filename)?;
    let cities: Vec<R> = Reader::from_reader(file)
        .deserialize()
        .filter_map(Result::ok)
        .collect();
    Ok(cities)
}

fn degrees_lat_lng_to_unit_sphere(lat: &f32, lng: &f32) -> [f32; 3] {
    let lat = lat.to_radians();
    let lng = lng.to_radians();
    [lat.cos() * lng.cos(), lat.cos() * lng.sin(), lat.sin()]
}

pub fn unit_sphere_squared_euclidean_to_metres(sq_euc_dist: f32) -> f32 {
    sq_euc_dist.sqrt() * EARTH_RADIUS_IN_M
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn finds_cologne() {
        let gc = ReverseGeocoder::new("../cities.csv");
        let (d, i) = gc.search(&50.93, &6.95).unwrap();
        let city = gc.get_city(i).unwrap();
        assert_eq!(i, 34938);
        assert_eq!(d, 370.00305);
        assert_eq!(city.name, "Koeln")
    }
}
