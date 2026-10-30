//! A hero exploring its location: characters, favors, plots and the Dark Path (§ 6.5).

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::engine::hero::training::Attribute;
use crate::engine::tokens::{Token, TokenKind, tokens_by_location};
use crate::game_data::GameData;
use crate::game_data::character::{CharacterAbility, CharacterAttribute};
use crate::game_data::corruption_card::Flaw;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::Resume;
use crate::model::game::Game;

/// Most Corruption cards a hero may hold to choose a Dark Path.
const DARK_PATH_MAX_CORRUPTION: usize = 3;

/// Hero gaining 1 favor once per turn when consulting a character (Noble Bloodline).
const NOBLE_BLOODLINE_HERO: &str = "eleanor";

/// What the hero chose to explore.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExploreChoice {
    /// Characters consulted for their favor.
    pub characters: Vec<String>,
    /// Characters consulted for their ability.
    pub character_abilities: Vec<String>,
    /// Number of favor tokens to take.
    pub favors: usize,
    /// Remove the plot of the location.
    pub plot: bool,
    /// Choose a Dark Path.
    pub dark_path: bool,
    /// Activate the event card in play on the location.
    pub event: bool,
}

/// Message to show after exploring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExploreNotice {
    /// A character was consulted.
    CharacterEncountered,
    /// The hero chose a Dark Path.
    DarkPath,
    /// Favor was taken.
    FavorTaken,
    /// A corruption card (Despairing, Mistrusted) stopped the hero taking more favor.
    FavorLimit,
    /// A plot was removed.
    PlotRemoved,
    /// The event card in play was activated.
    EventActivated,
}

impl Game {
    /// Tokens the hero may explore. An isolated hero can't meet characters; each
    /// hidden character is logged.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub fn exploration_tokens(
        &mut self,
        hero_id: i64,
        data: &GameData,
    ) -> Result<Vec<Token>, EngineError> {
        let hero = self.hero(hero_id)?;
        let isolated = hero.has_flaw(Flaw::Isolated, data);
        let mut tokens = tokens_by_location(self, data)
            .remove(&hero.location)
            .unwrap_or_default();
        if isolated {
            let hidden: Vec<String> = tokens
                .iter()
                .filter(|t| t.kind == TokenKind::Character)
                .map(|t| t.name.clone())
                .collect();
            tokens.retain(|t| t.kind != TokenKind::Character);
            for name in hidden {
                self.log(
                    ActorRef::Hero(hero_id),
                    "corruption.isolated",
                    &[("character", json!(name))],
                );
            }
        }
        Ok(tokens)
    }

    /// Resolves the hero's exploration choices. Choices asking a decision (training,
    /// a "Wretched" card, the event in play) come last; only one may wait at a time.
    ///
    /// # Returns
    /// The notice of the last resolved choice, if any.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when a chosen token isn't on the location, the plot
    /// costs more favor than the hero has, a Dark Path isn't allowed, or two decisions
    /// would wait at once.
    pub fn explore(
        &mut self,
        hero_id: i64,
        choice: &ExploreChoice,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Option<ExploreNotice>, EngineError> {
        self.trigger_held_event(hero_id, data)?;
        let mut notice = None;
        if choice.favors > 0 {
            notice = Some(self.pick_favors(hero_id, choice.favors, data)?);
        }
        if choice.plot {
            self.remove_plot_at_hero_location(hero_id, data)?;
            notice = Some(ExploreNotice::PlotRemoved);
        }
        let consulted = choice.characters.iter().map(|c| (c, false));
        for (character, ability) in
            consulted.chain(choice.character_abilities.iter().map(|c| (c, true)))
        {
            self.one_decision_at_a_time()?;
            self.consult_character(hero_id, character, ability, data, rng)?;
            notice = Some(ExploreNotice::CharacterEncountered);
        }
        if choice.dark_path {
            self.one_decision_at_a_time()?;
            self.choose_dark_path(hero_id, data, rng)?;
            notice = Some(ExploreNotice::DarkPath);
        }
        if choice.event {
            self.one_decision_at_a_time()?;
            self.activate_current_event(hero_id, data)?;
            notice = Some(ExploreNotice::EventActivated);
        }
        Ok(notice)
    }

    fn one_decision_at_a_time(&self) -> Result<(), EngineError> {
        if self.board.pending_choice.is_some() {
            return Err(EngineError::Refused(
                "answer the pending decision before exploring further".to_owned(),
            ));
        }
        Ok(())
    }

    /// Consults a character on the hero's location for its favor or its ability; the
    /// character goes back to the Favor and Character area (§ 6.5). An isolated hero
    /// can't consult characters; Eleanor gains 1 favor once per turn.
    fn consult_character(
        &mut self,
        hero_id: i64,
        character: &str,
        ability: bool,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let hero = self.hero(hero_id)?;
        let location = hero.location.clone();
        if hero.has_flaw(Flaw::Isolated, data) {
            return Err(EngineError::Refused(
                "an isolated hero can't consult".to_owned(),
            ));
        }
        if self.board.characters.get(character) != Some(&location) {
            return Err(EngineError::Refused(format!("{character} is not here")));
        }
        let def = data.character(character)?;
        self.board.characters.remove(character);
        let params = [
            ("location_name", json!(data.locations.name(&location))),
            ("character_name", json!(def.name)),
            ("count", json!(def.favor)),
        ];
        if ability {
            self.log(
                ActorRef::Hero(hero_id),
                "exploration.character_ability",
                &params,
            );
        } else {
            self.log(
                ActorRef::Hero(hero_id),
                "exploration.encounter_character",
                &params,
            );
        }
        self.noble_bloodline(hero_id, data)?;
        if ability {
            self.use_character_ability(hero_id, def.ability, rng)
        } else {
            self.hero_mut(hero_id)?.gain_favor(def.favor, data);
            Ok(())
        }
    }

    /// Eleanor's Noble Bloodline: once per turn, 1 favor when consulting a character
    /// or completing a quest.
    fn noble_bloodline(&mut self, hero_id: i64, data: &GameData) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        if hero.code != NOBLE_BLOODLINE_HERO || hero.turn_record.noble_bloodline {
            return Ok(());
        }
        hero.turn_record.noble_bloodline = true;
        hero.gain_favor(1, data);
        Ok(())
    }

    fn use_character_ability(
        &mut self,
        hero_id: i64,
        ability: CharacterAbility,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        match ability {
            CharacterAbility::ShadowPoolLoss { count } => {
                self.board.shadow_pool = (self.board.shadow_pool - count).max(0);
            }
            CharacterAbility::Horse => hero.items.horse = true,
            CharacterAbility::Level { attribute } => {
                hero.gain_level(match attribute {
                    CharacterAttribute::Strength => Attribute::Strength,
                    CharacterAttribute::Agility => Attribute::Agility,
                    CharacterAttribute::Wisdom => Attribute::Wisdom,
                });
            }
            CharacterAbility::AdvanceHeroes { cost } => {
                if hero.favor < cost {
                    return Err(EngineError::Refused("not enough favor".to_owned()));
                }
                hero.favor -= cost;
                self.board.story.heroes += 1;
            }
            CharacterAbility::Training { times } => {
                self.train_times(hero_id, times, Resume::Exploration, rng)?;
            }
            CharacterAbility::DiscardShadows { count } => {
                let hand = &mut self.sauron.shadow_hand;
                hand.shuffle(rng);
                let discarded = hand.split_off(hand.len().saturating_sub(count));
                self.board.decks.shadow.discard.extend(discarded);
            }
        }
        Ok(())
    }

    /// Dark Path (§ 6.5): once per turn, a hero with at most 3 Corruption cards gains
    /// 1 favor and 1 Corruption card.
    fn choose_dark_path(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        if hero.turn_record.dark_path {
            return Err(EngineError::Refused("one Dark Path per turn".to_owned()));
        }
        if hero.corruption_cards.len() > DARK_PATH_MAX_CORRUPTION {
            return Err(EngineError::Refused(
                "too corrupted for a Dark Path".to_owned(),
            ));
        }
        hero.turn_record.dark_path = true;
        hero.gain_favor(1, data);
        self.log(ActorRef::Hero(hero_id), "exploration.dark_path", &[]);
        self.peril_corruption(hero_id, Resume::Exploration, data, rng)?;
        Ok(())
    }

    /// Tells whether the hero may choose a Dark Path now.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub fn dark_path_allowed(&self, hero_id: i64) -> Result<bool, EngineError> {
        let hero = self.hero(hero_id)?;
        Ok(!hero.turn_record.dark_path && hero.corruption_cards.len() <= DARK_PATH_MAX_CORRUPTION)
    }

    fn pick_favors(
        &mut self,
        hero_id: i64,
        count: usize,
        data: &GameData,
    ) -> Result<ExploreNotice, EngineError> {
        let location = self.hero(hero_id)?.location.clone();
        for _ in 0..count {
            let position = self
                .board
                .favors
                .iter()
                .position(|f| *f == location)
                .ok_or_else(|| EngineError::Refused("no favor here".to_owned()))?;
            if self.hero_mut(hero_id)?.gain_favor(1, data) == 0 {
                self.log(ActorRef::Hero(hero_id), "corruption.favor_limit", &[]);
                return Ok(ExploreNotice::FavorLimit);
            }
            self.board.favors.remove(position);
            let params = [("location_name", json!(data.locations.name(&location)))];
            self.log(ActorRef::Hero(hero_id), "exploration.get_favor", &params);
        }
        Ok(ExploreNotice::FavorTaken)
    }

    fn remove_plot_at_hero_location(
        &mut self,
        hero_id: i64,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let location = self.hero(hero_id)?.location.clone();
        let found = self
            .board
            .plots
            .iter()
            .find(|(_, card)| {
                data.plot(**card)
                    .is_ok_and(|p| p.affect.as_deref() == Some(location.as_str()))
            })
            .map(|(slot, card)| (*slot, *card));
        let (slot, card) = found.ok_or_else(|| EngineError::Refused("no plot here".to_owned()))?;
        let cost = data.plot(card)?.favor_to_discard;
        let hero = self.hero_mut(hero_id)?;
        if cost > hero.favor {
            return Err(EngineError::Refused("not enough favor".to_owned()));
        }
        hero.favor -= cost;
        self.board.plots.remove(&slot);
        self.board.decks.plot.discard.push(card);
        let params = [("location_name", json!(data.locations.name(&location)))];
        self.log(ActorRef::Hero(hero_id), "exploration.plot.remove", &params);
        Ok(())
    }

    /// The hero leaves exploration to move again.
    ///
    /// # Errors
    /// [`EngineError::InvalidTransition`] outside exploration.
    pub fn exploration_next_movement(&mut self) -> Result<(), EngineError> {
        self.fire(BoardEvent::NextToHeroMovementScreen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, hero, rng};

    fn exploring_with(character: &str) -> Game {
        let mut g = game();
        g.board.state = BoardState::HeroExploration;
        g.heroes[0].location = "bree".to_owned();
        g.board
            .characters
            .insert(character.to_owned(), "bree".to_owned());
        g
    }

    fn consult(g: &mut Game, character: &str, ability: bool) -> Result<(), EngineError> {
        let choice = if ability {
            ExploreChoice {
                character_abilities: vec![character.to_owned()],
                ..ExploreChoice::default()
            }
        } else {
            ExploreChoice {
                characters: vec![character.to_owned()],
                ..ExploreChoice::default()
            }
        };
        g.explore(1, &choice, data(), &mut rng()).map(|_| ())
    }

    // § 6.5: a character gives its favor or its ability, then leaves the location.
    #[test]
    fn character_gives_favor_or_ability() {
        let mut g = exploring_with("gandalf");
        let wisdom = g.heroes[0].wisdom;
        consult(&mut g, "gandalf", true).unwrap();
        assert_eq!(g.heroes[0].wisdom, wisdom + 1);
        assert_eq!(g.heroes[0].favor, 0);
        assert!(g.board.characters.is_empty());

        let mut g = exploring_with("gandalf");
        consult(&mut g, "gandalf", false).unwrap();
        assert_eq!(g.heroes[0].favor, 2);
        assert_eq!(g.heroes[0].wisdom, wisdom);
    }

    // Dain II: train twice, one Skill card kept after the other.
    #[test]
    fn dain_trains_twice() {
        let mut g = exploring_with("dain");
        consult(&mut g, "dain", true).unwrap();
        g.answer_choice("keep_skill", &["skill:100".to_owned()], data(), &mut rng())
            .unwrap();
        assert!(g.board.pending_choice.is_some());
        g.answer_choice("keep_skill", &["skill:102".to_owned()], data(), &mut rng())
            .unwrap();
        assert_eq!(g.heroes[0].hand[g.heroes[0].hand.len() - 2..], [100, 102]);
        assert_eq!(g.board.state, BoardState::HeroExploration);
    }

    // Isolated: no consulting characters.
    #[test]
    fn isolated_hero_cannot_consult() {
        let mut g = exploring_with("gandalf");
        g.heroes[0].corruption_cards = vec![11];
        assert!(consult(&mut g, "gandalf", false).is_err());
    }

    // Eleanor's Noble Bloodline: 1 more favor, once per turn.
    #[test]
    fn noble_bloodline_once_per_turn() {
        let mut g = exploring_with("gandalf");
        g.heroes[0] = hero("eleanor");
        g.heroes[0].location = "bree".to_owned();
        g.board
            .characters
            .insert("aragorn".to_owned(), "bree".to_owned());
        consult(&mut g, "gandalf", false).unwrap();
        consult(&mut g, "aragorn", false).unwrap();
        assert_eq!(g.heroes[0].favor, 5);
    }

    // § 6.5 Dark Path: 1 favor and 1 Corruption card, once per turn, with at most 3
    // Corruption cards.
    #[test]
    fn dark_path_once_per_turn() {
        let mut g = exploring_with("gandalf");
        g.board.decks.corruption.draw = vec![14, 2];
        let dark = ExploreChoice {
            dark_path: true,
            ..ExploreChoice::default()
        };
        g.explore(1, &dark, data(), &mut rng()).unwrap();
        assert_eq!(g.heroes[0].favor, 1);
        assert_eq!(g.heroes[0].corruption_cards, vec![14]);
        assert!(g.explore(1, &dark, data(), &mut rng()).is_err());

        let mut g = exploring_with("gandalf");
        g.heroes[0].corruption_cards = vec![2, 3, 5, 8];
        assert!(!g.dark_path_allowed(1).unwrap());
        assert!(g.explore(1, &dark, data(), &mut rng()).is_err());
    }

    // Mistrusted: at most 1 favor gained per turn.
    #[test]
    fn mistrusted_hero_gains_one_favor_per_turn() {
        let mut g = exploring_with("gandalf");
        g.heroes[0].corruption_cards = vec![12];
        consult(&mut g, "gandalf", false).unwrap();
        assert_eq!(g.heroes[0].favor, 1);
    }
}
