//! Persistence of the `messages` table.

use sqlx::{Row, SqliteConnection};

use crate::db::columns::{actor_from_columns, actor_to_columns};
use crate::db::db_error::DbError;
use crate::model::actor_ref::ActorRef;
use crate::model::message::Message;

/// Messages sent or received by an actor of a board, newest first.
///
/// # Errors
/// Database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id, ?actor), err(level = "debug"))]
pub async fn for_actor(
    conn: &mut SqliteConnection,
    board_id: i64,
    actor: ActorRef,
) -> Result<Vec<Message>, DbError> {
    let (kind, hero_id) = actor_to_columns(actor);
    let rows = sqlx::query(
        "SELECT * FROM messages WHERE board_id = ? AND \
         ((sender_kind = ? AND sender_hero_id IS ?) OR (receiver_kind = ? AND receiver_hero_id IS ?)) \
         ORDER BY id DESC",
    )
    .bind(board_id)
    .bind(kind)
    .bind(hero_id)
    .bind(kind)
    .bind(hero_id)
    .fetch_all(conn)
    .await?;
    rows.iter()
        .map(|row| {
            let sender_kind: String = row.try_get("sender_kind")?;
            let receiver_kind: String = row.try_get("receiver_kind")?;
            Ok(Message {
                id: row.try_get("id")?,
                sender: actor_from_columns(&sender_kind, row.try_get("sender_hero_id")?),
                receiver: actor_from_columns(&receiver_kind, row.try_get("receiver_hero_id")?),
                text: row.try_get("text")?,
                created_at: row.try_get("created_at")?,
            })
        })
        .collect()
}

/// Stores a message.
///
/// # Errors
/// Database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id, ?sender, ?receiver, %text), err(level = "debug"))]
pub async fn create(
    conn: &mut SqliteConnection,
    board_id: i64,
    sender: ActorRef,
    receiver: ActorRef,
    text: &str,
) -> Result<(), DbError> {
    let (sender_kind, sender_hero) = actor_to_columns(sender);
    let (receiver_kind, receiver_hero) = actor_to_columns(receiver);
    sqlx::query(
        "INSERT INTO messages (board_id, sender_kind, sender_hero_id, receiver_kind, receiver_hero_id, text) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(board_id)
    .bind(sender_kind)
    .bind(sender_hero)
    .bind(receiver_kind)
    .bind(receiver_hero)
    .bind(text)
    .execute(conn)
    .await?;
    Ok(())
}
