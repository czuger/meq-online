//! Path and card terrain types.

use serde::Deserialize;

/// Terrain of a path between two locations, also printed on cards used to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    /// Path with no specific terrain: only the number of cards counts.
    Any,
    /// Desolation.
    Desolation,
    /// Forest.
    Forest,
    /// Hills.
    Hills,
    /// Mountains.
    Mountains,
    /// Sun (open land).
    Sun,
}
