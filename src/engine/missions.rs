//! Mission cards of both sides (§ 9).

use rand::RngExt;
use rand::rngs::StdRng;

use crate::model::game::Game;

/// Number of mission cards for each side.
const MISSION_CARDS: u32 = 5;

impl Game {
    /// Deals the secret missions of both sides.
    pub fn set_objectives(&mut self, rng: &mut StdRng) {
        self.board.heroes_objective = Some(rng.random_range(0..MISSION_CARDS));
        self.board.sauron_objective = Some(rng.random_range(0..MISSION_CARDS));
    }
}

/// Picture of a mission card.
pub fn objective_pic_path(side: &str, card: u32) -> String {
    format!("objectives/original/{side}/{side}_{card}.png")
}
