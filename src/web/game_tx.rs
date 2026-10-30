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
#[tracing::instrument(level = "debug", skip(state))]
pub async fn read_game(state: &AppState, board_id: i64) -> Result<Game, AppError> {
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let game = game_repo::load(&mut conn, board_id).await?;
    tracing::debug!(state = %game.board.state, "game read");
    Ok(game)
}

/// Loads a game, applies `change`, saves it and tells open pages when it changed.
///
/// Nothing is saved when `change` fails. Pages are not told about a change that left
/// the game as it was, so a screen that saves on display doesn't reload itself forever.
///
/// # Returns
/// The game after the change, and the value returned by `change`.
///
/// # Errors
/// [`AppError::Engine`] when the rules refuse the change, [`AppError::Db`] on
/// database failure.
#[tracing::instrument(level = "debug", skip(state, change))]
pub async fn change_game<T>(
    state: &AppState,
    board_id: i64,
    change: impl FnOnce(&mut Game, &GameData, &mut StdRng) -> Result<T, EngineError>,
) -> Result<(Game, T), AppError> {
    let mut tx = state.db.begin().await.map_err(DbError::from)?;
    let mut game = game_repo::load(&mut tx, board_id).await?;
    let before = game.clone();
    tracing::debug!(state = %before.board.state, "game loaded for a change");
    let mut rng: StdRng = rand::make_rng();
    let value = match change(&mut game, &state.data, &mut rng) {
        Ok(value) => value,
        Err(error) => {
            tracing::debug!(%error, "change refused, nothing saved");
            return Err(error.into());
        }
    };
    let changed = game != before;
    tracing::debug!(
        from = %before.board.state,
        to = %game.board.state,
        changed,
        new_logs = game.new_logs.len(),
        "change applied"
    );
    game_repo::save(&mut tx, &mut game).await?;
    tx.commit().await.map_err(DbError::from)?;
    tracing::debug!("change committed");
    if changed {
        state.refresh.board_changed(board_id);
    }
    Ok((game, value))
}
