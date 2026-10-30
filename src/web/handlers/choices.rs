//! The choice screen: a player answers the decision a card asks for.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::db::{DbError, user_repo};
use crate::engine::EngineError;
use crate::engine::events::event_step::event_pic_path;
use crate::engine::mobs::RINGWRAITHS;
use crate::engine::route::{RouteActor, route_for};
use crate::game_data::GameData;
use crate::game_data::shadow::ShadowTiming;
use crate::model::board::EventRef;
use crate::model::choice::{ChoiceOption, ChoiceSource, Chooser, PendingChoice};
use crate::model::game::Game;
use crate::web::access::BoardAccess;
use crate::web::app_error::AppError;
use crate::web::flash::redirect;
use crate::web::game_tx::{change_game, read_game};
use crate::web::page::render;
use crate::web::views::choice_label::{option_label, target_label, target_picture};
use crate::web::views::layout::Layout;
use crate::web::views::pictures::{image, plot_card, shadow_card};

/// Separates the option code from the target code in a `targets` field.
const TARGET_SEPARATOR: char = '|';

/// A target to pick, as shown.
struct TargetView {
    /// Form value, `<option>|<target>`.
    value: String,
    label: String,
    picture: Option<String>,
}

/// An option, as shown.
struct OptionView {
    code: String,
    label: String,
    targets: Vec<TargetView>,
    /// What to pick, e.g. "Choisissez de 1 à 2 cibles.", empty without targets.
    hint: String,
}

#[derive(Template)]
#[template(path = "choices_choice_screen.html")]
struct ChoicePage {
    layout: Layout,
    board_id: i64,
    chooser: String,
    card_pic: Option<String>,
    /// What the decision is about, when no card picture tells it.
    context: Option<String>,
    options: Vec<OptionView>,
    can_answer: bool,
    /// Card only the chooser may look at.
    seen_pic: Option<String>,
}

/// Heroes the user plays: their own and the computer seats.
async fn played_heroes(state: &AppState, game: &Game, user_id: i64) -> Result<Vec<i64>, AppError> {
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let mut played = Vec::new();
    for hero in &game.heroes {
        let computer = hero.user_id != user_id
            && user_repo::find(&mut conn, hero.user_id)
                .await?
                .is_some_and(|u| u.ai_index.is_some());
        if let (true, Some(id)) = (hero.user_id == user_id || computer, hero.id) {
            played.push(id);
        }
    }
    Ok(played)
}

/// Whether the user may answer: Sauron's user, or a player of the hero (or of any
/// hero when the heroes choose as a side).
fn may_answer(choice: &PendingChoice, game: &Game, user_id: i64, played: &[i64]) -> bool {
    match choice.chooser {
        Chooser::Sauron => game.sauron.user_id == user_id,
        Chooser::Hero(hero_id) => played.contains(&hero_id),
        Chooser::Heroes => !played.is_empty(),
    }
}

fn chooser_name(chooser: Chooser, game: &Game, data: &GameData) -> String {
    match chooser {
        Chooser::Sauron => "Sauron".to_owned(),
        Chooser::Hero(hero_id) => game
            .hero(hero_id)
            .ok()
            .and_then(|h| data.heroes.get(&h.code))
            .map_or_else(|| "Un héros".to_owned(), |d| d.name.clone()),
        Chooser::Heroes => "Les héros".to_owned(),
    }
}

fn layout(access: &BoardAccess, chooser: Chooser, game: &Game, data: &GameData) -> Layout {
    match chooser {
        Chooser::Sauron => Layout::sauron(&access.session, game),
        Chooser::Hero(hero_id) => Layout::hero(&access.session, game, hero_id, data),
        Chooser::Heroes => Layout::board(&access.session, game),
    }
}

/// Picture of the event, Shadow or Plot card the choice belongs to; other choices
/// show their cards as targets or explain themselves with [`choice_context`].
fn card_picture(source: &ChoiceSource) -> Option<String> {
    match *source {
        ChoiceSource::Event { stage, card, .. }
        | ChoiceSource::ExploreEvent { stage, card, .. } => Some(image(&format!(
            "events/{}",
            event_pic_path(EventRef { stage, card })
        ))),
        ChoiceSource::Shadow { card, .. } => Some(shadow_card(card).pic),
        ChoiceSource::PlotLocation { card, .. } => Some(plot_card(card).pic),
        _ => None,
    }
}

/// Card a Shadow card let Sauron look at ("Saruman's Lies"), shown to him only.
fn seen_picture(source: &ChoiceSource) -> Option<String> {
    let ChoiceSource::Shadow { picked, .. } = source else {
        return None;
    };
    let card = picked.iter().find_map(|t| t.strip_prefix("plot_card:"))?;
    Some(plot_card(card.parse().ok()?).pic)
}

/// French name of the moment a Shadow card may be played.
fn timing_name(timing: ShadowTiming) -> &'static str {
    match timing {
        ShadowTiming::ActionStep => "pendant l'étape d'action",
        ShadowTiming::HeroTurnStart => "au début du tour du héros",
        ShadowTiming::CombatStart => "au début du combat",
        ShadowTiming::EnteredLocation => "après l'entrée du héros dans un lieu",
        ShadowTiming::HeroDefeated => "après la défaite du héros",
    }
}

/// French text explaining a Shadow card decision.
fn shadow_context(source: &ChoiceSource, game: &Game, data: &GameData) -> Option<String> {
    let ChoiceSource::Shadow { card, heroes, .. } = source else {
        return None;
    };
    let shadow = data.shadow(*card).ok()?;
    let mut text = format!("{} : {}", shadow.name_fr, shadow.text_fr);
    if let [first, second] = heroes.as_slice() {
        let name = |id: &i64| chooser_name(Chooser::Hero(*id), game, data);
        text.push_str(&format!(
            " Les cartes cochées vont à {}, les autres à {}.",
            name(first),
            name(second)
        ));
    }
    Some(text)
}

/// French text explaining a choice without card picture.
fn choice_context(source: &ChoiceSource, game: &Game, data: &GameData) -> Option<String> {
    let text = match source {
        ChoiceSource::Event { .. }
        | ChoiceSource::ExploreEvent { .. }
        | ChoiceSource::Training { .. } => return None,
        ChoiceSource::Shadow { .. } => return shadow_context(source, game, data),
        ChoiceSource::ShadowWindow { timing, .. } => format!(
            "Sauron peut jouer une carte des Ténèbres {} (une seule par tour de joueur), ou passer.",
            timing_name(*timing)
        ),
        ChoiceSource::PlotLocation { .. } => {
            "Sauron choisit le lieu affecté par l'intrigue qu'il joue.".to_owned()
        }
        ChoiceSource::Threat { ambush: true, .. } => {
            "Embuscade : Sauron choisit le monstre ou le serviteur qui attaque le héros.".to_owned()
        }
        ChoiceSource::Threat { .. } => "Le héros entre dans un lieu menacé : Sauron choisit \
            le monstre ou le serviteur qui l'attaque, ou un péril si le lieu est périlleux."
            .to_owned(),
        ChoiceSource::HavenCombat { .. } => {
            "Un monstre ou un serviteur se trouve dans ce refuge : aucun combat n'a lieu sans \
             l'accord du héros."
                .to_owned()
        }
        ChoiceSource::PerilDraw { .. } => "Sauron a pioché des cartes Péril et choisit celle \
            que le héros doit résoudre, parmi celles qui affectent son lieu."
            .to_owned(),
        ChoiceSource::Peril { card, .. } => {
            let peril = data.peril(*card).ok()?;
            format!("{} : {}", peril.name_fr, peril.text_fr)
        }
        ChoiceSource::Damage { amount, .. } => format!(
            "Le héros subit {amount} dégâts : choisissez les cartes de la main à perdre, \
             le reste est pris sur la réserve de vie."
        ),
        ChoiceSource::Champion => champion_context(game)?,
        ChoiceSource::Wretched { .. } => "Carte de corruption Misérable : le tour du héros \
            se termine. Sauron peut l'envoyer au refuge le plus proche."
            .to_owned(),
    };
    Some(text)
}

/// French text of the champion choice, with the Ringwraiths the champion will fight.
fn champion_context(game: &Game) -> Option<String> {
    let ringwraiths = game.mobs.iter().find(|m| m.code == RINGWRAITHS)?;
    Some(format!(
        "Finale : chaque héros a mélangé ses cartes et pioché une nouvelle main. Les héros \
         choisissent leur champion, qui affronte les Nazgûl ({} points de vie, \
         endurance {}). S'il les vainc, les héros gagnent ; sinon, Sauron gagne.",
        ringwraiths.life, ringwraiths.fortitude
    ))
}

fn option_view(option: &ChoiceOption, game: &Game, data: &GameData, sauron: bool) -> OptionView {
    let targets = option
        .targets
        .iter()
        .map(|code| TargetView {
            value: format!("{}{TARGET_SEPARATOR}{code}", option.code),
            label: target_label(code, game, data, sauron),
            picture: target_picture(code, game, data),
        })
        .collect();
    let hint = match (option.min, option.max) {
        (_, 0) => String::new(),
        (min, max) if min == max => format!("Choisissez {max} cible(s)."),
        (min, max) => format!("Choisissez de {min} à {max} cible(s)."),
    };
    OptionView {
        code: option.code.clone(),
        label: option_label(&option.code).to_owned(),
        targets,
        hint,
    }
}

/// `GET /boards/{board_id}/choices/choice_screen`: the decision waiting, with the card
/// it comes from; only the chooser gets the form.
///
/// # Errors
/// [`AppError::NotFound`] without pending choice; database errors.
pub async fn choice_screen(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let choice = game
        .board
        .pending_choice
        .as_ref()
        .ok_or(AppError::NotFound)?;
    let me = access.session.user.id;
    let played = played_heroes(&state, &game, me).await?;
    let sauron = game.sauron.user_id == me;
    let page = ChoicePage {
        layout: layout(&access, choice.chooser, &game, &state.data),
        board_id: access.board_id,
        chooser: chooser_name(choice.chooser, &game, &state.data),
        card_pic: card_picture(&choice.source),
        context: choice_context(&choice.source, &game, &state.data),
        options: choice
            .options
            .iter()
            .map(|o| option_view(o, &game, &state.data, sauron))
            .collect(),
        can_answer: may_answer(choice, &game, me, &played),
        seen_pic: seen_picture(&choice.source).filter(|_| sauron),
    };
    render(&page, &page.layout)
}

/// Option and targets of a submitted answer; targets of other options are ignored.
fn parse_answer(fields: &[(String, String)]) -> Result<(String, Vec<String>), AppError> {
    let option = fields
        .iter()
        .find(|(name, _)| name == "option")
        .map(|(_, value)| value.clone())
        .ok_or_else(|| AppError::BadRequest("option missing".to_owned()))?;
    let targets = fields
        .iter()
        .filter(|(name, _)| name == "targets")
        .filter_map(|(_, value)| value.split_once(TARGET_SEPARATOR))
        .filter(|(code, _)| *code == option)
        .map(|(_, target)| target.to_owned())
        .collect();
    Ok((option, targets))
}

/// Screen of whoever plays next.
fn next_screen(game: &Game, board_id: i64) -> String {
    let actor = if game.sauron.active {
        game.sauron.id.map(RouteActor::Sauron)
    } else {
        game.heroes
            .iter()
            .find(|h| h.active)
            .and_then(|h| h.id)
            .map(RouteActor::Hero)
    };
    actor.map_or_else(
        || "/boards".to_owned(),
        |actor| route_for(game.board.state, board_id, actor),
    )
}

/// `POST /boards/{board_id}/choices/choose`: the chooser's answer, an `option` field
/// and `targets` fields valued `<option>|<target>`.
///
/// # Errors
/// [`AppError::BadRequest`] without option; [`AppError::Engine`] when the user isn't
/// the chooser or the rules refuse the answer; database errors.
pub async fn choose(
    State(state): State<AppState>,
    access: BoardAccess,
    Form(fields): Form<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    let (option, targets) = parse_answer(&fields)?;
    let me = access.session.user.id;
    let played = played_heroes(&state, &read_game(&state, access.board_id).await?, me).await?;
    let (game, ()) = change_game(&state, access.board_id, |game, data, rng| {
        let allowed = game
            .board
            .pending_choice
            .as_ref()
            .is_some_and(|c| may_answer(c, game, me, &played));
        if !allowed {
            return Err(EngineError::Refused("not the chooser".to_owned()));
        }
        game.answer_choice(&option, &targets, data, rng)
    })
    .await?;
    Ok(redirect(&next_screen(&game, access.board_id), None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answer_keeps_the_targets_of_its_option() {
        let fields = [
            ("option", "remove_monsters"),
            ("targets", "remove_monsters|monster:3"),
            ("targets", "place_favor|location:bree"),
            ("targets", "remove_monsters|monster:4"),
        ]
        .map(|(n, v)| (n.to_owned(), v.to_owned()));
        let (option, targets) = parse_answer(&fields).unwrap();
        assert_eq!(option, "remove_monsters");
        assert_eq!(targets, ["monster:3", "monster:4"]);
        assert!(parse_answer(&[]).is_err());
    }
}
