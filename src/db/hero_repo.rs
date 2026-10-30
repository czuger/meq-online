//! Persistence of the `heroes` table.

use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};

use crate::db::columns::{from_json, to_json};
use crate::db::db_error::DbError;
use crate::model::hero::Hero;

/// Heroes of a board, by ascending id.
///
/// # Errors
/// Decoding and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id), err(level = "debug"))]
pub async fn for_board(conn: &mut SqliteConnection, board_id: i64) -> Result<Vec<Hero>, DbError> {
    let rows = sqlx::query("SELECT * FROM heroes WHERE board_id = ? ORDER BY id")
        .bind(board_id)
        .fetch_all(conn)
        .await?;
    rows.iter().map(hero_from_row).collect()
}

/// Board of a hero.
///
/// # Errors
/// [`DbError::NotFound`] when the hero doesn't exist.
#[tracing::instrument(level = "debug", skip_all, fields(hero_id), err(level = "debug"))]
pub async fn board_id(conn: &mut SqliteConnection, hero_id: i64) -> Result<i64, DbError> {
    sqlx::query_scalar("SELECT board_id FROM heroes WHERE id = ?")
        .bind(hero_id)
        .fetch_optional(conn)
        .await?
        .ok_or(DbError::NotFound("hero"))
}

/// Inserts a new hero, or updates it when it already has an id.
///
/// # Errors
/// Encoding and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id, hero = %hero.code), err(level = "debug"))]
pub async fn save(
    conn: &mut SqliteConnection,
    board_id: i64,
    hero: &mut Hero,
) -> Result<(), DbError> {
    if hero.id.is_none() {
        let result = sqlx::query(
            "INSERT INTO heroes (board_id, user_id, code, location, fortitude, strength, agility, wisdom, \
             playing_order, life_pool, hand, rest_pool, damage_pool, items, turn_record, corruption_cards) \
             VALUES (?, ?, ?, '', 0, 0, 0, 0, 0, '[]', '[]', '[]', '[]', '{}', '{}', '[]')",
        )
        .bind(board_id)
        .bind(hero.user_id)
        .bind(&hero.code)
        .execute(&mut *conn)
        .await?;
        hero.id = Some(result.last_insert_rowid());
    }
    update(conn, hero).await
}

#[tracing::instrument(level = "debug", skip_all, fields(hero = %hero.code), err(level = "debug"))]
async fn update(conn: &mut SqliteConnection, hero: &Hero) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE heroes SET user_id = ?, code = ?, location = ?, fortitude = ?, strength = ?, agility = ?, \
         wisdom = ?, favor = ?, quest = ?, advanced_quest = ?, playing_order = ?, turn = ?, active = ?, turn_finished = ?, \
         damages_taken_this_turn = ?, temporary_damages = ?, \
         cards_drawn_this_round = ?, life_pool = ?, hand = ?, rest_pool = ?, damage_pool = ?, items = ?, levels = ?, turn_record = ?, \
         corruption_cards = ? WHERE id = ?",
    )
    .bind(hero.user_id)
    .bind(&hero.code)
    .bind(&hero.location)
    .bind(hero.fortitude)
    .bind(hero.strength)
    .bind(hero.agility)
    .bind(hero.wisdom)
    .bind(hero.favor)
    .bind(hero.quest)
    .bind(hero.advanced_quest)
    .bind(hero.playing_order)
    .bind(hero.turn)
    .bind(hero.active)
    .bind(hero.turn_finished)
    .bind(hero.damages_taken_this_turn)
    .bind(hero.temporary_damages)
    .bind(hero.cards_drawn_this_round)
    .bind(to_json(&hero.life_pool)?)
    .bind(to_json(&hero.hand)?)
    .bind(to_json(&hero.rest_pool)?)
    .bind(to_json(&hero.damage_pool)?)
    .bind(to_json(&hero.items)?)
    .bind(to_json(&hero.levels)?)
    .bind(to_json(&hero.turn_record)?)
    .bind(to_json(&hero.corruption_cards)?)
    .bind(hero.id)
    .execute(conn)
    .await?;
    Ok(())
}

fn hero_from_row(row: &SqliteRow) -> Result<Hero, DbError> {
    Ok(Hero {
        id: Some(row.try_get("id")?),
        user_id: row.try_get("user_id")?,
        code: row.try_get("code")?,
        location: row.try_get("location")?,
        fortitude: row.try_get("fortitude")?,
        strength: row.try_get("strength")?,
        agility: row.try_get("agility")?,
        wisdom: row.try_get("wisdom")?,
        favor: row.try_get("favor")?,
        quest: row.try_get("quest")?,
        advanced_quest: row.try_get("advanced_quest")?,
        playing_order: row.try_get("playing_order")?,
        turn: row.try_get("turn")?,
        active: row.try_get("active")?,
        turn_finished: row.try_get("turn_finished")?,
        damages_taken_this_turn: row.try_get("damages_taken_this_turn")?,
        temporary_damages: row.try_get("temporary_damages")?,
        cards_drawn_this_round: row.try_get("cards_drawn_this_round")?,
        life_pool: from_json(row.try_get("life_pool")?)?,
        hand: from_json(row.try_get("hand")?)?,
        rest_pool: from_json(row.try_get("rest_pool")?)?,
        damage_pool: from_json(row.try_get("damage_pool")?)?,
        items: from_json(row.try_get("items")?)?,
        levels: from_json(row.try_get("levels")?)?,
        turn_record: from_json(row.try_get("turn_record")?)?,
        corruption_cards: from_json(row.try_get("corruption_cards")?)?,
    })
}
