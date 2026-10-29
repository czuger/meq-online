//! Errors raised by game rules.

use thiserror::Error;

use crate::game_data::GameDataError;

/// A rule refused an action, or the game state is inconsistent.
#[derive(Debug, Error)]
pub enum EngineError {
    /// A static data lookup failed.
    #[error(transparent)]
    GameData(#[from] GameDataError),
    /// The board can't take this transition from its current state.
    #[error("invalid transition {event} from state {from}")]
    InvalidTransition {
        /// Current state.
        from: &'static str,
        /// Requested event.
        event: &'static str,
    },
    /// A combat round was resolved before both sides were ready.
    #[error("combat not ready: {0}")]
    CombatNotReady(&'static str),
    /// No combat is running on the board.
    #[error("no combat in progress")]
    NoCombat,
    /// A combat can't start while another one is running.
    #[error("a combat is already in progress")]
    CombatInProgress,
    /// A card was played that isn't in the player's hand or pool.
    #[error("card {card} is not available in {pile}")]
    CardNotAvailable {
        /// The card.
        card: u32,
        /// Where it was looked for.
        pile: &'static str,
    },
    /// An entity referenced by id doesn't exist on the board.
    #[error("{kind} {id} not found on this board")]
    NotFound {
        /// Kind of entity.
        kind: &'static str,
        /// Its id.
        id: i64,
    },
    /// An entity must be persisted before this action.
    #[error("{0} must be saved first")]
    Unpersisted(&'static str),
    /// The action isn't allowed by the rules in the current situation.
    #[error("action refused: {0}")]
    Refused(String),
}
