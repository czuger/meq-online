//! Sauron placing, moving and healing mobs.

use askama::Template;
use axum::Form;
use axum::extract::{Path, State};
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::game_data::mob_def::MobKind;
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::layout::Layout;

struct MobRow {
    id: i64,
    location: String,
    name: String,
    heal: Option<String>,
}

struct LocationChoice {
    code: String,
    name: String,
}

#[derive(Template)]
#[template(path = "sauron_mobs_index.html")]
struct IndexPage {
    layout: Layout,
    sauron_id: i64,
    mobs: Vec<MobRow>,
}

#[derive(Template)]
#[template(path = "sauron_mobs_new.html")]
struct NewPage {
    layout: Layout,
    sauron_id: i64,
    locations: Vec<LocationChoice>,
}

#[derive(Template)]
#[template(path = "sauron_mobs_edit.html")]
struct EditPage {
    layout: Layout,
    sauron_id: i64,
    mob_id: i64,
    locations: Vec<LocationChoice>,
}

/// Location chosen for a new monster.
#[derive(Debug, Deserialize)]
pub struct CreateForm {
    location: String,
}

/// Destination of a mob: the clicked button.
#[derive(Debug, Deserialize)]
pub struct MoveForm {
    button: String,
}

fn mobs_url(sauron_id: i64) -> String {
    format!("/sauron/{sauron_id}/sauron_mobs")
}

fn choices(codes: Vec<String>, state: &AppState) -> Vec<LocationChoice> {
    codes
        .into_iter()
        .map(|code| LocationChoice {
            name: state.data.locations.name(&code).to_owned(),
            code,
        })
        .collect()
}

/// `GET /sauron/{sauron_id}/sauron_mobs`: minions first, then monsters.
///
/// # Errors
/// Database and rendering errors.
pub async fn index(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let mut mobs: Vec<_> = game.mobs.iter().collect();
    mobs.sort_by(|a, b| {
        (a.kind != MobKind::Minion, &a.code).cmp(&(b.kind != MobKind::Minion, &b.code))
    });
    let rows = mobs
        .into_iter()
        .map(|m| MobRow {
            id: m.id.unwrap_or_default(),
            location: state.data.locations.name(&m.location).to_owned(),
            name: m.name.clone(),
            heal: (m.kind == MobKind::Minion && m.life < m.max_life)
                .then(|| format!("Soigner {} / {}", m.life, m.max_life)),
        })
        .collect();
    let page = IndexPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        mobs: rows,
    };
    render(&page, &page.layout)
}

/// `GET /sauron/{sauron_id}/sauron_mobs/new`.
///
/// # Errors
/// Database and rendering errors.
pub async fn new(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let locations = choices(game.monster_placement_locations(&state.data), &state);
    let page = NewPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        locations,
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/sauron_mobs`: draws a monster onto a location.
///
/// # Errors
/// Rule and database errors.
pub async fn create(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<CreateForm>,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, _| {
        game.place_new_monster(&form.location, data)
    })
    .await?;
    Ok(redirect(&mobs_url(access.sauron_id), None))
}

/// `GET /sauron/{sauron_id}/sauron_mobs/{mob_id}/edit`.
///
/// # Errors
/// Rule, database and rendering errors.
pub async fn edit(
    State(state): State<AppState>,
    access: SauronAccess,
    Path((_, mob_id)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let locations = choices(game.mob_destinations(mob_id, &state.data)?, &state);
    let page = EditPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        mob_id,
        locations,
    };
    render(&page, &page.layout)
}

/// `PATCH /sauron/{sauron_id}/sauron_mobs/{mob_id}` (or `POST` from the form).
///
/// # Errors
/// Rule and database errors.
pub async fn update(
    State(state): State<AppState>,
    access: SauronAccess,
    Path((_, mob_id)): Path<(i64, i64)>,
    Form(form): Form<MoveForm>,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, _| {
        game.move_mob(mob_id, &form.button, data)
    })
    .await?;
    Ok(redirect(
        &mobs_url(access.sauron_id),
        Some(Flash::notice("Monstre déplacé.")),
    ))
}

/// `GET /sauron/{sauron_id}/sauron_mobs/{mob_id}/heal`.
///
/// # Errors
/// Rule and database errors.
pub async fn heal(
    State(state): State<AppState>,
    access: SauronAccess,
    Path((_, mob_id)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, _, _| {
        game.heal_minion(mob_id)
    })
    .await?;
    Ok(redirect(
        &mobs_url(access.sauron_id),
        Some(Flash::notice("Serviteur soigné.")),
    ))
}
