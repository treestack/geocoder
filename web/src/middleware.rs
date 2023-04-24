use axum::http::{HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

static VERSION: &str = env!("CARGO_PKG_VERSION");

/// Add version number to response
pub(crate) async fn add_version<B>(req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let mut res = next.run(req).await;
    res.headers_mut().insert("X-Version", HeaderValue::from_static(VERSION));
    Ok(res)
}