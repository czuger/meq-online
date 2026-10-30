//! Logging: the console and, in debug mode, a detailed log file per day in the log folder.

use std::path::Path;

use thiserror::Error;
use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_appender::rolling::{InitError, RollingFileAppender, Rotation};
use tracing_subscriber::filter::ParseError;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::{SubscriberInitExt, TryInitError};
use tracing_subscriber::{EnvFilter, Layer, fmt};

use crate::config::Config;

/// Logging can't start.
#[derive(Debug, Error)]
pub enum LoggingError {
    /// The log folder can't be created.
    #[error("cannot create the log folder {path}: {source}")]
    Folder {
        /// Folder.
        path: String,
        /// Cause.
        source: std::io::Error,
    },
    /// The log file can't be opened.
    #[error("cannot open the log file: {0}")]
    File(#[from] InitError),
    /// `RUST_LOG` or `MEQ_LOG_FILTER` is invalid.
    #[error("invalid log filter: {0}")]
    Filter(#[from] ParseError),
    /// A global logger is already set.
    #[error("cannot install the logger: {0}")]
    Install(#[from] TryInitError),
}

/// Prefix of the daily debug log files (`meq-debug.YYYY-MM-DD.log`).
const LOG_FILE_PREFIX: &str = "meq-debug";

/// Installs the global logger: `RUST_LOG` (default `info`) on the console and, when
/// `config.debug_log` is set, `config.log_filter` in a daily file of `config.log_dir`,
/// with each instrumented step's entry and exit.
///
/// # Returns
/// The guard flushing the log file: keep it alive until the server stops.
///
/// # Errors
/// [`LoggingError`] when a filter is invalid, the file can't be opened or a logger is
/// already installed.
pub fn init(config: &Config) -> Result<Option<WorkerGuard>, LoggingError> {
    let console_filter = match std::env::var("RUST_LOG") {
        Ok(directives) => EnvFilter::try_new(directives)?,
        Err(_) => EnvFilter::new("info"),
    };
    let console = fmt::layer().with_filter(console_filter);
    if !config.debug_log {
        tracing_subscriber::registry().with(console).try_init()?;
        return Ok(None);
    }
    let (writer, guard) = NonBlockingBuilder::default()
        // A debug log missing lines would mislead: slow the game down rather than drop them.
        .lossy(false)
        .finish(log_file(&config.log_dir)?);
    let file = fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_filter(EnvFilter::try_new(&config.log_filter)?);
    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .try_init()?;
    tracing::info!(
        folder = %config.log_dir.display(),
        filter = %config.log_filter,
        "debug log enabled"
    );
    Ok(Some(guard))
}

/// Opens the daily rolling log file in `dir`, creating the folder.
fn log_file(dir: &Path) -> Result<RollingFileAppender, LoggingError> {
    std::fs::create_dir_all(dir).map_err(|source| LoggingError::Folder {
        path: dir.display().to_string(),
        source,
    })?;
    Ok(RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .filename_suffix("log")
        .build(dir)?)
}
