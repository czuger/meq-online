//! Errors raised while loading or querying static game data.

use std::path::PathBuf;

use thiserror::Error;

/// Failure to load a data file or to find a code in the loaded data.
#[derive(Debug, Error)]
pub enum GameDataError {
    /// A data file couldn't be read.
    #[error("cannot read {path}: {source}")]
    Io {
        /// File that failed.
        path: PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// A data file isn't valid YAML or doesn't match the expected shape.
    #[error("invalid data in {path}: {source}")]
    Yaml {
        /// File that failed.
        path: PathBuf,
        /// Underlying YAML error.
        source: serde_norway::Error,
    },
    /// A data file was expected to hold a mapping at the top level.
    #[error("{path} is not a YAML mapping")]
    NotAMapping {
        /// File that failed.
        path: PathBuf,
    },
    /// A code was looked up but doesn't exist in the data.
    #[error("unknown {kind} '{code}'")]
    UnknownCode {
        /// Kind of record looked up (hero, location, ...).
        kind: &'static str,
        /// The code that was not found.
        code: String,
    },
}

impl GameDataError {
    /// Builds an [`GameDataError::UnknownCode`].
    ///
    /// # Arguments
    /// * `kind` - kind of record looked up.
    /// * `code` - code that was not found.
    pub fn unknown(kind: &'static str, code: impl ToString) -> Self {
        Self::UnknownCode {
            kind,
            code: code.to_string(),
        }
    }
}
