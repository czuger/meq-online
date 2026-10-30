//! A hero exploring its location.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::engine::board_state::BoardState;
use crate::engine::exploration::{ExploreChoice, ExploreNotice};
use crate::engine::tokens::TokenKind;
use crate::engine::turn::TurnEnd;
use crate::game_data::GameData;
use crate::game_data::character::{CharacterAbility, CharacterAttribute};
use crate::model::game::Game;
use crate::web::access::HeroAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::game_layout::hero_layout;
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

/// A character the hero may consult for its favor or its ability.
struct Consult {
    code: String,
    name: String,
    favor: i32,
    power: String,
}

#[derive(Template)]
#[template(path = "explorations_show.html")]
struct ShowPage {
    layout: Layout,
    hero_id: i64,
    choices: Vec<Choice>,
    consults: Vec<Consult>,
    dark_path: bool,
    any_enabled: bool,
}

/// `GET /heroes/{hero_id}/exploration`. An isolated hero's hidden characters are logged.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
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
    let consults: Vec<Consult> = tokens
        .iter()
        .filter(|t| t.kind == TokenKind::Character)
        .filter_map(|t| {
            let def = state.data.character(&t.code).ok()?;
            Some(Consult {
                code: t.code.clone(),
                name: def.name.clone(),
                favor: def.favor,
                power: ability_label(def.ability),
            })
        })
        .collect();
    let dark_path = game.dark_path_allowed(hero_id)?;
    let page = ShowPage {
        layout: hero_layout(&state, &access.session, &game, hero_id).await?,
        hero_id,
        any_enabled: choices.iter().any(|c| c.enabled) || !consults.is_empty() || dark_path,
        choices,
        consults,
        dark_path,
    };
    render(&page, &page.layout)
}

/// French text of a character's ability.
fn ability_label(ability: CharacterAbility) -> String {
    match ability {
        CharacterAbility::ShadowPoolLoss { count } => {
            format!("retirer {count} influences de la réserve des Ténèbres")
        }
        CharacterAbility::Horse => "gagner l'objet Cheval".to_owned(),
        CharacterAbility::Level { attribute } => {
            let name = match attribute {
                CharacterAttribute::Strength => "force",
                CharacterAttribute::Agility => "agilité",
                CharacterAttribute::Wisdom => "sagesse",
            };
            format!("gagner 1 niveau de {name}")
        }
        CharacterAbility::AdvanceHeroes { cost } => {
            format!("payer {cost} faveurs pour avancer le marqueur des héros d'une case")
        }
        CharacterAbility::Training { times } => format!("recevoir {times} entraînement(s)"),
        CharacterAbility::DiscardShadows { count } => {
            format!("Sauron défausse {count} cartes des Ténèbres au hasard")
        }
    }
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
        // Characters are consulted with their own radio buttons.
        TokenKind::Character => None,
        TokenKind::Plot => {
            let hero = game.hero(hero_id).ok()?;
            let (_, card) = game.plot_at(&hero.location, data)?;
            let cost = data.plot(card).ok()?.favor_to_discard;
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
        TokenKind::Event => Some(Choice {
            field: "tokens[event][]",
            value: "event".to_owned(),
            text: format!("Activer la carte événement « {name} »"),
            enabled: true,
        }),
        TokenKind::Hero | TokenKind::Monster | TokenKind::Minion => None,
    }
}

/// `PUT /heroes/{hero_id}/exploration` (or `POST` from the form).
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, fields = ?fields), err(level = "debug"))]
pub async fn update(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(fields): Form<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    let mut choice = ExploreChoice::default();
    for (field, value) in fields {
        tracing::trace!(%field, %value, "exploration form field");
        if let Some(character) = field
            .strip_prefix("consult[")
            .and_then(|f| f.strip_suffix(']'))
        {
            match value.as_str() {
                "favor" => choice.characters.push(character.to_owned()),
                "ability" => choice.character_abilities.push(character.to_owned()),
                _ => {}
            }
            continue;
        }
        match field.as_str() {
            "tokens[favor][]" => choice.favors += 1,
            "tokens[plot][]" => choice.plot = true,
            "tokens[event][]" => choice.event = true,
            "dark_path" => choice.dark_path = true,
            _ => {}
        }
    }
    let hero_id = access.hero_id;
    let (game, notice) = change_game(&state, access.board_id, |game, data, rng| {
        game.explore(hero_id, &choice, data, rng)
    })
    .await?;
    if game.board.state == BoardState::ChoiceScreenBoardChoices {
        return Ok(redirect(
            &format!("/boards/{}/choices/choice_screen", access.board_id),
            None,
        ));
    }
    let text = notice.map(|n| match n {
        ExploreNotice::CharacterEncountered => "Personnage consulté.",
        ExploreNotice::DarkPath => "Vous avez choisi la voie des Ténèbres.",
        ExploreNotice::FavorTaken => "Faveur obtenue.",
        ExploreNotice::FavorLimit => {
            "Vous n'avez pas pu prendre toutes les faveurs à cause d'une carte de corruption."
        }
        ExploreNotice::PlotRemoved => "Intrigue retirée.",
        ExploreNotice::EventActivated => "Carte événement activée.",
    });
    Ok(redirect(
        &format!("/heroes/{hero_id}/exploration"),
        text.map(Flash::notice),
    ))
}

/// `POST /heroes/{hero_id}/exploration/next_movement`.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
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

/// `POST /heroes/{hero_id}/exploration/next_step`: the hero ends its turn.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn next_step(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let hero_id = access.hero_id;
    let (_, end) = change_game(&state, access.board_id, |game, data, rng| {
        game.finish_heroes_turn(hero_id, data, rng)
    })
    .await?;
    Ok(match end {
        TurnEnd::HeroDrawCards => redirect(&format!("/heroes/{hero_id}/draw_cards_screen"), None),
        TurnEnd::Boards => redirect(&format!("/boards/{}/play", access.board_id), None),
    })
}
