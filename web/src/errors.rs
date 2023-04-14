use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::env::VarError;
use std::net::AddrParseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("this coordinates are probably not on earth: lat {0}, lng {1}")]
    NotOnEarth(f32, f32),

    #[error("the city with index {0} was not found")]
    CityNotFound(usize),

    #[error("invalid configuration: {0}")]
    ConfigurationError(#[from] VarError),

    #[error("invalid configuration: {0}")]
    ParseError(#[from] AddrParseError),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, body) = to_response_tuple(self);
        (status, body).into_response()
    }
}

fn to_response_tuple(err: Error) -> (StatusCode, String) {
    match err {
        Error::NotOnEarth(_, _) => (StatusCode::NOT_FOUND, err.to_string()),
        Error::ConfigurationError(_) => (StatusCode::NOT_FOUND, err.to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}
