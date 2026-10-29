//! Server-sent events telling open pages to reload.

use std::convert::Infallible;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};

use crate::app_state::AppState;
use crate::refresh::RefreshMsg;
use crate::web::access::BoardAccess;

/// `GET /boards/{board_id}/events`: a `refresh` event each time the board changes.
///
/// A subscriber that falls behind also gets a `refresh`. The stream ends when the
/// server shuts down.
pub async fn stream(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let board_id = access.board_id;
    let events = BroadcastStream::new(state.refresh.subscribe())
        .take_while(|msg| !matches!(msg, Ok(RefreshMsg::Shutdown)))
        .filter(move |msg| match msg {
            Ok(RefreshMsg::Board(id)) => *id == board_id,
            Ok(RefreshMsg::Shutdown) => false,
            Err(_lagged) => true,
        })
        .map(|_| Ok(Event::default().event("refresh").data("refresh")));
    Sse::new(events).keep_alive(KeepAlive::default())
}
