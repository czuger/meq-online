//! Data shown by the page layout: menu, current actor and flash message.

use crate::game_data::GameData;
use crate::model::game::Game;
use crate::web::flash::Flash;
use crate::web::session::Session;

/// Board shortcuts in the menu.
#[derive(Debug, Clone)]
pub struct BoardNav {
    /// Board id.
    pub id: i64,
    /// Game turn.
    pub turn: i32,
}

/// The actor whose page is displayed.
#[derive(Debug, Clone)]
pub struct ActorNav {
    /// Name on the actor button.
    pub name: String,
    /// Actor page.
    pub url: String,
    /// Map page for this actor.
    pub map_url: String,
    /// Messages page for this actor.
    pub messages_url: String,
    /// Turn number (1 or 2) of a hero in a single-hero game.
    pub turn_badge: Option<i32>,
}

/// Everything the layout template needs.
#[derive(Debug, Clone, Default)]
pub struct Layout {
    /// Logged-in user's name.
    pub user_name: Option<String>,
    /// Board shown, if any.
    pub board: Option<BoardNav>,
    /// Actor shown, if any.
    pub actor: Option<ActorNav>,
    /// Flash message to display.
    pub flash: Option<Flash>,
}

impl Layout {
    /// Layout without user (login pages).
    pub fn anonymous() -> Self {
        Self::default()
    }

    /// Layout of a page outside any board.
    pub fn user(session: &Session) -> Self {
        Self {
            user_name: Some(session.user.name.clone()),
            flash: session.flash.clone(),
            ..Self::default()
        }
    }

    /// Layout of a board page with no specific actor.
    pub fn board(session: &Session, game: &Game) -> Self {
        Self {
            board: game.board.id.map(|id| BoardNav {
                id,
                turn: game.board.turn,
            }),
            ..Self::user(session)
        }
    }

    /// Layout of a hero's page.
    pub fn hero(session: &Session, game: &Game, hero_id: i64, data: &GameData) -> Self {
        let hero = game.heroes.iter().find(|h| h.id == Some(hero_id));
        let name = hero
            .and_then(|h| data.heroes.get(&h.code))
            .map_or_else(|| "Héros".to_owned(), |d| d.name.clone());
        let turn_badge = (game.heroes.len() == 1).then(|| hero.map_or(1, |h| h.turn));
        Self {
            actor: Some(ActorNav {
                name,
                url: format!("/heroes/{hero_id}"),
                map_url: format!("/maps/{hero_id}"),
                messages_url: format!("/heroes/{hero_id}/board_messages"),
                turn_badge,
            }),
            ..Self::board(session, game)
        }
    }

    /// Layout of a Sauron page.
    pub fn sauron(session: &Session, game: &Game) -> Self {
        let id = game.sauron.id.unwrap_or_default();
        Self {
            actor: Some(ActorNav {
                name: "Sauron".to_owned(),
                url: format!("/sauron/{id}"),
                map_url: format!("/maps/{id}/edit"),
                messages_url: format!("/sauron/{id}/board_messages"),
                turn_badge: None,
            }),
            ..Self::board(session, game)
        }
    }
}
