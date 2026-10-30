//! The map, seen by a hero or edited by Sauron.

use askama::Template;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::engine::influence::shadow_pool_left;
use crate::model::sauron_action::SauronAction;
use crate::web::access::{HeroAccess, SauronAccess};
use crate::web::app_error::AppError;
use crate::web::game_tx::read_game;
use crate::web::page::render;
use crate::web::views::layout::Layout;
use crate::web::views::map_view::{MapView, map_view};

#[derive(Template)]
#[template(path = "map.html")]
struct MapPage {
    layout: Layout,
    board_id: i64,
    /// Sauron's id when he edits the map.
    sauron_id: Option<i64>,
    map: MapView,
    /// Influence Sauron may place, shown to him.
    budget: Option<String>,
    /// Link ending a Place Influence action.
    finish_action: Option<String>,
}

/// `GET /maps/{hero_id}`: the map as seen by heroes.
///
/// # Errors
/// Database and rendering errors.
pub async fn show(State(state): State<AppState>, access: HeroAccess) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = MapPage {
        layout: Layout::hero(&access.session, &game, access.hero_id, &state.data),
        board_id: access.board_id,
        sauron_id: None,
        map: map_view(&game, &state.data, false),
        budget: None,
        finish_action: None,
    };
    render(&page, &page.layout)
}

/// `GET /maps/{sauron_id}/edit`: Sauron's map, where he places the influence he has.
///
/// # Errors
/// Database and rendering errors.
pub async fn edit(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    sauron_map_page(&state, &access).await
}

/// Sauron's map, with the influence he may place and, during a Place Influence action,
/// the link ending it.
///
/// # Errors
/// Database and rendering errors.
pub async fn sauron_map_page(
    state: &AppState,
    access: &SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(state, access.board_id).await?;
    let budget = &game.board.influence_budget;
    let in_action = game
        .board
        .current_action
        .is_some_and(|a| a.action == SauronAction::PlaceInfluence);
    let page = MapPage {
        layout: Layout::sauron(&access.session, &game),
        board_id: access.board_id,
        sauron_id: Some(access.sauron_id),
        map: map_view(&game, &state.data, true),
        budget: (budget.tokens > 0 || in_action).then(|| {
            format!(
                "Influence à placer : {} (dont {} au plus dans la réserve des Ténèbres). Cliquez sur un lieu pour y placer un jeton, Maj+clic pour reprendre un jeton que vous venez de placer.",
                budget.tokens,
                shadow_pool_left(budget)
            )
        }),
        finish_action: in_action
            .then(|| format!("/sauron/{}/sauron_actions/finish_action", access.sauron_id)),
    };
    render(&page, &page.layout)
}
