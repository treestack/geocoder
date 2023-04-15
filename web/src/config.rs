use crate::errors::Error::ConfigurationError;
use crate::Result;
use std::env;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use tracing::Level;

pub struct Configuration {
    pub loglevel: Level,
    pub bind_address: SocketAddr,
    pub data_file: String,
}

impl Configuration {
    pub fn from_env() -> Result<Configuration> {
        let default = Self::default();
        Ok(Self {
            loglevel: Level::from_str(env::var("LOGLEVEL")?.deref()).unwrap_or(default.loglevel),
            bind_address: env::var("BIND_ADDRESS")
                .map_err(ConfigurationError)
                .and_then(|v| Ok(SocketAddr::from_str(&v)?))
                .unwrap_or(default.bind_address), //TODO
            data_file: env::var("DATA_FILE").unwrap_or(default.data_file),
        })
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            loglevel: Level::INFO,
            bind_address: SocketAddr::from(([127, 0, 0, 1], 5353)),
            data_file: String::from("./cities500.txt"),
        }
    }
}
