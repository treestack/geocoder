mod config;
mod errors;
mod handlers;

use dotenvy::dotenv;
use once_cell::sync::OnceCell;
use salvo::logging::Logger;
use salvo::prelude::*;
use salvo::rate_limiter::{BasicQuota, FixedGuard, MemoryStore, RateLimiter, RemoteIpIssuer};
use std::env;
use salvo::CatcherImpl;

use crate::config::Configuration;
use crate::errors::Error;
use geocoder::{ReverseGeocoder};

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
    GEOCODER.set(ReverseGeocoder::new(&config.data_file)).ok();

    // Configure routing
    let router = Router::new()
        .hoop(Logger {})
        .hoop(CachingHeaders::new())
        .hoop(RateLimiter::new(
            FixedGuard::new(),
            MemoryStore::new(),
            RemoteIpIssuer,
            BasicQuota::per_second(1),
        ))
        .get(handlers::geocode);

    let service = Service::new(router)
        .with_catchers(Vec::new());

    // Start the server
    tracing::info!("Listening on {}", &config.bind_address);
    Server::new(TcpListener::bind(&config.bind_address))
        .serve(service)
        .await;
}
