//! Damage dealt to a hero outside combat (§ 6.3).

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::choices::{target, target_option};
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;
use crate::model::hero::Hero;

/// What dealing damage led to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dealt {
    /// Paid; play goes on.
    Paid,
    /// The hero chooses the hand cards paying it.
    Choice,
    /// The hero had too few cards and is defeated.
    Defeat,
}

/// Target code of the card at `index` in a hero's hand.
pub fn hand_target(hero_id: i64, index: usize) -> String {
    target("hand", format!("{hero_id}:{index}"))
}

/// Targets for every card of the hero's hand.
pub fn hand_targets(hero_id: i64, hero: &Hero) -> Vec<String> {
    (0..hero.hand.len())
        .map(|index| hand_target(hero_id, index))
        .collect()
}

/// Hand cards picked through [`hand_targets`].
///
/// # Errors
/// [`EngineError::Refused`] when a target isn't a card of the hero's hand.
pub fn picked_hand_cards(
    hero_id: i64,
    hero: &Hero,
    targets: &[String],
) -> Result<Vec<CardNumber>, EngineError> {
    targets
        .iter()
        .map(|t| {
            (0..hero.hand.len())
                .find(|index| hand_target(hero_id, *index) == *t)
                .and_then(|index| hero.hand.get(index).copied())
                .ok_or_else(|| EngineError::Refused(format!("{t} is not in hand")))
        })
        .collect()
}

impl Game {
    /// Deals `amount` damage to a hero outside combat: each damage costs a card from
    /// the hand or the top of the life pool (§ 6.3). The hero chooses when he holds
    /// cards; without enough cards left he is defeated (§ 8.8).
    ///
    /// # Arguments
    /// * `resume` - where play goes on after the hero's choice.
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, amount, ?resume), err(level = "debug"))]
    pub fn deal_damage(
        &mut self,
        hero_id: i64,
        amount: u32,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Dealt, EngineError> {
        let damage = usize::try_from(amount).unwrap_or(usize::MAX);
        if damage == 0 {
            return Ok(Dealt::Paid);
        }
        self.log(
            ActorRef::Hero(hero_id),
            "damage.dealt",
            &[("count", json!(amount))],
        );
        let hero = self.hero_mut(hero_id)?;
        if damage >= hero.life_pool.len() + hero.hand.len() {
            let mut everything = std::mem::take(&mut hero.life_pool);
            everything.append(&mut hero.hand);
            hero.damage_pool.extend(everything);
            self.start_hero_defeat(hero_id, data, rng)?;
            return Ok(Dealt::Defeat);
        }
        hero.temporary_damages = i32::try_from(amount).unwrap_or(i32::MAX);
        if hero.hand.is_empty() {
            hero.pay_damage(&[])?;
            return Ok(Dealt::Paid);
        }
        let targets = hand_targets(hero_id, hero);
        self.ask(PendingChoice {
            chooser: Chooser::Hero(hero_id),
            source: ChoiceSource::Damage { hero_id, amount },
            options: vec![target_option("pay_damage", targets, 0, damage)],
            resume,
        })?;
        Ok(Dealt::Choice)
    }

    /// The hero pays the damage with the picked hand cards, the rest from the life pool.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ?targets), err(level = "debug"))]
    pub(crate) fn apply_damage_choice(
        &mut self,
        hero_id: i64,
        targets: &[String],
    ) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        let cards = picked_hand_cards(hero_id, hero, targets)?;
        hero.pay_damage(&cards)
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};
    use crate::model::choice::Resume;

    use super::*;

    // § 6.3: the hero pays each damage from hand or life pool, in any combination.
    #[test]
    fn hero_chooses_cards_paying_the_damage() {
        let mut g = game();
        g.board.state = BoardState::HeroMovementScreen;
        g.heroes[0].hand = vec![1, 2, 3];
        let dealt = g
            .deal_damage(1, 2, Resume::Exploration, data(), &mut rng())
            .unwrap();
        assert_eq!(dealt, Dealt::Choice);
        let life = g.heroes[0].life_pool.len();
        g.answer_choice("pay_damage", &[hand_target(1, 1)], data(), &mut rng())
            .unwrap();
        let hero = &g.heroes[0];
        assert_eq!(hero.hand, vec![1, 3]);
        assert_eq!(hero.life_pool.len(), life - 1);
        assert_eq!(hero.damage_pool.len(), 2);
        assert_eq!(g.board.state, BoardState::HeroExploration);
    }

    // § 8.8: a hero without cards left in hand and life pool is defeated.
    #[test]
    fn damage_beyond_the_cards_left_defeats_the_hero() {
        let mut g = game();
        g.board.state = BoardState::ChoiceScreenBoardChoices;
        g.heroes[0].hand = vec![1];
        g.heroes[0].life_pool = vec![2];
        let dealt = g
            .deal_damage(1, 5, Resume::Exploration, data(), &mut rng())
            .unwrap();
        assert_eq!(dealt, Dealt::Defeat);
        assert_ne!(g.board.state, BoardState::ChoiceScreenBoardChoices);
    }
}
