//! Shadow cards Sauron plays during his Action step.

use axum::Form;
use axum::extract::State;
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::engine::route::{RouteActor, route_for};
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::forms::number;
use crate::web::game_tx::change_game;

/// Chosen Shadow card.
#[derive(Debug, Deserialize)]
pub struct PlayForm {
    card: String,
}

/// `POST /sauron/{sauron_id}/shadow_cards/play`: Sauron plays a Shadow card between two
/// actions; a card asking a decision opens the choice screen.
///
/// # Errors
/// [`AppError::BadRequest`] for a malformed card; rule and database errors.
pub async fn play(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<PlayForm>,
) -> Result<Response, AppError> {
    let card: u32 = number(&form.card, "card")?;
    let (game, ()) = change_game(&state, access.board_id, |game, data, rng| {
        game.play_action_shadow(card, data, rng)
    })
    .await?;
    let actor = if game.sauron.active {
        RouteActor::Sauron(access.sauron_id)
    } else {
        // A hero answers first ("Storms of Mordor").
        let hero = game.heroes.iter().find(|h| h.active).and_then(|h| h.id);
        hero.map_or(RouteActor::Sauron(access.sauron_id), RouteActor::Hero)
    };
    Ok(redirect(
        &route_for(game.board.state, access.board_id, actor),
        None,
    ))
}
