//! Colors of the monster pools.

use serde::{Deserialize, Serialize};

use crate::game_data::location::Color;

/// One of the five monster pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonsterPoolColor {
    /// Orange pool.
    Orange,
    /// Purple pool.
    Purple,
    /// Dark blue pool.
    DarkBlue,
    /// Brown pool.
    Brown,
    /// Dark green pool.
    DarkGreen,
}

impl MonsterPoolColor {
    /// All pools.
    pub const ALL: [Self; 5] = [
        Self::Orange,
        Self::Purple,
        Self::DarkBlue,
        Self::Brown,
        Self::DarkGreen,
    ];

    /// Pool a monster is drawn from when placed on a location of `color`.
    pub fn for_location_color(color: Color) -> Self {
        match color {
            Color::Orange | Color::Yellow => Self::Orange,
            Color::Purple | Color::White => Self::Purple,
            Color::LightBlue | Color::DarkBlue => Self::DarkBlue,
            Color::Brown | Color::Red => Self::Brown,
            Color::LightGreen | Color::DarkGreen => Self::DarkGreen,
        }
    }

    /// Snake-case name, used in token picture paths.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Orange => "orange",
            Self::Purple => "purple",
            Self::DarkBlue => "dark_blue",
            Self::Brown => "brown",
            Self::DarkGreen => "dark_green",
        }
    }
}
