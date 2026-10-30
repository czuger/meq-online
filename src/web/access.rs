//! Extractors checking that the user takes part in the board of a URL.

use std::collections::HashMap;

use axum::extract::{FromRequestParts, Path};
use axum::http::request::Parts;

use crate::app_state::AppState;
use crate::db::{DbError, hero_repo, sauron_repo, user_repo};
use crate::web::app_error::AppError;
use crate::web::session::Session;

/// A `/boards/{board_id}/...` URL opened by a member of the board.
#[derive(Debug, Clone)]
pub struct BoardAccess {
    /// The user.
    pub session: Session,
    /// The board.
    pub board_id: i64,
}

/// A `/heroes/{hero_id}/...` URL opened by a member of the hero's board.
#[derive(Debug, Clone)]
pub struct HeroAccess {
    /// The user.
    pub session: Session,
    /// The hero's board.
    pub board_id: i64,
    /// The hero.
    pub hero_id: i64,
}

/// A `/sauron/{sauron_id}/...` URL opened by a member of Sauron's board.
#[derive(Debug, Clone)]
pub struct SauronAccess {
    /// The user.
    pub session: Session,
    /// Sauron's board.
    pub board_id: i64,
    /// The Sauron player.
    pub sauron_id: i64,
}

/// Reads an integer path parameter.
#[tracing::instrument(level = "debug", skip_all, fields(%name), err(level = "debug"))]
async fn path_id(parts: &mut Parts, state: &AppState, name: &str) -> Result<i64, AppError> {
    let Path(params) = Path::<HashMap<String, String>>::from_request_parts(parts, state)
        .await
        .map_err(|_| AppError::NotFound)?;
    params
        .get(name)
        .and_then(|v| v.parse().ok())
        .ok_or(AppError::NotFound)
}

/// Checks board membership, the only ownership rule of the game.
#[tracing::instrument(level = "debug", skip_all, fields(board_id), err(level = "debug"))]
async fn ensure_member(state: &AppState, board_id: i64, session: &Session) -> Result<(), AppError> {
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    if user_repo::is_member(&mut conn, board_id, session.user.id).await? {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

impl FromRequestParts<AppState> for BoardAccess {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let board_id = path_id(parts, state, "board_id").await?;
        ensure_member(state, board_id, &session).await?;
        Ok(Self { session, board_id })
    }
}

impl FromRequestParts<AppState> for HeroAccess {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let hero_id = path_id(parts, state, "hero_id").await?;
        let mut conn = state.db.acquire().await.map_err(DbError::from)?;
        let board_id = hero_repo::board_id(&mut conn, hero_id).await?;
        drop(conn);
        ensure_member(state, board_id, &session).await?;
        Ok(Self {
            session,
            board_id,
            hero_id,
        })
    }
}

impl FromRequestParts<AppState> for SauronAccess {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;
        let sauron_id = path_id(parts, state, "sauron_id").await?;
        let mut conn = state.db.acquire().await.map_err(DbError::from)?;
        let board_id = sauron_repo::board_id(&mut conn, sauron_id).await?;
        drop(conn);
        ensure_member(state, board_id, &session).await?;
        Ok(Self {
            session,
            board_id,
            sauron_id,
        })
    }
}
