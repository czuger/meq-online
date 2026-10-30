//! A hero's turn: sheet, draw, rest, movement.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::engine::movement::{MoveOutcome, path_difficulty};
use crate::engine::rest::{Rest, RestOutcome};
use crate::game_data::GameData;
use crate::game_data::plot::StoryType;
use crate::model::game::Game;
use crate::model::hero::Hero;
use crate::web::access::HeroAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::forms::{card_list, number};
use crate::web::game_layout::hero_layout;
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::hero_info::HeroInfo;
use crate::web::views::layout::Layout;
use crate::web::views::pictures::{CardView, corruption_card, hero_hand, image};
use crate::web::views::table_view::MapTarget;
use crate::web::views::terrain_label::terrain_label;

#[derive(Template)]
#[template(path = "heroes_show.html")]
struct ShowPage {
    layout: Layout,
    info: HeroInfo,
    hand: Vec<CardView>,
    card_class: &'static str,
    sheet_pic: String,
    corruptions: Vec<CardView>,
}

#[derive(Template)]
#[template(path = "heroes_rest.html")]
struct RestPage {
    layout: Layout,
    hero_id: i64,
    can_heal: bool,
    /// Beravor outside a Haven may train instead of healing.
    can_train: bool,
    corruptions: Vec<CardView>,
}

#[derive(Template)]
#[template(path = "heroes_story_marker.html")]
struct StoryMarkerPage {
    layout: Layout,
    hero_id: i64,
    markers: Vec<(&'static str, &'static str)>,
}

struct Destination {
    code: String,
    label: String,
}

#[derive(Template)]
#[template(path = "heroes_movement.html")]
struct MovementPage {
    layout: Layout,
    hero_id: i64,
    hand: Vec<CardView>,
    card_class: &'static str,
    destinations: Vec<Destination>,
    argalad_can_use_power: bool,
}

#[derive(Template)]
#[template(path = "heroes_argalad.html")]
struct ArgaladPage {
    layout: Layout,
    hero_id: i64,
    monsters: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "heroes_draw_cards.html")]
struct DrawCardsPage {
    layout: Layout,
    hero_id: i64,
    left_to_draw: i32,
    drawn: i32,
    fortitude: i32,
    life_pool: usize,
    /// Why nothing more can be drawn, once `left_to_draw` is 0.
    blocked: Option<&'static str>,
}

/// Movement form: the destination is the clicked button.
#[derive(Debug, Deserialize)]
pub struct MoveForm {
    button: String,
    #[serde(default)]
    selected_cards: String,
}

/// Draw form.
#[derive(Debug, Deserialize)]
pub struct DrawForm {
    nb_cards: String,
}

/// Corruption card discarded while resting (empty: none).
#[derive(Debug, Deserialize)]
pub struct RestForm {
    #[serde(default)]
    corruption: Option<String>,
}

/// Chosen story marker.
#[derive(Debug, Deserialize)]
pub struct MarkerForm {
    marker: String,
}

fn hero(game: &Game, hero_id: i64) -> Result<&Hero, AppError> {
    Ok(game.hero(hero_id)?)
}

/// `GET /heroes/{hero_id}`: the hero sheet, when it is the hero's turn.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn show(State(state): State<AppState>, access: HeroAccess) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let hero = hero(&game, access.hero_id)?;
    if !hero.active {
        return Ok(redirect(
            &format!("/boards/{}/inactive_actor", access.board_id),
            None,
        ));
    }
    let page = ShowPage {
        layout: hero_layout(&state, &access.session, &game, access.hero_id)
            .await?
            .without_hand(),
        info: HeroInfo::new(hero, &state.data),
        hand: hero_hand(&state.data, &hero.code, &hero.hand),
        card_class: "small-card",
        sheet_pic: image(&format!("heroes/{}.jpg", hero.code)),
        corruptions: hero
            .corruption_cards
            .iter()
            .map(|c| corruption_card(*c))
            .collect(),
    };
    render(&page, &page.layout)
}

/// `GET /heroes/{hero_id}/rest_screen`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn rest_screen(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let hero = hero(&game, access.hero_id)?;
    let haven = state
        .data
        .locations
        .get(&hero.location)
        .is_ok_and(|l| l.haven);
    let survivalist = game.survivalist_rest(access.hero_id, &state.data)?;
    let page = RestPage {
        layout: hero_layout(&state, &access.session, &game, access.hero_id).await?,
        hero_id: access.hero_id,
        can_heal: haven || survivalist,
        can_train: survivalist,
        corruptions: hero
            .discardable_corruption_cards(&state.data)
            .into_iter()
            .map(corruption_card)
            .collect(),
    };
    render(&page, &page.layout)
}

/// How the hero rests: plain rest, heal, or train (Beravor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestKind {
    Rest,
    Heal,
    Train,
}

#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, ?kind, ?form), err(level = "debug"))]
async fn rest(
    state: AppState,
    access: HeroAccess,
    kind: RestKind,
    form: RestForm,
) -> Result<Response, AppError> {
    let discard = form
        .corruption
        .as_deref()
        .filter(|c| !c.is_empty())
        .map(|c| number(c, "corruption card"))
        .transpose()?;
    let hero_id = access.hero_id;
    let rest = Rest {
        heal: kind == RestKind::Heal,
        train: kind == RestKind::Train,
        discard,
    };
    let (_, outcome) = change_game(&state, access.board_id, |game, data, rng| {
        game.rest_hero(hero_id, rest, data, rng)
    })
    .await?;
    Ok(match outcome {
        RestOutcome::Movement => redirect(&format!("/heroes/{hero_id}/movement_screen"), None),
        RestOutcome::ChooseStoryMarker => redirect(
            &format!("/heroes/{hero_id}/after_rest_advance_story_marker_screen"),
            None,
        ),
        RestOutcome::Choice => redirect(
            &format!("/boards/{}/choices/choice_screen", access.board_id),
            None,
        ),
    })
}

/// `POST /heroes/{hero_id}/rest_rest` with an optional `corruption` field (the card to discard is optional).
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, form = ?form), err(level = "debug"))]
pub async fn rest_rest(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<RestForm>,
) -> Result<Response, AppError> {
    rest(state, access, RestKind::Rest, form).await
}

/// `POST /heroes/{hero_id}/rest_heal` with an optional `corruption` field (the card to discard is optional).
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, form = ?form), err(level = "debug"))]
pub async fn rest_heal(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<RestForm>,
) -> Result<Response, AppError> {
    rest(state, access, RestKind::Heal, form).await
}

/// `POST /heroes/{hero_id}/rest_train` with an optional `corruption` field: Beravor rests outside a Haven and
/// receives training.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, form = ?form), err(level = "debug"))]
pub async fn rest_train(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<RestForm>,
) -> Result<Response, AppError> {
    rest(state, access, RestKind::Train, form).await
}

/// `POST /heroes/{hero_id}/rest_skip`.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn rest_skip(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, _, _| game.skip_rest()).await?;
    Ok(redirect(
        &format!("/heroes/{}/movement_screen", access.hero_id),
        None,
    ))
}

/// `GET /heroes/{hero_id}/after_rest_advance_story_marker_screen`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn after_rest_advance_story_marker_screen(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let story = game.board.story;
    let lowest = story.ring.min(story.conquest).min(story.corruption);
    let markers = [
        ("ring", "Anneau", story.ring),
        ("conquest", "Conquête", story.conquest),
        ("corruption", "Corruption", story.corruption),
    ]
    .into_iter()
    .filter(|(_, _, value)| *value == lowest)
    .map(|(code, label, _)| (code, label))
    .collect();
    let page = StoryMarkerPage {
        layout: hero_layout(&state, &access.session, &game, access.hero_id).await?,
        hero_id: access.hero_id,
        markers,
    };
    render(&page, &page.layout)
}

/// `POST /heroes/{hero_id}/after_rest_advance_story_marker` with `marker=ring`.
///
/// # Errors
/// [`AppError::BadRequest`] for an unknown marker; rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, form = ?form), err(level = "debug"))]
pub async fn after_rest_advance_story_marker(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<MarkerForm>,
) -> Result<Response, AppError> {
    let marker = match form.marker.to_lowercase().as_str() {
        "ring" => StoryType::Ring,
        "conquest" => StoryType::Conquest,
        "corruption" => StoryType::Corruption,
        _ => return Err(AppError::BadRequest("unknown story marker".to_owned())),
    };
    change_game(&state, access.board_id, |game, data, _| {
        game.advance_chosen_story_marker(marker, data)
    })
    .await?;
    Ok(redirect(
        &format!("/heroes/{}/movement_screen", access.hero_id),
        None,
    ))
}

/// `GET /heroes/{hero_id}/movement_screen`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn movement_screen(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let hero = hero(&game, access.hero_id)?;
    let destinations: Vec<Destination> = state
        .data
        .paths
        .destinations(&hero.location)
        .iter()
        .map(|d| Destination {
            code: d.dest.clone(),
            label: format!(
                "{} - {} ({})",
                state.data.locations.name(&d.dest),
                terrain_label(d.path_type),
                path_difficulty(d, hero)
            ),
        })
        .collect();
    let page = MovementPage {
        layout: hero_layout(&state, &access.session, &game, access.hero_id)
            .await?
            .without_hand()
            .with_targets(
                destinations
                    .iter()
                    .filter_map(|d| {
                        MapTarget::new(&state.data, "move-form", "button", &d.code, d.label.clone())
                    })
                    .collect(),
            ),
        hero_id: access.hero_id,
        hand: hero_hand(&state.data, &hero.code, &hero.hand),
        card_class: "small-card selectable-card-selection-multiple",
        destinations,
        argalad_can_use_power: game.argalad_can_use_power(hero, &state.data),
    };
    render(&page, &page.layout)
}

/// `POST /heroes/{hero_id}/move`.
///
/// # Errors
/// [`AppError::BadRequest`] for malformed cards; rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, form = ?form), err(level = "debug"))]
pub async fn move_hero(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<MoveForm>,
) -> Result<Response, AppError> {
    let cards = card_list(&form.selected_cards)?;
    let (hero_id, board_id) = (access.hero_id, access.board_id);
    let (_, outcome) = change_game(&state, board_id, |game, data, rng| {
        game.move_hero(hero_id, &form.button, &cards, data, rng)
    })
    .await?;
    let movement = format!("/heroes/{hero_id}/movement_screen");
    Ok(match outcome {
        MoveOutcome::InvalidCards => redirect(
            &movement,
            Some(Flash::alert(
                "Les cartes choisies ne permettent pas d'emprunter ce chemin.",
            )),
        ),
        MoveOutcome::Hopeless => redirect(
            &movement,
            Some(Flash::alert(
                "La corruption Sans espoir vous interdit plus de 3 déplacements par tour.",
            )),
        ),
        MoveOutcome::SingleMove => redirect(
            &movement,
            Some(Flash::alert(
                "Tempête d'hiver : vous ne pouvez vous déplacer qu'une fois pendant ce tour.",
            )),
        ),
        MoveOutcome::Choice => redirect(&format!("/boards/{board_id}/choices/choice_screen"), None),
        MoveOutcome::Combat => redirect(
            &format!("/boards/{board_id}/combats/combat_setup_screen"),
            None,
        ),
        MoveOutcome::Exploration => redirect(&format!("/heroes/{hero_id}/exploration"), None),
    })
}

/// `POST /heroes/{hero_id}/movement_finished`.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn movement_finished(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, _, _| {
        game.movement_finished()
    })
    .await?;
    Ok(redirect(
        &format!("/heroes/{}/exploration", access.hero_id),
        None,
    ))
}

/// `GET /heroes/{hero_id}/argalad_power_screen`: Argalad sees the surrounding monsters.
///
/// # Errors
/// [`AppError::Engine`] when the hero isn't Argalad; database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn argalad_power_screen(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let hero_id = access.hero_id;
    let (game, ()) = change_game(&state, access.board_id, |game, _, _| {
        game.use_argalad_power(hero_id)
    })
    .await?;
    let hero = hero(&game, hero_id)?;
    let monsters = game
        .surrounding_monsters(hero, &state.data)
        .into_iter()
        .map(|m| {
            (
                state.data.locations.name(&m.location).to_owned(),
                m.name.clone(),
            )
        })
        .collect();
    let page = ArgaladPage {
        layout: hero_layout(&state, &access.session, &game, hero_id).await?,
        hero_id,
        monsters,
    };
    render(&page, &page.layout)
}

/// `GET /heroes/{hero_id}/draw_cards_screen`.
///
/// # Errors
/// Database and rendering errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn draw_cards_screen(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let hero = hero(&game, access.hero_id)?;
    let page = DrawCardsPage {
        layout: hero_layout(&state, &access.session, &game, access.hero_id).await?,
        hero_id: access.hero_id,
        left_to_draw: hero.cards_left_to_draw(&state.data),
        drawn: hero.cards_drawn_this_round,
        fortitude: hero.fortitude,
        life_pool: hero.life_pool.len(),
        blocked: draw_blocked(hero, &state.data),
    };
    render(&page, &page.layout)
}

/// Why the hero can't draw anymore this step, in the order `cards_left_to_draw` caps it.
fn draw_blocked(hero: &Hero, data: &GameData) -> Option<&'static str> {
    if hero.cards_left_to_draw(data) > 0 {
        None
    } else if hero.cards_drawn_this_round >= hero.fortitude {
        Some("Vous avez déjà pioché autant de cartes que votre endurance ce tour-ci.")
    } else if hero.life_pool.is_empty() {
        Some("Votre réserve de vie est vide.")
    } else {
        Some("Votre main est pleine : un héros désemparé ne garde pas plus de 7 cartes.")
    }
}

/// `POST /heroes/{hero_id}/draw_cards`.
///
/// # Errors
/// [`AppError::BadRequest`] for a malformed count; database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id, form = ?form), err(level = "debug"))]
pub async fn draw_cards(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<DrawForm>,
) -> Result<Response, AppError> {
    let count: usize = number(&form.nb_cards, "card count")?;
    let hero_id = access.hero_id;
    change_game(&state, access.board_id, |game, data, _| {
        game.hero_mut(hero_id)?.draw_step_cards(count, data);
        Ok(())
    })
    .await?;
    Ok(redirect(
        &format!("/heroes/{hero_id}/draw_cards_screen"),
        None,
    ))
}

/// `POST /heroes/{hero_id}/draw_cards_finished`.
///
/// # Errors
/// Rule and database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id, hero_id = access.hero_id), err(level = "debug"))]
pub async fn draw_cards_finished(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let hero_id = access.hero_id;
    change_game(&state, access.board_id, |game, data, rng| {
        game.draw_cards_finished(hero_id, data, rng)
    })
    .await?;
    Ok(redirect(&format!("/boards/{}/play", access.board_id), None))
}
