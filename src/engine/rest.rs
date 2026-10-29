//! Start of a hero's turn: drawing, resting and healing.

use rand::rngs::StdRng;

use crate::engine::board_state::BoardEvent;
use crate::engine::encounters::Encounter;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::plot::StoryType;
use crate::model::game::Game;

/// Where a hero goes after resting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestOutcome {
    /// Sauron's lowest marker advanced; the hero moves.
    Movement,
    /// Several markers are tied: the hero must choose which one advances.
    ChooseStoryMarker,
}

impl Game {
    /// The hero ends its draw; when no hero is drawing anymore, the current hero's
    /// turn starts with a check of its location.
    ///
    /// At the start of a round the first hero in playing order plays. Later in the
    /// round (after the next hero drew) that hero keeps playing.
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn draw_cards_finished(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        self.hero_mut(hero_id)?.active = false;
        if self.heroes_active() {
            return Ok(());
        }
        if self.heroes.iter().any(|h| h.turn_finished) {
            self.activate_current_hero()?;
        } else {
            self.set_first_hero_to_play()?;
        }
        self.check_rest(data, rng)
    }

    /// Before resting, the current hero may meet a monster; shadow fortresses forbid rest.
    fn check_rest(&mut self, data: &GameData, rng: &mut StdRng) -> Result<(), EngineError> {
        let hero_id = self.current_hero_id()?;
        if self.check_location_encounters(hero_id, data, rng)? == Encounter::Monster {
            return self.fire(BoardEvent::NextToCombatSetupScreenBoardCombats);
        }
        self.fire(BoardEvent::NextToHeroRestScreen)?;
        let location = &self.hero(hero_id)?.location;
        if data.locations.get(location)?.shadow_fortress {
            self.fire(BoardEvent::NextToHeroMovementScreen)?;
        }
        Ok(())
    }

    /// The hero rests, or heals when `heal` is set and it stands in a haven.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when healing outside a haven; transition errors.
    pub fn rest_hero(
        &mut self,
        hero_id: i64,
        heal: bool,
        data: &GameData,
    ) -> Result<RestOutcome, EngineError> {
        let hero = self.hero_mut(hero_id)?;
        if heal {
            if !data.locations.get(&hero.location)?.haven {
                return Err(EngineError::Refused(
                    "healing is only possible in a haven".to_owned(),
                ));
            }
            hero.heal();
        } else {
            hero.rest();
        }
        if self.board.story.advance_lowest(true) {
            self.fire(BoardEvent::NextToHeroMovementScreen)?;
            Ok(RestOutcome::Movement)
        } else {
            self.fire(BoardEvent::NextToAfterRestAdvanceStoryMarker)?;
            Ok(RestOutcome::ChooseStoryMarker)
        }
    }

    /// The hero skips resting.
    ///
    /// # Errors
    /// Transition errors.
    pub fn skip_rest(&mut self) -> Result<(), EngineError> {
        self.fire(BoardEvent::NextToHeroMovementScreen)
    }

    /// The hero picks the tied story marker to advance after resting.
    ///
    /// # Errors
    /// Transition errors.
    pub fn advance_chosen_story_marker(&mut self, marker: StoryType) -> Result<(), EngineError> {
        self.board.story.advance(marker, 1);
        self.fire(BoardEvent::NextToHeroMovementScreen)
    }
}
