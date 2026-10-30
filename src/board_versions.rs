//! Board versions: open pages poll the version of their board and follow the game
//! when it changed.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Version of each board, kept in memory and raised on every change.
#[derive(Debug, Clone)]
pub struct BoardVersions {
    /// Drawn at start, so a page opened before a restart sees a new version after it.
    run: u64,
    changes: Arc<Mutex<HashMap<i64, u64>>>,
}

impl BoardVersions {
    /// Creates the versions of a new server run.
    pub fn new() -> Self {
        Self {
            run: rand::random(),
            changes: Arc::default(),
        }
    }

    /// Current version of a board: an opaque text, different after each change.
    pub fn version(&self, board_id: i64) -> String {
        let changes = self.changes().get(&board_id).copied().unwrap_or_default();
        format!("{:x}-{changes}", self.run)
    }

    /// Raises the version of a board that changed.
    pub fn board_changed(&self, board_id: i64) {
        let mut changes = self.changes();
        let count = changes.entry(board_id).or_default();
        *count += 1;
        tracing::trace!(board_id, version = *count, "board version raised");
    }

    fn changes(&self) -> MutexGuard<'_, HashMap<i64, u64>> {
        // A panic while the lock was held leaves plain counters, still valid.
        self.changes.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for BoardVersions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_change_raises_only_its_board() {
        let versions = BoardVersions::new();
        let (one, two) = (versions.version(1), versions.version(2));
        versions.board_changed(1);
        assert_ne!(versions.version(1), one);
        assert_eq!(versions.version(2), two);
    }

    #[test]
    fn a_new_run_starts_from_another_version() {
        assert_ne!(
            BoardVersions::new().version(1),
            BoardVersions::new().version(1)
        );
    }
}
