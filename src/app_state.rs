//! State shared by every request.

use std::sync::Arc;

use sqlx::SqlitePool;

use crate::game_data::GameData;
use crate::refresh::RefreshHub;

/// Database, static data and refresh hub, cloned into every handler.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Database pool.
    pub db: SqlitePool,
    /// Static game data.
    pub data: Arc<GameData>,
    /// Live refresh broadcaster.
    pub refresh: RefreshHub,
}
