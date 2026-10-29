//! The logged-in user of a request.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app_state::AppState;
use crate::db::user_repo;
use crate::model::user::User;
use crate::web::app_error::AppError;
use crate::web::cookies::{SESSION_COOKIE, read_cookie};
use crate::web::flash::Flash;

/// The logged-in user, and the flash message sent with the request.
#[derive(Debug, Clone)]
pub struct Session {
    /// The user.
    pub user: User,
    /// Flash message to display on this page.
    pub flash: Option<Flash>,
}

impl FromRequestParts<AppState> for Session {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = read_cookie(&parts.headers, SESSION_COOKIE).ok_or(AppError::NotLoggedIn)?;
        let mut conn = state.db.acquire().await.map_err(crate::db::DbError::from)?;
        let user = user_repo::session_user(&mut conn, &token)
            .await?
            .ok_or(AppError::NotLoggedIn)?;
        Ok(Self {
            user,
            flash: Flash::from_headers(&parts.headers),
        })
    }
}
