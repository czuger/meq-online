//! Private messages between the players of a board.

use askama::Template;
use axum::Form;
use axum::extract::State;
use axum::response::Response;
use serde::Deserialize;

use crate::app_state::AppState;
use crate::db::{DbError, message_repo, user_repo};
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::web::access::{HeroAccess, SauronAccess};
use crate::web::app_error::AppError;
use crate::web::flash::{Flash, redirect};
use crate::web::game_tx::read_game;
use crate::web::page::render;
use crate::web::session::Session;
use crate::web::views::layout::Layout;

struct MessageRow {
    header: String,
    text: String,
    sent: bool,
}

#[derive(Template)]
#[template(path = "messages_index.html")]
struct IndexPage {
    layout: Layout,
    base_url: String,
    messages: Vec<MessageRow>,
}

#[derive(Template)]
#[template(path = "messages_new.html")]
struct NewPage {
    layout: Layout,
    base_url: String,
    receivers: Vec<(String, String)>,
}

/// New message form; the receiver is `sauron` or `hero-<id>`.
#[derive(Debug, Deserialize)]
pub struct MessageForm {
    receiver: String,
    text: String,
}

/// The actor whose messages are shown, and its page context.
struct Mailbox {
    session: Session,
    board_id: i64,
    actor: ActorRef,
    base_url: String,
}

impl From<HeroAccess> for Mailbox {
    fn from(access: HeroAccess) -> Self {
        Self {
            base_url: format!("/heroes/{}/board_messages", access.hero_id),
            actor: ActorRef::Hero(access.hero_id),
            session: access.session,
            board_id: access.board_id,
        }
    }
}

impl From<SauronAccess> for Mailbox {
    fn from(access: SauronAccess) -> Self {
        Self {
            base_url: format!("/sauron/{}/board_messages", access.sauron_id),
            actor: ActorRef::Sauron,
            session: access.session,
            board_id: access.board_id,
        }
    }
}

impl Mailbox {
    fn layout(&self, game: &Game, state: &AppState) -> Layout {
        match self.actor {
            ActorRef::Hero(id) => Layout::hero(&self.session, game, id, &state.data),
            ActorRef::Sauron | ActorRef::System => Layout::sauron(&self.session, game),
        }
    }
}

/// "Name (user)" of every actor of the board.
async fn actor_names(state: &AppState, game: &Game) -> Result<Vec<(ActorRef, String)>, AppError> {
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let mut user_name = async |id: i64| -> Result<String, AppError> {
        Ok(user_repo::find(&mut conn, id)
            .await?
            .map(|u| u.name)
            .unwrap_or_default())
    };
    let mut names = vec![(
        ActorRef::Sauron,
        format!("Sauron ({})", user_name(game.sauron.user_id).await?),
    )];
    for hero in &game.heroes {
        let hero_name = state
            .data
            .heroes
            .get(&hero.code)
            .map_or_else(|| hero.code.clone(), |d| d.name.clone());
        let id = hero.id.unwrap_or_default();
        names.push((
            ActorRef::Hero(id),
            format!("{hero_name} ({})", user_name(hero.user_id).await?),
        ));
    }
    Ok(names)
}

async fn index(state: AppState, mailbox: Mailbox) -> Result<Response, AppError> {
    let game = read_game(&state, mailbox.board_id).await?;
    let names = actor_names(&state, &game).await?;
    let name = |actor: ActorRef| {
        names
            .iter()
            .find(|(a, _)| *a == actor)
            .map(|(_, n)| n.clone())
            .unwrap_or_default()
    };
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    let messages = message_repo::for_actor(&mut conn, mailbox.board_id, mailbox.actor)
        .await?
        .into_iter()
        .map(|m| {
            let sent = m.sender == mailbox.actor;
            let header = if sent {
                format!("À : {}, le {}", name(m.receiver), m.created_at)
            } else {
                format!("De : {}, le {}", name(m.sender), m.created_at)
            };
            MessageRow {
                header,
                text: m.text,
                sent,
            }
        })
        .collect();
    let page = IndexPage {
        layout: mailbox.layout(&game, &state),
        base_url: mailbox.base_url.clone(),
        messages,
    };
    render(&page, &page.layout)
}

async fn new(state: AppState, mailbox: Mailbox) -> Result<Response, AppError> {
    let game = read_game(&state, mailbox.board_id).await?;
    let receivers = actor_names(&state, &game)
        .await?
        .into_iter()
        .filter(|(actor, _)| *actor != mailbox.actor)
        .map(|(actor, name)| {
            let value = match actor {
                ActorRef::Hero(id) => format!("hero-{id}"),
                ActorRef::Sauron | ActorRef::System => "sauron".to_owned(),
            };
            (value, name)
        })
        .collect();
    let page = NewPage {
        layout: mailbox.layout(&game, &state),
        base_url: mailbox.base_url.clone(),
        receivers,
    };
    render(&page, &page.layout)
}

async fn create(
    state: AppState,
    mailbox: Mailbox,
    form: MessageForm,
) -> Result<Response, AppError> {
    let game = read_game(&state, mailbox.board_id).await?;
    let receiver = match form.receiver.strip_prefix("hero-") {
        Some(id) => ActorRef::Hero(
            game.hero(
                id.parse()
                    .map_err(|_| AppError::BadRequest("receiver".to_owned()))?,
            )?
            .id
            .unwrap_or_default(),
        ),
        None if form.receiver == "sauron" => ActorRef::Sauron,
        None => return Err(AppError::BadRequest("receiver".to_owned())),
    };
    if form.text.trim().is_empty() {
        return Ok(redirect(
            &format!("{}/new", mailbox.base_url),
            Some(Flash::alert("Le message est vide.")),
        ));
    }
    let mut conn = state.db.acquire().await.map_err(DbError::from)?;
    message_repo::create(
        &mut conn,
        mailbox.board_id,
        mailbox.actor,
        receiver,
        &form.text,
    )
    .await?;
    Ok(redirect(
        &mailbox.base_url,
        Some(Flash::notice("Message envoyé.")),
    ))
}

/// `GET /heroes/{hero_id}/board_messages`.
///
/// # Errors
/// Database and rendering errors.
pub async fn hero_index(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    index(state, access.into()).await
}

/// `GET /heroes/{hero_id}/board_messages/new`.
///
/// # Errors
/// Database and rendering errors.
pub async fn hero_new(
    State(state): State<AppState>,
    access: HeroAccess,
) -> Result<Response, AppError> {
    new(state, access.into()).await
}

/// `POST /heroes/{hero_id}/board_messages`.
///
/// # Errors
/// [`AppError::BadRequest`] for an unknown receiver; database errors.
pub async fn hero_create(
    State(state): State<AppState>,
    access: HeroAccess,
    Form(form): Form<MessageForm>,
) -> Result<Response, AppError> {
    create(state, access.into(), form).await
}

/// `GET /sauron/{sauron_id}/board_messages`.
///
/// # Errors
/// Database and rendering errors.
pub async fn sauron_index(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    index(state, access.into()).await
}

/// `GET /sauron/{sauron_id}/board_messages/new`.
///
/// # Errors
/// Database and rendering errors.
pub async fn sauron_new(
    State(state): State<AppState>,
    access: SauronAccess,
) -> Result<Response, AppError> {
    new(state, access.into()).await
}

/// `POST /sauron/{sauron_id}/board_messages`.
///
/// # Errors
/// [`AppError::BadRequest`] for an unknown receiver; database errors.
pub async fn sauron_create(
    State(state): State<AppState>,
    access: SauronAccess,
    Form(form): Form<MessageForm>,
) -> Result<Response, AppError> {
    create(state, access.into(), form).await
}
