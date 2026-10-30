//! Asking a player to decide, checking the answer and resuming play.

use rand::rngs::StdRng;

use crate::engine::board_state::{BoardEvent, BoardState};
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::board::EventRef;
use crate::model::choice::{ChoiceOption, ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// Option code of declining an optional effect.
pub const PASS: &str = "pass";

/// Target code `kind:value`.
pub fn target(kind: &str, value: impl std::fmt::Display) -> String {
    format!("{kind}:{value}")
}

/// Splits a target code into its kind and values.
pub fn parse_target(code: &str) -> (&str, Vec<&str>) {
    let mut parts = code.split(':');
    let kind = parts.next().unwrap_or_default();
    (kind, parts.collect())
}

/// An option without targets.
pub fn plain_option(code: &str) -> ChoiceOption {
    ChoiceOption {
        code: code.to_owned(),
        targets: vec![],
        min: 0,
        max: 0,
    }
}

/// An option picking between `min` and `max` of `targets`, or `None` when there is
/// nothing to pick.
pub fn available_option(
    code: &str,
    targets: Vec<String>,
    min: usize,
    max: usize,
) -> Option<ChoiceOption> {
    (!targets.is_empty()).then(|| target_option(code, targets, min, max))
}

/// An option picking between `min` and `max` of `targets`.
pub fn target_option(code: &str, targets: Vec<String>, min: usize, max: usize) -> ChoiceOption {
    let max = max.min(targets.len());
    ChoiceOption {
        code: code.to_owned(),
        targets,
        min: min.min(max),
        max,
    }
}

impl Game {
    /// Waits for a player's decision on the choice screen; only the chooser is active.
    ///
    /// # Errors
    /// Transition errors.
    pub fn ask(&mut self, choice: PendingChoice) -> Result<(), EngineError> {
        if self.board.state != BoardState::ChoiceScreenBoardChoices {
            self.fire(BoardEvent::NextToChoiceScreenBoardChoices)?;
        }
        self.set_heroes_active(false);
        self.sauron.active = false;
        match choice.chooser {
            Chooser::Sauron => self.sauron.active = true,
            Chooser::Hero(hero_id) => self.hero_mut(hero_id)?.active = true,
            Chooser::Heroes => self.set_heroes_active(true),
        }
        self.board.pending_choice = Some(choice);
        Ok(())
    }

    /// Applies the chooser's answer: the option picked and its targets. Play resumes
    /// once no choice is left.
    ///
    /// # Errors
    /// [`EngineError::Refused`] without a pending choice, for an unknown option or
    /// targets outside it; errors of the effect applied.
    pub fn answer_choice(
        &mut self,
        option: &str,
        targets: &[String],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let choice = self
            .board
            .pending_choice
            .take()
            .ok_or_else(|| EngineError::Refused("no choice to answer".to_owned()))?;
        let picked = choice
            .options
            .iter()
            .find(|o| o.code == option)
            .ok_or_else(|| EngineError::Refused(format!("unknown option {option}")))?;
        check_targets(picked, targets)?;
        match choice.source {
            ChoiceSource::Event {
                stage,
                card,
                heroes,
            } => {
                let event = EventRef { stage, card };
                self.apply_event_choice(event, &heroes, option, targets, choice.resume, data, rng)?;
            }
            ChoiceSource::ExploreEvent {
                stage,
                card,
                hero_id,
            } => {
                let event = EventRef { stage, card };
                self.apply_explore_reward(
                    event,
                    hero_id,
                    option,
                    targets,
                    choice.resume,
                    data,
                    rng,
                )?;
            }
            ChoiceSource::Training { hero_id, cards } => {
                self.apply_training(hero_id, &cards, targets)?;
            }
        }
        if self.board.pending_choice.is_some() {
            return Ok(());
        }
        self.resume(choice.resume, data)
    }

    /// Plays on after the last choice.
    fn resume(&mut self, resume: Resume, data: &GameData) -> Result<(), EngineError> {
        match resume {
            Resume::ActionStep => {
                self.set_heroes_active(false);
                self.sauron.active = true;
                self.start_action_step(data)
            }
            Resume::Exploration => {
                self.fire(BoardEvent::NextToHeroExploration)?;
                self.activate_current_hero()
            }
        }
    }
}

fn check_targets(option: &ChoiceOption, targets: &[String]) -> Result<(), EngineError> {
    let mut unique: Vec<&String> = targets.iter().collect();
    unique.sort();
    unique.dedup();
    let known = targets.iter().all(|t| option.targets.contains(t));
    let count_ok = (option.min..=option.max).contains(&targets.len());
    if known && count_ok && unique.len() == targets.len() {
        Ok(())
    } else {
        Err(EngineError::Refused(format!(
            "pick {} to {} of the proposed targets",
            option.min, option.max
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_must_be_proposed_and_counted() {
        let option = target_option("x", vec!["a:1".into(), "a:2".into()], 1, 2);
        assert!(check_targets(&option, &["a:1".into()]).is_ok());
        assert!(check_targets(&option, &[]).is_err());
        assert!(check_targets(&option, &["a:3".into()]).is_err());
        assert!(check_targets(&option, &["a:1".into(), "a:1".into()]).is_err());
        assert_eq!(
            parse_target("influence:bree:2"),
            ("influence", vec!["bree", "2"])
        );
    }
}
