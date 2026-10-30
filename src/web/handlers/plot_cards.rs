//! Sauron's plot cards: drawing, keeping, playing and discarding.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::engine::board_state::BoardState;
use crate::engine::deck::SauronDeck;
use crate::engine::plots::PlotPlayed;
use crate::engine::route::{RouteActor, route_for};
use crate::web::access::SauronAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::forms::{card_list, number};
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::layout::Layout;
use crate::web::views::pictures::{CardView, plot_card};

#[derive(Template)]
#[template(path = "plot_cards_keep.html")]
struct KeepPage {
    layout: Layout,
    sauron_id: i64,
    cards: Vec<CardView>,
    to_keep: usize,
}

#[derive(Template)]
#[template(path = "plot_cards_play.html")]
struct PlayPage {
    layout: Layout,
    sauron_id: i64,
    playable: Vec<CardView>,
    unplayable: Vec<CardView>,
    free_slots: Vec<u8>,
}

#[derive(Template)]
#[template(path = "plot_cards_gollum.html")]
struct GollumPage {
    layout: Layout,
    sauron_id: i64,
    cards: Vec<CardView>,
}

#[derive(Template)]
#[template(path = "plot_cards_discard.html")]
struct DiscardPage {
    layout: Layout,
    sauron_id: i64,
    /// Plots in play; the card id is the slot.
    slots: Vec<CardView>,
}

/// Keep form.
#[derive(Debug, Deserialize)]
pub struct KeepForm {
    #[serde(default)]
    selected_cards: String,
}

/// Play form.
#[derive(Debug, Deserialize)]
pub struct PlayForm {
    selected_card: String,
    card_slot: String,
}

/// Form with one selected card (or slot).
#[derive(Debug, Deserialize)]
pub struct SelectedForm {
    selected_card: String,
}

fn actions_url(sauron_id: i64) -> String {
    format!("/sauron/{sauron_id}/sauron_actions/edit")
}

/// `GET /sauron/{sauron_id}/plot_cards/keep_screen`.
///
/// # Errors
/// Database and rendering errors.
pub async fn keep_screen(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = KeepPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        cards: game
            .sauron
            .drawn_plot
            .iter()
            .map(|c| plot_card(*c))
            .collect(),
        to_keep: game.cards_to_keep(SauronDeck::Plot, &state.data),
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/plot_cards/keep`, then back to the current step; during a
/// Draw Shadow and Plot Cards action, keeping the plot card ends the action.
///
/// # Errors
/// Rule and database errors.
pub async fn keep(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<KeepForm>,
) -> Result<Response, AppError> {
    let cards = card_list(&form.selected_cards)?;
    let (game, ()) = change_game(&state, access.board_id, |game, data, _| {
        game.keep_sauron_cards(SauronDeck::Plot, &cards, data)?;
        if game.board.state == BoardState::KeepScreenSauronPlotCards {
            game.finish_sauron_action()?;
        }
        Ok(())
    })
    .await?;
    let to = route_for(
        game.board.state,
        access.board_id,
        RouteActor::Sauron(access.sauron_id),
    );
    let text = if cards.len() == 1 {
        "Une carte intrigue a été gardée.".to_owned()
    } else {
        format!("{} cartes intrigue ont été gardées.", cards.len())
    };
    Ok(redirect(&to, Some(Flash::notice(text))))
}

/// `GET /sauron/{sauron_id}/plot_cards/play_screen`.
///
/// # Errors
/// Database and rendering errors.
pub async fn play_screen(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let (mut playable, mut unplayable) = (vec![], vec![]);
    for card in &game.sauron.plot_hand {
        if game.plot_playable(*card, &state.data)? {
            playable.push(plot_card(*card));
        } else {
            unplayable.push(plot_card(*card));
        }
    }
    let page = PlayPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        playable,
        unplayable,
        free_slots: game.free_plot_slots(),
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/plot_cards/play`.
///
/// # Errors
/// [`AppError::BadRequest`] for malformed fields; rule and database errors.
pub async fn play(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<PlayForm>,
) -> Result<Response, AppError> {
    let card: u32 = number(&form.selected_card, "card")?;
    let slot: u8 = number(&form.card_slot, "slot")?;
    let (_, played) = change_game(&state, access.board_id, |game, data, rng| {
        game.play_plot(card, slot, data, rng)
    })
    .await?;
    Ok(match played {
        PlotPlayed::Finished => redirect(&actions_url(access.sauron_id), None),
        PlotPlayed::LookForGollum => redirect(
            &format!(
                "/sauron/{}/plot_cards/look_for_gollum_cards",
                access.sauron_id
            ),
            None,
        ),
        PlotPlayed::ChooseLocation => redirect(
            &format!("/boards/{}/choices/choice_screen", access.board_id),
            None,
        ),
    })
}

/// `GET /sauron/{sauron_id}/plot_cards/play_finished`.
///
/// # Errors
/// Rule and database errors.
pub async fn play_finished(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    change_game(&state, access.board_id, |game, data, rng| {
        game.plot_play_finished(data, rng)
    })
    .await?;
    Ok(redirect(&actions_url(access.sauron_id), None))
}

/// `GET /sauron/{sauron_id}/plot_cards/look_for_gollum_cards`.
///
/// # Errors
/// Database and rendering errors.
pub async fn look_for_gollum_cards(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let page = GollumPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        cards: game
            .gollum_cards_available()
            .into_iter()
            .map(plot_card)
            .collect(),
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/plot_cards/get_gollum_card`.
///
/// # Errors
/// Rule and database errors.
pub async fn get_gollum_card(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<SelectedForm>,
) -> Result<Response, AppError> {
    let card: u32 = number(&form.selected_card, "card")?;
    change_game(&state, access.board_id, |game, data, rng| {
        game.take_gollum_card(card, data, rng)
    })
    .await?;
    Ok(redirect(&actions_url(access.sauron_id), None))
}

/// `GET /sauron/{sauron_id}/plot_cards/discard_screen`.
///
/// # Errors
/// Database and rendering errors.
pub async fn discard_screen(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let slots = game
        .board
        .plots
        .iter()
        .map(|(slot, card)| CardView {
            id: u32::from(*slot),
            pic: plot_card(*card).pic,
        })
        .collect();
    let page = DiscardPage {
        layout: Layout::sauron(&access.session, &game),
        sauron_id: access.sauron_id,
        slots,
    };
    render(&page, &page.layout)
}

/// `POST /sauron/{sauron_id}/plot_cards/discard`: `selected_card` is the slot; this
/// ends the Plot step.
///
/// # Errors
/// [`AppError::BadRequest`] for a malformed slot; rule and database errors.
pub async fn discard(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<SelectedForm>,
) -> Result<Response, AppError> {
    let slot: u8 = number(&form.selected_card, "slot")?;
    change_game(&state, access.board_id, |game, data, rng| {
        game.plot_step_discard(slot, data, rng)
    })
    .await?;
    Ok(redirect(&actions_url(access.sauron_id), None))
}
