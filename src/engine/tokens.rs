//! Tokens displayed on each location of the map.

use std::collections::BTreeMap;

use crate::engine::events::event_step::event_pic_path;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::model::game::Game;

/// Kind of token on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A character to meet.
    Character,
    /// A plot marker.
    Plot,
    /// A favor token.
    Favor,
    /// A hero.
    Hero,
    /// A face-down monster.
    Monster,
    /// A minion.
    Minion,
    /// The event marker of the event card in play.
    Event,
}

/// A token as shown on the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Kind of token.
    pub kind: TokenKind,
    /// Code of the character, hero or mob.
    pub code: String,
    /// Display name.
    pub name: String,
    /// Drawing order on the map.
    pub priority: i32,
    /// Picture seen by heroes.
    pub pic_path: String,
    /// Picture seen by Sauron (monsters only).
    pub sauron_pic_path: Option<String>,
    /// Name seen by Sauron (monsters only).
    pub sauron_name: Option<String>,
}

impl Token {
    fn new(kind: TokenKind, code: &str, name: &str, priority: i32, pic_path: String) -> Self {
        Self {
            kind,
            code: code.to_owned(),
            name: name.to_owned(),
            priority,
            pic_path,
            sauron_pic_path: None,
            sauron_name: None,
        }
    }
}

/// Every token of the board, grouped by location code.
///
/// # Arguments
/// * `game` - the board.
/// * `data` - static data, for character and hero names.
pub fn tokens_by_location(game: &Game, data: &GameData) -> BTreeMap<String, Vec<Token>> {
    let mut tokens: BTreeMap<String, Vec<Token>> = BTreeMap::new();
    let all = character_tokens(game, data)
        .chain(plot_tokens(game, data))
        .chain(favor_tokens(game))
        .chain(hero_tokens(game, data))
        .chain(event_token(game, data))
        .chain(
            game.mobs
                .iter()
                .map(|mob| (mob.location.clone(), mob_token(mob))),
        );
    for (location, token) in all {
        tokens.entry(location).or_default().push(token);
    }
    tokens
}

fn character_tokens<'a>(
    game: &'a Game,
    data: &'a GameData,
) -> impl Iterator<Item = (String, Token)> + 'a {
    game.board.characters.iter().map(|(character, location)| {
        let name = data
            .characters
            .get(character)
            .map_or(character.as_str(), |c| c.name.as_str());
        let pic = format!("characters/{character}.png");
        (
            location.clone(),
            Token::new(TokenKind::Character, character, name, 50, pic),
        )
    })
}

fn plot_tokens<'a>(
    game: &'a Game,
    data: &'a GameData,
) -> impl Iterator<Item = (String, Token)> + 'a {
    game.board.plots.iter().filter_map(|(slot, card)| {
        data.plot(*card).ok()?;
        let location = game.plot_location(*slot, data)?;
        let name = format!("Intrigue {slot}");
        Some((
            location,
            Token::new(
                TokenKind::Plot,
                "plot",
                &name,
                30,
                format!("plot_{slot}.png"),
            ),
        ))
    })
}

fn favor_tokens(game: &Game) -> impl Iterator<Item = (String, Token)> + '_ {
    game.board.favors.iter().map(|location| {
        (
            location.clone(),
            Token::new(
                TokenKind::Favor,
                "favor",
                "Faveur",
                100,
                "favor.png".to_owned(),
            ),
        )
    })
}

fn event_token(game: &Game, data: &GameData) -> Option<(String, Token)> {
    let event = game.board.current_event?;
    let def = data.event(event.stage, event.card).ok()?;
    let location = def.affect.clone()?;
    let pic = format!("events/{}", event_pic_path(event));
    Some((
        location,
        Token::new(TokenKind::Event, "event", &def.name, 20, pic),
    ))
}

fn hero_tokens<'a>(
    game: &'a Game,
    data: &'a GameData,
) -> impl Iterator<Item = (String, Token)> + 'a {
    game.heroes.iter().map(|hero| {
        let name = data
            .heroes
            .get(&hero.code)
            .map_or(hero.code.as_str(), |h| h.name.as_str());
        let pic = format!("heroes_tokens/{}.png", hero.code);
        (
            hero.location.clone(),
            Token::new(TokenKind::Hero, &hero.code, name, 10, pic),
        )
    })
}

fn mob_token(mob: &crate::model::mob::Mob) -> Token {
    match mob.kind {
        MobKind::Monster => {
            let pool = mob.pool.map_or("", |p| p.as_str());
            let mut token = Token::new(
                TokenKind::Monster,
                &mob.code,
                "Monstre",
                100,
                format!("monsters/tokens/covers/monsters_pool_{pool}.jpg"),
            );
            token.sauron_pic_path = Some(format!("monsters/tokens/sauron_map/{}.jpg", mob.code));
            token.sauron_name = Some(mob.name.clone());
            token
        }
        MobKind::Minion => Token::new(
            TokenKind::Minion,
            &mob.code,
            &mob.name,
            40,
            format!("monsters/tokens/covers/{}.png", mob.code),
        ),
    }
}
