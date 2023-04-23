use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid configuration: {0}")]
    ConfigurationError(#[from] envy::Error),

    #[error("please try again in a few seconds")]
    LockError(), //TODO: #[from] TryLockError<RwLockReadGuard<'_, ReverseGeocoder>>
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Error::LockError() => (
                StatusCode::SERVICE_UNAVAILABLE,
                [(header::RETRY_AFTER, "30")],
                self.to_string(),
            ).into_response(),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                self
            ).into_response(),
        }
    }
}
