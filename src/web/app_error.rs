//! The single error type of HTTP handlers.

use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use thiserror::Error;

use crate::db::DbError;
use crate::engine::EngineError;

/// Anything that can go wrong while handling a request.
#[derive(Debug, Error)]
pub enum AppError {
    /// No user is logged in.
    #[error("not logged in")]
    NotLoggedIn,
    /// The user isn't a member of the board.
    #[error("access denied")]
    Forbidden,
    /// The requested page or record doesn't exist.
    #[error("not found")]
    NotFound,
    /// The request is malformed.
    #[error("bad request: {0}")]
    BadRequest(String),
    /// A game rule refused the action.
    #[error(transparent)]
    Engine(#[from] EngineError),
    /// The database failed.
    #[error(transparent)]
    Db(#[from] DbError),
    /// A page couldn't be rendered.
    #[error(transparent)]
    Template(#[from] askama::Error),
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPage<'a> {
    title: &'a str,
    message: &'a str,
}

impl AppError {
    fn status_and_message(&self) -> (StatusCode, &'static str) {
        match self {
            Self::NotLoggedIn => (StatusCode::UNAUTHORIZED, "Vous devez être connecté."),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "Vous ne participez pas à cette partie.",
            ),
            Self::NotFound | Self::Db(DbError::NotFound(_)) => {
                (StatusCode::NOT_FOUND, "Cette page n'existe pas.")
            }
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, "La requête est invalide."),
            Self::Engine(EngineError::NotFound { .. }) => (
                StatusCode::NOT_FOUND,
                "Cet élément n'existe pas sur ce plateau.",
            ),
            Self::Engine(EngineError::GameData(_)) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Les données du jeu sont incohérentes.",
            ),
            Self::Engine(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "Cette action n'est pas possible maintenant.",
            ),
            Self::Db(_) | Self::Template(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Une erreur interne est survenue.",
            ),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if matches!(self, Self::NotLoggedIn) {
            return Redirect::to("/welcome").into_response();
        }
        let (status, message) = self.status_and_message();
        if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
        } else {
            tracing::info!(error = %self, "request refused");
        }
        let title = status.canonical_reason().unwrap_or("Erreur");
        match (ErrorPage { title, message }).render() {
            Ok(html) => (status, Html(html)).into_response(),
            Err(error) => {
                tracing::error!(%error, "error page failed to render");
                (status, message).into_response()
            }
        }
    }
}
