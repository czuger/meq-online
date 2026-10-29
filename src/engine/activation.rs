//! Which actors are expected to act.

use crate::engine::engine_error::EngineError;
use crate::model::game::Game;

impl Game {
    /// Makes the current hero the only active actor.
    ///
    /// # Errors
    /// Fails when no current hero is set or it isn't on the board.
    pub fn activate_current_hero(&mut self) -> Result<(), EngineError> {
        let current = self.current_hero_id()?;
        self.set_heroes_active(false);
        self.sauron.active = false;
        self.hero_mut(current)?.active = true;
        Ok(())
    }

    /// Hands over from the current hero to Sauron.
    ///
    /// # Errors
    /// Fails when no current hero is set or it isn't on the board.
    pub fn switch_to_sauron(&mut self) -> Result<(), EngineError> {
        let current = self.current_hero_id()?;
        self.hero_mut(current)?.active = false;
        self.sauron.active = true;
        Ok(())
    }

    /// Tells whether at least one hero is active.
    pub fn heroes_active(&self) -> bool {
        self.heroes.iter().any(|h| h.active)
    }

    /// Activates or deactivates every hero.
    pub fn set_heroes_active(&mut self, active: bool) {
        for hero in &mut self.heroes {
            hero.active = active;
        }
    }
}
