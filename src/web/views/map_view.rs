//! Positions of everything drawn on the map picture.

use crate::engine::tokens::{Token, TokenKind, tokens_by_location};
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::model::sauron_action::{ActionToken, SauronAction};
use crate::web::views::pictures::{image, plot_card};

/// Scale between the coordinates in the data and the pixels of the map picture.
const MUL_X: f64 = 2.5737;
const MUL_Y: f64 = 2.574;

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
        map_pic: image("map.jpg"),
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

/// Side of the clickable circle drawn over a location, in map pixels.
const TARGET_SIZE: f64 = 200.0;

/// Inline style centring a clickable circle on a location of the map.
///
/// # Returns
/// `None` for an unknown location.
pub fn target_style(data: &GameData, code: &str) -> Option<String> {
    let location = data.locations.get(code).ok()?;
    let half = TARGET_SIZE / 2.0;
    Some(place(
        f64::from(location.pos_y) * MUL_Y - half,
        f64::from(location.pos_x) * MUL_X - half,
    ))
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
                f64::from(l.pos_y) * MUL_Y + 50.33,
                f64::from(l.pos_x) * MUL_X - 78.33,
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
        11.67
    } else {
        0.0
    };
    let mut left = f64::from(x) * MUL_X - 63.33;
    let mut top = f64::from(y) * MUL_Y - 68.33 + decal;
    let (class, shift_x, shift_y) = match first.kind {
        TokenKind::Minion => ("token-on-map-pic-minion", 0.0, 0.0),
        TokenKind::Favor => ("token-on-map-pic-favor", 15.0, 15.0),
        TokenKind::Plot => ("token-on-map-pic", 11.67, 20.0),
        _ => ("token-on-map-pic", 6.67, 6.67),
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
        count_style: format!("top:{}px", 28.33 - decal),
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
        SauronAction::PlaceInfluence => 4105,
        SauronAction::DrawCards => 4353,
        SauronAction::Command => 4602,
    };
    format!(
        "top:2690px; left:{}px;",
        x + (i32::from(token.space) - 1) * 77
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::data;

    /// Size of `map.jpg`.
    const MAP_SIZE: (f64, f64) = (4890.0, 3362.0);

    fn position(style: &str) -> (f64, f64) {
        let value = |key: &str| {
            let start = style.find(key).expect("key") + key.len();
            let end = style[start..].find("px").expect("px") + start;
            style[start..end].parse::<f64>().expect("number")
        };
        (value("left:"), value("top:"))
    }

    #[test]
    fn every_location_target_lies_on_the_map() {
        for (code, _) in data().locations.iter() {
            let (left, top) = position(&target_style(data(), code).expect("known location"));
            assert!(
                (0.0..MAP_SIZE.0 - TARGET_SIZE).contains(&left)
                    && (0.0..MAP_SIZE.1 - TARGET_SIZE).contains(&top),
                "{code}: {left}, {top}"
            );
        }
    }

    // The tracks printed on the board: the last space of each must stay on the picture.
    #[test]
    fn board_tracks_lie_on_the_map() {
        let inside = |style: &str| {
            let (left, top) = position(style);
            (0.0..MAP_SIZE.0).contains(&left) && (0.0..MAP_SIZE.1).contains(&top)
        };
        assert!(pool_tokens(12).iter().all(|slot| inside(&slot.style)));
        for action in SauronAction::ALL {
            let last = ActionToken { action, space: 3 };
            assert!(inside(&action_style(last)), "{action:?}");
        }
    }

    #[test]
    fn unknown_locations_have_no_target() {
        assert!(target_style(data(), "nowhere").is_none());
    }
}
