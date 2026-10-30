//! Persistence of the `boards` table.

use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};

use crate::db::columns::{enum_from_text, enum_to_text, from_json, to_json};
use crate::db::db_error::DbError;
use crate::model::board::{Board, StoryMarkers};

const COLUMNS: &str = "id, state, max_heroes, waiting_heroes, turn, current_hero_id, winner, story_heroes, story_ring, \
    story_conquest, story_corruption, shadow_pool, influence, characters, favors, plots, sauron_actions, \
    sauron_actions_count, current_action, decks, monster_pools, monster_discards, ringwraiths_return, \
    influence_budget, last_event, current_event, held_event, pending_choice, turn_effects, heroes_objective, \
    sauron_objective, options";

/// Loads a board.
///
/// # Errors
/// [`DbError::NotFound`] when the board doesn't exist; decoding errors.
pub async fn find(conn: &mut SqliteConnection, board_id: i64) -> Result<Board, DbError> {
    let sql = format!("SELECT {COLUMNS} FROM boards WHERE id = ?");
    let row = sqlx::query(sqlx::AssertSqlSafe(sql))
        .bind(board_id)
        .fetch_optional(conn)
        .await?
        .ok_or(DbError::NotFound("board"))?;
    board_from_row(&row)
}

/// Ids of every board, most recently updated first.
///
/// # Errors
/// [`DbError::Sqlx`] on database failure.
pub async fn all_ids(conn: &mut SqliteConnection) -> Result<Vec<i64>, DbError> {
    let ids = sqlx::query_scalar("SELECT id FROM boards ORDER BY updated_at DESC, id DESC")
        .fetch_all(conn)
        .await?;
    Ok(ids)
}

/// Inserts a new board and stores its id in `board.id`.
///
/// # Errors
/// Encoding and database errors.
pub async fn insert(conn: &mut SqliteConnection, board: &mut Board) -> Result<(), DbError> {
    let result = sqlx::query(
        "INSERT INTO boards (state, max_heroes, turn, shadow_pool, influence, characters, favors, plots, \
         sauron_actions, decks, monster_pools) VALUES (?, ?, ?, ?, '{}', '{}', '[]', '{}', '[]', '{}', '{}')",
    )
    .bind(board.state.as_str())
    .bind(board.max_heroes)
    .bind(board.turn)
    .bind(board.shadow_pool)
    .execute(&mut *conn)
    .await?;
    board.id = Some(result.last_insert_rowid());
    update(conn, board).await
}

/// Saves every column of an existing board.
///
/// # Errors
/// [`DbError::NotFound`] when the board has no id; encoding and database errors.
pub async fn update(conn: &mut SqliteConnection, board: &Board) -> Result<(), DbError> {
    let id = board.id.ok_or(DbError::NotFound("board id"))?;
    let winner = board.winner.as_ref().map(enum_to_text).transpose()?;
    sqlx::query(
        "UPDATE boards SET state = ?, max_heroes = ?, waiting_heroes = ?, turn = ?, current_hero_id = ?, winner = ?, \
         story_heroes = ?, story_ring = ?, story_conquest = ?, story_corruption = ?, shadow_pool = ?, \
         influence = ?, characters = ?, favors = ?, plots = ?, sauron_actions = ?, sauron_actions_count = ?, \
         current_action = ?, decks = ?, monster_pools = ?, monster_discards = ?, ringwraiths_return = ?, influence_budget = ?, \
         last_event = ?, current_event = ?, held_event = ?, pending_choice = ?, turn_effects = ?, heroes_objective = ?, sauron_objective = ?, options = ?, \
         updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(board.state.as_str())
    .bind(board.max_heroes)
    .bind(to_json(&board.waiting_heroes)?)
    .bind(board.turn)
    .bind(board.current_hero_id)
    .bind(winner)
    .bind(board.story.heroes)
    .bind(board.story.ring)
    .bind(board.story.conquest)
    .bind(board.story.corruption)
    .bind(board.shadow_pool)
    .bind(to_json(&board.influence)?)
    .bind(to_json(&board.characters)?)
    .bind(to_json(&board.favors)?)
    .bind(to_json(&board.plots)?)
    .bind(to_json(&board.sauron_actions)?)
    .bind(board.sauron_actions_count)
    .bind(optional_json(&board.current_action)?)
    .bind(to_json(&board.decks)?)
    .bind(to_json(&board.monster_pools)?)
    .bind(to_json(&board.monster_discards)?)
    .bind(board.ringwraiths_return)
    .bind(to_json(&board.influence_budget)?)
    .bind(optional_json(&board.last_event)?)
    .bind(optional_json(&board.current_event)?)
    .bind(optional_json(&board.held_event)?)
    .bind(optional_json(&board.pending_choice)?)
    .bind(to_json(&board.turn_effects)?)
    .bind(board.heroes_objective)
    .bind(board.sauron_objective)
    .bind(to_json(&board.options)?)
    .bind(id)
    .execute(conn)
    .await?;
    Ok(())
}

fn board_from_row(row: &SqliteRow) -> Result<Board, DbError> {
    let winner: Option<String> = row.try_get("winner")?;
    Ok(Board {
        id: Some(row.try_get("id")?),
        state: row.try_get::<String, _>("state")?.parse()?,
        max_heroes: row.try_get("max_heroes")?,
        waiting_heroes: from_json(row.try_get("waiting_heroes")?)?,
        turn: row.try_get("turn")?,
        current_hero_id: row.try_get("current_hero_id")?,
        winner: winner.as_deref().map(enum_from_text).transpose()?,
        story: StoryMarkers {
            heroes: row.try_get("story_heroes")?,
            ring: row.try_get("story_ring")?,
            conquest: row.try_get("story_conquest")?,
            corruption: row.try_get("story_corruption")?,
        },
        shadow_pool: row.try_get("shadow_pool")?,
        influence: from_json(row.try_get("influence")?)?,
        characters: from_json(row.try_get("characters")?)?,
        favors: from_json(row.try_get("favors")?)?,
        plots: from_json(row.try_get("plots")?)?,
        sauron_actions: from_json(row.try_get("sauron_actions")?)?,
        sauron_actions_count: row.try_get("sauron_actions_count")?,
        current_action: optional_from_json(row, "current_action")?,
        decks: from_json(row.try_get("decks")?)?,
        monster_pools: from_json(row.try_get("monster_pools")?)?,
        monster_discards: from_json(row.try_get("monster_discards")?)?,
        ringwraiths_return: row.try_get("ringwraiths_return")?,
        influence_budget: from_json(row.try_get("influence_budget")?)?,
        last_event: optional_from_json(row, "last_event")?,
        current_event: optional_from_json(row, "current_event")?,
        held_event: optional_from_json(row, "held_event")?,
        pending_choice: optional_from_json(row, "pending_choice")?,
        turn_effects: from_json(row.try_get("turn_effects")?)?,
        heroes_objective: row.try_get("heroes_objective")?,
        sauron_objective: row.try_get("sauron_objective")?,
        options: from_json(row.try_get("options")?)?,
    })
}

/// Encodes an optional value as a nullable JSON column.
fn optional_json<T: Serialize>(value: &Option<T>) -> Result<Option<String>, DbError> {
    value.as_ref().map(to_json).transpose()
}

/// Decodes a nullable JSON column.
fn optional_from_json<T: DeserializeOwned>(
    row: &SqliteRow,
    column: &str,
) -> Result<Option<T>, DbError> {
    row.try_get::<Option<String>, _>(column)?
        .as_deref()
        .map(from_json)
        .transpose()
}
