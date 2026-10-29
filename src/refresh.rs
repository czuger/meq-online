//! Live refresh: tells open pages that a board changed.

use tokio::sync::broadcast;

/// Message sent to open pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshMsg {
    /// A board changed.
    Board(i64),
    /// The server is stopping; streams must end.
    Shutdown,
}

/// Broadcasts board changes to every open event stream.
#[derive(Debug, Clone)]
pub struct RefreshHub {
    sender: broadcast::Sender<RefreshMsg>,
}

/// Messages kept for slow subscribers before they lag.
const CAPACITY: usize = 64;

impl RefreshHub {
    /// Creates a hub with no subscriber.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CAPACITY);
        Self { sender }
    }

    /// Announces that a board changed.
    pub fn board_changed(&self, board_id: i64) {
        // An error only means nobody is listening right now.
        if self.sender.send(RefreshMsg::Board(board_id)).is_err() {
            tracing::trace!(board_id, "no refresh subscriber");
        }
    }

    /// Ends every open stream, so the server can shut down.
    pub fn shutdown(&self) {
        if self.sender.send(RefreshMsg::Shutdown).is_err() {
            tracing::trace!("no refresh subscriber at shutdown");
        }
    }

    /// Subscribes to future messages.
    pub fn subscribe(&self) -> broadcast::Receiver<RefreshMsg> {
        self.sender.subscribe()
    }
}

impl Default for RefreshHub {
    fn default() -> Self {
        Self::new()
    }
}
