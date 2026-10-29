//! A hero exploring its location.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::engine::exploration::{ExploreChoice, ExploreNotice};
use crate::engine::tokens::TokenKind;
use crate::engine::turn::TurnEnd;
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::web::access::HeroAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::game_tx::change_game;
use crate::web::page::render;
use crate::web::views::layout::Layout;

/// A checkbox of the exploration form.
struct Choice {
    field: &'static str,
    value: String,
    text: String,
    enabled: bool,
}

#[derive(Template)]
#[template(path = "explorations_show.html")]
struct ShowPage {
    layout: Layout,
    hero_id: i64,
    choices: Vec<Choice>,
    any_enabled: bool,
}

/// `GET /heroes/{hero_id}/exploration`. An isolated hero's hidden characters are logged.
///
/// # Errors
/// Rule and database errors.
pub async fn show(State(state): State<AppState>, access: HeroAccess) -> Result<Response, AppError> {
    let hero_id = access.hero_id;
    let (game, tokens) = change_game(&state, access.board_id, |game, data, _| {
        game.exploration_tokens(hero_id, data)
    })
    .await?;
    let choices: Vec<Choice> = tokens
        .iter()
        .filter_map(|t| choice(t.kind, &t.code, &t.name, &game, hero_id, &state.data))
        .collect();
    let page = ShowPage {
        layout: Layout::hero(&access.session, &game, hero_id, &state.data),
        hero_id,
        any_enabled: choices.iter().any(|c| c.enabled),
        choices,
    };
    render(&page, &page.layout)
}

fn choice(
    kind: TokenKind,
    code: &str,
    name: &str,
    game: &Game,
    hero_id: i64,
    data: &GameData,
) -> Option<Choice> {
    match kind {
        TokenKind::Favor => Some(Choice {
            field: "tokens[favor][]",
            value: code.to_owned(),
            text: "Gagner une faveur".to_owned(),
            enabled: true,
        }),
        TokenKind::Character => Some(Choice {
            field: "tokens[character][]",
            value: code.to_owned(),
            text: format!("Rencontrer {name} et gagner 2 faveurs"),
            enabled: true,
        }),
        TokenKind::Plot => {
            let hero = game.hero(hero_id).ok()?;
            let cost = game
                .board
                .plots
                .values()
                .filter_map(|card| data.plot(*card).ok())
                .find(|p| p.affect.as_deref() == Some(hero.location.as_str()))?
                .favor_to_discard;
            let enabled = cost <= hero.favor;
            let text = if enabled {
                "Dépenser des faveurs pour retirer l'intrigue locale"
            } else {
                "Vous n'avez pas assez de faveurs pour retirer l'intrigue locale"
            };
            Some(Choice {
                field: "tokens[plot][]",
                value: "plot".to_owned(),
                text: text.to_owned(),
                enabled,
            })
        }
        TokenKind::Hero | TokenKind::Monster | TokenKind::Minion => None,
    }
}

/// `PUT /heroes/{hero_id}/exploration` (or `POST` from the form).
///
/// # Errors
/// Rule and database errors.
pub async fn update(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(fields): Form<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    let mut choice = ExploreChoice::default();
    for (field, value) in fields {
        match field.as_str() {
            "tokens[character][]" => choice.characters.push(value),
            "tokens[favor][]" => choice.favors += 1,
            "tokens[plot][]" => choice.plot = true,
            _ => {}
        }
    }
    let hero_id = access.hero_id;
    let (_, notice) = change_game(&state, access.board_id, |game, data, _| {
        game.explore(hero_id, &choice, data)
    })
    .await?;
    let text = notice.map(|n| match n {
        ExploreNotice::CharacterEncountered => "Personnage rencontré.",
        ExploreNotice::FavorTaken => "Faveur obtenue.",
        ExploreNotice::Dispairing => {
            "Vous n'avez pas pu prendre toutes les faveurs à cause de la corruption Désespéré."
        }
        ExploreNotice::PlotRemoved => "Intrigue retirée.",
    });
    Ok(redirect(
        &format!("/heroes/{hero_id}/exploration"),
        text.map(Flash::notice),
    ))
}

/// `GET /heroes/{hero_id}/exploration/next_movement`.
///
/// # Errors
/// Rule and database errors.
pub async fn next_movement(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, _, _| {
        game.exploration_next_movement()
    })
    .await?;
    Ok(redirect(
        &format!("/heroes/{}/movement_screen", access.hero_id),
        None,
    ))
}

/// `GET /heroes/{hero_id}/exploration/next_step`: the hero ends its turn.
///
/// # Errors
/// Rule and database errors.
pub async fn next_step(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let hero_id = access.hero_id;
    let (_, end) = change_game(&state, access.board_id, |game, data, _| {
        game.finish_heroes_turn(hero_id, data)
    })
    .await?;
    Ok(match end {
        TurnEnd::HeroDrawCards => redirect(&format!("/heroes/{hero_id}/draw_cards_screen"), None),
        TurnEnd::Boards => redirect("/boards", None),
    })
}
