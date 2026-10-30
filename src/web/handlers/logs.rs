//! Game log.

use std::collections::BTreeMap;

use askama::Template;
use axum::extract::State;
use axum::response::Response;

use crate::app_state::AppState;
use crate::db::{DbError, log_repo, user_repo};
use crate::model::actor_ref::ActorRef;
use crate::web::access::BoardAccess;
use crate::web::app_error::AppError;
use crate::web::game_tx::read_game;
use crate::web::page::render;
use crate::web::views::layout::Layout;
use crate::web::views::log_text::{log_card_pic, log_text};

struct LogRow {
    /// `(user name, actor name)`, or `None` for automatic steps.
    actor: Option<(String, String)>,
    text: String,
    card_pic: Option<String>,
}

#[derive(Template)]
#[template(path = "logs_show.html")]
struct LogsPage {
    layout: Layout,
    rows: Vec<LogRow>,
}

/// `GET /boards/{board_id}/logs`: the log, newest first.
///
/// # Errors
/// [`AppError::Db`] on database failure.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id), err(level = "debug"))]
pub async fn show(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let logs = log_repo::for_board(&mut conn, access.board_id).await?;
    let mut names = BTreeMap::new();
    for user_id in std::iter::once(game.sauron.user_id).chain(game.heroes.iter().map(|h| h.user_id))
    {
        tracing::trace!(user_id, "loading player name");
        if let Some(user) = user_repo::find(&mut conn, user_id).await? {
            names.insert(user_id, user.name);
        }
    }
    let user_name = |id: i64| names.get(&id).cloned().unwrap_or_default();
    let rows = logs
        .iter()
        .map(|log| {
            let actor = match log.entry.actor {
                ActorRef::System => None,
                ActorRef::Sauron => Some((user_name(game.sauron.user_id), "Sauron".to_owned())),
                ActorRef::Hero(id) => game.heroes.iter().find(|h| h.id == Some(id)).map(|h| {
                    let name = state
                        .data
                        .heroes
                        .get(&h.code)
                        .map_or_else(|| h.code.clone(), |d| d.name.clone());
                    (user_name(h.user_id), name)
                }),
            };
            LogRow {
                actor,
                text: log_text(&log.entry, &state.data),
                card_pic: log_card_pic(&log.entry),
            }
        })
        .collect();
    let layout = Layout::board(&access.session, &game);
    render(
        &LogsPage {
            layout: layout.clone(),
            rows,
        },
        &layout,
    )
}
