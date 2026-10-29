//! Static definition of a monster or minion.

use serde::{Deserialize, Serialize};

/// Whether a mob is a monster (drawn from a pool) or a named minion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MobKind {
    /// Monster token drawn from a colored pool.
    Monster,
    /// Named minion of Sauron.
    Minion,
}

/// A mob as described in `data/mobs.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct MobDef {
    /// Display name.
    pub name: String,
    /// Hand size at the start of a combat.
    pub fortitude: i32,
    /// Strength available during a combat.
    pub strength: i32,
    /// Wisdom.
    pub wisdom: i32,
    /// Starting life; missing for mobs not finished in the data (uruk_hai).
    #[serde(default)]
    pub life: Option<i32>,
    /// Code of the attack deck used in combat.
    pub attack_deck: String,
    /// Monster or minion.
    #[serde(rename = "type")]
    pub kind: MobKind,
}
