//! A defeated hero (§ 8.9).

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::engine::shadows::shadow_window::ShadowWindow;
use crate::engine::turn::TurnEnd;
use crate::game_data::GameData;
use crate::game_data::corruption_card::Flaw;
use crate::game_data::plot::StoryType;
use crate::game_data::shadow::ShadowTiming;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::Resume;
use crate::model::game::Game;
use crate::model::hero::{Hero, Item};

/// What a defeated hero gives up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefeatLoss {
    /// One favor token.
    Favor,
    /// One Item card.
    Item(Item),
}

/// The defeated hero's decisions; `None` where the rules leave no choice.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DefeatChoice {
    /// Story marker to advance when several are tied for the lowest.
    pub marker: Option<StoryType>,
    /// What to lose when the hero has several options.
    pub loss: Option<DefeatLoss>,
}

/// What follows the hero's defeat choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefeatOutcome {
    /// Several Havens are as close: Sauron picks one.
    ChooseHaven,
    /// The hero recovered in a Haven and its turn ended.
    TurnEnded(TurnEnd),
}

/// What a hero may lose to a defeat: one favor, or one of its Items.
pub fn defeat_losses(hero: &Hero) -> Vec<DefeatLoss> {
    let favor = (hero.favor > 0).then_some(DefeatLoss::Favor);
    favor
        .into_iter()
        .chain(hero.items.held().into_iter().map(DefeatLoss::Item))
        .collect()
}

/// Havens at the fewest locations from `location` (§ 8.9).
pub fn closest_havens<'d>(location: &str, data: &'d GameData) -> Vec<&'d str> {
    let distances = data.paths.distances_from(location);
    let havens: Vec<(&str, u32)> = data
        .locations
        .iter()
        .filter(|(_, l)| l.haven)
        .filter_map(|(code, _)| distances.get(code.as_str()).map(|d| (code.as_str(), *d)))
        .collect();
    let nearest = havens.iter().map(|(_, d)| *d).min();
    havens
        .into_iter()
        .filter(|(_, d)| Some(*d) == nearest)
        .map(|(code, _)| code)
        .collect()
}

impl Game {
    /// A hero is defeated, in combat or not: the defeat screen opens when he has
    /// decisions to make, otherwise the defeat resolves at once.
    ///
    /// # Returns
    /// `None` when the hero must choose on the defeat screen.
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn start_hero_defeat(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Option<DefeatOutcome>, EngineError> {
        self.fire(BoardEvent::NextToHeroDefeatScreen)?;
        self.activate_current_hero()?;
        if self.defeat_needs_choice(hero_id)? {
            return Ok(None);
        }
        self.resolve_hero_defeat(hero_id, DefeatChoice::default(), data, rng)
            .map(Some)
    }

    /// Spaces the lowest Sauron marker advances when the hero rests or is defeated:
    /// 2 for a despondent hero, 1 otherwise.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub fn story_penalty(&self, hero_id: i64, data: &GameData) -> Result<i32, EngineError> {
        let despondent = self.hero(hero_id)?.has_flaw(Flaw::Despondent, data);
        Ok(if despondent { 2 } else { 1 })
    }

    /// Tells whether the defeated hero has a decision to make: tied story markers, or
    /// several things it could lose.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub fn defeat_needs_choice(&self, hero_id: i64) -> Result<bool, EngineError> {
        let tied = self.board.story.lowest_sauron_markers().len() > 1;
        Ok(tied || defeat_losses(self.hero(hero_id)?).len() > 1)
    }

    /// Resolves the steps of a defeat: advance the lowest Sauron marker, lose a favor
    /// or an Item, then go to the closest Haven, recover and end the turn.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when a needed choice is missing or not allowed;
    /// lookup and transition errors.
    pub fn resolve_hero_defeat(
        &mut self,
        hero_id: i64,
        choice: DefeatChoice,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<DefeatOutcome, EngineError> {
        let marker = self.defeat_marker(choice.marker)?;
        let loss = self.defeat_loss(hero_id, choice.loss)?;
        let spaces = self.story_penalty(hero_id, data)?;
        self.board.story.advance(marker, spaces);
        self.log(
            ActorRef::Hero(hero_id),
            "story.advance",
            &[("marker_name", json!(marker)), ("count", json!(spaces))],
        );
        if let Some(loss) = loss {
            self.lose_to_defeat(hero_id, loss)?;
        }

        let location = self.hero(hero_id)?.location.clone();
        match closest_havens(&location, data).as_slice() {
            [haven] => Ok(DefeatOutcome::TurnEnded(
                self.send_to_haven(hero_id, haven, data, rng)?,
            )),
            _ => {
                self.fire(BoardEvent::NextToSauronHavenScreen)?;
                self.sauron.active = true;
                Ok(DefeatOutcome::ChooseHaven)
            }
        }
    }

    /// Sauron sends the defeated current hero to one of the closest Havens.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when `haven` isn't one of the closest Havens; lookup
    /// and transition errors.
    pub fn choose_defeat_haven(
        &mut self,
        haven: &str,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<TurnEnd, EngineError> {
        let hero_id = self.current_hero_id()?;
        let location = self.hero(hero_id)?.location.clone();
        let haven = closest_havens(&location, data)
            .into_iter()
            .find(|h| *h == haven)
            .ok_or_else(|| EngineError::Refused(format!("{haven} is not a closest haven")))?;
        self.sauron.active = false;
        self.send_to_haven(hero_id, haven, data, rng)
    }

    fn defeat_marker(&self, chosen: Option<StoryType>) -> Result<StoryType, EngineError> {
        let lowest = self.board.story.lowest_sauron_markers();
        match (lowest.as_slice(), chosen) {
            ([only], _) => Ok(*only),
            (_, Some(marker)) if lowest.contains(&marker) => Ok(marker),
            _ => Err(EngineError::Refused(
                "choose one of the lowest story markers".to_owned(),
            )),
        }
    }

    fn defeat_loss(
        &self,
        hero_id: i64,
        chosen: Option<DefeatLoss>,
    ) -> Result<Option<DefeatLoss>, EngineError> {
        let losses = defeat_losses(self.hero(hero_id)?);
        match (losses.as_slice(), chosen) {
            ([], _) => Ok(None),
            ([only], _) => Ok(Some(*only)),
            (_, Some(loss)) if losses.contains(&loss) => Ok(Some(loss)),
            _ => Err(EngineError::Refused(
                "choose between a favor and an item".to_owned(),
            )),
        }
    }

    fn lose_to_defeat(&mut self, hero_id: i64, loss: DefeatLoss) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        match loss {
            DefeatLoss::Favor => {
                hero.favor_loss(1);
                self.log(ActorRef::Hero(hero_id), "defeat.lose_favor", &[]);
            }
            DefeatLoss::Item(item) => {
                hero.items.remove(item);
                let params = [("item", json!(item))];
                self.log(ActorRef::Hero(hero_id), "defeat.lose_item", &params);
            }
        }
        Ok(())
    }

    /// The defeated hero recovers in the Haven; Sauron may then play a Shadow card
    /// (§ 5.9) before the hero's turn ends.
    fn send_to_haven(
        &mut self,
        hero_id: i64,
        haven: &str,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<TurnEnd, EngineError> {
        let hero = self.hero_mut(hero_id)?;
        hero.location = haven.to_owned();
        hero.heal(rng);
        let params = [("location", json!(data.locations.name(haven)))];
        self.log(ActorRef::Hero(hero_id), "defeat.haven", &params);
        let window = ShadowWindow {
            timing: ShadowTiming::HeroDefeated,
            hero_id,
        };
        if self.open_shadow_window(window, Resume::EndTurn, data)? {
            return Ok(TurnEnd::Boards);
        }
        self.finish_heroes_turn(hero_id, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};

    fn defeated(location: &str) -> Game {
        let mut g = game();
        g.board.state = BoardState::HeroDefeatScreen;
        g.board.story.ring = 1;
        g.board.story.conquest = 1;
        g.heroes[0].location = location.to_owned();
        g.heroes[0].turn = 2;
        g
    }

    // § 8.9 step 3: the closest Haven by number of locations.
    #[test]
    fn closest_haven_by_number_of_locations() {
        assert_eq!(closest_havens("rivendell", data()), vec!["rivendell"]);
        assert_eq!(closest_havens("bree", data()), vec!["fornost"]);
    }

    // § 8.9: lowest marker +1, one favor lost, healed in the closest Haven.
    #[test]
    fn defeat_without_choice_resolves_at_once() {
        let mut g = defeated("bree");
        g.heroes[0].favor = 2;
        g.heroes[0].damage_pool = vec![1, 2];
        assert!(!g.defeat_needs_choice(1).unwrap());
        g.resolve_hero_defeat(1, DefeatChoice::default(), data(), &mut rng())
            .unwrap();
        assert_eq!(g.board.story.corruption, 1);
        assert_eq!(g.board.story.ring, 1);
        let hero = &g.heroes[0];
        assert_eq!(hero.favor, 1);
        assert_eq!(hero.location, "fornost");
        assert!(hero.damage_pool.is_empty());
    }

    // § 8.9 steps 1 and 2: the hero chooses among tied markers and between favor and item.
    #[test]
    fn hero_chooses_marker_and_loss() {
        let mut g = defeated("bree");
        g.board.story.ring = 0;
        g.heroes[0].favor = 2;
        g.heroes[0].items.horse = true;
        assert!(g.defeat_needs_choice(1).unwrap());
        let refused = g.resolve_hero_defeat(1, DefeatChoice::default(), data(), &mut rng());
        assert!(refused.is_err());
        let choice = DefeatChoice {
            marker: Some(StoryType::Corruption),
            loss: Some(DefeatLoss::Item(Item::Horse)),
        };
        g.resolve_hero_defeat(1, choice, data(), &mut rng())
            .unwrap();
        assert_eq!(g.board.story.corruption, 1);
        assert_eq!(g.heroes[0].favor, 2);
        assert!(!g.heroes[0].items.horse);
    }

    // § 8.9 step 3: Sauron breaks ties between equally close Havens.
    #[test]
    fn sauron_breaks_haven_ties() {
        let tied = data()
            .locations
            .iter()
            .map(|(code, _)| code.as_str())
            .find(|code| closest_havens(code, data()).len() > 1)
            .expect("a location equally close to two havens");
        let havens = closest_havens(tied, data());
        let mut g = defeated(tied);
        let outcome = g
            .resolve_hero_defeat(1, DefeatChoice::default(), data(), &mut rng())
            .unwrap();
        assert_eq!(outcome, DefeatOutcome::ChooseHaven);
        assert_eq!(g.board.state, BoardState::SauronHavenScreen);
        assert!(
            g.choose_defeat_haven("the_shire", data(), &mut rng())
                .is_err()
        );
        g.choose_defeat_haven(havens[1], data(), &mut rng())
            .unwrap();
        assert_eq!(g.heroes[0].location, havens[1]);
    }
}
