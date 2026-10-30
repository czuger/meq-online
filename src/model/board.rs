//! A game board and its shared state.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine::board_state::BoardState;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::monster_pool_color::MonsterPoolColor;
use crate::model::choice::PendingChoice;
use crate::model::sauron_action::{ActionInProgress, ActionToken, InfluenceBudget};

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
    /// Stage I Event cards.
    pub event: Pile,
    /// Stage II Event cards.
    #[serde(default)]
    pub event_ii: Pile,
    /// Stage III Event cards.
    #[serde(default)]
    pub event_iii: Pile,
    /// Skill cards.
    #[serde(default)]
    pub skill: Pile,
    /// Corruption cards.
    pub corruption: Pile,
}

/// An event card: its stage deck and number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventRef {
    /// Stage deck (1 to 3).
    pub stage: u8,
    /// Card number in the deck.
    pub card: u32,
}

/// An event card a hero keeps until it triggers ("Trust in Friendship").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldEvent {
    /// The card.
    pub event: EventRef,
    /// Its holder.
    pub hero_id: i64,
    /// Favor placed on the card.
    pub favor: i32,
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
    /// Number of heroes in the game.
    pub max_heroes: i32,
    /// Heroes whose seat waits for a player to join.
    pub waiting_heroes: Vec<String>,
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
    /// Sauron's action tokens on the action tracks, oldest first (at most three).
    pub sauron_actions: Vec<ActionToken>,
    /// Actions Sauron took this turn.
    pub sauron_actions_count: i32,
    /// The action Sauron is resolving.
    pub current_action: Option<ActionInProgress>,
    /// Card piles.
    pub decks: Decks,
    /// Remaining monster codes in each pool, top first.
    pub monster_pools: BTreeMap<MonsterPoolColor, Vec<String>>,
    /// Monster tokens discarded after a combat or a reveal, by pool.
    pub monster_discards: BTreeMap<MonsterPoolColor, Vec<String>>,
    /// The Ringwraiths were defeated and come back at Sauron's next Action step.
    pub ringwraiths_return: bool,
    /// Influence Sauron may place now.
    pub influence_budget: InfluenceBudget,
    /// Last resolved event card.
    pub last_event: Option<EventRef>,
    /// Event card in play on the Current Event Card space (green flag).
    pub current_event: Option<EventRef>,
    /// Event card held by a hero.
    pub held_event: Option<HeldEvent>,
    /// Decision the game waits for.
    pub pending_choice: Option<PendingChoice>,
    /// Secret objective card of the heroes.
    pub heroes_objective: Option<u32>,
    /// Secret objective card of Sauron.
    pub sauron_objective: Option<u32>,
}
