mod config;
mod errors;
mod handlers;

use axum::error_handling::HandleErrorLayer;
use axum::routing::get;
use axum::{BoxError, Router};
use dotenvy::dotenv;
use once_cell::sync::OnceCell;
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use notify::{Event, RecursiveMode, Watcher};
use notify::event::DataChange::Content;
use notify::event::ModifyKind::Data;
use notify::EventKind::Modify;
use tokio::signal;
use tower::ServiceBuilder;
use tower_governor::errors::display_error;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::trace::TraceLayer;

use crate::config::Configuration;
use crate::errors::Error;
use geocoder::ReverseGeocoder;

static GEOCODER: OnceCell<ReverseGeocoder> = OnceCell::new();

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
    boot_geocoder(&config.data_file);

    let data_file = config.data_file.clone();
    let watcher_fn = move |res: notify::Result<Event>| {
        tracing::debug!("Received watcher event: {:?}", res);
        match res {
            Ok(Event { kind: Modify(Data(Content)), .. }) => boot_geocoder(&data_file),
            _ => ()
        }
    };

    let mut watcher = notify::recommended_watcher(watcher_fn)
        .expect("Unable to initialize watcher");

    match watcher.watch(&Path::new(&config.data_file), RecursiveMode::NonRecursive) {
        Ok(()) => tracing::info!("Watching data file for changes"),
        Err(e) => tracing::error!("Unable to watch data file: {}", e)
    }

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
        .layer(TraceLayer::new_for_http())
        .layer(
            ServiceBuilder::new()
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

fn boot_geocoder(path: &str) {
    //TODO: Either buffer requests or fail with 503 while geocoder is loading
    let geocoder = ReverseGeocoder::from_file(path);
    GEOCODER.set(geocoder).ok();
}