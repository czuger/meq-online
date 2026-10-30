//! Developer login and logout.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::header::{LOCATION, SET_COOKIE};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::db::{DbError, user_repo};
use crate::web::app_error::AppError;
use crate::web::cookies::{SESSION_COOKIE, clear_cookie, read_cookie, set_cookie};
use crate::web::flash::{Flash, redirect};
use crate::web::page::render;
use crate::web::views::layout::Layout;

/// Provider name of developer logins.
const DEV_PROVIDER: &str = "developer";

#[derive(Template)]
#[template(path = "sessions_new.html")]
struct NewSessionPage {
    layout: Layout,
}

/// Login form.
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    name: String,
    email: String,
}

/// `GET /sessions/new`: the developer login form.
///
/// # Errors
/// [`AppError::Template`] when rendering fails.
#[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
pub async fn new(headers: HeaderMap) -> Result<Response, AppError> {
    let layout = Layout {
        flash: Flash::from_headers(&headers),
        ..Layout::anonymous()
    };
    render(
        &NewSessionPage {
            layout: layout.clone(),
        },
        &layout,
    )
}

/// `POST /auth/developer/callback`: logs in with a name and email (no password).
///
/// # Errors
/// [`AppError::Db`] on database failure.
#[tracing::instrument(level = "debug", skip_all, fields(name = %form.name), err(level = "debug"))]
pub async fn create(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    let (name, email) = (form.name.trim(), form.email.trim());
    if name.is_empty() || email.is_empty() {
        return Ok(redirect(
            "/sessions/new",
            Some(Flash::alert(
                "Le nom et l'adresse e-mail sont obligatoires.",
            )),
        ));
    }
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let user = user_repo::find_or_create(&mut conn, DEV_PROVIDER, email, name, email).await?;
    let token = user_repo::create_session(&mut conn, user.id).await?;
    Ok(with_cookie("/boards", &set_cookie(SESSION_COOKIE, &token)))
}

/// `GET /auth/failure`.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn failure() -> Response {
    redirect(
        "/sessions/new",
        Some(Flash::alert(
            "L'authentification a échoué, veuillez réessayer.",
        )),
    )
}

/// `DELETE /sessions` (or `POST` from a form): logs out.
///
/// # Errors
/// [`AppError::Db`] on database failure.
#[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
pub async fn destroy(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    if let Some(token) = read_cookie(&headers, SESSION_COOKIE) {
        let mut conn = state.db.acquire().await.map_err(DbError::from)?;
        user_repo::delete_session(&mut conn, &token).await?;
    }
    Ok(with_cookie("/", &clear_cookie(SESSION_COOKIE)))
}

fn with_cookie(to: &str, cookie: &str) -> Response {
    let mut response = (
        axum::http::StatusCode::SEE_OTHER,
        [(LOCATION, to.to_owned())],
    )
        .into_response();
    match cookie.parse() {
        Ok(value) => {
            response.headers_mut().insert(SET_COOKIE, value);
        }
        Err(error) => tracing::error!(%error, "session cookie dropped"),
    }
    response
}
