//! Sauron's sheet and setup.

use askama::Template;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::engine::deck::SauronDeck;
use crate::model::game::Game;
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::game_layout::sauron_layout;
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::layout::Layout;
use crate::web::views::pictures::{CardView, plot_card, shadow_card};

#[derive(Template)]
#[template(path = "sauron_show.html")]
struct ShowPage {
    layout: Layout,
    shadow_hand: Vec<CardView>,
    plot_hand: Vec<CardView>,
    plots: Vec<CardView>,
}

#[derive(Template)]
#[template(path = "sauron_setup.html")]
struct SetupPage {
    layout: Layout,
    sauron_id: i64,
    influence_left: i32,
    plot_cards_to_keep: usize,
}

/// Plots in play, by slot.
pub fn plots_in_play(game: &Game) -> Vec<CardView> {
    game.board.plots.values().map(|c| plot_card(*c)).collect()
}

/// `GET /sauron/{sauron_id}`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
pub async fn show(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = ShowPage {
        layout: sauron_layout(&state, &access.session, &game)
            .await?
            .without_hand(),
        shadow_hand: game
            .sauron
            .shadow_hand
            .iter()
            .map(|c| shadow_card(*c))
            .collect(),
        plot_hand: game
            .sauron
            .plot_hand
            .iter()
            .map(|c| plot_card(*c))
            .collect(),
        plots: plots_in_play(&game),
    };
    render(&page, &page.layout)
}

/// `GET /sauron/{sauron_id}/setup_screen`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
pub async fn setup_screen(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = SetupPage {
        layout: sauron_layout(&state, &access.session, &game)
            .await?
            .map_table(|table| table.with_influence_editor(access.sauron_id)),
        sauron_id: access.sauron_id,
        influence_left: game.board.influence_budget.tokens,
        plot_cards_to_keep: game.cards_to_keep(SauronDeck::Plot, &state.data),
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/setup_finished`.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
pub async fn setup_finished(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, rng| {
        game.sauron_setup_finished(data, rng)
    })
    .await?;
    // The Event step may ask a choice before the Action step.
    Ok(redirect(&format!("/boards/{}/play", access.board_id), None))
}
