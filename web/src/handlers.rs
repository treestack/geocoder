use crate::{Result, SharedState};
use axum::extract::{Query, State};
use axum::{Json};
use geocoder::City;
use geojson::{Feature, GeoJson, Geometry, JsonObject, Value};
use serde::Deserialize;

#[derive(Deserialize)]
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

    let gc = state.read().expect(""); //TODO
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
