//! Board list, creation, joining and story track.

use std::collections::BTreeMap;

use askama::Template;
use axum::extract::{Query, State};
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::db::{DbError, board_repo, game_repo, user_repo};
use crate::engine::board_state::BoardState;
use crate::engine::route::{RouteActor, route_for};
use crate::engine::setup::new_game;
use crate::engine::story::Dominance;
use crate::model::board::Winner;
use crate::model::game::Game;
use crate::model::user::User;
use crate::web::access::BoardAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::game_tx::read_game;
use crate::web::page::render;
use crate::web::session::Session;
use crate::web::views::layout::Layout;
use crate::web::views::state_label::state_label;

/// Hero columns of the board list, in display order.
const HERO_COLUMNS: [&str; 5] = ["argalad", "beravor", "eleanor", "eometh", "thalin"];

/// A player cell: a link when it is this player's turn.
struct PlayerCell {
    name: String,
    url: Option<String>,
}

struct BoardRow {
    id: i64,
    players: String,
    turn: i32,
    status: String,
    can_join: bool,
    sauron: Option<PlayerCell>,
    heroes: Vec<Option<PlayerCell>>,
}

#[derive(Template)]
#[template(path = "boards_index.html")]
struct IndexPage {
    layout: Layout,
    rows: Vec<BoardRow>,
}

#[derive(Template)]
#[template(path = "welcome.html")]
struct WelcomePage {
    layout: Layout,
}

#[derive(Template)]
#[template(path = "boards_new.html")]
struct NewPage {
    layout: Layout,
}

#[derive(Template)]
#[template(path = "boards_join.html")]
struct JoinPage {
    layout: Layout,
    board_id: i64,
}

#[derive(Template)]
#[template(path = "boards_inactive.html")]
struct InactivePage {
    layout: Layout,
    state: &'static str,
}

struct Counter {
    label: &'static str,
    value: String,
}

#[derive(Template)]
#[template(path = "boards_story.html")]
struct StoryPage {
    layout: Layout,
    markers: Vec<Counter>,
    distances: Vec<Counter>,
    stage: i32,
}

/// Query of the board list.
#[derive(Debug, Deserialize)]
pub struct IndexQuery {
    all: Option<String>,
}

/// `GET /welcome`.
///
/// # Errors
/// [`AppError::Template`] when rendering fails.
pub async fn welcome(headers: axum::http::HeaderMap) -> Result<Response, AppError> {
    let layout = Layout {
        flash: Flash::from_headers(&headers),
        ..Layout::anonymous()
    };
    render(
        &WelcomePage {
            layout: layout.clone(),
        },
        &layout,
    )
}

/// `GET /boards` and `GET /`: the user's boards, or every board with `?all=true`.
///
/// # Errors
/// [`AppError::Db`] on database failure.
pub async fn index(
    State(state): State<AppState>,
    session: Session,
    Query(query): Query<IndexQuery>,
) -> Result<Response, AppError> {
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let ids = if query.all.is_some() {
        board_repo::all_ids(&mut conn).await?
    } else {
        user_repo::member_board_ids(&mut conn, session.user.id).await?
    };
    let mut users = BTreeMap::new();
    let mut rows = Vec::with_capacity(ids.len());
    for id in ids {
        let game = game_repo::load(&mut conn, id).await?;
        for user_id in
            std::iter::once(game.sauron.user_id).chain(game.heroes.iter().map(|h| h.user_id))
        {
            if !users.contains_key(&user_id)
                && let Some(user) = user_repo::find(&mut conn, user_id).await?
            {
                users.insert(user_id, user);
            }
        }
        rows.push(board_row(&game, &users, &session.user, &state));
    }
    let layout = Layout::user(&session);
    render(
        &IndexPage {
            layout: layout.clone(),
            rows,
        },
        &layout,
    )
}

fn board_row(game: &Game, users: &BTreeMap<i64, User>, me: &User, state: &AppState) -> BoardRow {
    let user_name = |id: i64| users.get(&id).map_or_else(String::new, |u| u.name.clone());
    let mut players = vec![format!("Sauron ({})", user_name(game.sauron.user_id))];
    players.extend(
        game.heroes
            .iter()
            .map(|h| format!("{} ({})", hero_name(state, &h.code), user_name(h.user_id))),
    );
    let status = match game.board.winner {
        Some(Winner::Heroes) => "Victoire des héros",
        Some(Winner::Sauron) => "Victoire de Sauron",
        Some(Winner::Draw) => "Égalité",
        None => "En cours",
    };
    BoardRow {
        id: game.board.id.unwrap_or_default(),
        players: players.join(", "),
        turn: game.board.turn,
        status: status.to_owned(),
        can_join: i32::try_from(game.heroes.len()).unwrap_or(i32::MAX) < game.board.max_heroes,
        sauron: sauron_cell(game, me),
        heroes: hero_cells(game, users, me, state),
    }
}

fn hero_name(state: &AppState, code: &str) -> String {
    state
        .data
        .heroes
        .get(code)
        .map_or_else(|| code.to_owned(), |h| h.name.clone())
}

/// Link to the current screen for an active actor of a running game.
fn actor_url(game: &Game, active: bool, actor: RouteActor) -> Option<String> {
    let playing = game.board.state != BoardState::Finished;
    (active && playing)
        .then(|| route_for(game.board.state, game.board.id.unwrap_or_default(), actor))
}

fn sauron_cell(game: &Game, me: &User) -> Option<PlayerCell> {
    (game.sauron.user_id == me.id).then(|| PlayerCell {
        name: "Sauron".to_owned(),
        url: actor_url(
            game,
            game.sauron.active,
            RouteActor::Sauron(game.sauron.id.unwrap_or_default()),
        ),
    })
}

/// Hero cells in [`HERO_COLUMNS`] order; computer seats are played by the board members.
fn hero_cells(
    game: &Game,
    users: &BTreeMap<i64, User>,
    me: &User,
    state: &AppState,
) -> Vec<Option<PlayerCell>> {
    let controls = |user_id: i64| {
        user_id == me.id || users.get(&user_id).is_some_and(|u| u.ai_index.is_some())
    };
    HERO_COLUMNS
        .iter()
        .map(|code| {
            let hero = game
                .heroes
                .iter()
                .find(|h| h.code == *code && controls(h.user_id))?;
            Some(PlayerCell {
                name: hero_name(state, code),
                url: actor_url(
                    game,
                    hero.active,
                    RouteActor::Hero(hero.id.unwrap_or_default()),
                ),
            })
        })
        .collect()
}

/// `GET /boards/new`.
///
/// # Errors
/// [`AppError::Template`] when rendering fails.
pub async fn new(session: Session) -> Result<Response, AppError> {
    let layout = Layout::user(&session);
    render(
        &NewPage {
            layout: layout.clone(),
        },
        &layout,
    )
}

/// `POST /boards`: creates a game where the user plays Sauron and computer seats play
/// the heroes.
///
/// # Errors
/// [`AppError::Engine`] when the static data can't build a game; database errors.
pub async fn create(State(state): State<AppState>, session: Session) -> Result<Response, AppError> {
    let mut tx = state.db.begin().await.map_err(DbError::from)?;
    let ai = user_repo::ai_users(&mut tx, 4).await?;
    let seats = [0, 1, 2, 3].map(|i| ai.get(i).copied().unwrap_or(session.user.id));
    let mut rng: rand::rngs::StdRng = rand::make_rng();
    let mut game = new_game(session.user.id, seats, &state.data, &mut rng)?;
    game_repo::save(&mut tx, &mut game).await?;
    let board_id = game.board.id.ok_or(AppError::NotFound)?;
    user_repo::add_member(&mut tx, board_id, session.user.id).await?;
    tx.commit().await.map_err(DbError::from)?;
    Ok(redirect(
        "/boards",
        Some(Flash::notice("La partie a été créée.")),
    ))
}

/// `GET /boards/{board_id}/join`.
///
/// # Errors
/// [`AppError::Template`] when rendering fails.
pub async fn join_new(
    session: Session,
    axum::extract::Path(board_id): axum::extract::Path<i64>,
) -> Result<Response, AppError> {
    let layout = Layout::user(&session);
    render(
        &JoinPage {
            layout: layout.clone(),
            board_id,
        },
        &layout,
    )
}

/// `POST /boards/{board_id}/join`: every seat is taken at creation, so joining is refused.
pub async fn join(_session: Session) -> Response {
    redirect("/boards", Some(Flash::alert("Cette partie est complète.")))
}

/// `GET /boards/{board_id}/inactive_actor`.
///
/// # Errors
/// [`AppError::Db`] when the board can't be loaded.
pub async fn inactive_actor(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let layout = Layout::board(&access.session, &game);
    render(
        &InactivePage {
            layout: layout.clone(),
            state: state_label(game.board.state),
        },
        &layout,
    )
}

/// `GET /boards/{board_id}/story_screen`.
///
/// # Errors
/// [`AppError::Db`] when the board can't be loaded.
pub async fn story_screen(
    State(state): State<AppState>,
    access: BoardAccess,
) -> Result<Response, AppError> {
    let game = read_game(&state, access.board_id).await?;
    let story = game.board.story;
    let data = story.data();
    let counter = |label, value: i32| Counter {
        label,
        value: value.to_string(),
    };
    let dominance = match data.dominance {
        Dominance::Heroes => "Héros",
        Dominance::Sauron => "Sauron",
        Dominance::Nobody => "Personne",
    };
    let page = StoryPage {
        layout: Layout::board(&access.session, &game),
        markers: vec![
            counter("Marqueur des héros", story.heroes),
            counter("Marqueur de l'Anneau", story.ring),
            counter("Marqueur de la conquête", story.conquest),
            counter("Marqueur de la corruption", story.corruption),
        ],
        distances: vec![
            counter("Héros jusqu'à la fin", data.heroes_to_final),
            counter("Sauron jusqu'à la fin", data.sauron_to_final),
            counter("Sauron jusqu'à l'Ombre", data.sauron_to_shadowfall),
            Counter {
                label: "Domination",
                value: dominance.to_owned(),
            },
        ],
        stage: story.stage(),
    };
    render(&page, &page.layout)
}
