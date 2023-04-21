mod config;
mod errors;
mod handlers;

use axum::error_handling::HandleErrorLayer;
use axum::routing::get;
use axum::{BoxError, Router};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tower::ServiceBuilder;
use tower_governor::errors::display_error;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::trace::TraceLayer;

use crate::config::Configuration;
use crate::errors::Error;
use geocoder::ReverseGeocoder;

pub struct AppState {
    geocoder: ReverseGeocoder,
}

pub type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    let config = Configuration::from_env().expect("Invalid configuration");

    // Initialize logger
    tracing_subscriber::fmt()
        .with_max_level(config.loglevel)
        .json()
        .init();

    tracing::info!("Geocoder launched. Initializing now");

    // Dump env
    for (key, value) in env::vars() {
        tracing::trace!("{key}: {value}");
    }

    // Boot geocoder
    tracing::info!("Loading city data and populating tree");
    let shared_state = Arc::new(AppState {
        geocoder: ReverseGeocoder::from_file(&config.data_file),
    });

    // Rate limiter
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_millisecond(300)
            .burst_size(10)
            .finish()
            .unwrap(),
    );

    // Configure routes
    let app = Router::new()
        .route("/", get(handlers::geocode))
        .with_state(shared_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(HandleErrorLayer::new(|e: BoxError| async move {
                    display_error(e)
                }))
                .layer(GovernorLayer {
                    config: Box::leak(governor_conf),
                }),
        );

    // Start the server
    tracing::info!("Listening on {}", &config.bind_address);
    axum::Server::bind(&config.bind_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Handle shutdown signal
///
/// cf. https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::warn!("Shutdown signal received, starting graceful shutdown");
}
