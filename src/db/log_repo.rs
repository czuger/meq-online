//! Persistence of the `logs` table.

use sqlx::{Row, SqliteConnection};

use crate::db::columns::{actor_from_columns, actor_to_columns, from_json, to_json};
use crate::db::db_error::DbError;
use crate::model::log_entry::LogEntry;

/// A stored log entry with its creation time.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredLog {
    /// The entry.
    pub entry: LogEntry,
    /// Creation time, as stored by SQLite.
    pub created_at: String,
}

/// Appends entries to a board's log.
///
/// # Errors
/// Encoding and database errors.
pub async fn append(
    conn: &mut SqliteConnection,
    board_id: i64,
    entries: &[LogEntry],
) -> Result<(), DbError> {
    for entry in entries {
        let (kind, hero_id) = actor_to_columns(entry.actor);
        sqlx::query("INSERT INTO logs (board_id, actor_kind, hero_id, action, params) VALUES (?, ?, ?, ?, ?)")
            .bind(board_id)
            .bind(kind)
            .bind(hero_id)
            .bind(&entry.action)
            .bind(to_json(&entry.params)?)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Log of a board, newest first.
///
/// # Errors
/// Decoding and database errors.
pub async fn for_board(
    conn: &mut SqliteConnection,
    board_id: i64,
) -> Result<Vec<StoredLog>, DbError> {
    let rows = sqlx::query("SELECT * FROM logs WHERE board_id = ? ORDER BY id DESC")
        .bind(board_id)
        .fetch_all(conn)
        .await?;
    rows.iter()
        .map(|row| {
            let kind: String = row.try_get("actor_kind")?;
            Ok(StoredLog {
                entry: LogEntry {
                    actor: actor_from_columns(&kind, row.try_get("hero_id")?),
                    action: row.try_get("action")?,
                    params: from_json(row.try_get("params")?)?,
                },
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}
