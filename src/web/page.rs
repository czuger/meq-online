//! Rendering templates into responses.

use askama::Template;
use axum::http::header::SET_COOKIE;
use axum::response::{Html, IntoResponse, Response};

use crate::web::app_error::AppError;
use crate::web::cookies::{FLASH_COOKIE, clear_cookie};
use crate::web::views::layout::Layout;

/// Renders a page; a displayed flash message is cleared so it shows only once.
///
/// # Errors
/// [`AppError::Template`] when rendering fails.
pub fn render(template: &impl Template, layout: &Layout) -> Result<Response, AppError> {
    let mut response = Html(template.render()?).into_response();
    if layout.flash.is_some() {
        match clear_cookie(FLASH_COOKIE).parse() {
            Ok(value) => {
                response.headers_mut().insert(SET_COOKIE, value);
            }
            Err(error) => tracing::warn!(%error, "flash cookie not cleared"),
        }
    }
    Ok(response)
}
