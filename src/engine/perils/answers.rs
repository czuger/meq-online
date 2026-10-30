//! Decisions taken while resolving a Peril card.

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::choices::{
    PASS, available_option, parse_target, plain_option, target, target_option,
};
use crate::engine::deck::SauronDeck;
use crate::engine::engine_error::EngineError;
use crate::engine::hero::damage::picked_hand_cards;
use crate::engine::perils::effects::{
    BALROG, HOUNDS_DAMAGE, ORC, PERIL_DAMAGE, PERIL_FAVOR, option, target_id,
};
use crate::game_data::GameData;
use crate::game_data::peril::{PerilCode, PerilEffect};
use crate::game_data::plot::StoryType;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{Chooser, Resume};
use crate::model::game::Game;

/// Corruption cards Sauron draws for "Ill-met Company".
const ILL_MET_DRAW: usize = 2;

/// A decision within a Peril card, as answered.
pub(crate) struct PerilAnswer<'a> {
    /// The hero resolving the card.
    pub hero_id: i64,
    /// The Peril card.
    pub card: PerilCode,
    /// Cards drawn for the effect.
    pub drawn: &'a [u32],
    /// Option picked.
    pub option: &'a str,
    /// Targets picked.
    pub targets: &'a [String],
}

impl Game {
    /// Applies a decision of a Peril card.
    ///
    /// # Returns
    /// Where play goes on when no other decision is pending.
    pub(crate) fn apply_peril_choice(
        &mut self,
        answer: &PerilAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let go_on = Resume::Exploration;
        let hero_id = answer.hero_id;
        match answer.option {
            PASS => {}
            option::DISCARD_SHIELDS => self.hounds_discard(hero_id, answer, data, rng)?,
            option::FAVOR => {
                self.hero_mut(hero_id)?.gain_favor(1, data);
            }
            option::TRAINING => self.train(hero_id, go_on, rng)?,
            option::DAMAGE => {
                self.deal_damage(hero_id, PERIL_DAMAGE, go_on, data, rng)?;
            }
            option::SAURON_DRAWS => {
                self.draw_to_sauron_hand(SauronDeck::Plot, 1, rng);
                self.draw_to_sauron_hand(SauronDeck::Shadow, 1, rng);
            }
            option::ADVANCE_CORRUPTION => self.advance_corruption_marker(hero_id),
            option::LOSE_FAVOR => self.hero_mut(hero_id)?.favor_loss(PERIL_FAVOR),
            option::CORRUPTION => self.ill_met_draw(hero_id, answer.card, rng)?,
            option::GIVE_CORRUPTION => return self.ill_met_give(answer, data, rng),
            option::FIGHT => self.peril_fight(hero_id, answer.card, data, rng)?,
            option::MINION => {
                let mob_id = target_id(first(answer.targets)?)?;
                self.start_peril_combat(hero_id, mob_id, answer.card, data, rng)?;
            }
            option::DISCARD_CARDS => {
                let hero = self.hero_mut(hero_id)?;
                hero.hand_to_rest(&picked_hand_cards(hero_id, hero, answer.targets)?)?;
            }
            option::PLACE_INFLUENCE => self.peril_influence(answer, data)?,
            option::PLACE_MONSTER => {
                for location in locations(answer.targets) {
                    self.place_new_monster(location, data, rng)?;
                }
            }
            option::MOVE_MINION => self.peril_minion_move(answer, data, rng)?,
            option::DRAW_PLOT => self.draw_to_sauron_hand(SauronDeck::Plot, 1, rng),
            other => return Err(EngineError::Refused(format!("unknown option {other}"))),
        }
        Ok(go_on)
    }

    /// "The Weakness of Men": the black (corruption) marker advances 1 space.
    fn advance_corruption_marker(&mut self, hero_id: i64) {
        self.board.story.advance(StoryType::Corruption, 1);
        let params = [
            ("marker_name", json!(StoryType::Corruption)),
            ("count", json!(1)),
        ];
        self.log(ActorRef::Hero(hero_id), "story.advance", &params);
    }

    /// "The Hounds of Sauron": discarded cards reduce the damage by their shields; with
    /// no damage left the hero gains 1 favor or training.
    fn hounds_discard(
        &mut self,
        hero_id: i64,
        answer: &PerilAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        let cards = picked_hand_cards(hero_id, hero, answer.targets)?;
        let code = hero.code.clone();
        hero.hand_to_rest(&cards)?;
        let mut shields = 0;
        for card in &cards {
            shields += data.hero_card(&code, *card)?.defense.max(0);
        }
        let damage = HOUNDS_DAMAGE.saturating_sub(u32::try_from(shields).unwrap_or(u32::MAX));
        if damage > 0 {
            self.deal_damage(hero_id, damage, Resume::Exploration, data, rng)?;
            return Ok(());
        }
        let options = vec![plain_option(option::FAVOR), plain_option(option::TRAINING)];
        self.ask_peril(
            hero_id,
            answer.card,
            Chooser::Hero(hero_id),
            options,
            vec![],
        )
    }

    /// "Ill-met Company": Sauron draws two Corruption cards to pick one for the hero.
    fn ill_met_draw(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let drawn = self.draw_corruption_cards(ILL_MET_DRAW, rng);
        let targets = drawn.iter().map(|c| target("corruption", c)).collect();
        let options = vec![target_option(option::GIVE_CORRUPTION, targets, 1, 1)];
        self.ask_peril(hero_id, card, Chooser::Sauron, options, drawn)
    }

    /// Sauron gives the hero one of the drawn Corruption cards and discards the other.
    fn ill_met_give(
        &mut self,
        answer: &PerilAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let given = answer
            .drawn
            .iter()
            .copied()
            .find(|c| answer.targets.contains(&target("corruption", c)))
            .ok_or_else(|| EngineError::Refused("choose a corruption card".to_owned()))?;
        let others = answer.drawn.iter().copied().filter(|c| *c != given);
        self.board.decks.corruption.discard.extend(others);
        self.give_corruption(answer.hero_id, given, data, rng)?;
        let params = [("corruption", json!(data.corruption_card(given)?.name))];
        self.log(ActorRef::Hero(answer.hero_id), "corruption.gain", &params);
        if self.end_turn_if_wretched(answer.hero_id, given, data)? {
            return Ok(Resume::EndTurn);
        }
        Ok(Resume::Exploration)
    }

    /// The monster a Peril card created on the hero's location fights him.
    fn peril_fight(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let code = match data.peril(card)?.effect {
            PerilEffect::KhazadDum => BALROG,
            _ => ORC,
        };
        let location = self.hero(hero_id)?.location.clone();
        let mob_id = self
            .mobs
            .iter()
            .rev()
            .find(|m| m.code == code && m.location == location && m.pool.is_none())
            .and_then(|m| m.id)
            .ok_or(EngineError::Unpersisted("mob"))?;
        self.start_peril_combat(hero_id, mob_id, card, data, rng)?;
        Ok(())
    }

    /// Sauron places 1 influence on each picked location; for "Undying Regret" he may
    /// then place a monster token.
    fn peril_influence(
        &mut self,
        answer: &PerilAnswer,
        data: &GameData,
    ) -> Result<(), EngineError> {
        for location in locations(answer.targets) {
            self.add_influence(location, 1, data);
            let params = [("location", json!(data.locations.name(location)))];
            self.log(ActorRef::Sauron, "peril.influence", &params);
        }
        if data.peril(answer.card)?.effect != PerilEffect::UndyingRegret {
            return Ok(());
        }
        let spots = self.hero_region_targets(answer.hero_id, true, data)?;
        let Some(place) = available_option(option::PLACE_MONSTER, spots, 0, 1) else {
            return Ok(());
        };
        self.ask_peril(
            answer.hero_id,
            answer.card,
            Chooser::Sauron,
            vec![place],
            vec![],
        )
    }

    /// "Nine for Mortal Men": a commanded minion entering the hero's location fights him.
    fn peril_minion_move(
        &mut self,
        answer: &PerilAnswer,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let (_, values) = parse_target(first(answer.targets)?);
        let (Some(id), Some(to)) = (values.first(), values.get(1)) else {
            return Err(EngineError::Refused("choose a minion move".to_owned()));
        };
        let mob_id: i64 = id
            .parse()
            .map_err(|_| EngineError::Refused(format!("bad minion {id}")))?;
        let mob = self.mob_mut(mob_id)?;
        mob.location = (*to).to_owned();
        let name = mob.name.clone();
        let params = [
            ("name", json!(name)),
            ("location", json!(data.locations.name(to))),
        ];
        self.log(ActorRef::Sauron, "command.move", &params);
        if self.hero(answer.hero_id)?.location == *to {
            self.start_peril_combat(answer.hero_id, mob_id, answer.card, data, rng)?;
        }
        Ok(())
    }
}

fn first(targets: &[String]) -> Result<&String, EngineError> {
    targets
        .first()
        .ok_or_else(|| EngineError::Refused("choose a target".to_owned()))
}

/// Location codes of `location:<code>` targets.
fn locations(targets: &[String]) -> impl Iterator<Item = &str> {
    targets.iter().filter_map(|t| t.strip_prefix("location:"))
}
