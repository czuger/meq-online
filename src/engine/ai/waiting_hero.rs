//! The computer hero the game waits on, if any.

use crate::engine::board_state::BoardState;
use crate::model::choice::Chooser;
use crate::model::game::Game;

/// The computer hero expected to act now.
///
/// # Arguments
/// * `computer` - ids of the heroes played by the computer.
///
/// # Returns
/// `None` when the game waits on Sauron, on a human hero, or on no one.
pub fn waiting_computer_hero(game: &Game, computer: &[i64]) -> Option<i64> {
    let is_computer = |id: &i64| computer.contains(id);
    match game.board.state {
        BoardState::HeroDrawCardsScreen => game
            .heroes
            .iter()
            .filter(|h| h.active)
            .filter_map(|h| h.id)
            .find(is_computer),
        BoardState::HeroRestScreen
        | BoardState::AfterRestAdvanceStoryMarker
        | BoardState::HeroMovementScreen
        | BoardState::HeroExploration
        | BoardState::HeroDefeatScreen => game.board.current_hero_id.filter(is_computer),
        BoardState::CombatSetupScreenBoardCombats | BoardState::CardsLossScreenBoardCombats => {
            combat_hero(game).filter(is_computer)
        }
        BoardState::PlayCombatCardScreenBoardCombats => combat_card_hero(game, computer),
        BoardState::ChoiceScreenBoardChoices => choice_hero(game, computer),
        _ => None,
    }
}

fn combat_hero(game: &Game) -> Option<i64> {
    game.combat.as_ref().map(|c| c.hero_id)
}

/// The combat's computer hero while it still has to choose its card, or once the
/// combat is over and must be ended.
fn combat_card_hero(game: &Game, computer: &[i64]) -> Option<i64> {
    let hero_id = combat_hero(game).filter(|id| computer.contains(id))?;
    let deciding = game.hero(hero_id).is_ok_and(|h| h.active);
    (deciding || combat_over(game)).then_some(hero_id)
}

/// Whether the combat may be ended, read on a copy: refreshing the result marks a
/// side with an empty hand exhausted.
fn combat_over(game: &Game) -> bool {
    let mut copy = game.clone();
    copy.refresh_combat_result()
        .is_ok_and(|r| r.hero_defeated || r.mob_defeated || (r.hero_exhausted && r.mob_exhausted))
}

/// The computer hero answering the pending choice. A choice of the heroes as a side
/// is left to a human hero when there is one.
fn choice_hero(game: &Game, computer: &[i64]) -> Option<i64> {
    let choice = game.board.pending_choice.as_ref()?;
    match choice.chooser {
        Chooser::Sauron => None,
        Chooser::Hero(id) => computer.contains(&id).then_some(id),
        Chooser::Heroes => {
            let ids: Vec<i64> = game.heroes.iter().filter_map(|h| h.id).collect();
            let all_computer = ids.iter().all(|id| computer.contains(id));
            if all_computer {
                ids.first().copied()
            } else {
                None
            }
        }
    }
}
