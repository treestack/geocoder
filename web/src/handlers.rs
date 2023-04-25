use crate::{Result, SharedState};
use axum::extract::{Query, State};
use axum::Json;
use geocoder::City;
use geojson::{Feature, GeoJson, Geometry, JsonObject, Value};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct GeocodeParameters {
    lat: f32,
    lng: f32,
    details: Option<bool>,
    results: Option<usize>,
}

pub async fn geocode(
    State(state): State<SharedState>,
    Query(pos): Query<GeocodeParameters>,
) -> Result<Json<Vec<GeoJson>>> {
    let GeocodeParameters {
        lat,
        lng,
        details,
        results,
    } = pos;

    let gc = state.try_read()?;
    let results = gc.search(lat, lng, results.unwrap_or(1));

    let response = results
        .iter()
        .map(|(d, c)| to_feature(&c, *d, details.unwrap_or(false)))
        .collect();

    Ok(Json(response))
}

fn to_feature(city: &City, distance: u32, include_details: bool) -> GeoJson {
    let city = city.clone();

    let point = Value::Point(vec![city.longitude as f64, city.latitude as f64]);

    let mut properties = JsonObject::new();
    properties.insert(String::from("distanceToQuery"), distance.into());

    if include_details {
        properties.insert(String::from("featureCode"), city.feature_code.into());
        properties.insert(String::from("countryCode"), city.country_code.into());
        properties.insert(String::from("cc2"), city.cc2.into());
        properties.insert(String::from("admin1Code"), city.admin1_code.into());
        properties.insert(String::from("admin2Code"), city.admin2_code.into());
        properties.insert(String::from("admin3Code"), city.admin3_code.into());
        properties.insert(String::from("admin4Code"), city.admin4_code.into());
        properties.insert(String::from("population"), city.population.into());
        properties.insert(String::from("elevation"), city.elevation.into());
        properties.insert(String::from("dem"), city.dem.into());
        properties.insert(String::from("timezone"), city.timezone.into());
        properties.insert(
            String::from("modificationDate"),
            city.modification_date.into(),
        );
    }

    let mut foreign_members = JsonObject::new();
    foreign_members.insert(String::from("name"), city.name.into());
    foreign_members.insert(String::from("id"), city.id.into());

    GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(Geometry::new(point)),
        id: None,
        properties: Some(properties),
        foreign_members: Some(foreign_members),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error::LockError;
    use geocoder::ReverseGeocoder;
    use std::sync::{Arc, RwLock};
    use tracing_test::traced_test;

    fn test_city() -> City {
        City {
            id: 0,
            name: "Erkelenz".to_string(),
            asciiname: "erkelenz".to_string(),
            alternatenames: "erkelenz".to_string(),
            latitude: 51.0,
            longitude: 6.0,
            feature_class: "P".to_string(),
            feature_code: "PPLA2".to_string(),
            country_code: "DE".to_string(),
            cc2: "DE".to_string(),
            admin1_code: "".to_string(),
            admin2_code: "".to_string(),
            admin3_code: "".to_string(),
            admin4_code: "".to_string(),
            population: None,
            elevation: None,
            dem: "".to_string(),
            timezone: "".to_string(),
            modification_date: "".to_string(),
        }
    }

    #[test]
    #[traced_test]
    fn returns_error_when_geocoder_busy() {
        let state = Arc::new(RwLock::new(ReverseGeocoder::default()));

        let _lock = state.write().unwrap();

        let result = tokio_test::block_on(geocode(
            State(state.clone()),
            Query(GeocodeParameters::default()),
        ));

        assert_eq!(LockError(), result.unwrap_err());
    }

    //noinspection SpellCheckingInspection
    #[test]
    #[traced_test]
    fn returns_cities_without_details() {
        let erkelenz: City = test_city();
        let state = Arc::new(RwLock::new(ReverseGeocoder::new(vec![erkelenz.clone()])));
        let query = GeocodeParameters::default();

        let result = tokio_test::block_on(geocode(State(state), Query(query))).unwrap();

        let city = result.0.first().unwrap();
        let expected = to_feature(&erkelenz, 5511, false);
        assert_eq!(&expected, city);
    }
}
