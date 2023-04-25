use crate::errors::Error::ConfigurationError;
use crate::Result;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::net::SocketAddr;
use tracing::Level;

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Configuration {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "default_loglevel")]
    pub loglevel: Level,
    #[serde(default = "default_bind_address")]
    pub bind_address: SocketAddr,
    #[serde(default = "default_data_file")]
    pub data_file: String,
    #[serde(default = "default_watch_for_changes")]
    pub watch_for_changes: bool,
    #[serde(default = "default_allow_origin")]
    pub allow_origin: String,
}

fn default_loglevel() -> Level {
    Level::INFO
}
fn default_bind_address() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 5353))
}
fn default_data_file() -> String {
    String::from("./cities.txt")
}
fn default_watch_for_changes() -> bool {
    true
}
fn default_allow_origin() -> String {
    return String::from("*");
}

impl Configuration {
    pub fn from_env() -> Result<Configuration> {
        envy::prefixed("GEOCODER_")
            .from_env::<Configuration>()
            .map_err(ConfigurationError)
    }
}
