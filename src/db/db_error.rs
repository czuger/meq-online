//! Errors raised by the persistence layer.

use thiserror::Error;

use crate::engine::board_state::UnknownBoardState;

/// A database operation failed or returned inconsistent data.
#[derive(Debug, Error)]
pub enum DbError {
    /// SQLite or the driver failed.
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    /// A JSON column couldn't be encoded or decoded.
    #[error("invalid JSON column: {0}")]
    Json(#[from] serde_json::Error),
    /// A board state column holds an unknown value.
    #[error(transparent)]
    State(#[from] UnknownBoardState),
    /// A row required by a game is missing.
    #[error("{0} not found")]
    NotFound(&'static str),
}
