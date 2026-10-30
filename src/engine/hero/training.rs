//! A hero improving: training, level tokens and favor (§ 7).

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::choices::{target, target_option};
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::corruption_card::Flaw;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;
use crate::model::hero::Hero;

/// Increases of one attribute at most (§ 7).
const MAX_LEVELS: u8 = 2;

/// Skill cards drawn when training.
const TRAINING_DRAW: usize = 2;

/// Most favor a despairing hero may hold.
const DESPAIRING_FAVOR_LIMIT: i32 = 3;

/// Most favor a mistrusted hero may gain per turn.
const MISTRUSTED_FAVOR_LIMIT: i32 = 1;

/// A hero attribute raised by level tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attribute {
    /// Fortitude.
    Fortitude,
    /// Strength.
    Strength,
    /// Agility.
    Agility,
    /// Wisdom.
    Wisdom,
}

impl Hero {
    /// Gains one level of an attribute; a third increase is lost (§ 7).
    ///
    /// # Returns
    /// `true` when the attribute increased.
    #[tracing::instrument(level = "debug", skip_all, fields(?attribute))]
    pub fn gain_level(&mut self, attribute: Attribute) -> bool {
        let (level, value) = match attribute {
            Attribute::Fortitude => (&mut self.levels.fortitude, &mut self.fortitude),
            Attribute::Strength => (&mut self.levels.strength, &mut self.strength),
            Attribute::Agility => (&mut self.levels.agility, &mut self.agility),
            Attribute::Wisdom => (&mut self.levels.wisdom, &mut self.wisdom),
        };
        if *level >= MAX_LEVELS {
            return false;
        }
        *level += 1;
        *value += 1;
        true
    }

    /// Gains favor; a despairing hero never holds more than 3, a mistrusted one gains
    /// at most 1 per turn.
    ///
    /// # Returns
    /// The favor actually gained.
    #[tracing::instrument(level = "debug", skip_all, fields(amount))]
    pub fn gain_favor(&mut self, amount: i32, data: &GameData) -> i32 {
        let mut gained = amount.max(0);
        if self.has_flaw(Flaw::Mistrusted, data) {
            gained = gained.min((MISTRUSTED_FAVOR_LIMIT - self.turn_record.favor_gained).max(0));
        }
        if self.has_flaw(Flaw::Dispairing, data) {
            gained = gained.min((DESPAIRING_FAVOR_LIMIT - self.favor).max(0));
        }
        self.favor += gained;
        self.turn_record.favor_gained += gained;
        gained
    }
}

impl Game {
    /// Training (§ 7): the hero draws the top two Skill cards and keeps one in hand.
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ?resume), err(level = "debug"))]
    pub fn train(
        &mut self,
        hero_id: i64,
        resume: Resume,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        self.train_times(hero_id, 1, resume, rng)
    }

    /// Receives training `times` times in a row, one choice after the other.
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, times, ?resume), err(level = "debug"))]
    pub fn train_times(
        &mut self,
        hero_id: i64,
        times: u8,
        resume: Resume,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        for left in (0..times).rev() {
            tracing::trace!(
                left,
                draw = self.board.decks.skill.draw.len(),
                "training draw"
            );
            let pile = &mut self.board.decks.skill;
            if pile.draw.len() < TRAINING_DRAW {
                pile.draw.append(&mut pile.discard);
                pile.draw.shuffle(rng);
                tracing::trace!(pile = "pile.draw", order = ?pile.draw, "shuffled");
            }
            let count = TRAINING_DRAW.min(pile.draw.len());
            let cards: Vec<CardNumber> = pile.draw.drain(..count).collect();
            if cards.len() <= 1 {
                self.hero_mut(hero_id)?.hand.extend(cards);
                continue;
            }
            let targets = cards.iter().map(|c| target("skill", c)).collect();
            return self.ask(PendingChoice {
                chooser: Chooser::Hero(hero_id),
                source: ChoiceSource::Training {
                    hero_id,
                    cards,
                    more: left,
                },
                options: vec![target_option("keep_skill", targets, 1, 1)],
                resume,
            });
        }
        Ok(())
    }

    /// The hero keeps the chosen Skill card; the other is discarded faceup.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ?targets), err(level = "debug"))]
    pub(crate) fn apply_training(
        &mut self,
        hero_id: i64,
        cards: &[CardNumber],
        targets: &[String],
    ) -> Result<(), EngineError> {
        let kept = cards
            .iter()
            .copied()
            .find(|c| targets.contains(&target("skill", c)))
            .ok_or_else(|| EngineError::Refused("choose a skill card".to_owned()))?;
        self.hero_mut(hero_id)?.hand.push(kept);
        let others = cards.iter().copied().filter(|c| *c != kept);
        self.board.decks.skill.discard.extend(others);
        self.log(
            ActorRef::Hero(hero_id),
            "training.keep",
            &[("skill", json!(kept))],
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, hero, rng};

    // § 7: each attribute may be increased at most twice.
    #[test]
    fn two_levels_per_attribute() {
        let mut h = hero("argalad");
        let wisdom = h.wisdom;
        assert!(h.gain_level(Attribute::Wisdom));
        assert!(h.gain_level(Attribute::Wisdom));
        assert!(!h.gain_level(Attribute::Wisdom));
        assert_eq!(h.wisdom, wisdom + 2);
    }

    // § 7 training: draw two Skill cards, keep one, discard the other.
    #[test]
    fn training_keeps_one_skill() {
        let mut g = game();
        g.board.state = BoardState::HeroExploration;
        g.board.decks.skill.draw = vec![100, 101, 102];
        g.train(1, Resume::Exploration, &mut rng()).unwrap();
        let hand = g.heroes[0].hand.len();
        g.answer_choice("keep_skill", &["skill:101".to_owned()], data(), &mut rng())
            .unwrap();
        assert_eq!(g.heroes[0].hand.len(), hand + 1);
        assert_eq!(g.heroes[0].hand.last(), Some(&101));
        assert_eq!(g.board.decks.skill.discard, vec![100]);
        assert_eq!(g.board.state, BoardState::HeroExploration);
    }
}
