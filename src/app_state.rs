//! State shared by every request.

use std::sync::Arc;

use sqlx::SqlitePool;

use crate::board_versions::BoardVersions;
use crate::game_data::GameData;

/// Database, static data and board versions, cloned into every handler.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Database pool.
    pub db: SqlitePool,
    /// Static game data.
    pub data: Arc<GameData>,
    /// Board versions polled by open pages.
    pub versions: BoardVersions,
}
