//! Seats of a board (Sauron and the heroes), shown above the game table.

use std::collections::BTreeMap;

use crate::engine::route::{RouteActor, route_for};
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::model::user::User;
use crate::web::views::pictures::image;

/// Users of a board's seats, by id.
pub type Players = BTreeMap<i64, User>;

/// One seat of the board.
#[derive(Debug, Clone)]
pub struct SeatView {
    /// Sauron or the hero's name.
    pub name: String,
    /// Name of the user playing the seat.
    pub player: String,
    /// Token picture.
    pub pic: String,
    /// The seat is expected to act.
    pub active: bool,
    /// The seat whose table is displayed.
    pub viewing: bool,
    /// Link to the seat's current screen or sheet, for seats the user plays.
    pub url: Option<String>,
}

/// Whether `me` plays the seat owned by `owner`: their own seat, or a computer seat
/// (played by the board members).
pub fn controls(owner: i64, me: &User, players: &Players) -> bool {
    owner == me.id || players.get(&owner).is_some_and(|u| u.ai_index.is_some())
}

/// The seats of a game, Sauron first.
///
/// # Arguments
/// * `viewing` - the seat whose table is displayed.
pub fn seats(
    game: &Game,
    data: &GameData,
    players: &Players,
    me: &User,
    viewing: Option<RouteActor>,
) -> Vec<SeatView> {
    let board_id = game.board.id.unwrap_or_default();
    let player = |id: i64| {
        players
            .get(&id)
            .map_or_else(String::new, |u| u.name.clone())
    };
    let link = |owner: i64, active: bool, actor: RouteActor, sheet: String| {
        let playing = active && game.board.winner.is_none();
        controls(owner, me, players)
            .then(|| playing.then(|| route_for(game.board.state, board_id, actor)))
            .map(|screen| screen.unwrap_or(sheet))
    };
    let sauron_id = game.sauron.id.unwrap_or_default();
    let sauron_actor = RouteActor::Sauron(sauron_id);
    let mut seats = vec![SeatView {
        name: "Sauron".to_owned(),
        player: player(game.sauron.user_id),
        pic: image("influence.png"),
        active: game.sauron.active,
        viewing: viewing == Some(sauron_actor),
        url: link(
            game.sauron.user_id,
            game.sauron.active,
            sauron_actor,
            format!("/sauron/{sauron_id}"),
        ),
    }];
    seats.extend(game.heroes.iter().map(|hero| {
        let id = hero.id.unwrap_or_default();
        let actor = RouteActor::Hero(id);
        SeatView {
            name: data
                .heroes
                .get(&hero.code)
                .map_or_else(|| hero.code.clone(), |d| d.name.clone()),
            player: player(hero.user_id),
            pic: image(&format!("heroes_tokens/{}.png", hero.code)),
            active: hero.active,
            viewing: viewing == Some(actor),
            url: link(hero.user_id, hero.active, actor, format!("/heroes/{id}")),
        }
    }));
    seats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::{data, game};

    fn user(id: i64, ai: bool) -> User {
        User {
            id,
            provider: "dev".to_owned(),
            uid: id.to_string(),
            name: format!("joueur {id}"),
            email: String::new(),
            ai_index: ai.then_some(0),
        }
    }

    #[test]
    fn links_only_the_seats_the_user_plays() {
        let mut game = game();
        game.sauron.user_id = 1;
        game.heroes[0].user_id = 2;
        let players = Players::from([(1, user(1, false)), (2, user(2, false))]);
        let seats = seats(&game, data(), &players, &user(1, false), None);
        assert!(seats[0].url.is_some());
        assert!(seats[1].url.is_none());
    }

    #[test]
    fn computer_seats_are_played_by_members() {
        let players = Players::from([(5, user(5, true))]);
        assert!(controls(5, &user(1, false), &players));
        assert!(!controls(6, &user(1, false), &players));
    }
}
