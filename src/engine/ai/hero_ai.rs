//! Plays the computer heroes: each move is tried on a copy of the game and kept
//! when the rules accept it and it changed something.

use rand::rngs::StdRng;

use crate::engine::EngineError;
use crate::engine::ai::candidates::candidates;
use crate::engine::ai::waiting_hero::waiting_computer_hero;
use crate::game_data::GameData;
use crate::model::game::Game;

/// Moves after which the computer heroes stop, against a move that never ends a turn.
const MAX_AI_STEPS: usize = 300;

impl Game {
    /// Plays the computer heroes while the game waits on one of them. When none of a
    /// hero's moves is accepted, it stops and leaves the seat to the board members.
    ///
    /// # Arguments
    /// * `computer` - ids of the heroes played by the computer.
    ///
    /// # Returns
    /// The number of moves played.
    ///
    /// # Errors
    /// Lookup errors while listing a hero's moves.
    #[tracing::instrument(level = "debug", skip(self, data, rng), err(level = "debug"))]
    pub fn play_computer_heroes(
        &mut self,
        computer: &[i64],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<usize, EngineError> {
        for step in 0..MAX_AI_STEPS {
            let Some(hero_id) = waiting_computer_hero(self, computer) else {
                return Ok(step);
            };
            tracing::trace!(step, hero_id, state = %self.board.state, "computer hero step");
            if !self.play_one_move(hero_id, data, rng)? {
                tracing::warn!(hero_id, state = %self.board.state, "no move accepted for the computer hero");
                return Ok(step);
            }
        }
        tracing::warn!(state = %self.board.state, "computer heroes stopped at the step limit");
        Ok(MAX_AI_STEPS)
    }

    /// Plays the first of the hero's moves the rules accept.
    ///
    /// # Returns
    /// Whether a move was played.
    fn play_one_move(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<bool, EngineError> {
        for candidate in candidates(self, hero_id, data, rng)? {
            tracing::trace!(?candidate, "computer hero tries");
            let mut copy = self.clone();
            match candidate.apply(&mut copy, hero_id, data, rng) {
                Ok(()) if progressed(self, &copy) => {
                    tracing::debug!(hero_id, ?candidate, to = %copy.board.state, "computer hero move");
                    *self = copy;
                    return Ok(true);
                }
                Ok(()) => tracing::trace!("move changed nothing"),
                Err(error) => tracing::trace!(%error, "move refused"),
            }
        }
        Ok(false)
    }
}

/// Whether a move changed the game. A move that only wrote log lines (a favor
/// refused over the corruption limit) changed nothing and would be played forever.
fn progressed(before: &Game, after: &Game) -> bool {
    let Game {
        board,
        sauron,
        heroes,
        mobs,
        combat,
        new_logs: _,
    } = after;
    *board != before.board
        || *sauron != before.sauron
        || *heroes != before.heroes
        || *mobs != before.mobs
        || *combat != before.combat
}
