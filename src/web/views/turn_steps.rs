//! Steps of the current turn, shown as a progress bar above the game table.

use crate::engine::board_state::BoardState;
use crate::model::choice::Resume;
use crate::model::combat::CombatOrigin;
use crate::model::game::Game;

/// Steps of Sauron's turn with a screen (the Hero Rally step is automatic).
const SAURON_STEPS: [&str; 5] = [
    "Histoire",
    "Intrigue",
    "Événement",
    "Actions",
    "Pioche des héros",
];

/// Steps of a hero's turn, in the engine's order (§ 6).
const HERO_STEPS: [&str; 4] = ["Embuscade", "Repos", "Déplacement", "Exploration"];

/// Whose turn the steps belong to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnSide {
    /// Sauron's turn.
    Sauron,
    /// A hero's turn.
    Hero,
}

/// One step of the progress bar.
#[derive(Debug, Clone)]
pub struct StepView {
    /// French name.
    pub label: &'static str,
    /// CSS class: `done`, `current` or `todo`.
    pub status: &'static str,
}

/// Progress through the current turn.
#[derive(Debug, Clone)]
pub struct TurnSteps {
    /// Whose turn it is.
    pub side: TurnSide,
    /// The steps, in order.
    pub steps: Vec<StepView>,
}

/// Steps of the turn in progress, or `None` outside turns (setup, Finale, game over).
pub fn turn_steps(game: &Game) -> Option<TurnSteps> {
    let (side, current) = current_step(game)?;
    let labels: &[&'static str] = match side {
        TurnSide::Sauron => &SAURON_STEPS,
        TurnSide::Hero => &HERO_STEPS,
    };
    let steps = labels
        .iter()
        .enumerate()
        .map(|(index, label)| StepView {
            label,
            status: match index.cmp(&current) {
                std::cmp::Ordering::Less => "done",
                std::cmp::Ordering::Equal => "current",
                std::cmp::Ordering::Greater => "todo",
            },
        })
        .collect();
    Some(TurnSteps { side, steps })
}

/// Side and index of the current step.
fn current_step(game: &Game) -> Option<(TurnSide, usize)> {
    use BoardState as S;
    match game.board.state {
        S::PlayScreenSauronPlotCards | S::LookForGollumCardsSauronPlotCards => {
            Some((TurnSide::Sauron, 1))
        }
        S::EditSauronSauronActions
        | S::PlaceInfluenceScreenSauronActions
        | S::KeepScreenSauronPlotCards
        | S::CommandScreenSauronMobs => Some((TurnSide::Sauron, 3)),
        S::HeroDrawCardsScreen => Some((TurnSide::Sauron, 4)),
        S::HeroRestScreen | S::AfterRestAdvanceStoryMarker => Some((TurnSide::Hero, 1)),
        S::HeroMovementScreen => Some((TurnSide::Hero, 2)),
        S::HeroExploration | S::HeroDefeatScreen | S::SauronHavenScreen => {
            Some((TurnSide::Hero, 3))
        }
        S::CombatSetupScreenBoardCombats
        | S::PlayCombatCardScreenBoardCombats
        | S::CardsLossScreenBoardCombats => combat_step(game),
        S::ChoiceScreenBoardChoices => choice_step(game),
        S::Created
        | S::WaitingForPlayers
        | S::SauronSetupScreen
        | S::FinishHeroTurn
        | S::FinishSauronTurn
        | S::Finished => None,
    }
}

fn combat_step(game: &Game) -> Option<(TurnSide, usize)> {
    match game.combat.as_ref()?.origin {
        CombatOrigin::Ambush | CombatOrigin::Shadow { .. } => Some((TurnSide::Hero, 0)),
        CombatOrigin::Travel | CombatOrigin::Peril { .. } => Some((TurnSide::Hero, 2)),
        CombatOrigin::Finale => None,
    }
}

/// A pending choice belongs to the step before the one it resumes.
fn choice_step(game: &Game) -> Option<(TurnSide, usize)> {
    match game.board.pending_choice.as_ref()?.resume {
        Resume::EventStep => Some((TurnSide::Sauron, 1)),
        Resume::ActionStep => Some((TurnSide::Sauron, 2)),
        Resume::SauronActions => Some((TurnSide::Sauron, 3)),
        Resume::Ambush | Resume::Rest => Some((TurnSide::Hero, 0)),
        Resume::AfterRest => Some((TurnSide::Hero, 1)),
        Resume::Movement | Resume::TravelThreats => Some((TurnSide::Hero, 2)),
        Resume::Exploration | Resume::EndTurn => Some((TurnSide::Hero, 3)),
        Resume::CombatSetup => combat_step(game),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::game;

    fn game_in(state: BoardState) -> Game {
        let mut game = game();
        game.board.state = state;
        game
    }

    #[test]
    fn hero_movement_marks_earlier_steps_done() {
        let steps = turn_steps(&game_in(BoardState::HeroMovementScreen)).expect("hero turn");
        assert_eq!(steps.side, TurnSide::Hero);
        let statuses: Vec<_> = steps.steps.iter().map(|s| s.status).collect();
        assert_eq!(statuses, ["done", "done", "current", "todo"]);
    }

    #[test]
    fn sauron_actions_are_the_fourth_step() {
        let steps = turn_steps(&game_in(BoardState::EditSauronSauronActions)).expect("Sauron turn");
        assert_eq!(steps.side, TurnSide::Sauron);
        assert_eq!(steps.steps[3].status, "current");
    }

    #[test]
    fn setup_and_end_have_no_steps() {
        assert!(turn_steps(&game_in(BoardState::SauronSetupScreen)).is_none());
        assert!(turn_steps(&game_in(BoardState::Finished)).is_none());
    }
}
