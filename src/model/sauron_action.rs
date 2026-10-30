//! Sauron's action tracks and the action he is resolving (§ 5.7).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

/// One of Sauron's three action tracks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SauronAction {
    /// Place Influence.
    PlaceInfluence,
    /// Draw Shadow and Plot Cards.
    DrawCards,
    /// Command Monsters and Minions.
    Command,
}

impl SauronAction {
    /// The three tracks, in board order.
    pub const ALL: [Self; 3] = [Self::PlaceInfluence, Self::DrawCards, Self::Command];

    /// Snake-case name, used in forms and token codes.
    pub fn code(self) -> &'static str {
        match self {
            Self::PlaceInfluence => "place_influence",
            Self::DrawCards => "draw_cards",
            Self::Command => "command",
        }
    }

    /// The track named `code`.
    pub fn from_code(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.code() == code)
    }
}

/// An action token on a track space; space 1 is the leftmost.
///
/// Stored as `"<track>_<space>"` (`"command_1"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ActionToken {
    /// The track.
    pub action: SauronAction,
    /// The space, 1 to 3.
    pub space: u8,
}

impl fmt::Display for ActionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.action.code(), self.space)
    }
}

impl From<ActionToken> for String {
    fn from(token: ActionToken) -> Self {
        token.to_string()
    }
}

/// Error returned for a malformed action token code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown action token '{0}'")]
pub struct UnknownActionToken(pub String);

impl TryFrom<String> for ActionToken {
    type Error = UnknownActionToken;

    fn try_from(code: String) -> Result<Self, Self::Error> {
        let parsed = code.rsplit_once('_').and_then(|(track, space)| {
            let action = SauronAction::from_code(track)?;
            let space: u8 = space.parse().ok()?;
            (1..=3).contains(&space).then_some(Self { action, space })
        });
        parsed.ok_or(UnknownActionToken(code))
    }
}

/// The action Sauron is resolving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionInProgress {
    /// The track.
    pub action: SauronAction,
    /// X, the number on the space just covered.
    pub value: i32,
    /// Commands left (Command).
    #[serde(default)]
    pub commands_left: i32,
    /// A monster token was placed by this action (Command).
    #[serde(default)]
    pub monster_placed: bool,
}

/// Influence Sauron may place now: from a Place Influence action, the starting plot or
/// an undefeated monster or minion.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfluenceBudget {
    /// Tokens left to place.
    #[serde(default)]
    pub tokens: i32,
    /// Most tokens of this budget that may go to the Shadow Pool.
    #[serde(default)]
    pub shadow_pool_max: i32,
    /// Tokens placed from this budget, by location; they may be taken back.
    #[serde(default)]
    pub placed: BTreeMap<String, i32>,
    /// Tokens put in the Shadow Pool from this budget; they may be taken back.
    #[serde(default)]
    pub shadow_pool_placed: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_codes_round_trip() {
        let token = ActionToken::try_from("place_influence_2".to_owned()).unwrap();
        assert_eq!(token.action, SauronAction::PlaceInfluence);
        assert_eq!(token.space, 2);
        assert_eq!(String::from(token), "place_influence_2");
        assert!(ActionToken::try_from("command_4".to_owned()).is_err());
        assert!(ActionToken::try_from("rest_1".to_owned()).is_err());
    }
}
