//! Hero movement along the map paths.

use rand::rngs::StdRng;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::engine::shadows::shadow_window::ShadowWindow;
use crate::engine::threats::Threat;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::corruption_card::Flaw;
use crate::game_data::location_paths::Destination;
use crate::game_data::shadow::ShadowTiming;
use crate::model::choice::Resume;
use crate::model::game::Game;
use crate::model::hero::Hero;
use crate::model::mob::Mob;

/// Travel steps a hopeless hero may take per turn.
const HOPELESS_TRAVELS: u8 = 3;

/// Result of a move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveOutcome {
    /// The selected cards can't pay for the path; nothing changed.
    InvalidCards,
    /// A hopeless hero already took his 3 Travel steps; nothing changed.
    Hopeless,
    /// "Winter Storm": the hero already moved this turn; nothing changed.
    SingleMove,
    /// A combat starts.
    Combat,
    /// Sauron (or the hero, in a Haven) decides what threatens the hero.
    Choice,
    /// The hero explores the new location.
    Exploration,
}

/// Number of cards needed for a path, after the horse bonus.
pub fn path_difficulty(path: &Destination, hero: &Hero) -> u32 {
    if hero.items.horse {
        path.difficulty.saturating_sub(1).max(1)
    } else {
        path.difficulty
    }
}

/// Tells whether the selected cards pay for the path to `destination`.
///
/// One card whose terrain matches the path is enough; otherwise one card per
/// difficulty point is needed.
///
/// # Errors
/// [`EngineError::GameData`] when the path or a card is unknown.
pub fn validate_movement(
    hero: &Hero,
    destination: &str,
    cards: &[CardNumber],
    data: &GameData,
) -> Result<bool, EngineError> {
    let path = data.paths.path(&hero.location, destination)?;
    let mut remaining = path_difficulty(path, hero);
    for card in cards {
        let def = data.hero_card(&hero.code, *card)?;
        if def.movement_type == Some(path.path_type) {
            return Ok(true);
        }
        remaining = remaining.saturating_sub(1);
        if remaining == 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

impl Game {
    /// Moves a hero, paying with cards from its hand, then resolves the monster or
    /// minion attack or the Peril of its new location; the hero then explores (§ 6.5).
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when a card isn't in hand; lookup and
    /// transition errors.
    pub fn move_hero(
        &mut self,
        hero_id: i64,
        destination: &str,
        cards: &[CardNumber],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<MoveOutcome, EngineError> {
        let hero = self.hero(hero_id)?;
        if hero.has_flaw(Flaw::Hopeless, data) && hero.turn_record.travels >= HOPELESS_TRAVELS {
            return Ok(MoveOutcome::Hopeless);
        }
        if self.board.turn_effects.single_move && hero.turn_record.travels >= 1 {
            return Ok(MoveOutcome::SingleMove);
        }
        if !validate_movement(hero, destination, cards, data)? {
            return Ok(MoveOutcome::InvalidCards);
        }
        let hero = self.hero_mut(hero_id)?;
        hero.hand_to_rest(cards)?;
        hero.location = destination.to_owned();
        hero.turn_record.travels += 1;

        let window = ShadowWindow {
            timing: ShadowTiming::EnteredLocation,
            hero_id,
        };
        if self.open_shadow_window(window, Resume::TravelThreats, data)? {
            return Ok(MoveOutcome::Choice);
        }
        match self.travel_threats(hero_id, data, rng)? {
            Threat::None => Ok(MoveOutcome::Exploration),
            Threat::Combat => Ok(MoveOutcome::Combat),
            Threat::Choice => Ok(MoveOutcome::Choice),
        }
    }

    /// Combat or Peril on the location the hero just entered (§ 6.5); the hero
    /// explores when nothing threatens him.
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn travel_threats(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Threat, EngineError> {
        let threat = self.face_location_threats(hero_id, false, data, rng)?;
        if threat == Threat::None {
            self.fire(BoardEvent::NextToHeroExploration)?;
        }
        Ok(threat)
    }

    /// The hero stops moving and explores its location.
    ///
    /// # Errors
    /// [`EngineError::InvalidTransition`] outside the movement step.
    pub fn movement_finished(&mut self) -> Result<(), EngineError> {
        self.fire(BoardEvent::NextToHeroExploration)
    }

    /// Monsters next to the hero, revealed by Argalad's power.
    pub fn surrounding_monsters(&self, hero: &Hero, data: &GameData) -> Vec<&Mob> {
        let near = data.paths.connected(&hero.location);
        self.mobs
            .iter()
            .filter(|m| {
                m.kind == crate::game_data::mob_def::MobKind::Monster
                    && near.contains(&m.location.as_str())
            })
            .collect()
    }

    /// Tells whether Argalad may look at the surrounding monsters now.
    pub fn argalad_can_use_power(&self, hero: &Hero, data: &GameData) -> bool {
        hero.code == "argalad"
            && !hero.turn_record.argalad
            && !self.surrounding_monsters(hero, data).is_empty()
    }

    /// Argalad looks at the surrounding monsters (once per turn).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the hero isn't Argalad.
    pub fn use_argalad_power(&mut self, hero_id: i64) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        if hero.code != "argalad" {
            return Err(EngineError::Refused(
                "only Argalad has this power".to_owned(),
            ));
        }
        hero.turn_record.argalad = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, hero, rng};

    // Hopeless: at most 3 Travel steps per turn.
    #[test]
    fn hopeless_hero_moves_three_times() {
        let mut g = game();
        g.heroes[0].corruption_cards = vec![1];
        g.heroes[0].location = "barad_dur".to_owned();
        g.heroes[0].turn_record.travels = 3;
        g.board.state = BoardState::HeroMovementScreen;
        let outcome = g.move_hero(1, "mount_doom", &[4], data(), &mut rng());
        assert_eq!(outcome.unwrap(), MoveOutcome::Hopeless);
        assert_eq!(g.heroes[0].location, "barad_dur");
    }

    #[test]
    fn matching_terrain_or_enough_cards() {
        let mut h = hero("argalad");
        h.location = "barad_dur".to_owned();
        // barad_dur -> mount_doom is a sun path of difficulty 2; card 4 (Fall Back) is sun.
        assert!(validate_movement(&h, "mount_doom", &[4], data()).unwrap());
        // Card 6 (Parry) is hills: two cards are needed.
        assert!(!validate_movement(&h, "mount_doom", &[6], data()).unwrap());
        assert!(validate_movement(&h, "mount_doom", &[6, 6], data()).unwrap());
        h.items.horse = true;
        assert!(validate_movement(&h, "mount_doom", &[6], data()).unwrap());
    }
}
