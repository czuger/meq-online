//! Asking a player to decide, checking the answer and resuming play.

use rand::rngs::StdRng;

use crate::engine::board_state::{BoardEvent, BoardState};
use crate::engine::engine_error::EngineError;
use crate::engine::perils::answers::PerilAnswer;
use crate::engine::shadows::answers::ShadowAnswer;
use crate::engine::shadows::shadow_window::ShadowWindow;
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
        let resume = self.apply_answer(choice.source, option, targets, choice.resume, data, rng)?;
        // An answer may ask another choice, start a combat, defeat the hero or end his
        // turn: play only resumes from a settled choice screen.
        if self.board.pending_choice.is_some()
            || self.board.state != BoardState::ChoiceScreenBoardChoices
        {
            return Ok(());
        }
        self.resume(resume, data, rng)
    }

    /// Applies an answer to what its choice belongs to.
    ///
    /// # Returns
    /// Where play goes on, usually `resume`.
    fn apply_answer(
        &mut self,
        source: ChoiceSource,
        option: &str,
        targets: &[String],
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        match source {
            ChoiceSource::Event { .. } | ChoiceSource::ExploreEvent { .. } => {
                self.apply_event_answer(source, option, targets, resume, data, rng)?;
            }
            ChoiceSource::Training {
                hero_id,
                cards,
                more,
            } => {
                self.apply_training(hero_id, &cards, targets)?;
                self.train_times(hero_id, more, resume, rng)?;
            }
            ChoiceSource::Threat { hero_id, ambush } => {
                self.apply_threat_choice(hero_id, ambush, option, targets, data, rng)?;
            }
            ChoiceSource::HavenCombat { hero_id, ambush } => {
                self.apply_haven_choice(hero_id, ambush, option, data, rng)?;
            }
            ChoiceSource::PerilDraw { hero_id, cards } => {
                return self.apply_peril_draw(hero_id, &cards, targets, data, rng);
            }
            ChoiceSource::Peril {
                hero_id,
                card,
                drawn,
            } => {
                let answer = PerilAnswer {
                    hero_id,
                    card,
                    drawn: &drawn,
                    option,
                    targets,
                };
                return self.apply_peril_choice(&answer, data, rng);
            }
            ChoiceSource::Damage { hero_id, .. } => self.apply_damage_choice(hero_id, targets)?,
            ChoiceSource::Wretched { hero_id } => {
                self.apply_wretched_choice(hero_id, option, targets, data)?;
            }
            ChoiceSource::ShadowWindow { .. }
            | ChoiceSource::Shadow { .. }
            | ChoiceSource::PlotLocation { .. } => {
                return self.apply_shadow_answer(source, option, targets, resume, data, rng);
            }
        }
        Ok(resume)
    }

    /// Applies an answer about a Shadow card or the location of a Plot card.
    fn apply_shadow_answer(
        &mut self,
        source: ChoiceSource,
        option: &str,
        targets: &[String],
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        match source {
            ChoiceSource::ShadowWindow { timing, hero_id } => {
                let window = ShadowWindow { timing, hero_id };
                self.apply_shadow_window(window, option, targets, resume, data, rng)
            }
            ChoiceSource::Shadow {
                card,
                hero_id,
                heroes,
                picked,
            } => {
                let answer = ShadowAnswer {
                    card,
                    hero_id,
                    heroes: &heroes,
                    picked: &picked,
                    option,
                    targets,
                    resume,
                };
                self.apply_shadow_choice(&answer, data, rng)
            }
            ChoiceSource::PlotLocation { card, slot } => {
                self.apply_plot_location(card, slot, targets, data)?;
                Ok(resume)
            }
            _ => Ok(resume),
        }
    }

    /// Applies an answer about an event card.
    fn apply_event_answer(
        &mut self,
        source: ChoiceSource,
        option: &str,
        targets: &[String],
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        match source {
            ChoiceSource::Event {
                stage,
                card,
                heroes,
            } => {
                let event = EventRef { stage, card };
                self.apply_event_choice(event, &heroes, option, targets, resume, data, rng)
            }
            ChoiceSource::ExploreEvent {
                stage,
                card,
                hero_id,
            } => {
                let event = EventRef { stage, card };
                self.apply_explore_reward(event, hero_id, option, targets, resume, data, rng)
            }
            _ => Ok(()),
        }
    }

    /// Plays on after the last choice.
    fn resume(
        &mut self,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        match resume {
            Resume::ActionStep | Resume::SauronActions | Resume::EventStep => {
                self.resume_sauron(resume, data, rng)
            }
            Resume::Exploration => {
                self.fire(BoardEvent::NextToHeroExploration)?;
                self.activate_current_hero()
            }
            Resume::Movement => {
                self.fire(BoardEvent::NextToHeroMovementScreen)?;
                self.activate_current_hero()
            }
            Resume::Rest => {
                self.activate_current_hero()?;
                self.enter_rest_step(data)
            }
            Resume::AfterRest => {
                self.activate_current_hero()?;
                let hero_id = self.current_hero_id()?;
                self.advance_story_after_rest(hero_id, data).map(|_| ())
            }
            Resume::EndTurn => {
                let hero_id = self.current_hero_id()?;
                self.finish_heroes_turn(hero_id, data).map(|_| ())
            }
            Resume::Ambush => {
                self.activate_current_hero()?;
                self.ambush_step(data, rng)
            }
            Resume::TravelThreats => {
                self.activate_current_hero()?;
                let hero_id = self.current_hero_id()?;
                self.travel_threats(hero_id, data, rng).map(|_| ())
            }
            Resume::CombatSetup => {
                self.fire(BoardEvent::NextToCombatSetupScreenBoardCombats)?;
                self.activate_current_hero()
            }
        }
    }

    /// Plays on in Sauron's turn: the Event step, the Action step or between actions.
    fn resume_sauron(
        &mut self,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        self.set_heroes_active(false);
        self.sauron.active = true;
        match resume {
            Resume::EventStep => self.plot_play_finished(data, rng),
            Resume::SauronActions => self.fire(BoardEvent::NextToEditSauronSauronActions),
            _ => self.start_action_step(data),
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
