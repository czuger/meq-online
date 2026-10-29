//! Runtime settings read from the environment.

use std::net::SocketAddr;
use std::path::PathBuf;

use thiserror::Error;

/// A setting has an invalid value.
#[derive(Debug, Error)]
#[error("invalid value for {name}: {value}")]
pub struct ConfigError {
    /// Environment variable.
    pub name: &'static str,
    /// Its value.
    pub value: String,
}

/// Server settings.
#[derive(Debug, Clone)]
pub struct Config {
    /// Address to listen on (`MEQ_BIND`, default `127.0.0.1:3000`).
    pub bind: SocketAddr,
    /// SQLite URL (`DATABASE_URL`, default `sqlite://meq.sqlite3`).
    pub database_url: String,
    /// Folder of the YAML game data (`MEQ_DATA_DIR`, default `data`).
    pub data_dir: PathBuf,
    /// Folder of static files (`MEQ_STATIC_DIR`, default `static`).
    pub static_dir: PathBuf,
}

impl Config {
    /// Reads the settings from the environment.
    ///
    /// # Errors
    /// [`ConfigError`] when `MEQ_BIND` isn't a socket address.
    pub fn from_env() -> Result<Self, ConfigError> {
        let var =
            |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.to_owned());
        let bind = var("MEQ_BIND", "127.0.0.1:3000");
        Ok(Self {
            bind: bind.parse().map_err(|_| ConfigError {
                name: "MEQ_BIND",
                value: bind.clone(),
            })?,
            database_url: var("DATABASE_URL", "sqlite://meq.sqlite3"),
            data_dir: var("MEQ_DATA_DIR", "data").into(),
            static_dir: var("MEQ_STATIC_DIR", "static").into(),
        })
    }
}
