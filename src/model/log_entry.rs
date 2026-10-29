//! A line of the game log.

use serde_json::{Map, Value};

use crate::model::actor_ref::ActorRef;

/// An event recorded in the board log.
#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    /// Who did it.
    pub actor: ActorRef,
    /// Dotted action key (`combat.inc_strength`, ...), used to pick the message.
    pub action: String,
    /// Values interpolated in the message.
    pub params: Map<String, Value>,
}

impl LogEntry {
    /// Builds a log entry.
    ///
    /// # Arguments
    /// * `actor` - who did it.
    /// * `action` - dotted action key.
    /// * `params` - `(name, value)` pairs interpolated in the message.
    pub fn new(actor: ActorRef, action: impl Into<String>, params: &[(&str, Value)]) -> Self {
        Self {
            actor,
            action: action.into(),
            params: params
                .iter()
                .map(|(name, value)| ((*name).to_owned(), value.clone()))
                .collect(),
        }
    }
}
