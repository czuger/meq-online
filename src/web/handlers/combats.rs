//! Combat screens of a board.

use askama::Template;
use axum::Form;
use axum::extract::{Path, State};
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::engine::combat::flow::{CardPlayed, CombatEnd, CombatResult, CombatSide};
use crate::engine::route::{RouteActor, route_for};
use crate::engine::turn::TurnEnd;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::model::combat::CombatModifiers;
use crate::model::game::Game;
use crate::model::played_card::PlayedCard;
use crate::web::access::BoardAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::forms::{card_list, number};
use crate::web::game_tx::change_game;
use crate::web::page::render;
use crate::web::views::hero_info::HeroInfo;
use crate::web::views::layout::Layout;
use crate::web::views::pictures::{CardView, hero_hand, image, mob_card};

/// Number of past cards shown for each side.
const HISTORY: usize = 6;

/// Counters of one side of the combat.
struct Fighter {
    name: String,
    used_strength: i32,
    strength: String,
    damages_taken: i32,
    life: i32,
}

#[derive(Template)]
#[template(path = "combats_show.html")]
struct ShowPage {
    layout: Layout,
    board_id: i64,
    hero: Fighter,
    mob: Fighter,
    result: CombatResult,
    hero_can_play: bool,
    sauron_can_play: bool,
    hero_history: Vec<String>,
    mob_history: Vec<String>,
}

#[derive(Template)]
#[template(path = "combats_setup.html")]
struct SetupPage {
    layout: Layout,
    board_id: i64,
    info: HeroInfo,
    hand: Vec<CardView>,
    card_class: &'static str,
    /// Agility the hero may spend in this combat.
    agility: i32,
    /// Shadow card changes to the hero's attributes, explained.
    modifiers: Option<String>,
}

/// French explanation of the changes Shadow cards made to the hero for this combat.
fn modifiers_note(modifiers: CombatModifiers) -> Option<String> {
    if modifiers == CombatModifiers::default() {
        return None;
    }
    let agility = if modifiers.no_agility {
        "agilité réduite à 0".to_owned()
    } else {
        format!("agilité {:+}", modifiers.agility)
    };
    Some(format!(
        "Carte des Ténèbres : {agility}, force {:+} pour ce combat.",
        modifiers.strength
    ))
}

#[derive(Template)]
#[template(path = "combats_play_card.html")]
struct PlayCardPage {
    layout: Layout,
    board_id: i64,
    fighter: Fighter,
    hand: Vec<CardView>,
    card_class: &'static str,
    action: &'static str,
    declare_action: &'static str,
    can_declare: bool,
    monster_pic: Option<String>,
}

#[derive(Template)]
#[template(path = "combats_cards_loss.html")]
struct CardsLossPage {
    layout: Layout,
    board_id: i64,
    damages: i32,
    life: usize,
    hand: Vec<CardView>,
    card_class: &'static str,
}

/// Combat setup form: agility spent to draw cards.
#[derive(Debug, Deserialize)]
pub struct SetupForm {
    draw: String,
}

/// Card choice form.
#[derive(Debug, Deserialize)]
pub struct PlayForm {
    selected_card: String,
}

/// Card loss form.
#[derive(Debug, Deserialize)]
pub struct LossForm {
    #[serde(default)]
    selected_cards: String,
}

fn fighters(game: &Game, data: &GameData) -> Result<(Fighter, Fighter), AppError> {
    let combat = game.combat.as_ref().ok_or(AppError::NotFound)?;
    let hero = game.hero(combat.hero_id)?;
    let mob = game
        .mobs
        .get(game.mob_index(combat.mob_id)?)
        .ok_or(AppError::NotFound)?;
    let hero_life = i32::try_from(hero.life_pool.len() + hero.hand.len()).unwrap_or(i32::MAX);
    Ok((
        Fighter {
            name: HeroInfo::new(hero, data).name,
            used_strength: combat.hero_strength_used,
            strength: combat
                .hero_strength
                .map_or_else(|| "?".to_owned(), |s| s.to_string()),
            damages_taken: hero.damages_taken_this_turn,
            life: hero_life,
        },
        Fighter {
            name: mob.name.clone(),
            used_strength: combat.mob_strength_used,
            strength: mob.strength.to_string(),
            damages_taken: mob.damages_taken_this_turn,
            life: mob.life,
        },
    ))
}

fn history(cards: &[PlayedCard], folder: &str) -> Vec<String> {
    let skip = cards.len().saturating_sub(HISTORY);
    cards
        .iter()
        .skip(skip)
        .filter(|c| c.card.is_some())
        .map(|c| image(&format!("{folder}/{}", c.pic_path)))
        .collect()
}

/// `GET /boards/{board_id}/combats`: the combat status; empty hands exhaust their side.
///
/// # Errors
/// [`AppError::NotFound`] without combat; database errors.
pub async fn show(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let (game, result) = change_game(&state, access.board_id, |game, _, _| {
        game.refresh_combat_result()
    })
    .await?;
    let combat = game.combat.as_ref().ok_or(AppError::NotFound)?;
    let (hero, mob) = fighters(&game, &state.data)?;
    let hero_active = game.hero(combat.hero_id).is_ok_and(|h| h.active);
    let me = access.session.user.id;
    let page = ShowPage {
        layout: Layout::board(&access.session, &game),
        board_id: access.board_id,
        hero,
        mob,
        result,
        hero_can_play: hero_active && !combat.hero_exhausted,
        sauron_can_play: game.sauron.user_id == me && game.sauron.active && !combat.mob_exhausted,
        hero_history: history(&combat.hero_cards, "hero_cards/actions"),
        mob_history: history(&combat.mob_cards, "monsters"),
    };
    render(&page, &page.layout)
}

/// `GET /boards/{board_id}/combats/combat_setup_screen`.
///
/// # Errors
/// [`AppError::NotFound`] without combat; database errors.
pub async fn combat_setup_screen(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let game = crate::web::game_tx::read_game(&state, access.board_id).await?;
    let combat = game.combat.as_ref().ok_or(AppError::NotFound)?;
    let hero = game.hero(combat.hero_id)?;
    let page = SetupPage {
        layout: Layout::hero(&access.session, &game, combat.hero_id, &state.data),
        board_id: access.board_id,
        info: HeroInfo::new(hero, &state.data),
        hand: hero_hand(&state.data, &hero.code, &hero.hand),
        card_class: "small-card",
        agility: combat.modifiers.agility(hero.agility),
        modifiers: modifiers_note(combat.modifiers),
    };
    render(&page, &page.layout)
}

/// `POST /boards/{board_id}/combats/combat_setup`: agility split between cards drawn
/// and strength.
///
/// # Errors
/// Rule and database errors.
pub async fn combat_setup(
    State(state): State<AppState>,
    access: BoardAccess,
    Form(form): Form<SetupForm>,
) -> Result<Response, AppError> {
    let agility_to_draw: i32 = number(&form.draw, "agility")?;
    change_game(&state, access.board_id, |game, data, rng| {
        game.combat_setup(agility_to_draw, data, rng)
    })
    .await?;
    Ok(redirect(
        &format!(
            "/boards/{}/combats/hero/play_combat_card_screen",
            access.board_id
        ),
        None,
    ))
}

/// `GET /boards/{board_id}/combats/{side}/play_combat_card_screen` (`side` is `hero` or `sauron`).
///
/// # Errors
/// [`AppError::NotFound`] for an unknown side or without combat; database errors.
pub async fn play_combat_card_screen(
    State(state): State<AppState>,
    access: BoardAccess,
    Path((_, side)): Path<(i64, String)>,
) -> Result<Response, AppError> {
    let game = crate::web::game_tx::read_game(&state, access.board_id).await?;
    let combat = game.combat.as_ref().ok_or(AppError::NotFound)?;
    let (hero_fighter, mob_fighter) = fighters(&game, &state.data)?;
    let page = match side.as_str() {
        "hero" => {
            let hero = game.hero(combat.hero_id)?;
            PlayCardPage {
                layout: Layout::hero(&access.session, &game, combat.hero_id, &state.data),
                board_id: access.board_id,
                fighter: hero_fighter,
                hand: hero_hand(&state.data, &hero.code, &hero.hand),
                card_class: "small-card selectable-card-selection-unique",
                action: "play_combat_card_hero",
                declare_action: "declare_exhaustion_hero",
                can_declare: hero.active,
                monster_pic: None,
            }
        }
        "sauron" => {
            let mob = game
                .mobs
                .get(game.mob_index(combat.mob_id)?)
                .ok_or(AppError::NotFound)?;
            PlayCardPage {
                layout: Layout::sauron(&access.session, &game),
                board_id: access.board_id,
                fighter: mob_fighter,
                hand: mob
                    .hand
                    .iter()
                    .map(|c| mob_card(&state.data, &mob.attack_deck, *c))
                    .collect(),
                card_class: "small-card selectable-card-selection-unique",
                action: "play_combat_card_mob",
                declare_action: "declare_exhaustion_mob",
                can_declare: game.sauron.active
                    && !game.hero(combat.hero_id).is_ok_and(|h| h.active),
                monster_pic: (mob.kind == MobKind::Monster)
                    .then(|| image(&format!("monsters/monsters/{}.jpg", mob.code))),
            }
        }
        _ => return Err(AppError::NotFound),
    };
    render(&page, &page.layout)
}

fn after_card(board_id: i64, played: CardPlayed) -> Response {
    match played {
        CardPlayed::Waiting => redirect("/boards", None),
        CardPlayed::Resolved => redirect(&format!("/boards/{board_id}/combats"), None),
        CardPlayed::CardsLoss => redirect(
            &format!("/boards/{board_id}/combats/cards_loss_screen"),
            None,
        ),
    }
}

async fn declare(
    state: AppState,
    access: BoardAccess,
    side: CombatSide,
) -> Result<Response, AppError> {
    let board_id = access.board_id;
    let (_, played) = change_game(&state, board_id, |game, data, _| {
        game.declare_exhaustion(side, data)
    })
    .await?;
    Ok(after_card(board_id, played))
}

/// `POST /boards/{board_id}/combats/declare_exhaustion_hero`.
///
/// # Errors
/// Rule and database errors.
pub async fn declare_exhaustion_hero(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    declare(state, access, CombatSide::Hero).await
}

/// `POST /boards/{board_id}/combats/declare_exhaustion_mob`.
///
/// # Errors
/// Rule and database errors.
pub async fn declare_exhaustion_mob(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    declare(state, access, CombatSide::Mob).await
}

async fn play_card(
    state: AppState,
    access: BoardAccess,
    side: CombatSide,
    form: PlayForm,
) -> Result<Response, AppError> {
    let card: u32 = number(&form.selected_card, "card")?;
    let board_id = access.board_id;
    let (_, played) = change_game(&state, board_id, |game, data, _| {
        game.play_combat_card(side, card, data)
    })
    .await?;
    Ok(after_card(board_id, played))
}

/// `POST /boards/{board_id}/combats/play_combat_card_hero`.
///
/// # Errors
/// Rule and database errors.
pub async fn play_combat_card_hero(
    State(state): State<AppState>,
    access: BoardAccess,
    Form(form): Form<PlayForm>,
) -> Result<Response, AppError> {
    play_card(state, access, CombatSide::Hero, form).await
}

/// `POST /boards/{board_id}/combats/play_combat_card_mob`.
///
/// # Errors
/// Rule and database errors.
pub async fn play_combat_card_mob(
    State(state): State<AppState>,
    access: BoardAccess,
    Form(form): Form<PlayForm>,
) -> Result<Response, AppError> {
    play_card(state, access, CombatSide::Mob, form).await
}

/// `GET /boards/{board_id}/combats/cards_loss_screen`.
///
/// # Errors
/// [`AppError::NotFound`] without combat; database errors.
pub async fn cards_loss_screen(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let game = crate::web::game_tx::read_game(&state, access.board_id).await?;
    let combat = game.combat.as_ref().ok_or(AppError::NotFound)?;
    let hero = game.hero(combat.hero_id)?;
    let page = CardsLossPage {
        layout: Layout::hero(&access.session, &game, combat.hero_id, &state.data),
        board_id: access.board_id,
        damages: hero.temporary_damages,
        life: hero.life_pool.len(),
        hand: hero_hand(&state.data, &hero.code, &hero.hand),
        card_class: "small-card selectable-card-selection-multiple",
    };
    render(&page, &page.layout)
}

/// `POST /boards/{board_id}/combats/cards_loss`.
///
/// # Errors
/// Rule and database errors.
pub async fn cards_loss(
    State(state): State<AppState>,
    access: BoardAccess,
    Form(form): Form<LossForm>,
) -> Result<Response, AppError> {
    let cards = card_list(&form.selected_cards)?;
    let board_id = access.board_id;
    change_game(&state, board_id, |game, _, _| {
        game.combat_cards_loss(&cards)
    })
    .await?;
    Ok(redirect(&format!("/boards/{board_id}/combats"), None))
}

/// `GET /boards/{board_id}/combats/terminate`.
///
/// # Errors
/// Rule and database errors.
pub async fn terminate(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let (game, (end, hero_id)) = change_game(&state, access.board_id, |game, data, rng| {
        let hero_id = game.combat.as_ref().map(|c| c.hero_id).unwrap_or_default();
        Ok((game.terminate_combat(data, rng)?, hero_id))
    })
    .await?;
    Ok(match end {
        CombatEnd::Movement => redirect(&format!("/heroes/{hero_id}/movement_screen"), None),
        CombatEnd::Rest => redirect(
            &route_for(game.board.state, access.board_id, RouteActor::Hero(hero_id)),
            None,
        ),
        CombatEnd::Exploration => redirect(&format!("/heroes/{hero_id}/exploration"), None),
        CombatEnd::Choice => redirect(
            &format!("/boards/{}/choices/choice_screen", access.board_id),
            None,
        ),
        CombatEnd::Defeat => redirect(&format!("/heroes/{hero_id}/defeat_screen"), None),
        CombatEnd::ChooseHaven => redirect("/boards", None),
        CombatEnd::TurnEnded(TurnEnd::HeroDrawCards) => {
            redirect(&format!("/heroes/{hero_id}/draw_cards_screen"), None)
        }
        CombatEnd::TurnEnded(TurnEnd::Boards) => redirect("/boards", None),
    })
}
