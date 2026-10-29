//! All map locations.

use crate::game_data::code_map::CodeMap;
use crate::game_data::game_data_error::GameDataError;
use crate::game_data::location::{Color, Location};

/// Every location of the map, in file order, by code.
#[derive(Debug, Clone)]
pub struct Locations {
    map: CodeMap<String, Location>,
}

impl Locations {
    /// Wraps the loaded locations.
    ///
    /// # Arguments
    /// * `map` - locations by code, in file order.
    pub fn new(map: CodeMap<String, Location>) -> Self {
        Self { map }
    }

    /// Looks up a location.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the code doesn't exist.
    pub fn get(&self, code: &str) -> Result<&Location, GameDataError> {
        self.map
            .get(code)
            .ok_or_else(|| GameDataError::unknown("location", code))
    }

    /// Display name of a location, or the code itself when it is unknown.
    pub fn name<'a>(&'a self, code: &'a str) -> &'a str {
        self.map.get(code).map_or(code, |l| l.name.as_str())
    }

    /// Tells whether the code is a known location.
    pub fn exists(&self, code: &str) -> bool {
        self.map.contains_key(code)
    }

    /// Iterates over `(code, location)` in file order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Location)> {
        self.map.iter()
    }

    /// `(code, location)` sorted by display name, for select boxes.
    pub fn sorted_by_name(&self) -> Vec<(&String, &Location)> {
        let mut list: Vec<_> = self.map.iter().collect();
        list.sort_by(|a, b| a.1.name.cmp(&b.1.name));
        list
    }

    /// Code of the haven of a region color.
    ///
    /// Some regions have no haven of their own and use a neighbour's.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when no haven matches the color.
    pub fn haven_for_color(&self, color: Color) -> Result<&str, GameDataError> {
        let color = match color {
            Color::Yellow => Color::LightGreen,
            Color::Purple => Color::Orange,
            Color::Red | Color::Brown => Color::LightBlue,
            other => other,
        };
        self.map
            .iter()
            .find(|(_, l)| l.color_code == color && l.haven)
            .map(|(code, _)| code.as_str())
            .ok_or_else(|| GameDataError::unknown("haven for color", format!("{color:?}")))
    }
}
