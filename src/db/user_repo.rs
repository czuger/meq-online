//! Persistence of users, sessions and board membership.

use rand::RngExt;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqliteConnection};

use crate::db::db_error::DbError;
use crate::model::user::User;

/// Names of the computer seats.
const AI_USER_NAMES: [&str; 5] = ["IA Alice", "IA Bob", "IA Marc", "IA Luc", "IA Ida"];

/// Finds a user by provider identity, creating it on first login.
///
/// # Errors
/// Database errors.
pub async fn find_or_create(
    conn: &mut SqliteConnection,
    provider: &str,
    uid: &str,
    name: &str,
    email: &str,
) -> Result<User, DbError> {
    sqlx::query("INSERT INTO users (provider, uid, name, email) VALUES (?, ?, ?, ?) ON CONFLICT (provider, uid) DO NOTHING")
        .bind(provider)
        .bind(uid)
        .bind(name)
        .bind(email)
        .execute(&mut *conn)
        .await?;
    let row = sqlx::query("SELECT * FROM users WHERE provider = ? AND uid = ?")
        .bind(provider)
        .bind(uid)
        .fetch_one(conn)
        .await?;
    user_from_row(&row)
}

/// A user by id.
///
/// # Errors
/// Database errors.
pub async fn find(conn: &mut SqliteConnection, user_id: i64) -> Result<Option<User>, DbError> {
    let row = sqlx::query("SELECT * FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_optional(conn)
        .await?;
    row.as_ref().map(user_from_row).transpose()
}

/// Ids of the users of the first `count` computer seats, created when missing.
///
/// # Errors
/// Database errors.
pub async fn ai_users(conn: &mut SqliteConnection, count: usize) -> Result<Vec<i64>, DbError> {
    let mut ids = Vec::with_capacity(count);
    for (index, name) in AI_USER_NAMES.iter().enumerate().take(count) {
        let email = format!("{}@foo.bar", name.replace(' ', ""));
        let uid = format!("ai-{index}");
        sqlx::query(
            "INSERT INTO users (provider, uid, name, email, ai_index) VALUES ('ai', ?, ?, ?, ?) \
             ON CONFLICT (provider, uid) DO NOTHING",
        )
        .bind(&uid)
        .bind(name)
        .bind(&email)
        .bind(i64::try_from(index).unwrap_or(i64::MAX))
        .execute(&mut *conn)
        .await?;
        let id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE provider = 'ai' AND uid = ?")
            .bind(&uid)
            .fetch_one(&mut *conn)
            .await?;
        ids.push(id);
    }
    Ok(ids)
}

/// Opens a session for a user.
///
/// # Returns
/// The random session token to store in the cookie.
///
/// # Errors
/// Database errors.
pub async fn create_session(conn: &mut SqliteConnection, user_id: i64) -> Result<String, DbError> {
    let token = format!("{:032x}", rand::rng().random::<u128>());
    sqlx::query("INSERT INTO sessions (token, user_id) VALUES (?, ?)")
        .bind(&token)
        .bind(user_id)
        .execute(conn)
        .await?;
    Ok(token)
}

/// The user of a session token.
///
/// # Errors
/// Database errors.
pub async fn session_user(
    conn: &mut SqliteConnection,
    token: &str,
) -> Result<Option<User>, DbError> {
    let row = sqlx::query(
        "SELECT users.* FROM users JOIN sessions ON sessions.user_id = users.id WHERE token = ?",
    )
    .bind(token)
    .fetch_optional(conn)
    .await?;
    row.as_ref().map(user_from_row).transpose()
}

/// Closes a session.
///
/// # Errors
/// Database errors.
pub async fn delete_session(conn: &mut SqliteConnection, token: &str) -> Result<(), DbError> {
    sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(token)
        .execute(conn)
        .await?;
    Ok(())
}

/// Adds a user to a board's members.
///
/// # Errors
/// Database errors.
pub async fn add_member(
    conn: &mut SqliteConnection,
    board_id: i64,
    user_id: i64,
) -> Result<(), DbError> {
    sqlx::query("INSERT OR IGNORE INTO board_members (board_id, user_id) VALUES (?, ?)")
        .bind(board_id)
        .bind(user_id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Tells whether a user is a member of a board.
///
/// # Errors
/// Database errors.
pub async fn is_member(
    conn: &mut SqliteConnection,
    board_id: i64,
    user_id: i64,
) -> Result<bool, DbError> {
    let found: Option<i64> =
        sqlx::query_scalar("SELECT 1 FROM board_members WHERE board_id = ? AND user_id = ?")
            .bind(board_id)
            .bind(user_id)
            .fetch_optional(conn)
            .await?;
    Ok(found.is_some())
}

/// Boards a user is a member of, most recently updated first.
///
/// # Errors
/// Database errors.
pub async fn member_board_ids(
    conn: &mut SqliteConnection,
    user_id: i64,
) -> Result<Vec<i64>, DbError> {
    let ids = sqlx::query_scalar(
        "SELECT boards.id FROM boards JOIN board_members ON board_members.board_id = boards.id \
         WHERE board_members.user_id = ? ORDER BY boards.updated_at DESC, boards.id DESC",
    )
    .bind(user_id)
    .fetch_all(conn)
    .await?;
    Ok(ids)
}

fn user_from_row(row: &SqliteRow) -> Result<User, DbError> {
    Ok(User {
        id: row.try_get("id")?,
        provider: row.try_get("provider")?,
        uid: row.try_get("uid")?,
        name: row.try_get("name")?,
        email: row.try_get("email")?,
        ai_index: row.try_get("ai_index")?,
    })
}
