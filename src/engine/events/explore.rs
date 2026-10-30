//! Event cards triggered by an exploring hero: the card in play (green flag) and the
//! card a hero holds (§ 5.6).

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::choices::{
    PASS, available_option, parse_target, plain_option, target, target_option,
};
use crate::engine::engine_error::EngineError;
use crate::engine::events::abilities::area_locations;
use crate::engine::hero::training::Attribute;
use crate::game_data::GameData;
use crate::game_data::event::{Area, EventAbility, ExploreReward};
use crate::game_data::mob_def::MobKind;
use crate::model::actor_ref::ActorRef;
use crate::model::board::EventRef;
use crate::model::choice::{ChoiceOption, ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// Character moved by some rewards.
const GANDALF: &str = "gandalf";

impl Game {
    /// The event in play when it affects the hero's location.
    pub fn event_at_hero(&self, hero_id: i64, data: &GameData) -> Option<EventRef> {
        let event = self.board.current_event?;
        let location = &self.hero(hero_id).ok()?.location;
        let affect = data.event(event.stage, event.card).ok()?.affect.as_ref()?;
        (affect == location).then_some(event)
    }

    /// The exploring hero activates the event in play on its location: the hero picks
    /// one of its rewards (or declines, and the card stays). A card without reward is
    /// just discarded.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when no event in play affects the hero's location;
    /// lookup and choice errors.
    pub fn activate_current_event(
        &mut self,
        hero_id: i64,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let event = self
            .event_at_hero(hero_id, data)
            .ok_or_else(|| EngineError::Refused("no event in play here".to_owned()))?;
        let EventAbility::OnExplore { rewards } = &data.event(event.stage, event.card)?.ability
        else {
            return Ok(());
        };
        if rewards.is_empty() {
            self.discard_current_event();
            self.log_event(hero_id, event, data)?;
            return Ok(());
        }
        let mut options: Vec<ChoiceOption> = rewards
            .iter()
            .filter_map(|r| self.reward_option(hero_id, r, data))
            .collect();
        options.push(plain_option(PASS));
        self.ask(PendingChoice {
            chooser: Chooser::Hero(hero_id),
            source: ChoiceSource::ExploreEvent {
                stage: event.stage,
                card: event.card,
                hero_id,
            },
            options,
            resume: Resume::Exploration,
        })
    }

    /// The option offering a reward, when the hero can take it.
    fn reward_option(
        &self,
        hero_id: i64,
        reward: &ExploreReward,
        data: &GameData,
    ) -> Option<ChoiceOption> {
        let hero = self.hero(hero_id).ok()?;
        let code = reward.code();
        let all_locations = || data.locations.iter().map(|(c, _)| target("location", c));
        match reward {
            ExploreReward::AdvanceHeroes { cost } | ExploreReward::RetreatCorruption { cost }
                if hero.favor < *cost =>
            {
                None
            }
            ExploreReward::DiscardCorruption => {
                let cards = hero
                    .corruption_cards
                    .iter()
                    .map(|c| target("corruption", c));
                available_option(code, cards.collect(), 1, 1)
            }
            ExploreReward::MoveAnywhere => available_option(code, all_locations().collect(), 1, 1),
            ExploreReward::GandalfToInfluence => {
                let influenced = self
                    .board
                    .influence
                    .iter()
                    .filter(|(_, v)| **v > 0)
                    .map(|(l, _)| target("location", l));
                available_option(code, influenced.collect(), 1, 1)
            }
            ExploreReward::BoatAndMove { location } => Some(target_option(
                code,
                vec![target("location", location)],
                0,
                1,
            )),
            ExploreReward::FavorAndRemoveMonster { .. } => {
                let all: Vec<String> = data.locations.iter().map(|(c, _)| c.clone()).collect();
                let monsters = self.mob_targets(&all, MobKind::Monster);
                Some(target_option(code, monsters, 0, 1))
            }
            ExploreReward::GandalfFarAway { distance } => {
                let distances = data.paths.distances_from(&hero.location);
                let far = data
                    .locations
                    .iter()
                    .filter(|(c, _)| distances.get(c.as_str()).is_none_or(|d| d >= distance))
                    .map(|(c, _)| target("location", c));
                Some(target_option(code, far.collect(), 0, 1))
            }
            _ => Some(plain_option(code)),
        }
    }

    /// Applies the reward the exploring hero picked; the event card is then discarded.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply_explore_reward(
        &mut self,
        event: EventRef,
        hero_id: i64,
        option: &str,
        targets: &[String],
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if option == PASS {
            return Ok(());
        }
        let EventAbility::OnExplore { rewards } = &data.event(event.stage, event.card)?.ability
        else {
            return Ok(());
        };
        let reward = rewards
            .iter()
            .find(|r| r.code() == option)
            .cloned()
            .ok_or_else(|| EngineError::Refused(format!("unknown reward {option}")))?;
        self.discard_current_event();
        self.log_event(hero_id, event, data)?;
        let location = |kind| first_value(targets, kind).map(str::to_owned);
        match reward {
            ExploreReward::Training => return self.train(hero_id, resume, rng),
            ExploreReward::DiscardCorruption => {
                if let Some(card) = first_value(targets, "corruption").and_then(|c| c.parse().ok())
                {
                    self.remove_corruption_card(hero_id, card, 0, data)?;
                }
            }
            ExploreReward::MoveAnywhere | ExploreReward::BoatAndMove { .. } => {
                if matches!(reward, ExploreReward::BoatAndMove { .. }) {
                    self.hero_mut(hero_id)?.items.boat = true;
                }
                if let Some(to) = location("location") {
                    self.hero_mut(hero_id)?.location = to;
                }
            }
            ExploreReward::GandalfToInfluence => {
                if let Some(to) = location("location") {
                    self.place_character(ActorRef::Hero(hero_id), GANDALF, &to, data)?;
                    self.clear_monsters(&[to], data)?;
                }
            }
            ExploreReward::GandalfFarAway { .. } => {
                if let Some(to) = location("location") {
                    self.place_character(ActorRef::Hero(hero_id), GANDALF, &to, data)?;
                }
                self.board.shadow_pool = (self.board.shadow_pool - 1).max(0);
            }
            ExploreReward::FavorAndRemoveMonster { count } => {
                self.hero_mut(hero_id)?.gain_favor(count, data);
                self.remove_token_targets(targets, data)?;
            }
            reward => self.apply_simple_reward(hero_id, &reward, data)?,
        }
        Ok(())
    }

    /// Rewards without targets.
    fn apply_simple_reward(
        &mut self,
        hero_id: i64,
        reward: &ExploreReward,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let location = self.hero(hero_id)?.location.clone();
        match *reward {
            ExploreReward::RemoveAdjacentInfluence => {
                for adjacent in data.paths.connected(&location) {
                    self.remove_influence(adjacent, 1);
                }
                self.trim_unsupported_influence(data);
            }
            ExploreReward::AdvanceHeroes { cost } => {
                self.hero_mut(hero_id)?.favor -= cost;
                self.board.story.heroes += 1;
            }
            ExploreReward::RetreatCorruption { cost } => {
                self.hero_mut(hero_id)?.favor -= cost;
                self.board.story.corruption = (self.board.story.corruption - 1).max(0);
            }
            ExploreReward::GainFortitude => {
                self.hero_mut(hero_id)?.gain_level(Attribute::Fortitude);
            }
            ExploreReward::Favor { count } => {
                self.hero_mut(hero_id)?.gain_favor(count, data);
            }
            ExploreReward::Boat => self.hero_mut(hero_id)?.items.boat = true,
            ExploreReward::ShadowPoolLoss { count } => {
                self.board.shadow_pool = (self.board.shadow_pool - count).max(0);
            }
            ExploreReward::DamageToRest => {
                let hero = self.hero_mut(hero_id)?;
                let damage = std::mem::take(&mut hero.damage_pool);
                hero.rest_pool.extend(damage);
            }
            ExploreReward::FavorAndClearMonsters { count } => {
                self.hero_mut(hero_id)?.gain_favor(count, data);
                self.clear_monsters(&area_locations(&Area::Near(location), data), data)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Discards every monster token on `locations`.
    fn clear_monsters(&mut self, locations: &[String], data: &GameData) -> Result<(), EngineError> {
        let monsters = self.mob_targets(locations, MobKind::Monster);
        self.remove_token_targets(&monsters, data)
    }

    fn log_event(
        &mut self,
        hero_id: i64,
        event: EventRef,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let name = data.event(event.stage, event.card)?.name.clone();
        self.log(
            ActorRef::Hero(hero_id),
            "event.activated",
            &[("event", json!(name))],
        );
        Ok(())
    }

    /// "Trust in Friendship": its holder exploring a location with a character or
    /// another hero gains the favor on the card, which is then discarded.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub fn trigger_held_event(&mut self, hero_id: i64, data: &GameData) -> Result<(), EngineError> {
        let Some(held) = self.board.held_event.filter(|h| h.hero_id == hero_id) else {
            return Ok(());
        };
        let location = self.hero(hero_id)?.location.clone();
        let character = self.board.characters.values().any(|l| *l == location);
        let other_hero = self
            .heroes
            .iter()
            .any(|h| h.id != Some(hero_id) && h.location == location);
        if !character && !other_hero {
            return Ok(());
        }
        self.board.held_event = None;
        self.hero_mut(hero_id)?.gain_favor(held.favor, data);
        self.discard_event(held.event);
        self.log_event(hero_id, held.event, data)
    }
}

/// The first value of the first target of a kind.
fn first_value<'a>(targets: &'a [String], kind: &str) -> Option<&'a str> {
    targets.iter().find_map(|t| {
        let (k, values) = parse_target(t);
        (k == kind).then(|| values.first().copied()).flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};

    fn exploring(location: &str, event: EventRef) -> Game {
        let mut g = game();
        g.board.state = BoardState::HeroExploration;
        g.heroes[0].location = location.to_owned();
        g.board.current_event = Some(event);
        g
    }

    // § 5.6: a hero exploring the affected location activates the card, which is then
    // discarded; declining leaves it in play.
    #[test]
    fn exploring_hero_activates_the_card_in_play() {
        // I/5 "Gandalf Visits Frodo": spend 3 favor, hero marker +1.
        let event = EventRef { stage: 1, card: 5 };
        let mut g = exploring("the_shire", event);
        g.heroes[0].favor = 3;
        g.activate_current_event(1, data()).unwrap();
        assert_eq!(g.board.state, BoardState::ChoiceScreenBoardChoices);
        g.answer_choice(PASS, &[], data(), &mut rng()).unwrap();
        assert_eq!(g.board.current_event, Some(event));
        assert_eq!(g.board.state, BoardState::HeroExploration);

        g.activate_current_event(1, data()).unwrap();
        g.answer_choice("advance_heroes", &[], data(), &mut rng())
            .unwrap();
        assert_eq!(g.board.story.heroes, 1);
        assert_eq!(g.heroes[0].favor, 0);
        assert_eq!(g.board.current_event, None);
        assert_eq!(g.board.decks.event.discard, vec![5]);
    }

    // § 5.6: story cards are just discarded when explored.
    #[test]
    fn story_card_is_discarded_when_explored() {
        let event = EventRef { stage: 2, card: 13 };
        let mut g = exploring("near_harad", event);
        g.activate_current_event(1, data()).unwrap();
        assert_eq!(g.board.current_event, None);
        assert!(g.activate_current_event(1, data()).is_err());
    }

    // "Trust in Friendship": the holder gets the favor back next to a character.
    #[test]
    fn held_card_gives_its_favor() {
        let mut g = game();
        g.board.held_event = Some(crate::model::board::HeldEvent {
            event: EventRef { stage: 1, card: 3 },
            hero_id: 1,
            favor: 2,
        });
        g.trigger_held_event(1, data()).unwrap();
        assert_eq!(g.heroes[0].favor, 0);
        let location = g.heroes[0].location.clone();
        g.board.characters.insert("gandalf".to_owned(), location);
        g.trigger_held_event(1, data()).unwrap();
        assert_eq!(g.heroes[0].favor, 2);
        assert!(g.board.held_event.is_none());
    }
}
