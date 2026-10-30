//! Sauron's Action step: the action tracks, influence and the shadow pool.

use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::engine::influence::InfluenceRefusal;
use crate::engine::route::{RouteActor, route_for};
use crate::engine::sauron_actions::space_values;
use crate::model::sauron_action::SauronAction;
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::forms::number;
use crate::web::game_tx::{change_game, read_game};
use crate::web::handlers::maps::sauron_map_page;
use crate::web::page::render;
use crate::web::views::layout::Layout;

/// One space of a track.
struct Space {
    value: i32,
    covered: bool,
}

/// One action track.
struct Track {
    code: &'static str,
    label: &'static str,
    spaces: Vec<Space>,
    available: bool,
}

#[derive(Template)]
#[template(path = "sauron_actions_edit.html")]
struct EditPage {
    layout: Layout,
    sauron_id: i64,
    actions_taken: i32,
    actions_per_turn: i32,
    tracks: Vec<Track>,
    action_in_progress: Option<String>,
    can_terminate: bool,
}

/// Chosen action track.
#[derive(Debug, Deserialize)]
pub struct StartForm {
    action: String,
}

/// Influence form sent by the map.
#[derive(Debug, Deserialize)]
pub struct InfluenceForm {
    location: String,
    val: String,
}

/// Answer to an influence change.
#[derive(Debug, Serialize)]
pub struct InfluenceAnswer {
    result: bool,
    message: Option<String>,
    location: String,
    val: i32,
}

/// Shadow pool form sent by the map.
#[derive(Debug, Deserialize)]
pub struct PoolForm {
    current_val: String,
}

/// Answer to a shadow pool change.
#[derive(Debug, Serialize)]
pub struct PoolAnswer {
    pool: i32,
    message: Option<String>,
}

fn track_label(action: SauronAction) -> &'static str {
    match action {
        SauronAction::PlaceInfluence => "Placer de l'influence",
        SauronAction::DrawCards => "Piocher des cartes des Ténèbres et intrigue",
        SauronAction::Command => "Commander monstres et serviteurs",
    }
}

/// French reason of a refused influence change.
fn refusal_message(refusal: InfluenceRefusal, name: &str) -> String {
    match refusal {
        InfluenceRefusal::NoConnection => {
            format!("Aucune liaison avec une forteresse des Ténèbres depuis {name}.")
        }
        InfluenceRefusal::MaxReached => format!(
            "Impossible de placer l'influence à {name} : elle dépasserait la valeur de la forteresse des Ténèbres liée."
        ),
        InfluenceRefusal::Haven => {
            format!("Impossible de placer de l'influence à {name} : c'est un refuge.")
        }
        InfluenceRefusal::NoToken => "Vous n'avez plus d'influence à placer.".to_owned(),
        InfluenceRefusal::NotPlacedNow => {
            "Vous ne pouvez reprendre que l'influence que vous venez de placer.".to_owned()
        }
        InfluenceRefusal::ShadowPoolFull => {
            "Vous ne pouvez plus placer d'influence dans la réserve des Ténèbres.".to_owned()
        }
    }
}

/// `GET /sauron/{sauron_id}/sauron_actions/edit`: the action tracks.
///
/// # Errors
/// Database and rendering errors.
pub async fn edit(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let can_start = game.board.current_action.is_none() && game.sauron_actions_left() > 0;
    let tracks = SauronAction::ALL
        .into_iter()
        .map(|action| Track {
            code: action.code(),
            label: track_label(action),
            spaces: space_values(action)
                .into_iter()
                .zip(1..)
                .map(|(value, space)| Space {
                    value,
                    covered: game
                        .board
                        .sauron_actions
                        .iter()
                        .any(|t| t.action == action && t.space == space),
                })
                .collect(),
            available: can_start && game.next_action_space(action).is_some(),
        })
        .collect();
    let page = EditPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        actions_taken: game.board.sauron_actions_count,
        actions_per_turn: game.sauron_actions_per_turn(),
        tracks,
        action_in_progress: game.board.current_action.map(|_| {
            route_for(
                game.board.state,
                access.board_id,
                RouteActor::Sauron(access.sauron_id),
            )
        }),
        can_terminate: game.sauron_actions_left() == 0,
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/sauron_actions/start`: places a token on a track.
///
/// # Errors
/// [`AppError::BadRequest`] for an unknown track; rule and database errors.
pub async fn start(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<StartForm>,
) -> Result<Response, AppError> {
    let action = SauronAction::from_code(&form.action)
        .ok_or_else(|| AppError::BadRequest("unknown action".to_owned()))?;
    let (game, ()) = change_game(&state, access.board_id, |game, _, rng| {
        game.start_sauron_action(action, rng)
    })
    .await?;
    let to = route_for(
        game.board.state,
        access.board_id,
        RouteActor::Sauron(access.sauron_id),
    );
    Ok(redirect(&to, None))
}

/// `GET /sauron/{sauron_id}/sauron_actions/place_influence_screen`: the map, to place
/// the action's influence.
///
/// # Errors
/// Database and rendering errors.
pub async fn place_influence_screen(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    sauron_map_page(&state, &access).await
}

/// `GET /sauron/{sauron_id}/sauron_actions/finish_action`: ends the action in progress.
///
/// # Errors
/// Rule and database errors.
pub async fn finish_action(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, _, _| {
        game.finish_sauron_action()
    })
    .await?;
    Ok(redirect(
        &format!("/sauron/{}/sauron_actions/edit", access.sauron_id),
        None,
    ))
}

/// `POST /sauron/{sauron_id}/sauron_actions/set_influence`.
///
/// # Errors
/// [`AppError::BadRequest`] for a malformed value; database errors.
pub async fn set_influence(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<InfluenceForm>,
) -> Result<Response, AppError> {
    let value: i32 = number(&form.val, "influence")?;
    let location = form.location.clone();
    let (_, refusal) = change_game(&state, access.board_id, |game, data, _| {
        Ok(game.set_influence(&location, value, data))
    })
    .await?;
    let name = state.data.locations.name(&form.location).to_owned();
    let message = refusal.map(|r| refusal_message(r, &name));
    Ok(Json(InfluenceAnswer {
        result: message.is_none(),
        message,
        location: form.location,
        val: value,
    })
    .into_response())
}

/// `GET /sauron/{sauron_id}/sauron_actions/terminate`: Sauron ends his turn.
///
/// # Errors
/// Rule and database errors.
pub async fn terminate(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, _, _| {
        game.terminate_sauron_turn()
    })
    .await?;
    Ok(redirect("/boards", None))
}

/// `PATCH /shadow_pools/{sauron_id}/update_from_map`: fills (value 0) or empties a slot.
///
/// # Returns
/// The shadow pool and the reason of a refusal, as JSON.
///
/// # Errors
/// [`AppError::BadRequest`] for a malformed value; database errors.
pub async fn update_shadow_pool(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<PoolForm>,
) -> Result<Response, AppError> {
    let current: i32 = number(&form.current_val, "current value")?;
    let (game, result) = change_game(&state, access.board_id, |game, _, _| {
        Ok(game.change_shadow_pool(current == 0))
    })
    .await?;
    Ok(Json(PoolAnswer {
        pool: game.board.shadow_pool,
        message: result.err().map(|r| refusal_message(r, "")),
    })
    .into_response())
}
