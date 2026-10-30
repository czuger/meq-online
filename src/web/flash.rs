//! One-shot messages shown on the page after a redirect.

use axum::http::header::{LOCATION, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::web::cookies::{FLASH_COOKIE, decode, encode, read_cookie, set_cookie};

/// Kind of flash message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashKind {
    /// Information.
    Notice,
    /// Warning.
    Alert,
}

/// A message to display once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flash {
    /// Kind of message.
    pub kind: FlashKind,
    /// Text in French.
    pub text: String,
}

impl Flash {
    /// An information message.
    pub fn notice(text: impl Into<String>) -> Self {
        Self {
            kind: FlashKind::Notice,
            text: text.into(),
        }
    }

    /// A warning message.
    pub fn alert(text: impl Into<String>) -> Self {
        Self {
            kind: FlashKind::Alert,
            text: text.into(),
        }
    }

    /// Bootstrap classes of the message box.
    pub fn css_class(&self) -> &'static str {
        match self.kind {
            FlashKind::Notice => "flash",
            FlashKind::Alert => "flash flash-error",
        }
    }

    /// Flash message sent back by the browser, if any.
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let raw = decode(&read_cookie(headers, FLASH_COOKIE)?);
        let (kind, text) = raw.split_once('|')?;
        let kind = if kind == "alert" {
            FlashKind::Alert
        } else {
            FlashKind::Notice
        };
        Some(Self {
            kind,
            text: text.to_owned(),
        })
    }

    fn cookie_value(&self) -> String {
        let kind = match self.kind {
            FlashKind::Notice => "notice",
            FlashKind::Alert => "alert",
        };
        encode(&format!("{kind}|{}", self.text))
    }
}

/// A "303 See Other" redirect, optionally carrying a flash message.
pub fn redirect(to: &str, flash: Option<Flash>) -> Response {
    let mut response = (StatusCode::SEE_OTHER, [(LOCATION, to.to_owned())]).into_response();
    if let Some(flash) = flash {
        let cookie = set_cookie(FLASH_COOKIE, &flash.cookie_value());
        match cookie.parse() {
            Ok(value) => {
                response.headers_mut().insert(SET_COOKIE, value);
            }
            Err(error) => tracing::warn!(%error, "flash cookie dropped"),
        }
    }
    response
}
