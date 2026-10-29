//! A private message between players of a board.

use crate::model::actor_ref::ActorRef;

/// A message sent from one actor to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Database id.
    pub id: i64,
    /// Sender.
    pub sender: ActorRef,
    /// Receiver.
    pub receiver: ActorRef,
    /// Text.
    pub text: String,
    /// Creation time, as stored by SQLite.
    pub created_at: String,
}
