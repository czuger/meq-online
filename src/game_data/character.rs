//! Static definition of a character.

use serde::Deserialize;

/// A character heroes can meet on the map.
#[derive(Debug, Clone, Deserialize)]
pub struct Character {
    /// Display name.
    pub name: String,
    /// Power description.
    pub power: String,
}
