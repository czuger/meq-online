//! Steps of the current turn, shown as a progress bar above the game table.

use crate::engine::board_state::BoardState;
use crate::model::choice::Resume;
use crate::model::combat::CombatOrigin;
use crate::model::game::Game;
use crate::model::turn_step::TurnStep;
use crate::web::views::step_label::step_label;
use crate::web::views::turn_log::TurnLog;

/// Steps of Sauron's turn with a screen (the Hero Rally step is automatic).
const SAURON_STEPS: [TurnStep; 5] = [
    TurnStep::Story,
    TurnStep::Plot,
    TurnStep::Event,
    TurnStep::Action,
    TurnStep::HeroDraw,
];

/// Steps of a hero's turn, in the engine's order (§ 6).
const HERO_STEPS: [TurnStep; 4] = [
    TurnStep::Ambush,
    TurnStep::Rest,
    TurnStep::Movement,
    TurnStep::Exploration,
];

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
    /// CSS class: `done`, `skipped`, `current` or `todo`.
    pub status: &'static str,
    /// Why the step couldn't be played.
    pub reason: Option<&'static str>,
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
///
/// # Arguments
/// * `game` - the board.
/// * `turn_log` - what the latest turns did, for the steps that couldn't be played.
pub fn turn_steps(game: &Game, turn_log: &TurnLog) -> Option<TurnSteps> {
    let (side, current) = current_step(game)?;
    let order: &[TurnStep] = match side {
        TurnSide::Sauron => &SAURON_STEPS,
        TurnSide::Hero => &HERO_STEPS,
    };
    let skipped = turn_log.skipped_steps(side == TurnSide::Sauron);
    let steps = order
        .iter()
        .enumerate()
        .map(|(index, step)| {
            let reason = skipped.iter().find(|(s, _)| s == step).map(|(_, r)| *r);
            let reason = reason.filter(|_| index < current);
            StepView {
                label: step_label(*step),
                status: match index.cmp(&current) {
                    std::cmp::Ordering::Less if reason.is_some() => "skipped",
                    std::cmp::Ordering::Less => "done",
                    std::cmp::Ordering::Equal => "current",
                    std::cmp::Ordering::Greater => "todo",
                },
                reason,
            }
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
    use serde_json::json;

    use super::*;
    use crate::engine::test_support::{data, game};
    use crate::model::actor_ref::ActorRef;
    use crate::model::log_entry::LogEntry;
    use crate::model::turn_step::{STEP_SKIPPED, TURN_HERO};

    fn game_in(state: BoardState) -> Game {
        let mut game = game();
        game.board.state = state;
        game
    }

    #[test]
    fn hero_movement_marks_earlier_steps_done() {
        let steps = turn_steps(
            &game_in(BoardState::HeroMovementScreen),
            &TurnLog::default(),
        )
        .expect("hero turn");
        assert_eq!(steps.side, TurnSide::Hero);
        let statuses: Vec<_> = steps.steps.iter().map(|s| s.status).collect();
        assert_eq!(statuses, ["done", "done", "current", "todo"]);
    }

    #[test]
    fn a_step_that_could_not_be_played_is_marked_with_its_reason() {
        let entries = [
            LogEntry::new(ActorRef::Hero(1), TURN_HERO, &[("hero", json!("argalad"))]),
            LogEntry::new(
                ActorRef::System,
                STEP_SKIPPED,
                &[("step", json!("rest")), ("reason", json!("mob_here"))],
            ),
        ];
        let refs: Vec<_> = entries.iter().collect();
        let game = game_in(BoardState::HeroMovementScreen);
        let log = TurnLog::new(&refs, &game, data());
        let steps = turn_steps(&game, &log).expect("hero turn");
        let statuses: Vec<_> = steps.steps.iter().map(|s| s.status).collect();
        assert_eq!(statuses, ["done", "skipped", "current", "todo"]);
        assert!(steps.steps[1].reason.is_some());
    }

    #[test]
    fn sauron_actions_are_the_fourth_step() {
        let steps = turn_steps(
            &game_in(BoardState::EditSauronSauronActions),
            &TurnLog::default(),
        )
        .expect("Sauron turn");
        assert_eq!(steps.side, TurnSide::Sauron);
        assert_eq!(steps.steps[3].status, "current");
    }

    #[test]
    fn setup_and_end_have_no_steps() {
        assert!(turn_steps(&game_in(BoardState::SauronSetupScreen), &TurnLog::default()).is_none());
        assert!(turn_steps(&game_in(BoardState::Finished), &TurnLog::default()).is_none());
    }
}
