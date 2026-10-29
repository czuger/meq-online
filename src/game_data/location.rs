//! Static definition of a map location.

use serde::{Deserialize, Serialize};

/// Region color printed on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    /// Orange.
    Orange,
    /// Yellow.
    Yellow,
    /// Purple.
    Purple,
    /// Light blue.
    LightBlue,
    /// Dark blue.
    DarkBlue,
    /// White.
    White,
    /// Brown.
    Brown,
    /// Red.
    Red,
    /// Light green.
    LightGreen,
    /// Dark green.
    DarkGreen,
}

/// A location of the map.
#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    /// Display name.
    pub name: String,
    /// Region display name.
    pub region: String,
    /// Region code.
    pub region_code: String,
    /// Region color.
    pub color_code: Color,
    /// Entering it may trigger a peril.
    pub perilous: bool,
    /// Heroes can heal here.
    pub haven: bool,
    /// Heroes can't rest here.
    #[serde(default)]
    pub shadow_fortress: bool,
    /// Horizontal position on the map picture.
    pub pos_x: i32,
    /// Vertical position on the map picture.
    pub pos_y: i32,
}
