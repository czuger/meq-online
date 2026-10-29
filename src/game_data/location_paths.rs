//! Paths between map locations.

use serde::Deserialize;

use crate::game_data::code_map::CodeMap;
use crate::game_data::game_data_error::GameDataError;
use crate::game_data::terrain::Terrain;

/// One path leaving a location.
#[derive(Debug, Clone, Deserialize)]
pub struct Destination {
    /// Code of the reached location.
    pub dest: String,
    /// Terrain of the path.
    pub path_type: Terrain,
    /// Number of cards needed when no card matches the terrain.
    pub difficulty: u32,
}

/// All paths leaving a location.
#[derive(Debug, Clone, Deserialize)]
pub struct PathNode {
    /// Paths leaving the location.
    pub destinations: Vec<Destination>,
}

/// The map graph.
#[derive(Debug, Clone)]
pub struct LocationPaths {
    map: CodeMap<String, PathNode>,
}

impl LocationPaths {
    /// Wraps the loaded graph.
    ///
    /// # Arguments
    /// * `map` - path nodes by location code.
    pub fn new(map: CodeMap<String, PathNode>) -> Self {
        Self { map }
    }

    /// Paths leaving a location; empty when the location has no entry.
    pub fn destinations(&self, location: &str) -> &[Destination] {
        self.map
            .get(location)
            .map_or(&[], |node| node.destinations.as_slice())
    }

    /// Codes of the locations directly connected to `location`.
    pub fn connected(&self, location: &str) -> Vec<&str> {
        self.destinations(location)
            .iter()
            .map(|d| d.dest.as_str())
            .collect()
    }

    /// The path from `source` to `dest`.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the two locations aren't connected.
    pub fn path(&self, source: &str, dest: &str) -> Result<&Destination, GameDataError> {
        self.destinations(source)
            .iter()
            .find(|d| d.dest == dest)
            .ok_or_else(|| GameDataError::unknown("path", format!("{source} -> {dest}")))
    }
}
