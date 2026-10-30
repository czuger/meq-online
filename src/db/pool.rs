//! Database connection and schema setup.

use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::db::db_error::DbError;

/// Opens the SQLite database at `url` (created if missing) and applies `schema.sql`.
///
/// In-memory databases (`sqlite::memory:`) use a single connection, since each
/// connection would otherwise see its own empty database.
///
/// # Errors
/// [`DbError::Sqlx`] when the database can't be opened or the schema fails.
#[tracing::instrument(level = "debug", skip_all, fields(%url), err(level = "debug"))]
pub async fn connect(url: &str) -> Result<SqlitePool, DbError> {
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let max_connections = if url.contains(":memory:") { 1 } else { 5 };
    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect_with(options)
        .await?;
    sqlx::raw_sql(include_str!("../../schema.sql"))
        .execute(&pool)
        .await?;
    Ok(pool)
}
