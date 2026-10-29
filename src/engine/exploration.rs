//! A hero exploring its location: characters, favors and plots.

use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::engine::tokens::{Token, TokenKind, tokens_by_location};
use crate::game_data::GameData;
use crate::game_data::corruption_card::Flaw;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;

/// Favor gained when meeting a character.
const CHARACTER_FAVOR: i32 = 2;

/// Favor above which a dispairing hero can't take more.
const DISPAIRING_FAVOR_LIMIT: i32 = 3;

/// What the hero chose to explore.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExploreChoice {
    /// Characters to meet.
    pub characters: Vec<String>,
    /// Number of favor tokens to take.
    pub favors: usize,
    /// Remove the plot of the location.
    pub plot: bool,
}

/// Message to show after exploring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExploreNotice {
    /// A character was met.
    CharacterEncountered,
    /// Favor was taken.
    FavorTaken,
    /// A dispairing hero couldn't take more favor.
    Dispairing,
    /// A plot was removed.
    PlotRemoved,
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

    /// Resolves the hero's exploration choices.
    ///
    /// # Returns
    /// The notice of the last resolved choice, if any.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when a chosen token isn't on the location or the
    /// plot costs more favor than the hero has.
    pub fn explore(
        &mut self,
        hero_id: i64,
        choice: &ExploreChoice,
        data: &GameData,
    ) -> Result<Option<ExploreNotice>, EngineError> {
        let mut notice = None;
        for character in &choice.characters {
            self.meet_character(hero_id, character, data)?;
            notice = Some(ExploreNotice::CharacterEncountered);
        }
        if choice.favors > 0 {
            notice = Some(self.pick_favors(hero_id, choice.favors, data)?);
        }
        if choice.plot {
            self.remove_plot_at_hero_location(hero_id, data)?;
            notice = Some(ExploreNotice::PlotRemoved);
        }
        Ok(notice)
    }

    fn meet_character(
        &mut self,
        hero_id: i64,
        character: &str,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let location = self.hero(hero_id)?.location.clone();
        if self.board.characters.get(character) != Some(&location) {
            return Err(EngineError::Refused(format!("{character} is not here")));
        }
        self.hero_mut(hero_id)?.favor += CHARACTER_FAVOR;
        self.board.characters.remove(character);
        let params = [
            ("location_name", json!(data.locations.name(&location))),
            ("character_name", json!(character)),
        ];
        self.log(
            ActorRef::Hero(hero_id),
            "exploration.encounter_character",
            &params,
        );
        Ok(())
    }

    fn pick_favors(
        &mut self,
        hero_id: i64,
        count: usize,
        data: &GameData,
    ) -> Result<ExploreNotice, EngineError> {
        let dispairing = self.hero(hero_id)?.has_flaw(Flaw::Dispairing, data);
        let location = self.hero(hero_id)?.location.clone();
        for _ in 0..count {
            if dispairing && self.hero(hero_id)?.favor >= DISPAIRING_FAVOR_LIMIT {
                self.log(ActorRef::Hero(hero_id), "corruption.dispairing", &[]);
                return Ok(ExploreNotice::Dispairing);
            }
            let position = self
                .board
                .favors
                .iter()
                .position(|f| *f == location)
                .ok_or_else(|| EngineError::Refused("no favor here".to_owned()))?;
            self.board.favors.remove(position);
            self.hero_mut(hero_id)?.favor += 1;
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
