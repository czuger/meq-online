//! The Sauron player of a board.

use crate::game_data::combat_card_def::CardNumber;

/// Sauron's private state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sauron {
    /// Database id; `None` until persisted.
    pub id: Option<i64>,
    /// Owning user.
    pub user_id: i64,
    /// Sauron is expected to act.
    pub active: bool,
    /// Plot cards in hand.
    pub plot_hand: Vec<CardNumber>,
    /// Shadow cards in hand.
    pub shadow_hand: Vec<CardNumber>,
    /// Plot cards drawn, waiting to be kept or returned.
    pub drawn_plot: Vec<CardNumber>,
    /// Shadow cards drawn, waiting to be kept or discarded.
    pub drawn_shadow: Vec<CardNumber>,
}
