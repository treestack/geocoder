use crate::errors::Error::{CityNotFound, NotOnEarth};
use crate::{Result, GEOCODER};
use axum::extract::Query;
use axum::Json;
use geocoder::City;
use geojson::{Feature, GeoJson, Geometry, JsonObject, Value};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GeocodeParameters {
    lat: f32,
    lng: f32,
}

pub async fn geocode(Query(pos): Query<GeocodeParameters>) -> Result<Json<GeoJson>> {
    let gc = GEOCODER.get().unwrap();
    let GeocodeParameters { lat, lng } = pos;

    let (d, idx) = gc.search(&lat, &lng).ok_or(NotOnEarth(lat, lng))?;
    let city = gc
        .get_city(idx)
        .ok_or(CityNotFound(idx))
        .map(|c| to_feature(idx, c, d))?;

    Ok(Json(city))
}

fn to_feature(idx: usize, city: &City, distance: f32) -> GeoJson {
    let city = city.clone();

    let point = Value::Point(vec![city.lng as f64, city.lat as f64]);

    let mut properties = JsonObject::new();
    properties.insert(String::from("admin1"), city.admin1.into());
    properties.insert(String::from("admin2"), city.admin2.into());
    properties.insert(String::from("country"), city.country.into());
    properties.insert(String::from("distanceToQuery"), distance.into());

    let mut foreign_members = JsonObject::new();
    foreign_members.insert(String::from("name"), city.name.into());
    foreign_members.insert(String::from("id"), idx.into());

    GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(Geometry::new(point)),
        id: None,
        properties: Some(properties),
        foreign_members: Some(foreign_members),
    })
}
