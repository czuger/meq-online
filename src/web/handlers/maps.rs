//! The map, seen by a hero or edited by Sauron.

use askama::Template;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::engine::board_state::BoardState;
use crate::engine::influence::shadow_pool_left;
use crate::engine::route::{RouteActor, route_for};
use crate::model::sauron_action::SauronAction;
use crate::web::access::{HeroAccess, SauronAccess};
use crate::web::app_error::AppError;
use crate::web::game_layout::{hero_layout, sauron_layout};
use crate::web::game_tx::read_game;
use crate::web::page::render;
use crate::web::views::layout::Layout;

#[derive(Template)]
#[template(path = "map.html")]
struct MapPage {
    layout: Layout,
    /// Influence Sauron may place, shown to him.
    budget: Option<String>,
    /// Link ending a Place Influence action.
    finish_action: Option<String>,
    /// The seat's current screen, when it is its turn.
    back_url: Option<String>,
}

/// `GET /maps/{hero_id}`: the map as seen by heroes.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn show(State(state): State<AppState>, access: HeroAccess) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = MapPage {
        layout: hero_layout(&state, &access.session, &game, access.hero_id).await?,
        budget: None,
        finish_action: None,
        back_url: game.hero(access.hero_id)?.active.then(|| {
            route_for(
                game.board.state,
                access.board_id,
                RouteActor::Hero(access.hero_id),
            )
        }),
    };
    render(&page, &page.layout)
}

/// `GET /maps/{sauron_id}/edit`: Sauron's map, where he places the influence he has.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
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
#[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
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
        layout: sauron_layout(state, &access.session, &game)
            .await?
            .map_table(|table| table.with_influence_editor(access.sauron_id)),
        budget: (budget.tokens > 0 || in_action).then(|| {
            format!(
                "Influence à placer : {} (dont {} au plus dans la réserve des Ténèbres). Cliquez sur un lieu pour y placer un jeton, Maj+clic pour reprendre un jeton que vous venez de placer.",
                budget.tokens,
                shadow_pool_left(budget)
            )
        }),
        finish_action: in_action
            .then(|| format!("/sauron/{}/sauron_actions/finish_action", access.sauron_id)),
        back_url: (game.sauron.active
            && game.board.state != BoardState::PlaceInfluenceScreenSauronActions)
            .then(|| {
                route_for(
                    game.board.state,
                    access.board_id,
                    RouteActor::Sauron(access.sauron_id),
                )
            }),
    };
    render(&page, &page.layout)
}
