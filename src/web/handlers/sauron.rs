//! Sauron's sheet and setup.

use askama::Template;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::model::game::Game;
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
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
    plot_cards_to_draw: u32,
    plots: Vec<CardView>,
}

/// Plots in play, by slot.
pub fn plots_in_play(game: &Game) -> Vec<CardView> {
    game.board.plots.values().map(|c| plot_card(*c)).collect()
}

/// `GET /sauron/{sauron_id}`.
///
/// # Errors
/// Database and rendering errors.
pub async fn show(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = ShowPage {
        layout: Layout::sauron(&access.session, &game),
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
pub async fn setup_screen(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let starting_plot = game
        .board
        .plots
        .values()
        .next()
        .copied()
        .unwrap_or_default();
    let to_draw = state
        .data
        .plot(starting_plot)
        .ok()
        .and_then(|p| p.plot_cards)
        .map_or(2, |d| d.draw);
    let page = SetupPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        plot_cards_to_draw: to_draw,
        plots: plots_in_play(&game),
    };
    render(&page, &page.layout)
}

/// `GET /sauron/{sauron_id}/setup_finished`.
///
/// # Errors
/// Rule and database errors.
pub async fn setup_finished(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, rng| {
        game.sauron_setup_finished(data, rng)
    })
    .await?;
    Ok(redirect(
        &format!("/sauron/{}/sauron_actions/edit", access.sauron_id),
        None,
    ))
}
