//! Card effects lasting until the end of a player's turn or a later step.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::board::PlotSlot;

/// Effects of Shadow and Plot cards that outlive the moment they were played.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnEffects {
    /// Sauron played a Shadow card during the current player's turn (§ 5.9).
    #[serde(default)]
    pub shadow_played: bool,
    /// Location treated as perilous until the end of the hero's turn ("His Arm Has
    /// Grown Long").
    #[serde(default)]
    pub perilous_location: Option<String>,
    /// The hero may move only once during his Travel step ("Winter Storm").
    #[serde(default)]
    pub single_move: bool,
    /// Spaces the green marker advances less during the next Story step ("Grievous
    /// Loss").
    #[serde(default)]
    pub heroes_story_malus: i32,
    /// Location picked by Sauron for the active plots that let him choose it.
    #[serde(default)]
    pub plot_locations: BTreeMap<PlotSlot, String>,
}

impl TurnEffects {
    /// A new player's turn starts: the effects of the previous turn end.
    pub fn start_turn(&mut self) {
        self.shadow_played = false;
        self.perilous_location = None;
        self.single_move = false;
    }
}
