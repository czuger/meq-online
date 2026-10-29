//! Loading and saving a whole [`Game`].

use sqlx::SqliteConnection;

use crate::db::db_error::DbError;
use crate::db::{board_repo, combat_repo, hero_repo, log_repo, mob_repo, sauron_repo};
use crate::model::game::Game;

/// Loads a board with all its pieces.
///
/// # Errors
/// [`DbError::NotFound`] when the board or its Sauron is missing; decoding errors.
pub async fn load(conn: &mut SqliteConnection, board_id: i64) -> Result<Game, DbError> {
    Ok(Game {
        board: board_repo::find(conn, board_id).await?,
        sauron: sauron_repo::for_board(conn, board_id).await?,
        heroes: hero_repo::for_board(conn, board_id).await?,
        mobs: mob_repo::for_board(conn, board_id).await?,
        combat: combat_repo::for_board(conn, board_id).await?,
        new_logs: vec![],
    })
}

/// Saves a game: inserts it when new, fills in the ids of new pieces and appends
/// its new log entries. Run it inside a transaction.
///
/// # Errors
/// Encoding and database errors.
pub async fn save(conn: &mut SqliteConnection, game: &mut Game) -> Result<(), DbError> {
    let board_id = match game.board.id {
        Some(id) => id,
        None => {
            board_repo::insert(conn, &mut game.board).await?;
            game.board.id.ok_or(DbError::NotFound("board id"))?
        }
    };
    sauron_repo::save(conn, board_id, &mut game.sauron).await?;
    for hero in &mut game.heroes {
        hero_repo::save(conn, board_id, hero).await?;
    }
    combat_repo::replace(conn, board_id, None).await?;
    mob_repo::sync(conn, board_id, &mut game.mobs).await?;
    combat_repo::replace(conn, board_id, game.combat.as_ref()).await?;
    board_repo::update(conn, &game.board).await?;
    log_repo::append(conn, board_id, &game.new_logs).await?;
    game.new_logs.clear();
    Ok(())
}
