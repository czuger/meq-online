//! A game board and its shared state.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::board_state::BoardState;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::monster_pool_color::MonsterPoolColor;

/// Plot slot number (1 to 3).
pub type PlotSlot = u8;

/// Positions of the four story markers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StoryMarkers {
    /// Heroes' marker.
    pub heroes: i32,
    /// Sauron's ring marker.
    pub ring: i32,
    /// Sauron's conquest marker.
    pub conquest: i32,
    /// Sauron's corruption marker.
    pub corruption: i32,
}

/// A draw pile and its discard pile.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pile {
    /// Cards to draw, top first.
    pub draw: Vec<CardNumber>,
    /// Discarded cards.
    pub discard: Vec<CardNumber>,
}

/// The board's card piles.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decks {
    /// Sauron's plot cards.
    pub plot: Pile,
    /// Sauron's shadow cards.
    pub shadow: Pile,
    /// Event cards.
    pub event: Pile,
    /// Corruption cards.
    pub corruption: Pile,
}

/// Winner of a finished game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Winner {
    /// The heroes won.
    Heroes,
    /// Sauron won.
    Sauron,
    /// Both reached their goal on the same turn.
    Draw,
}

/// A game board.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    /// Database id; `None` until persisted.
    pub id: Option<i64>,
    /// Current game phase.
    pub state: BoardState,
    /// Maximum number of heroes.
    pub max_heroes: i32,
    /// Game turn, starting at 1.
    pub turn: i32,
    /// Hero playing now.
    pub current_hero_id: Option<i64>,
    /// Set when the game is finished.
    pub winner: Option<Winner>,
    /// Story track.
    pub story: StoryMarkers,
    /// Influence available to Sauron.
    pub shadow_pool: i32,
    /// Influence tokens by location code.
    pub influence: BTreeMap<String, i32>,
    /// Location of each character on the map.
    pub characters: BTreeMap<String, String>,
    /// Locations holding a favor token (one entry per token).
    pub favors: Vec<String>,
    /// Plot card in each used slot.
    pub plots: BTreeMap<PlotSlot, u32>,
    /// Sauron action markers placed this turn.
    pub sauron_actions: Vec<String>,
    /// Number of times Sauron updated his actions this turn.
    pub sauron_actions_count: i32,
    /// Card piles.
    pub decks: Decks,
    /// Remaining monster codes in each pool, top first.
    pub monster_pools: BTreeMap<MonsterPoolColor, Vec<String>>,
    /// Last drawn event card.
    pub last_event_card: Option<u32>,
    /// Secret objective card of the heroes.
    pub heroes_objective: Option<u32>,
    /// Secret objective card of Sauron.
    pub sauron_objective: Option<u32>,
}
