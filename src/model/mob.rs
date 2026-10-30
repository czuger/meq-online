//! A monster or minion on a board.

use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::mob_def::MobKind;
use crate::game_data::monster_pool_color::MonsterPoolColor;

/// Code of the blank monster token that only means "nothing here".
pub const NOTHING_CODE: &str = "nothing";

/// A monster or minion and its game state.
#[derive(Debug, Clone, PartialEq)]
pub struct Mob {
    /// Database id; `None` until persisted.
    pub id: Option<i64>,
    /// Monster or minion.
    pub kind: MobKind,
    /// Code in `data/mobs.yaml`, or [`NOTHING_CODE`].
    pub code: String,
    /// Display name.
    pub name: String,
    /// Location code.
    pub location: String,
    /// Pool the monster was drawn from (monsters only).
    pub pool: Option<MonsterPoolColor>,
    /// Hand size at the start of a combat.
    pub fortitude: i32,
    /// Strength available in a combat.
    pub strength: i32,
    /// Remaining life.
    pub life: i32,
    /// Starting life.
    pub max_life: i32,
    /// Attack deck code.
    pub attack_deck: String,
    /// Cards in hand during a combat.
    pub hand: Vec<CardNumber>,
    /// Damage taken during the current combat round.
    pub damages_taken_this_turn: i32,
    /// Received a command during Sauron's current action.
    pub commanded: bool,
}
