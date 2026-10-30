//! Persistence of the `mobs` table.

use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};

use crate::db::columns::{enum_from_text, enum_to_text, from_json, to_json};
use crate::db::db_error::DbError;
use crate::model::mob::Mob;

/// Mobs of a board, by ascending id.
///
/// # Errors
/// Decoding and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id), err(level = "debug"))]
pub async fn for_board(conn: &mut SqliteConnection, board_id: i64) -> Result<Vec<Mob>, DbError> {
    let rows = sqlx::query("SELECT * FROM mobs WHERE board_id = ? ORDER BY id")
        .bind(board_id)
        .fetch_all(conn)
        .await?;
    rows.iter().map(mob_from_row).collect()
}

/// Makes the stored mobs of a board match `mobs`: removed ones are deleted, new ones
/// inserted (their ids are filled in), the others updated.
///
/// # Errors
/// Encoding and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id), err(level = "debug"))]
pub async fn sync(
    conn: &mut SqliteConnection,
    board_id: i64,
    mobs: &mut [Mob],
) -> Result<(), DbError> {
    let stored: Vec<i64> = sqlx::query_scalar("SELECT id FROM mobs WHERE board_id = ?")
        .bind(board_id)
        .fetch_all(&mut *conn)
        .await?;
    for id in stored
        .iter()
        .filter(|id| !mobs.iter().any(|m| m.id == Some(**id)))
    {
        tracing::trace!(mob_id = id, "deleting mob");
        sqlx::query("DELETE FROM mobs WHERE id = ?")
            .bind(id)
            .execute(&mut *conn)
            .await?;
    }
    for mob in mobs.iter_mut() {
        tracing::trace!(mob = %mob.code, id = ?mob.id, "saving mob");
        save(conn, board_id, mob).await?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all, fields(board_id, mob = %mob.code), err(level = "debug"))]
async fn save(conn: &mut SqliteConnection, board_id: i64, mob: &mut Mob) -> Result<(), DbError> {
    if mob.id.is_none() {
        let result = sqlx::query(
            "INSERT INTO mobs (board_id, kind, code, name, location, fortitude, strength, life, max_life, \
             attack_deck, hand) VALUES (?, ?, ?, ?, ?, 0, 0, 0, 0, '', '[]')",
        )
        .bind(board_id)
        .bind(enum_to_text(&mob.kind)?)
        .bind(&mob.code)
        .bind(&mob.name)
        .bind(&mob.location)
        .execute(&mut *conn)
        .await?;
        mob.id = Some(result.last_insert_rowid());
    }
    sqlx::query(
        "UPDATE mobs SET kind = ?, code = ?, name = ?, location = ?, pool = ?, fortitude = ?, strength = ?, \
         life = ?, max_life = ?, attack_deck = ?, hand = ?, damages_taken_this_turn = ?, commanded = ? WHERE id = ?",
    )
    .bind(enum_to_text(&mob.kind)?)
    .bind(&mob.code)
    .bind(&mob.name)
    .bind(&mob.location)
    .bind(mob.pool.as_ref().map(enum_to_text).transpose()?)
    .bind(mob.fortitude)
    .bind(mob.strength)
    .bind(mob.life)
    .bind(mob.max_life)
    .bind(&mob.attack_deck)
    .bind(to_json(&mob.hand)?)
    .bind(mob.damages_taken_this_turn)
    .bind(mob.commanded)
    .bind(mob.id)
    .execute(conn)
    .await?;
    Ok(())
}

fn mob_from_row(row: &SqliteRow) -> Result<Mob, DbError> {
    let pool: Option<String> = row.try_get("pool")?;
    Ok(Mob {
        id: Some(row.try_get("id")?),
        kind: enum_from_text(row.try_get("kind")?)?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        location: row.try_get("location")?,
        pool: pool.as_deref().map(enum_from_text).transpose()?,
        fortitude: row.try_get("fortitude")?,
        strength: row.try_get("strength")?,
        life: row.try_get("life")?,
        max_life: row.try_get("max_life")?,
        attack_deck: row.try_get("attack_deck")?,
        hand: from_json(row.try_get("hand")?)?,
        damages_taken_this_turn: row.try_get("damages_taken_this_turn")?,
        commanded: row.try_get("commanded")?,
    })
}
