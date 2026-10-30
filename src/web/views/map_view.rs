//! Positions of everything drawn on the map picture.

use crate::engine::tokens::{Token, TokenKind, tokens_by_location};
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::model::sauron_action::{ActionToken, SauronAction};
use crate::web::views::pictures::{image, plot_card};

/// Scale between the coordinates in the data and the displayed map picture.
const MUL_X: f64 = 1.5442;
const MUL_Y: f64 = 1.5444;

/// Influence counter of a location.
#[derive(Debug, Clone)]
pub struct InfluencePawn {
    /// Location code.
    pub code: String,
    /// Inline style placing the pawn.
    pub style: String,
    /// Influence on the location.
    pub value: i32,
}

/// Top token of a location, with a counter when several are stacked.
#[derive(Debug, Clone)]
pub struct MapToken {
    /// Picture.
    pub pic: String,
    /// Inline style placing the token.
    pub style: String,
    /// Names of every token of the location.
    pub title: String,
    /// Number of tokens on the location.
    pub count: usize,
    /// Inline style placing the counter.
    pub count_style: String,
    /// CSS class of the picture.
    pub class: &'static str,
}

/// Slot of the shadow pool track.
#[derive(Debug, Clone)]
pub struct PoolToken {
    /// Slot number, from 0.
    pub number: usize,
    /// Inline style placing the slot.
    pub style: String,
    /// 1 when filled.
    pub value: i32,
}

/// Plot card shown next to the map.
#[derive(Debug, Clone)]
pub struct MapPlot {
    /// Slot number.
    pub slot: u8,
    /// Picture.
    pub pic: String,
}

/// Everything drawn on the map.
#[derive(Debug, Clone)]
pub struct MapView {
    /// Map picture.
    pub map_pic: String,
    /// Influence pawn picture.
    pub influence_pic: String,
    /// Sauron action marker picture.
    pub action_pic: String,
    /// Influence counters.
    pub pawns: Vec<InfluencePawn>,
    /// Tokens.
    pub tokens: Vec<MapToken>,
    /// Shadow pool track.
    pub pool: Vec<PoolToken>,
    /// Inline styles placing Sauron's action markers.
    pub actions: Vec<String>,
    /// Plots in play.
    pub plots: Vec<MapPlot>,
}

/// Builds the map of a game, as seen by Sauron (`sauron` set) or by heroes.
pub fn map_view(game: &Game, data: &GameData, sauron: bool) -> MapView {
    MapView {
        map_pic: image("map_half.jpg"),
        influence_pic: image("influence.png"),
        action_pic: image("sauron_action.png"),
        pawns: influence_pawns(game, data),
        tokens: map_tokens(game, data, sauron),
        pool: pool_tokens(game.board.shadow_pool),
        actions: game
            .board
            .sauron_actions
            .iter()
            .map(|token| action_style(*token))
            .collect(),
        plots: game
            .board
            .plots
            .iter()
            .map(|(slot, card)| MapPlot {
                slot: *slot,
                pic: plot_card(*card).pic,
            })
            .collect(),
    }
}

fn place(top: f64, left: f64) -> String {
    format!("top:{top}px; left:{left}px;")
}

fn influence_pawns(game: &Game, data: &GameData) -> Vec<InfluencePawn> {
    data.locations
        .iter()
        .filter(|(_, l)| !l.haven)
        .map(|(code, l)| InfluencePawn {
            code: code.clone(),
            style: place(
                f64::from(l.pos_y) * MUL_Y + 15.0,
                f64::from(l.pos_x) * MUL_X - 55.0,
            ),
            value: game.board.influence.get(code).copied().unwrap_or(0),
        })
        .collect()
}

fn map_tokens(game: &Game, data: &GameData, sauron: bool) -> Vec<MapToken> {
    tokens_by_location(game, data)
        .iter()
        .filter_map(|(code, tokens)| {
            let location = data.locations.get(code).ok()?;
            let first = tokens.iter().min_by_key(|t| t.priority)?;
            Some(map_token(
                first,
                tokens,
                (location.pos_x, location.pos_y),
                sauron,
            ))
        })
        .collect()
}

fn map_token(first: &Token, tokens: &[Token], (x, y): (i32, i32), sauron: bool) -> MapToken {
    let shown_name = |t: &Token| {
        if sauron {
            t.sauron_name.clone().unwrap_or_else(|| t.name.clone())
        } else {
            t.name.clone()
        }
    };
    let decal = if first.kind == TokenKind::Plot {
        7.0
    } else {
        0.0
    };
    let mut left = f64::from(x) * MUL_X - 23.0 - 15.0;
    let mut top = f64::from(y) * MUL_Y - 25.0 - 16.0 + decal;
    let (class, shift_x, shift_y) = match first.kind {
        TokenKind::Minion => ("token-on-map-pic-minion", 0.0, 0.0),
        TokenKind::Favor => ("token-on-map-pic-favor", 9.0, 9.0),
        TokenKind::Plot => ("token-on-map-pic", 7.0, 12.0),
        _ => ("token-on-map-pic", 4.0, 4.0),
    };
    left += shift_x;
    top += shift_y;
    let pic = match (&first.sauron_pic_path, sauron) {
        (Some(pic), true) => pic.clone(),
        _ => first.pic_path.clone(),
    };
    MapToken {
        pic: image(&pic),
        style: place(top, left),
        title: tokens.iter().map(shown_name).collect::<Vec<_>>().join(", "),
        count: tokens.len(),
        count_style: format!("top:{}px", 17.0 - decal),
        class,
    }
}

fn pool_tokens(shadow_pool: i32) -> Vec<PoolToken> {
    (0..12usize)
        .map(|number| {
            let (row, column) = (number / 4, number % 4);
            let top = 1183.0 * MUL_Y + 35.0 * MUL_Y * row as f64;
            let left = 1695.0 * MUL_X + 38.5 * MUL_X * column as f64;
            let filled = i32::try_from(number).unwrap_or(i32::MAX) < shadow_pool;
            PoolToken {
                number,
                style: place(top, left),
                value: i32::from(filled),
            }
        })
        .collect()
}

/// Position of a Sauron action token on the action tracks of the map.
fn action_style(token: ActionToken) -> String {
    let x = match token.action {
        SauronAction::PlaceInfluence => 2463,
        SauronAction::DrawCards => 2612,
        SauronAction::Command => 2761,
    };
    format!(
        "top:1614px; left:{}px;",
        x + (i32::from(token.space) - 1) * 46
    )
}
