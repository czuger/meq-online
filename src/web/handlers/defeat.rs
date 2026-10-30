//! A defeated hero's choices, and Sauron's choice of its Haven.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::engine::defeat::{
    DefeatChoice, DefeatLoss, DefeatOutcome, closest_havens, defeat_losses,
};
use crate::engine::turn::TurnEnd;
use crate::game_data::plot::StoryType;
use crate::model::hero::Item;
use crate::web::access::{HeroAccess, SauronAccess};
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::hero_info::HeroInfo;
use crate::web::views::layout::Layout;

/// A radio button: `(value, label)`.
type Choice = (&'static str, String);

#[derive(Template)]
#[template(path = "heroes_defeat.html")]
struct DefeatPage {
    layout: Layout,
    hero_id: i64,
    markers: Vec<Choice>,
    losses: Vec<Choice>,
}

#[derive(Template)]
#[template(path = "sauron_haven.html")]
struct HavenPage {
    layout: Layout,
    sauron_id: i64,
    hero_name: String,
    havens: Vec<(String, String)>,
}

/// The defeated hero's choices.
#[derive(Debug, Deserialize)]
pub struct DefeatForm {
    #[serde(default)]
    marker: Option<String>,
    #[serde(default)]
    loss: Option<String>,
}

/// Sauron's choice of Haven.
#[derive(Debug, Deserialize)]
pub struct HavenForm {
    haven: String,
}

fn marker_choice(marker: StoryType) -> Choice {
    match marker {
        StoryType::Ring => ("ring", "Anneau".to_owned()),
        StoryType::Conquest => ("conquest", "Conquête".to_owned()),
        StoryType::Corruption => ("corruption", "Corruption".to_owned()),
    }
}

fn loss_choice(loss: DefeatLoss) -> Choice {
    match loss {
        DefeatLoss::Favor => ("favor", "Une faveur".to_owned()),
        DefeatLoss::Item(Item::Horse) => ("horse", "L'objet Cheval".to_owned()),
        DefeatLoss::Item(Item::Boat) => ("boat", "L'objet Bateau".to_owned()),
    }
}

fn parse_marker(code: &str) -> Result<StoryType, AppError> {
    match code {
        "ring" => Ok(StoryType::Ring),
        "conquest" => Ok(StoryType::Conquest),
        "corruption" => Ok(StoryType::Corruption),
        _ => Err(AppError::BadRequest("unknown story marker".to_owned())),
    }
}

fn parse_loss(code: &str) -> Result<DefeatLoss, AppError> {
    match code {
        "favor" => Ok(DefeatLoss::Favor),
        "horse" => Ok(DefeatLoss::Item(Item::Horse)),
        "boat" => Ok(DefeatLoss::Item(Item::Boat)),
        _ => Err(AppError::BadRequest("unknown loss".to_owned())),
    }
}

/// `GET /heroes/{hero_id}/defeat_screen`.
///
/// # Errors
/// Database and rendering errors.
pub async fn defeat_screen(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let hero = game.hero(access.hero_id)?;
    let markers = game.board.story.lowest_sauron_markers();
    let losses = defeat_losses(hero);
    let page = DefeatPage {
        layout: Layout::hero(&access.session, &game, access.hero_id, &state.data),
        hero_id: access.hero_id,
        markers: if markers.len() > 1 {
            markers.into_iter().map(marker_choice).collect()
        } else {
            vec![]
        },
        losses: if losses.len() > 1 {
            losses.into_iter().map(loss_choice).collect()
        } else {
            vec![]
        },
    };
    render(&page, &page.layout)
}

/// `POST /heroes/{hero_id}/defeat`.
///
/// # Errors
/// [`AppError::BadRequest`] for unknown choices; rule and database errors.
pub async fn defeat(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<DefeatForm>,
) -> Result<Response, AppError> {
    let choice = DefeatChoice {
        marker: form.marker.as_deref().map(parse_marker).transpose()?,
        loss: form.loss.as_deref().map(parse_loss).transpose()?,
    };
    let hero_id = access.hero_id;
    let (_, outcome) = change_game(&state, access.board_id, |game, data, rng| {
        game.resolve_hero_defeat(hero_id, choice, data, rng)
    })
    .await?;
    Ok(match outcome {
        DefeatOutcome::TurnEnded(TurnEnd::HeroDrawCards) => {
            redirect(&format!("/heroes/{hero_id}/draw_cards_screen"), None)
        }
        DefeatOutcome::TurnEnded(TurnEnd::Boards) | DefeatOutcome::ChooseHaven => {
            redirect("/boards", None)
        }
    })
}

/// `GET /sauron/{sauron_id}/haven_screen`.
///
/// # Errors
/// Database and rendering errors.
pub async fn haven_screen(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let hero = game.hero(game.current_hero_id()?)?;
    let havens = closest_havens(&hero.location, &state.data)
        .into_iter()
        .map(|code| (code.to_owned(), state.data.locations.name(code).to_owned()))
        .collect();
    let page = HavenPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        hero_name: HeroInfo::new(hero, &state.data).name,
        havens,
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/haven`.
///
/// # Errors
/// Rule and database errors.
pub async fn haven(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<HavenForm>,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, rng| {
        game.choose_defeat_haven(&form.haven, data, rng)
    })
    .await?;
    Ok(redirect("/boards", None))
}
