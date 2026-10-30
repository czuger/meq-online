//! Board list, creation, joining and story track.

use std::collections::BTreeMap;

use askama::Template;
use axum::Form;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::db::{DbError, board_repo, game_repo, user_repo};
use crate::engine::board_state::BoardState;
use crate::engine::missions::objective_pic_path;
use crate::engine::route::{RouteActor, route_for};
use crate::engine::setup::{HeroSeat, MAX_HEROES, new_game};
use crate::engine::story::Dominance;
use crate::model::board::Winner;
use crate::model::game::Game;
use crate::model::game_options::GameOptions;
use crate::model::user::User;
use crate::web::access::BoardAccess;
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::game_tx::read_game;
use crate::web::page::render;
use crate::web::session::Session;
use crate::web::views::layout::Layout;
use crate::web::views::mission_label::{heroes_mission_name, sauron_mission_name};
use crate::web::views::pictures::image;
use crate::web::views::seat_view::controls;

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
    /// The user plays a seat of the board.
    member: bool,
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
    heroes: Vec<(&'static str, String)>,
}

#[derive(Template)]
#[template(path = "boards_join.html")]
struct JoinPage {
    layout: Layout,
    board_id: i64,
    heroes: Vec<(String, String)>,
}

struct Counter {
    label: &'static str,
    value: String,
}

/// A Mission card shown to its side.
struct MissionView {
    side: &'static str,
    name: &'static str,
    pic: String,
}

#[derive(Template)]
#[template(path = "boards_story.html")]
struct StoryPage {
    layout: Layout,
    markers: Vec<Counter>,
    distances: Vec<Counter>,
    stage: i32,
    winner: Option<&'static str>,
    missions: Vec<MissionView>,
    options: Vec<&'static str>,
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
#[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
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
        tracing::trace!(board_id = id, "listing board");
        let game = game_repo::load(&mut conn, id).await?;
        for user_id in
            std::iter::once(game.sauron.user_id).chain(game.heroes.iter().map(|h| h.user_id))
        {
            tracing::trace!(user_id, "board player");
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
    let status = winner_label(game).unwrap_or("En cours");
    BoardRow {
        id: game.board.id.unwrap_or_default(),
        players: players.join(", "),
        turn: game.board.turn,
        status: status.to_owned(),
        can_join: !game.board.waiting_heroes.is_empty(),
        member: game.sauron.user_id == me.id || game.heroes.iter().any(|h| h.user_id == me.id),
        sauron: sauron_cell(game, me),
        heroes: hero_cells(game, users, me, state),
    }
}

fn winner_label(game: &Game) -> Option<&'static str> {
    game.board.winner.map(|winner| match winner {
        Winner::Heroes => "Victoire des héros",
        Winner::Sauron => "Victoire de Sauron",
    })
}

/// Missions the user may see: each side sees its own, everyone sees both once the
/// game is over (§ 3, § 9).
fn visible_missions(game: &Game, user_id: i64) -> Vec<MissionView> {
    let over = game.board.winner.is_some();
    let sauron = over || game.sauron.user_id == user_id;
    let heroes =
        over || game.sauron.user_id != user_id || game.heroes.iter().any(|h| h.user_id == user_id);
    let mut missions = Vec::new();
    if let (true, Some(card), Some(mission)) =
        (sauron, game.board.sauron_objective, game.sauron_mission())
    {
        missions.push(MissionView {
            side: "Mission de Sauron",
            name: sauron_mission_name(mission),
            pic: image(&objective_pic_path("sauron", card)),
        });
    }
    if let (true, Some(card), Some(mission)) =
        (heroes, game.board.heroes_objective, game.heroes_mission())
    {
        missions.push(MissionView {
            side: "Mission des héros",
            name: heroes_mission_name(mission),
            pic: image(&objective_pic_path("heroes", card)),
        });
    }
    missions
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
    HERO_COLUMNS
        .iter()
        .map(|code| {
            let hero = game
                .heroes
                .iter()
                .find(|h| h.code == *code && controls(h.user_id, me, users))?;
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
#[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
pub async fn new(State(state): State<AppState>, session: Session) -> Result<Response, AppError> {
    let layout = Layout::user(&session);
    let heroes = HERO_COLUMNS
        .iter()
        .map(|code| (*code, hero_name(&state, code)))
        .collect();
    render(
        &NewPage {
            layout: layout.clone(),
            heroes,
        },
        &layout,
    )
}

/// `POST /boards`: creates a game where the user plays Sauron. The chosen heroes are
/// open seats for players to join (`seats=open`), or computer seats played by the
/// board members.
///
/// # Errors
/// [`AppError::Engine`] for a wrong choice of heroes; database errors.
#[tracing::instrument(level = "debug", skip_all, fields(fields = ?fields), err(level = "debug"))]
pub async fn create(
    State(state): State<AppState>,
    session: Session,
    Form(fields): Form<Vec<(String, String)>>,
) -> Result<Response, AppError> {
    let codes: Vec<&str> = HERO_COLUMNS
        .into_iter()
        .filter(|code| fields.iter().any(|(k, v)| k == "heroes" && v == code))
        .collect();
    let open = fields.iter().any(|(k, v)| k == "seats" && v == "open");
    let checked = |name: &str| fields.iter().any(|(k, _)| k == name);
    let options = GameOptions {
        advanced_dominance: checked("advanced_dominance"),
        high_stakes: checked("high_stakes"),
    };
    if codes.is_empty() || codes.len() > MAX_HEROES {
        return Ok(redirect(
            "/boards/new",
            Some(Flash::alert("Choisissez de un à trois héros.")),
        ));
    }
    let mut tx = state.db.begin().await.map_err(DbError::from)?;
    let ai = user_repo::ai_users(&mut tx, codes.len()).await?;
    let seats: Vec<HeroSeat<'_>> = codes
        .iter()
        .zip(ai)
        .map(|(code, user_id)| HeroSeat {
            code,
            user_id,
            open,
        })
        .collect();
    let mut rng: rand::rngs::StdRng = rand::make_rng();
    let mut game = new_game(session.user.id, &seats, &state.data, &mut rng)?;
    game.board.options = options;
    game_repo::save(&mut tx, &mut game).await?;
    let board_id = game.board.id.ok_or(AppError::NotFound)?;
    user_repo::add_member(&mut tx, board_id, session.user.id).await?;
    tx.commit().await.map_err(DbError::from)?;
    Ok(redirect(
        &format!("/boards/{board_id}/play"),
        Some(Flash::notice("La partie a été créée.")),
    ))
}

/// `GET /boards/{board_id}/join`: the heroes still waiting for a player.
///
/// # Errors
/// [`AppError::Db`] when the board can't be loaded.
#[tracing::instrument(level = "debug", skip_all, fields(board_id), err(level = "debug"))]
pub async fn join_new(
    State(state): State<AppState>,
    session: Session,
    Path(board_id): Path<i64>,
) -> Result<Response, AppError> {
    let game = read_game(&state, board_id).await?;
    let layout = Layout::user(&session);
    let heroes = game
        .board
        .waiting_heroes
        .iter()
        .map(|code| (code.clone(), hero_name(&state, code)))
        .collect();
    render(
        &JoinPage {
            layout: layout.clone(),
            board_id,
            heroes,
        },
        &layout,
    )
}

/// Hero chosen when joining.
#[derive(Debug, Deserialize)]
pub struct JoinForm {
    hero: String,
}

/// `POST /boards/{board_id}/join`: the user takes an open hero seat.
///
/// # Errors
/// [`AppError::Engine`] when the seat isn't open; database errors.
#[tracing::instrument(level = "debug", skip_all, fields(board_id, form = ?form), err(level = "debug"))]
pub async fn join(
    State(state): State<AppState>,
    session: Session,
    Path(board_id): Path<i64>,
    Form(form): Form<JoinForm>,
) -> Result<Response, AppError> {
    let mut tx = state.db.begin().await.map_err(DbError::from)?;
    let mut game = game_repo::load(&mut tx, board_id).await?;
    game.join_hero_seat(&form.hero, session.user.id)?;
    game_repo::save(&mut tx, &mut game).await?;
    user_repo::add_member(&mut tx, board_id, session.user.id).await?;
    tx.commit().await.map_err(DbError::from)?;
    state.versions.board_changed(board_id);
    Ok(redirect(
        &format!("/boards/{board_id}/play"),
        Some(Flash::notice("Vous avez rejoint la partie.")),
    ))
}

/// `GET /boards/{board_id}/story_screen`.
///
/// # Errors
/// [`AppError::Db`] when the board can't be loaded.
#[tracing::instrument(level = "debug", skip_all, fields(board_id = access.board_id, user_id = access.session.user.id), err(level = "debug"))]
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
    let dominance = match game.dominance(&state.data)? {
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
        winner: winner_label(&game),
        missions: visible_missions(&game, access.session.user.id),
        options: option_labels(game.board.options),
    };
    render(&page, &page.layout)
}

/// French names of the optional rules in use (§ 10.4).
fn option_labels(options: GameOptions) -> Vec<&'static str> {
    [
        (options.advanced_dominance, "Domination avancée"),
        (options.high_stakes, "Enjeux élevés à l'étape III"),
    ]
    .into_iter()
    .filter_map(|(used, label)| used.then_some(label))
    .collect()
}
