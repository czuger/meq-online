//! Sauron placing characters by hand.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::model::actor_ref::ActorRef;
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::layout::Layout;

struct LocationOption {
    code: String,
    name: String,
    selected: bool,
}

struct CharacterRow {
    code: String,
    name: String,
    options: Vec<LocationOption>,
}

#[derive(Template)]
#[template(path = "characters_edit.html")]
struct EditPage {
    layout: Layout,
    sauron_id: i64,
    characters: Vec<CharacterRow>,
}

/// `GET /characters/{sauron_id}/edit`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id), err(level = "debug"))]
pub async fn edit(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let locations = state.data.locations.sorted_by_name();
    let characters = state
        .data
        .characters
        .iter()
        .map(|(code, character)| {
            let current = game.board.characters.get(code);
            CharacterRow {
                code: code.clone(),
                name: character.name.clone(),
                options: locations
                    .iter()
                    .map(|(loc, l)| LocationOption {
                        code: (*loc).clone(),
                        name: l.name.clone(),
                        selected: current == Some(*loc),
                    })
                    .collect(),
            }
        })
        .collect();
    let page = EditPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        characters,
    };
    render(&page, &page.layout)
}

/// `PATCH /characters/{sauron_id}` (or `POST` from the form): places or removes characters.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, sauron_id = access.sauron_id, fields = ?fields), err(level = "debug"))]
pub async fn update(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(fields): Form<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    let wanted: Vec<(String, String)> = fields
        .into_iter()
        .filter_map(|(field, location)| {
            let code = field.strip_prefix("characters[")?.strip_suffix(']')?;
            Some((code.to_owned(), location))
        })
        .collect();
    change_game(&state, access.board_id, |game, data, _| {
        for (code, location) in &wanted {
            tracing::trace!(%code, %location, "character placement");
            if location.is_empty() {
                game.remove_character(ActorRef::Sauron, code, data)?;
            } else {
                game.place_character(ActorRef::Sauron, code, location, data)?;
            }
        }
        Ok(())
    })
    .await?;
    Ok(redirect(
        &format!("/characters/{}/edit", access.sauron_id),
        None,
    ))
}
