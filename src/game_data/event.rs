//! Static definition of an event card.

use serde::Deserialize;

/// A character placed by an event card.
#[derive(Debug, Clone, Deserialize)]
pub struct EventCharacter {
    /// Character code.
    pub name: String,
    /// Location code.
    pub location: String,
}

/// An event card.
#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    /// Character to place, if any.
    #[serde(default)]
    pub character: Option<EventCharacter>,
    /// Locations receiving a favor token.
    #[serde(default)]
    pub favors: Vec<String>,
}
