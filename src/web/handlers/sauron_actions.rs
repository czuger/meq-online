//! Sauron's action markers, influence and shadow pool.

use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::engine::sauron_turn::InfluenceRefusal;
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::forms::number;
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::layout::Layout;

/// Action markers in display columns: `(code, label)`.
const ACTION_COLUMNS: [[(&str, &str); 3]; 3] = [
    [
        ("place_influence_1", "Placer 6 influences"),
        ("place_influence_2", "Placer 5 influences"),
        ("place_influence_3", "Placer 4 influences"),
    ],
    [
        ("draw_cards_1", "Piocher 2 cartes"),
        ("draw_cards_2", "Piocher 2 cartes"),
        ("draw_cards_3", "Piocher 1 carte"),
    ],
    [
        ("command_1", "Commander 3"),
        ("command_2", "Commander 2"),
        ("command_3", "Commander 1"),
    ],
];

struct ActionBox {
    code: &'static str,
    label: &'static str,
    checked: bool,
}

#[derive(Template)]
#[template(path = "sauron_actions_edit.html")]
struct EditPage {
    layout: Layout,
    sauron_id: i64,
    used_actions: i32,
    columns: Vec<Vec<ActionBox>>,
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

/// `GET /sauron/{sauron_id}/sauron_actions/edit`.
///
/// # Errors
/// Database and rendering errors.
pub async fn edit(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let columns = ACTION_COLUMNS
        .iter()
        .map(|column| {
            column
                .iter()
                .map(|(code, label)| ActionBox {
                    code,
                    label,
                    checked: game.board.sauron_actions.iter().any(|a| a == code),
                })
                .collect()
        })
        .collect();
    let page = EditPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        used_actions: game.board.sauron_actions_count,
        columns,
    };
    render(&page, &page.layout)
}

/// `PATCH /sauron/{sauron_id}/sauron_actions`: replaces the action markers.
///
/// # Returns
/// The number of updates this turn, as JSON.
///
/// # Errors
/// Database errors.
pub async fn update(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(fields): Form<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    let known = |code: &str| ACTION_COLUMNS.iter().flatten().any(|(c, _)| *c == code);
    let actions: Vec<String> = fields
        .into_iter()
        .filter(|(field, value)| (field == "actions" || field == "actions[]") && known(value))
        .map(|(_, value)| value)
        .collect();
    let (_, count) = change_game(&state, access.board_id, |game, _, _| {
        Ok(game.update_sauron_actions(actions))
    })
    .await?;
    Ok(Json(count).into_response())
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
    let message = refusal.map(|r| match r {
        InfluenceRefusal::NoConnection => format!("Aucune liaison avec une forteresse des Ténèbres depuis {name}."),
        InfluenceRefusal::MaxReached => {
            format!("Impossible de placer l'influence à {name} : elle dépasserait la valeur de la forteresse des Ténèbres liée.")
        }
    });
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
/// The new shadow pool, as JSON.
///
/// # Errors
/// [`AppError::BadRequest`] for a malformed value; database errors.
pub async fn update_shadow_pool(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<PoolForm>,
) -> Result<Response, AppError> {
    let current: i32 = number(&form.current_val, "current value")?;
    let (_, pool) = change_game(&state, access.board_id, |game, _, _| {
        Ok(game.update_shadow_pool_from_map(current))
    })
    .await?;
    Ok(Json(pool).into_response())
}
