use geojson::{Feature, GeoJson, Geometry, JsonObject, JsonValue, Value};
use salvo::prelude::Json;
use salvo::{handler, Request};
use geocoder::City;
use crate::errors::Error::{CityNotFound, MissingParameter, NotOnEarth};
use crate::{GEOCODER, Result};

#[handler]
pub async fn geocode(req: &mut Request) -> Result<Json<GeoJson>> {
    let lat = req.query::<f32>("lat").ok_or(MissingParameter("lat"))?;
    let lng = req.query::<f32>("lng").ok_or(MissingParameter("lng"))?;

    let gc = GEOCODER.get().unwrap();

    let (d, idx) = gc.search(&lat, &lng).ok_or(NotOnEarth(lat, lng))?;
    let city = gc
        .get_city(idx)
        .ok_or(CityNotFound(idx))
        .map(|c| to_feature(c, d))?;

    Ok(Json(city))
}

fn to_feature(city: &City, distance: f32) -> GeoJson {
    let city = city.clone();

    let point = Value::Point(vec![city.lng as f64, city.lat as f64]);

    let mut properties = JsonObject::new();
    properties.insert(String::from("name"), JsonValue::from(city.name));
    properties.insert(String::from("admin1"), JsonValue::from(city.admin1));
    properties.insert(String::from("admin2"), JsonValue::from(city.admin2));
    properties.insert(String::from("country"), JsonValue::from(city.country));
    properties.insert(String::from("distanceToQuery"), JsonValue::from(distance));

    GeoJson::Feature(Feature {
        bbox: None,
        geometry: Some(Geometry::new(point)),
        id: None,
        properties: Some(properties),
        foreign_members: None,
    })
}