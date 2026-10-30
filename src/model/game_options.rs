//! Optional rules chosen when the game is created (§ 10.4).

use serde::{Deserialize, Serialize};

/// Optional rules in use on a board; all players agree on them before the game.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameOptions {
    /// Advanced dominance: the side needing fewer Story steps to reach the Finale is
    /// dominant, Sauron's active Plot cards staying in play.
    #[serde(default)]
    pub advanced_dominance: bool,
    /// High stakes stage III: in stage III, Sauron picks the marker a defeated hero
    /// advances while the heroes are dominant, and the heroes advance their marker for
    /// each minion they defeat while Sauron is.
    #[serde(default)]
    pub high_stakes: bool,
}
