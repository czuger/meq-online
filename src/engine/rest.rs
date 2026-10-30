//! Start of a hero's turn: drawing, resting and healing.

use rand::rngs::StdRng;

use crate::engine::board_state::BoardEvent;
use crate::engine::encounters::Encounter;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::corruption_card::CorruptionCode;
use crate::game_data::plot::StoryType;
use crate::model::game::Game;

/// How a hero rests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rest {
    /// Heal as well (in a Haven only).
    pub heal: bool,
    /// Corruption card to discard.
    pub discard: Option<CorruptionCode>,
}

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
        if self.check_location_encounters(hero_id, true, data, rng)? == Encounter::Monster {
            return self.fire(BoardEvent::NextToCombatSetupScreenBoardCombats);
        }
        self.fire(BoardEvent::NextToHeroRestScreen)?;
        let location = &self.hero(hero_id)?.location;
        if data.locations.get(location)?.shadow_fortress {
            self.fire(BoardEvent::NextToHeroMovementScreen)?;
        }
        Ok(())
    }

    /// The hero rests, or heals when `heal` is set and it stands in a haven, and may
    /// discard one corruption card; then the lowest Sauron marker advances (§ 6.1).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when healing outside a haven or the corruption card
    /// can't be paid; transition errors.
    pub fn rest_hero(
        &mut self,
        hero_id: i64,
        rest: Rest,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<RestOutcome, EngineError> {
        if let Some(card) = rest.discard {
            self.discard_corruption_card(hero_id, card, data)?;
        }
        let heal = rest.heal;
        let hero = self.hero_mut(hero_id)?;
        if heal {
            if !data.locations.get(&hero.location)?.haven {
                return Err(EngineError::Refused(
                    "healing is only possible in a haven".to_owned(),
                ));
            }
            hero.heal(rng);
        } else {
            hero.rest(rng);
        }
        if self.board.story.advance_lowest() {
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

    /// The hero picks the tied story marker to advance after resting (§ 6.1).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when `marker` isn't one of the lowest; transition errors.
    pub fn advance_chosen_story_marker(&mut self, marker: StoryType) -> Result<(), EngineError> {
        if !self.board.story.lowest_sauron_markers().contains(&marker) {
            return Err(EngineError::Refused(
                "only one of the lowest markers may advance".to_owned(),
            ));
        }
        self.board.story.advance(marker, 1);
        self.fire(BoardEvent::NextToHeroMovementScreen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};

    // § 6.1: the lowest Sauron marker advances; on a tie the hero chooses.
    #[test]
    fn rest_advances_the_lowest_marker_or_asks() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        g.board.story.ring = 1;
        g.board.story.conquest = 1;
        assert_eq!(
            g.rest_hero(1, Rest::default(), data(), &mut rng()).unwrap(),
            RestOutcome::Movement
        );
        assert_eq!(g.board.story.corruption, 1);

        g.board.state = BoardState::HeroRestScreen;
        assert_eq!(
            g.rest_hero(1, Rest::default(), data(), &mut rng()).unwrap(),
            RestOutcome::ChooseStoryMarker
        );
        assert!(g.advance_chosen_story_marker(StoryType::Ring).is_ok());
        assert_eq!(g.board.story.ring, 2);
    }

    #[test]
    fn only_a_lowest_marker_may_be_chosen() {
        let mut g = game();
        g.board.state = BoardState::AfterRestAdvanceStoryMarker;
        g.board.story.ring = 1;
        assert!(g.advance_chosen_story_marker(StoryType::Ring).is_err());
    }

    // § 6.1: when resting, a corruption card is discarded by spending its favor; its
    // malus ends.
    #[test]
    fn resting_hero_pays_to_discard_a_corruption_card() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        g.board.decks.corruption.draw = vec![14];
        g.gain_random_corruption(1, data(), &mut rng()).unwrap();
        let strength = g.heroes[0].strength;
        let rest = Rest {
            heal: false,
            discard: Some(14),
        };
        g.heroes[0].favor = 1;
        assert!(g.rest_hero(1, rest, data(), &mut rng()).is_err());
        g.heroes[0].favor = 3;
        g.rest_hero(1, rest, data(), &mut rng()).unwrap();
        let hero = &g.heroes[0];
        assert_eq!(hero.favor, 1);
        assert_eq!(hero.strength, strength + 1);
        assert!(hero.corruption_cards.is_empty());
        assert_eq!(g.board.decks.corruption.discard, vec![14]);
    }
}
