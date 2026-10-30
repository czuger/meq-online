//! Loading a game for a request, and changing it in one transaction.

use rand::rngs::StdRng;
use sqlx::SqliteConnection;

use crate::app_state::AppState;
use crate::db::{DbError, game_repo, user_repo};
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

/// Loads a game, applies `change`, lets the AI play the computer heroes the game then
/// waits on, saves it and raises the board version when it changed.
///
/// Nothing is saved when `change` fails. A change that left the game as it was keeps the
/// version, so a screen that saves on display doesn't reload itself forever.
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
    play_computer_heroes(&mut tx, &mut game, &state.data, &mut rng).await?;
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
        state.versions.board_changed(board_id);
    }
    Ok((game, value))
}

/// Lets the AI play the computer heroes, unless the board members play them by hand.
/// An AI failure is logged: the moves it played before are valid and kept.
///
/// # Errors
/// [`AppError::Db`] when the computer users can't be read.
async fn play_computer_heroes(
    conn: &mut SqliteConnection,
    game: &mut Game,
    data: &GameData,
    rng: &mut StdRng,
) -> Result<(), AppError> {
    if game.board.options.computer_heroes_by_hand {
        return Ok(());
    }
    let users = user_repo::computer_user_ids(conn).await?;
    let computer: Vec<i64> = game
        .heroes
        .iter()
        .filter(|h| users.contains(&h.user_id))
        .filter_map(|h| h.id)
        .collect();
    if computer.is_empty() {
        return Ok(());
    }
    match game.play_computer_heroes(&computer, data, rng) {
        Ok(moves) => tracing::debug!(moves, "computer heroes played"),
        Err(error) => tracing::warn!(%error, "computer heroes failed"),
    }
    Ok(())
}
