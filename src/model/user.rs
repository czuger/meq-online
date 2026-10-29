//! A registered user.

/// A user who can join boards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// Database id.
    pub id: i64,
    /// Login provider (`dev`, `ai`, ...).
    pub provider: String,
    /// Identifier at the provider.
    pub uid: String,
    /// Display name.
    pub name: String,
    /// Email address.
    pub email: String,
    /// Index of the computer seat this user stands for, if any.
    pub ai_index: Option<i64>,
}
