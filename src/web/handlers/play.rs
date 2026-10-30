//! The play page of a board: it opens the screen of a seat the user plays when the
//! game waits on it, or shows what the game waits for; and the status the game table
//! polls to follow the game.

use askama::Template;
use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::game_data::GameData;
use crate::model::choice::Chooser;
use crate::model::game::Game;
use crate::web::access::BoardAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::game_layout::{board_players, my_screen, viewer_layout};
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::layout::Layout;
use crate::web::views::state_label::state_label;

#[derive(Template)]
#[template(path = "boards_inactive.html")]
struct WaitingPage {
    layout: Layout,
    board_id: i64,
    /// Who the game waits on.
    waiting_for: String,
    /// What the game waits for.
    state: &'static str,
    /// The game is over.
    finished: bool,
}

/// Version of the board, and where the table should be when it changed.
#[derive(Debug, Serialize)]
struct Status {
    /// Current version of the board.
    version: String,
    /// Screen of a seat the user plays that the game waits on; only computed when the
    /// version changed.
    url: Option<String>,
}

/// Query of the status: the version the page shows.
#[derive(Debug, Deserialize)]
pub struct StatusQuery {
    /// Version of the board when the page was displayed.
    version: Option<String>,
}

/// `GET /boards/{board_id}/play`: the screen of the user's seat the game waits on, or
/// the waiting page. A game left waiting on a computer hero (after a server restart)
/// lets the AI play first.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id), err(level = "debug"))]
pub async fn play(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let (game, ()) = change_game(&state, access.board_id, |_, _, _| Ok(())).await?;
    let players = board_players(&state, &game).await?;
    if let Some(url) = my_screen(&game, &players, &access.session) {
        return Ok(redirect(&url, None));
    }
    let layout = viewer_layout(&state, &access.session, &game).await?;
    render(
        &WaitingPage {
            layout: layout.clone(),
            board_id: access.board_id,
            waiting_for: waiting_for(&game, &state.data),
            state: state_label(game.board.state),
            finished: game.board.winner.is_some(),
        },
        &layout,
    )
}

/// `GET /boards/{board_id}/status?version=…`: JSON `{ "version": …, "url": … }`, the
/// board version and the screen the user should be on.
///
/// The game is only loaded when the version differs from the page's, so polling an
/// unchanged board stays cheap.
///
/// # Errors
/// Database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, version = ?query.version), err(level = "debug"))]
pub async fn status(
    State(state): State<AppState>,
    access: BoardAccess,
    Query(query): Query<StatusQuery>,
) -> Result<Response, AppError> {
    let version = state.versions.version(access.board_id);
    if query.version.as_deref() == Some(version.as_str()) {
        return Ok(Json(Status { version, url: None }).into_response());
    }
    let game = read_game(&state, access.board_id).await?;
    let players = board_players(&state, &game).await?;
    let url = my_screen(&game, &players, &access.session);
    tracing::debug!(%version, ?url, "board changed");
    Ok(Json(Status { version, url }).into_response())
}

/// `GET /boards/{board_id}/inactive_actor`: kept for old links, same as the play page.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id), err(level = "debug"))]
pub async fn inactive_actor(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    play(State(state), access).await
}

/// Names of the seats the game waits on.
fn waiting_for(game: &Game, data: &GameData) -> String {
    let hero_name = |id: i64| {
        game.heroes
            .iter()
            .find(|h| h.id == Some(id))
            .and_then(|h| data.heroes.get(&h.code))
            .map_or_else(|| "un héros".to_owned(), |d| d.name.clone())
    };
    if let Some(choice) = &game.board.pending_choice {
        return match choice.chooser {
            Chooser::Sauron => "Sauron".to_owned(),
            Chooser::Hero(id) => hero_name(id),
            Chooser::Heroes => "les héros".to_owned(),
        };
    }
    let names: Vec<String> = game
        .sauron
        .active
        .then(|| "Sauron".to_owned())
        .into_iter()
        .chain(
            game.heroes
                .iter()
                .filter(|h| h.active)
                .filter_map(|h| h.id)
                .map(hero_name),
        )
        .collect();
    if names.is_empty() {
        "personne".to_owned()
    } else {
        names.join(", ")
    }
}
