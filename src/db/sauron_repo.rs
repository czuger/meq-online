//! Persistence of the `saurons` table.

use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};

use crate::db::columns::{from_json, to_json};
use crate::db::db_error::DbError;
use crate::model::sauron::Sauron;

/// The Sauron player of a board.
///
/// # Errors
/// [`DbError::NotFound`] when the board has no Sauron; decoding errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id), err(level = "debug"))]
pub async fn for_board(conn: &mut SqliteConnection, board_id: i64) -> Result<Sauron, DbError> {
    let row = sqlx::query("SELECT * FROM saurons WHERE board_id = ?")
        .bind(board_id)
        .fetch_optional(conn)
        .await?
        .ok_or(DbError::NotFound("sauron"))?;
    sauron_from_row(&row)
}

/// Board of a Sauron player.
///
/// # Errors
/// [`DbError::NotFound`] when the Sauron doesn't exist.
#[tracing::instrument(level = "debug", skip_all, fields(sauron_id), err(level = "debug"))]
pub async fn board_id(conn: &mut SqliteConnection, sauron_id: i64) -> Result<i64, DbError> {
    sqlx::query_scalar("SELECT board_id FROM saurons WHERE id = ?")
        .bind(sauron_id)
        .fetch_optional(conn)
        .await?
        .ok_or(DbError::NotFound("sauron"))
}

/// Inserts the Sauron player, or updates it when it already has an id.
///
/// # Errors
/// Encoding and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id), err(level = "debug"))]
pub async fn save(
    conn: &mut SqliteConnection,
    board_id: i64,
    sauron: &mut Sauron,
) -> Result<(), DbError> {
    if sauron.id.is_none() {
        let result = sqlx::query(
            "INSERT INTO saurons (board_id, user_id, plot_hand, shadow_hand, drawn_plot, drawn_shadow) \
             VALUES (?, ?, '[]', '[]', '[]', '[]')",
        )
        .bind(board_id)
        .bind(sauron.user_id)
        .execute(&mut *conn)
        .await?;
        sauron.id = Some(result.last_insert_rowid());
    }
    sqlx::query(
        "UPDATE saurons SET user_id = ?, active = ?, plot_hand = ?, shadow_hand = ?, drawn_plot = ?, \
         drawn_shadow = ? WHERE id = ?",
    )
    .bind(sauron.user_id)
    .bind(sauron.active)
    .bind(to_json(&sauron.plot_hand)?)
    .bind(to_json(&sauron.shadow_hand)?)
    .bind(to_json(&sauron.drawn_plot)?)
    .bind(to_json(&sauron.drawn_shadow)?)
    .bind(sauron.id)
    .execute(conn)
    .await?;
    Ok(())
}

fn sauron_from_row(row: &SqliteRow) -> Result<Sauron, DbError> {
    Ok(Sauron {
        id: Some(row.try_get("id")?),
        user_id: row.try_get("user_id")?,
        active: row.try_get("active")?,
        plot_hand: from_json(row.try_get("plot_hand")?)?,
        shadow_hand: from_json(row.try_get("shadow_hand")?)?,
        drawn_plot: from_json(row.try_get("drawn_plot")?)?,
        drawn_shadow: from_json(row.try_get("drawn_shadow")?)?,
    })
}
