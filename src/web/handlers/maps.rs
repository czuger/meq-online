//! The map, seen by a hero or edited by Sauron.

use askama::Template;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
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
    };
    render(&page, &page.layout)
}

/// `GET /maps/{sauron_id}/edit`: Sauron's map, where he sets influence and shadow pool.
///
/// # Errors
/// Database and rendering errors.
pub async fn edit(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = MapPage {
        layout: Layout::sauron(&access.session, &game),
        board_id: access.board_id,
        sauron_id: Some(access.sauron_id),
        map: map_view(&game, &state.data, true),
    };
    render(&page, &page.layout)
}
