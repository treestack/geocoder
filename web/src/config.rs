use std::env;
use std::ops::Deref;
use std::str::FromStr;
use tracing::Level;
use crate::{Result};

pub struct Configuration {
    pub loglevel: Level,
    pub bind_address: String,
    pub data_file: String
}

impl Configuration {
    pub fn from_env() -> Result<Configuration> {
        let default = Self::default();
        Ok(Self {
            loglevel: Level::from_str(env::var("LOGLEVEL")?.deref()).unwrap_or(default.loglevel),
            bind_address: env::var("BIND_ADDRESS").unwrap_or(default.bind_address),
            data_file: env::var("DATA_FILE").unwrap_or(default.data_file)
        })
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            loglevel: Level::INFO,
            bind_address: String::from("0.0.0.0:5353"),
            data_file: String::from("./cities.csv")
        }
    }
}