//! Everything stored for one board, loaded and saved as a unit.

use crate::model::board::Board;
use crate::model::combat::Combat;
use crate::model::hero::Hero;
use crate::model::log_entry::LogEntry;
use crate::model::mob::Mob;
use crate::model::sauron::Sauron;

/// A board with all its pieces.
///
/// Requests load the whole game, let the engine change it, and save it back in one
/// transaction. Entries in `new_logs` are appended to the log on save.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    /// The board.
    pub board: Board,
    /// The Sauron player.
    pub sauron: Sauron,
    /// Heroes, by ascending id.
    pub heroes: Vec<Hero>,
    /// Monsters and minions on the map.
    pub mobs: Vec<Mob>,
    /// Combat in progress.
    pub combat: Option<Combat>,
    /// Log entries produced since the game was loaded.
    pub new_logs: Vec<LogEntry>,
}
