//! Layouts of a seat's screens, framed by the game table: loads the users of the
//! seats and the latest log lines the table shows.

use crate::app_state::AppState;
use crate::db::{DbError, log_repo, user_repo};
use crate::engine::board_state::BoardState;
use crate::engine::route::{RouteActor, route_for, screen_side};
use crate::model::choice::Chooser;
use crate::model::game::Game;
use crate::web::app_error::AppError;
use crate::web::session::Session;
use crate::web::views::layout::Layout;
use crate::web::views::seat_view::{Players, controls};
use crate::web::views::table_view::TableView;
use crate::web::views::turn_log::TurnLog;

/// Log entries read to tell the latest turns next to the map.
const TURN_LOG_ENTRIES: i64 = 300;

/// Users of the board's seats.
///
/// # Errors
/// [`AppError::Db`] on database failure.
pub async fn board_players(state: &AppState, game: &Game) -> Result<Players, AppError> {
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let mut players = Players::new();
    for user_id in std::iter::once(game.sauron.user_id).chain(game.heroes.iter().map(|h| h.user_id))
    {
        tracing::trace!(user_id, "loading player");
        if players.contains_key(&user_id) {
            continue;
        }
        if let Some(user) = user_repo::find(&mut conn, user_id).await? {
            players.insert(user_id, user);
        }
    }
    Ok(players)
}

/// The game table as seen by `viewer`.
///
/// # Errors
/// [`AppError::Db`] on database failure.
pub async fn table(
    state: &AppState,
    session: &Session,
    game: &Game,
    viewer: RouteActor,
) -> Result<TableView, AppError> {
    let players = board_players(state, game).await?;
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let board_id = game.board.id.unwrap_or_default();
    let logs = log_repo::recent(&mut conn, board_id, TURN_LOG_ENTRIES).await?;
    let entries: Vec<_> = logs.iter().rev().map(|log| &log.entry).collect();
    let turn_log = TurnLog::new(&entries, game, &state.data);
    Ok(TableView::new(
        game,
        &state.data,
        &players,
        &session.user,
        viewer,
        turn_log,
        state.versions.version(board_id),
    ))
}

/// Layout of a hero's screen, framed by the table.
///
/// # Errors
/// [`AppError::Db`] on database failure.
pub async fn hero_layout(
    state: &AppState,
    session: &Session,
    game: &Game,
    hero_id: i64,
) -> Result<Layout, AppError> {
    let table = table(state, session, game, RouteActor::Hero(hero_id)).await?;
    Ok(Layout::hero(session, game, hero_id, &state.data).with_table(table))
}

/// Layout of a Sauron screen, framed by the table.
///
/// # Errors
/// [`AppError::Db`] on database failure.
pub async fn sauron_layout(
    state: &AppState,
    session: &Session,
    game: &Game,
) -> Result<Layout, AppError> {
    let viewer = RouteActor::Sauron(game.sauron.id.unwrap_or_default());
    let table = table(state, session, game, viewer).await?;
    Ok(Layout::sauron(session, game).with_table(table))
}

/// Layout of a board screen shared by both sides (combat, choices), framed by the
/// table of the seat the user plays; a plain board layout for onlookers.
///
/// # Errors
/// [`AppError::Db`] on database failure.
pub async fn viewer_layout(
    state: &AppState,
    session: &Session,
    game: &Game,
) -> Result<Layout, AppError> {
    let players = board_players(state, game).await?;
    match viewer_seat(game, &players, session) {
        Some(RouteActor::Hero(id)) => hero_layout(state, session, game, id).await,
        Some(RouteActor::Sauron(_)) => sauron_layout(state, session, game).await,
        None => Ok(Layout::board(session, game)),
    }
}

/// The seat whose table the user sees: an active seat they play, else their Sauron,
/// their own hero, or a computer hero they play.
pub fn viewer_seat(game: &Game, players: &Players, session: &Session) -> Option<RouteActor> {
    let me = &session.user;
    let sauron = game.sauron.id.map(RouteActor::Sauron);
    let plays_sauron = game.sauron.user_id == me.id;
    let hero = |active: bool, own: bool| {
        game.heroes
            .iter()
            .find(|h| {
                (!active || h.active)
                    && if own {
                        h.user_id == me.id
                    } else {
                        controls(h.user_id, me, players)
                    }
            })
            .and_then(|h| h.id)
            .map(RouteActor::Hero)
    };
    if plays_sauron && game.sauron.active {
        return sauron;
    }
    hero(true, false)
        .or_else(|| plays_sauron.then_some(sauron).flatten())
        .or_else(|| hero(false, true))
        .or_else(|| hero(false, false))
}

/// Screen of a seat the user plays that the game waits on, if any.
pub fn my_screen(game: &Game, players: &Players, session: &Session) -> Option<String> {
    if game.board.winner.is_some() {
        return None;
    }
    let side = screen_side(game.board.state);
    let actor = acting_seats(game, players, session)
        .into_iter()
        .find(|actor| side.includes(*actor))?;
    Some(route_for(
        game.board.state,
        game.board.id.unwrap_or_default(),
        actor,
    ))
}

/// Seats the user plays that the game waits on, Sauron first: the chooser of a
/// pending choice, otherwise the active seats.
fn acting_seats(game: &Game, players: &Players, session: &Session) -> Vec<RouteActor> {
    let me = &session.user;
    let sauron = game
        .sauron
        .id
        .filter(|_| game.sauron.user_id == me.id)
        .map(RouteActor::Sauron);
    let heroes = |wanted: &dyn Fn(i64) -> bool| -> Vec<RouteActor> {
        game.heroes
            .iter()
            .filter(|h| controls(h.user_id, me, players))
            .filter_map(|h| h.id.filter(|id| wanted(*id)))
            .map(RouteActor::Hero)
            .collect()
    };
    let choice = game
        .board
        .pending_choice
        .as_ref()
        .filter(|_| game.board.state == BoardState::ChoiceScreenBoardChoices);
    match choice.map(|c| c.chooser) {
        Some(Chooser::Sauron) => sauron.into_iter().collect(),
        Some(Chooser::Hero(hero_id)) => heroes(&|id| id == hero_id),
        Some(Chooser::Heroes) => heroes(&|_| true),
        None => {
            let active = |id: i64| game.heroes.iter().any(|h| h.id == Some(id) && h.active);
            sauron
                .filter(|_| game.sauron.active)
                .into_iter()
                .chain(heroes(&active))
                .collect()
        }
    }
}
