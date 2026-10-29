//! Reference to whoever performs an action.

use serde::{Deserialize, Serialize};

/// Who did something: the game itself, Sauron or a hero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorRef {
    /// Automatic game step.
    System,
    /// The Sauron player.
    Sauron,
    /// A hero, by id.
    Hero(i64),
}
