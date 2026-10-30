//! Persistence of the `combats` table.

use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};

use crate::db::columns::{from_json, to_json};
use crate::db::db_error::DbError;
use crate::model::combat::Combat;

/// Combat in progress on a board.
///
/// # Errors
/// Decoding and database errors.
pub async fn for_board(
    conn: &mut SqliteConnection,
    board_id: i64,
) -> Result<Option<Combat>, DbError> {
    let row = sqlx::query("SELECT * FROM combats WHERE board_id = ?")
        .bind(board_id)
        .fetch_optional(conn)
        .await?;
    row.as_ref().map(combat_from_row).transpose()
}

/// Replaces the combat of a board (`None` ends it).
///
/// # Errors
/// Encoding and database errors.
pub async fn replace(
    conn: &mut SqliteConnection,
    board_id: i64,
    combat: Option<&Combat>,
) -> Result<(), DbError> {
    sqlx::query("DELETE FROM combats WHERE board_id = ?")
        .bind(board_id)
        .execute(&mut *conn)
        .await?;
    let Some(combat) = combat else { return Ok(()) };
    sqlx::query(
        "INSERT INTO combats (board_id, hero_id, mob_id, origin, hero_strength, hero_secret_card, mob_secret_card, \
         hero_strength_used, mob_strength_used, hero_exhausted, mob_exhausted, hero_cards, mob_cards) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(board_id)
    .bind(combat.hero_id)
    .bind(combat.mob_id)
    .bind(to_json(&combat.origin)?)
    .bind(combat.hero_strength)
    .bind(combat.hero_secret_card)
    .bind(combat.mob_secret_card)
    .bind(combat.hero_strength_used)
    .bind(combat.mob_strength_used)
    .bind(combat.hero_exhausted)
    .bind(combat.mob_exhausted)
    .bind(to_json(&combat.hero_cards)?)
    .bind(to_json(&combat.mob_cards)?)
    .execute(conn)
    .await?;
    Ok(())
}

fn combat_from_row(row: &SqliteRow) -> Result<Combat, DbError> {
    Ok(Combat {
        hero_id: row.try_get("hero_id")?,
        mob_id: row.try_get("mob_id")?,
        origin: from_json(row.try_get("origin")?)?,
        hero_strength: row.try_get("hero_strength")?,
        hero_secret_card: row.try_get("hero_secret_card")?,
        mob_secret_card: row.try_get("mob_secret_card")?,
        hero_strength_used: row.try_get("hero_strength_used")?,
        mob_strength_used: row.try_get("mob_strength_used")?,
        hero_exhausted: row.try_get("hero_exhausted")?,
        mob_exhausted: row.try_get("mob_exhausted")?,
        hero_cards: from_json(row.try_get("hero_cards")?)?,
        mob_cards: from_json(row.try_get("mob_cards")?)?,
    })
}
