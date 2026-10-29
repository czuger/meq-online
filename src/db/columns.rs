//! Conversions between Rust values and SQLite columns.

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::db::db_error::DbError;
use crate::model::actor_ref::ActorRef;

/// Encodes a value as a JSON text column.
///
/// # Errors
/// [`DbError::Json`] when the value can't be serialized.
pub fn to_json<T: Serialize>(value: &T) -> Result<String, DbError> {
    Ok(serde_json::to_string(value)?)
}

/// Decodes a JSON text column.
///
/// # Errors
/// [`DbError::Json`] when the text doesn't match `T`.
pub fn from_json<T: DeserializeOwned>(text: &str) -> Result<T, DbError> {
    Ok(serde_json::from_str(text)?)
}

/// Stores a unit enum as its serde name (`"monster"`, `"dark_blue"`, ...).
///
/// # Errors
/// [`DbError::Json`] when the value doesn't serialize to a string.
pub fn enum_to_text<T: Serialize>(value: &T) -> Result<String, DbError> {
    match serde_json::to_value(value)? {
        serde_json::Value::String(text) => Ok(text),
        other => Ok(other.to_string()),
    }
}

/// Reads a unit enum stored by [`enum_to_text`].
///
/// # Errors
/// [`DbError::Json`] when the text isn't a variant of `T`.
pub fn enum_from_text<T: DeserializeOwned>(text: &str) -> Result<T, DbError> {
    Ok(serde_json::from_value(serde_json::Value::String(
        text.to_owned(),
    ))?)
}

/// Splits an actor into its `(kind, hero_id)` columns.
pub fn actor_to_columns(actor: ActorRef) -> (&'static str, Option<i64>) {
    match actor {
        ActorRef::System => ("system", None),
        ActorRef::Sauron => ("sauron", None),
        ActorRef::Hero(id) => ("hero", Some(id)),
    }
}

/// Rebuilds an actor from its `(kind, hero_id)` columns.
///
/// A hero whose row was deleted reads back as [`ActorRef::System`].
pub fn actor_from_columns(kind: &str, hero_id: Option<i64>) -> ActorRef {
    match (kind, hero_id) {
        ("sauron", _) => ActorRef::Sauron,
        ("hero", Some(id)) => ActorRef::Hero(id),
        _ => ActorRef::System,
    }
}
