//! Loading a game for a request, and changing it in one transaction.

use rand::rngs::StdRng;

use crate::app_state::AppState;
use crate::db::{DbError, game_repo};
use crate::engine::EngineError;
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::web::app_error::AppError;

/// Loads a game for display.
///
/// # Errors
/// [`AppError::Db`] when loading fails.
pub async fn read_game(state: &AppState, board_id: i64) -> Result<Game, AppError> {
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    Ok(game_repo::load(&mut conn, board_id).await?)
}

/// Loads a game, applies `change`, saves it and tells open pages it changed.
///
/// Nothing is saved when `change` fails.
///
/// # Returns
/// The game after the change, and the value returned by `change`.
///
/// # Errors
/// [`AppError::Engine`] when the rules refuse the change, [`AppError::Db`] on
/// database failure.
pub async fn change_game<T>(
    state: &AppState,
    board_id: i64,
    change: impl FnOnce(&mut Game, &GameData, &mut StdRng) -> Result<T, EngineError>,
) -> Result<(Game, T), AppError> {
    let mut tx = state.db.begin().await.map_err(DbError::from)?;
    let mut game = game_repo::load(&mut tx, board_id).await?;
    let mut rng: StdRng = rand::make_rng();
    let value = change(&mut game, &state.data, &mut rng)?;
    game_repo::save(&mut tx, &mut game).await?;
    tx.commit().await.map_err(DbError::from)?;
    state.refresh.board_changed(board_id);
    Ok((game, value))
}
