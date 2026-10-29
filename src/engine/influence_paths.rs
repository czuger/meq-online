//! How much influence a location may hold, given its links to the shadow strongholds.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::game_data::location_paths::LocationPaths;

/// Sauron's strongholds, sources of influence.
const STRONGHOLDS: [&str; 3] = ["dol_guldur", "barad_dur", "mount_gundabad"];

/// Influence limits computed from the influence already on the map.
pub struct InfluencePaths<'a> {
    influence: &'a BTreeMap<String, i32>,
    paths: &'a LocationPaths,
}

impl<'a> InfluencePaths<'a> {
    /// Prepares computations on the current influence.
    ///
    /// # Arguments
    /// * `influence` - influence by location.
    /// * `paths` - map graph.
    pub fn new(influence: &'a BTreeMap<String, i32>, paths: &'a LocationPaths) -> Self {
        Self { influence, paths }
    }

    /// Maximum influence a new token may bring to `location` (0 when unconnected).
    pub fn max_for_new_token(&self, location: &str) -> i32 {
        self.max_from_strongholds(location).unwrap_or(0)
    }

    /// Maximum influence an existing token may keep on `location` (1 when unconnected).
    pub fn max_for_existing_token(&self, location: &str) -> i32 {
        self.max_from_strongholds(location).unwrap_or(1)
    }

    /// Highest influence among the strongholds linked to `location` by influenced locations.
    fn max_from_strongholds(&self, location: &str) -> Option<i32> {
        STRONGHOLDS
            .iter()
            .filter(|stronghold| self.connected(location, stronghold))
            .map(|stronghold| self.influence.get(*stronghold).copied().unwrap_or(0))
            .max()
    }

    /// Tells whether `destination` can be reached from `start` walking only through
    /// locations holding at least 1 influence.
    fn connected(&self, start: &str, destination: &str) -> bool {
        let mut visited = BTreeSet::from([start]);
        let mut frontier = VecDeque::from([start]);
        while let Some(current) = frontier.pop_front() {
            if current == destination {
                return true;
            }
            for next in self.paths.connected(current) {
                let influenced = self.influence.get(next).copied().unwrap_or(0) >= 1;
                if influenced && visited.insert(next) {
                    frontier.push_back(next);
                }
            }
        }
        false
    }
}
