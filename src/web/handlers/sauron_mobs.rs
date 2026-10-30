//! Sauron placing, moving and healing mobs.

use askama::Template;
use axum::Form;
use axum::extract::{Path, State};
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::game_data::mob_def::MobKind;
use crate::model::sauron_action::SauronAction;
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::game_layout::sauron_layout;
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::layout::Layout;
use crate::web::views::table_view::MapTarget;

struct MobRow {
    id: i64,
    location: String,
    name: String,
    can_move: bool,
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
    /// Commands left, during a Command action.
    commands_left: Option<i32>,
    can_place: bool,
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

/// The locations as clickable targets of the form `form_id` on the table's map.
fn targets(
    locations: &[LocationChoice],
    form_id: &'static str,
    name: &'static str,
    state: &AppState,
) -> Vec<MapTarget> {
    locations
        .iter()
        .filter_map(|l| MapTarget::new(&state.data, form_id, name, &l.code, l.name.clone()))
        .collect()
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

/// `GET /sauron/{sauron_id}/sauron_mobs` (and `.../command_screen`): minions first,
/// then monsters, with the commands Sauron may give during a Command action.
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
    let command = game
        .board
        .current_action
        .filter(|a| a.action == SauronAction::Command);
    let commanding = command.is_some_and(|c| c.commands_left > 0);
    let near_hero = |location: &str| game.heroes.iter().any(|h| h.location == location);
    let rows = mobs
        .into_iter()
        .map(|m| MobRow {
            id: m.id.unwrap_or_default(),
            location: state.data.locations.name(&m.location).to_owned(),
            name: m.name.clone(),
            can_move: commanding && !m.commanded,
            heal: (commanding
                && !m.commanded
                && m.kind == MobKind::Minion
                && m.life < m.max_life
                && !near_hero(&m.location))
            .then(|| format!("Soigner {} / {}", m.life, m.max_life)),
        })
        .collect();
    let page = IndexPage {
        layout: sauron_layout(&state, &access.session, &game).await?,
        sauron_id: access.sauron_id,
        mobs: rows,
        commands_left: command.map(|c| c.commands_left),
        can_place: commanding && command.is_some_and(|c| !c.monster_placed),
    };
    render(&page, &page.layout)
}

/// `GET /sauron/{sauron_id}/sauron_mobs/new`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
pub async fn new(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let locations = choices(game.monster_placement_locations(&state.data), &state);
    let page = NewPage {
        layout: sauron_layout(&state, &access.session, &game)
            .await?
            .with_targets(targets(&locations, "place-mob-form", "location", &state)),
        sauron_id: access.sauron_id,
        locations,
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/sauron_mobs`: command placing a monster token.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id, form = ?form), err(level = "debug"))]
pub async fn create(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<CreateForm>,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, rng| {
        game.command_place_monster(&form.location, data, rng)
    })
    .await?;
    Ok(redirect(&mobs_url(access.sauron_id), None))
}

/// `GET /sauron/{sauron_id}/sauron_mobs/{mob_id}/edit`.
///
/// # Errors
/// Rule, database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
pub async fn edit(
    State(state): State<AppState>,
    access: SauronAccess,
    Path((_, mob_id)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let locations = choices(game.mob_destinations(mob_id, &state.data)?, &state);
    let page = EditPage {
        layout: sauron_layout(&state, &access.session, &game)
            .await?
            .with_targets(targets(&locations, "move-mob-form", "button", &state)),
        sauron_id: access.sauron_id,
        mob_id,
        locations,
    };
    render(&page, &page.layout)
}

/// `PATCH /sauron/{sauron_id}/sauron_mobs/{mob_id}` (or `POST` from the form): command
/// moving a mob.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id, form = ?form), err(level = "debug"))]
pub async fn update(
    State(state): State<AppState>,
    access: SauronAccess,
    Path((_, mob_id)): Path<(i64, i64)>,
    Form(form): Form<MoveForm>,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, _| {
        game.command_move_mob(mob_id, &form.button, data)
    })
    .await?;
    Ok(redirect(
        &mobs_url(access.sauron_id),
        Some(Flash::notice("Déplacement effectué.")),
    ))
}

/// `POST /sauron/{sauron_id}/sauron_mobs/{mob_id}/heal`: command healing a minion.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
pub async fn heal(
    State(state): State<AppState>,
    access: SauronAccess,
    Path((_, mob_id)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, _, _| {
        game.command_heal_minion(mob_id)
    })
    .await?;
    Ok(redirect(
        &mobs_url(access.sauron_id),
        Some(Flash::notice("Serviteur soigné.")),
    ))
}
