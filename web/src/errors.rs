use std::sync::{PoisonError, RwLockReadGuard};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use geocoder::ReverseGeocoder;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid configuration: {0}")]
    ConfigurationError(#[from] envy::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, body) = to_response_tuple(self);
        (status, body).into_response()
    }
}

#[allow(unreachable_patterns)]
fn to_response_tuple(err: Error) -> (StatusCode, String) {
    match err {
        Error::ConfigurationError(_) => (StatusCode::NOT_FOUND, err.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
