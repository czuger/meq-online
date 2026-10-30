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
    /// Write the detailed debug log file (`MEQ_DEBUG_LOG`, `1`/`0`; on by default in
    /// debug builds, off in release builds).
    pub debug_log: bool,
    /// Folder of the debug log files (`MEQ_LOG_DIR`, default `logs`).
    pub log_dir: PathBuf,
    /// Filter of the debug log file (`MEQ_LOG_FILTER`, `RUST_LOG` syntax, default
    /// [`DEFAULT_LOG_FILTER`]). The console keeps using `RUST_LOG`.
    pub log_filter: String,
}

/// Debug log filter: everything from the game, requests and SQL statements.
pub const DEFAULT_LOG_FILTER: &str = "meq_online=trace,tower_http=trace,sqlx=debug,info";

impl Config {
    /// Reads the settings from the environment.
    ///
    /// # Errors
    /// [`ConfigError`] when `MEQ_BIND` isn't a socket address or `MEQ_DEBUG_LOG` isn't a
    /// flag.
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
            debug_log: debug_log_flag(std::env::var("MEQ_DEBUG_LOG").ok())?,
            log_dir: var("MEQ_LOG_DIR", "logs").into(),
            log_filter: var("MEQ_LOG_FILTER", DEFAULT_LOG_FILTER),
        })
    }
}

/// Reads `MEQ_DEBUG_LOG`; without it, dev (debug) builds log and release builds don't.
fn debug_log_flag(value: Option<String>) -> Result<bool, ConfigError> {
    let Some(value) = value else {
        return Ok(cfg!(debug_assertions));
    };
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "on" | "yes" => Ok(true),
        "0" | "false" | "off" | "no" => Ok(false),
        _ => Err(ConfigError {
            name: "MEQ_DEBUG_LOG",
            value,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_log_defaults_to_the_build_mode() {
        assert_eq!(debug_log_flag(None).ok(), Some(cfg!(debug_assertions)));
    }

    #[test]
    fn debug_log_flag_values() {
        assert_eq!(debug_log_flag(Some("on".to_owned())).ok(), Some(true));
        assert_eq!(debug_log_flag(Some("0".to_owned())).ok(), Some(false));
        assert!(debug_log_flag(Some("maybe".to_owned())).is_err());
    }
}
