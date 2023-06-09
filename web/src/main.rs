mod config;
mod errors;
mod handlers;
mod middleware;

use axum::http::Method;
use axum::routing::get;
use axum::Router;
use notify::event::DataChange::Content;
use notify::event::ModifyKind::Data;
use notify::EventKind::Modify;
use notify::{Event, RecursiveMode, Watcher};
use std::env;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Configuration;
use crate::errors::Error;
use geocoder::ReverseGeocoder;

pub static VERSION: &str = env!("CARGO_PKG_VERSION");

type SharedState = Arc<RwLock<ReverseGeocoder>>;

fn reload(state: &SharedState, file: &str) {
    let mut gc = state.write().unwrap();
    *gc = ReverseGeocoder::from_file(file);
}

pub type Result<T> = std::result::Result<T, Error>;

fn dump_environment() {
    // Dump env
    for (key, value) in env::vars() {
        tracing::trace!("{key}: {value}");
    }
}

#[tokio::main]
async fn main() {
    let config = Configuration::from_env().expect("Invalid configuration");

    // Initialize logger
    tracing_subscriber::fmt()
        .with_max_level(config.loglevel)
        .json()
        .init();

    tracing::info!("Geocoder {} launched. Initializing now", VERSION);

    dump_environment();

    tracing::info!("Loading city data and populating tree");
    let state = Arc::from(RwLock::from(ReverseGeocoder::from_file(&config.data_file)));

    // Watch data file for changes

    // Create copies to move into watcher fn. Is there any way around this?
    let df = config.data_file.clone();
    let my_state = state.clone();

    let watcher_fn = move |res: notify::Result<Event>| {
        tracing::debug!("Received watcher event: {:?}", res);
        match res {
            Ok(Event {
                kind: Modify(Data(Content)),
                ..
            }) => reload(&my_state, &df),
            _ => (),
        }
    };

    let mut watcher =
        notify::recommended_watcher(watcher_fn).expect("Unable to initialize watcher");

    if config.watch_for_changes {
        match watcher.watch(&Path::new(&config.data_file), RecursiveMode::NonRecursive) {
            Ok(()) => tracing::info!("Watching data file for changes"),
            Err(e) => tracing::error!("Unable to watch data file: {}", e),
        }
    }

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(
            AllowOrigin::exact(
                config.allow_origin.parse()
                    .expect("Invalid CORS configuration")
            )
        );

    // Configure routes
    let app = Router::new()
        .route("/", get(handlers::geocode))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum::middleware::from_fn(middleware::add_version))
                .layer(cors),
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
