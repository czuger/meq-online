//! Lookups and bookkeeping shared by all rules working on a [`Game`].

use serde_json::Value;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::hero::Hero;
use crate::model::log_entry::LogEntry;
use crate::model::mob::Mob;

impl Game {
    /// Fires a state machine event on the board.
    ///
    /// # Errors
    /// [`EngineError::InvalidTransition`] when the event isn't allowed now.
    pub fn fire(&mut self, event: BoardEvent) -> Result<(), EngineError> {
        let from = self.board.state;
        match from.transition(event) {
            Ok(to) => {
                tracing::debug!(board_id = ?self.board.id, %from, event = event.as_str(), %to, "board state");
                self.board.state = to;
                Ok(())
            }
            Err(error) => {
                tracing::debug!(board_id = ?self.board.id, %from, event = event.as_str(), "board event refused");
                Err(error)
            }
        }
    }

    /// Records a log entry, saved with the game.
    ///
    /// # Arguments
    /// * `actor` - who did it.
    /// * `action` - dotted action key.
    /// * `params` - values interpolated in the message.
    pub fn log(&mut self, actor: ActorRef, action: &str, params: &[(&str, Value)]) {
        tracing::debug!(board_id = ?self.board.id, ?actor, action, ?params, "game log");
        self.new_logs.push(LogEntry::new(actor, action, params));
    }

    /// Position of a hero in `heroes`.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero isn't on this board.
    pub fn hero_index(&self, hero_id: i64) -> Result<usize, EngineError> {
        self.heroes
            .iter()
            .position(|h| h.id == Some(hero_id))
            .ok_or(EngineError::NotFound {
                kind: "hero",
                id: hero_id,
            })
    }

    /// A hero of this board.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero isn't on this board.
    pub fn hero(&self, hero_id: i64) -> Result<&Hero, EngineError> {
        let index = self.hero_index(hero_id)?;
        self.heroes.get(index).ok_or(EngineError::NotFound {
            kind: "hero",
            id: hero_id,
        })
    }

    /// A hero of this board, mutable.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero isn't on this board.
    pub fn hero_mut(&mut self, hero_id: i64) -> Result<&mut Hero, EngineError> {
        let index = self.hero_index(hero_id)?;
        self.heroes.get_mut(index).ok_or(EngineError::NotFound {
            kind: "hero",
            id: hero_id,
        })
    }

    /// Id of the hero playing now.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when no hero is playing.
    pub fn current_hero_id(&self) -> Result<i64, EngineError> {
        self.board
            .current_hero_id
            .ok_or_else(|| EngineError::Refused("no current hero".to_owned()))
    }

    /// Position of a mob in `mobs`.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the mob isn't on this board.
    pub fn mob_index(&self, mob_id: i64) -> Result<usize, EngineError> {
        self.mobs
            .iter()
            .position(|m| m.id == Some(mob_id))
            .ok_or(EngineError::NotFound {
                kind: "mob",
                id: mob_id,
            })
    }

    /// A mob of this board, mutable.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the mob isn't on this board.
    pub fn mob_mut(&mut self, mob_id: i64) -> Result<&mut Mob, EngineError> {
        let index = self.mob_index(mob_id)?;
        self.mobs.get_mut(index).ok_or(EngineError::NotFound {
            kind: "mob",
            id: mob_id,
        })
    }
}
