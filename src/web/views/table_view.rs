//! The game table framing every screen of a seat: seats, turn steps, map, recent
//! events and the seat's cards.

use crate::engine::route::RouteActor;
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::model::user::User;
use crate::web::views::hero_info::HeroInfo;
use crate::web::views::map_view::{MapView, map_view, target_style};
use crate::web::views::pictures::{
    CardView, corruption_card, hero_hand, image, plot_card, shadow_card,
};
use crate::web::views::seat_view::{Players, SeatView, seats};
use crate::web::views::state_label::state_label;
use crate::web::views::turn_log::TurnLog;
use crate::web::views::turn_steps::{TurnSteps, turn_steps};

/// A hero's sheet and cards, below the map.
#[derive(Debug, Clone)]
pub struct HeroTray {
    /// Counters.
    pub info: HeroInfo,
    /// Cards in hand.
    pub hand: Vec<CardView>,
    /// Hero sheet picture.
    pub sheet_pic: String,
    /// Corruption cards.
    pub corruptions: Vec<CardView>,
}

/// Sauron's cards, below the map.
#[derive(Debug, Clone)]
pub struct SauronTray {
    /// Shadow cards in hand.
    pub shadow_hand: Vec<CardView>,
    /// Plot cards in hand.
    pub plot_hand: Vec<CardView>,
    /// Plots in play.
    pub plots: Vec<CardView>,
}

/// A location the current step lets the player click on the map: a submit button of
/// the step's form, drawn over the location.
#[derive(Debug, Clone)]
pub struct MapTarget {
    /// Id of the form the button submits.
    pub form_id: &'static str,
    /// Name of the submitted field.
    pub name: &'static str,
    /// Submitted value (the location code).
    pub value: String,
    /// Tooltip.
    pub label: String,
    /// Inline style placing the button.
    pub style: String,
}

impl MapTarget {
    /// A button of form `form_id` submitting `name=code` from the location `code`.
    ///
    /// # Returns
    /// `None` for an unknown location.
    pub fn new(
        data: &GameData,
        form_id: &'static str,
        name: &'static str,
        code: &str,
        label: String,
    ) -> Option<Self> {
        Some(Self {
            form_id,
            name,
            value: code.to_owned(),
            style: target_style(data, code)?,
            label,
        })
    }
}

/// Everything the game table shows around a screen.
#[derive(Debug, Clone)]
pub struct TableView {
    /// Board id.
    pub board_id: i64,
    /// Board version when the page was displayed, polled by the page to follow the game.
    pub version: String,
    /// Game turn.
    pub turn: i32,
    /// Story stage (1 to 3).
    pub stage: i32,
    /// What the game waits for, in French.
    pub phase: &'static str,
    /// Steps of the current turn.
    pub steps: Option<TurnSteps>,
    /// Seats, Sauron first.
    pub seats: Vec<SeatView>,
    /// The map.
    pub map: MapView,
    /// Sauron's id when he may place influence by clicking the map.
    pub influence_editor: Option<i64>,
    /// Locations the current step lets the player click.
    pub map_targets: Vec<MapTarget>,
    /// What the latest turns did, step by step.
    pub turn_log: TurnLog,
    /// The viewing hero's sheet and cards.
    pub hero_tray: Option<HeroTray>,
    /// Sauron's cards, when Sauron is viewing.
    pub sauron_tray: Option<SauronTray>,
    /// Whether the tray shows the hero's hand (screens selecting cards show their own).
    pub show_hand: bool,
    /// Messages page of the viewing seat.
    pub messages_url: String,
}

impl TableView {
    /// The table as seen by a seat.
    ///
    /// # Arguments
    /// * `players` - users of the board's seats.
    /// * `me` - the user viewing the page.
    /// * `viewer` - the seat whose table is displayed.
    /// * `turn_log` - what the latest turns did.
    /// * `version` - board version.
    pub fn new(
        game: &Game,
        data: &GameData,
        players: &Players,
        me: &User,
        viewer: RouteActor,
        turn_log: TurnLog,
        version: String,
    ) -> Self {
        let sauron = matches!(viewer, RouteActor::Sauron(_));
        let messages_url = match viewer {
            RouteActor::Hero(id) => format!("/heroes/{id}/board_messages"),
            RouteActor::Sauron(id) => format!("/sauron/{id}/board_messages"),
        };
        Self {
            board_id: game.board.id.unwrap_or_default(),
            version,
            turn: game.board.turn,
            stage: game.board.story.stage(),
            phase: state_label(game.board.state),
            steps: turn_steps(game, &turn_log),
            seats: seats(game, data, players, me, Some(viewer)),
            map: map_view(game, data, sauron),
            influence_editor: None,
            map_targets: Vec::new(),
            turn_log,
            hero_tray: hero_tray(game, data, viewer),
            sauron_tray: sauron.then(|| sauron_tray(game)),
            show_hand: true,
            messages_url,
        }
    }

    /// Lets Sauron place influence by clicking the map.
    #[must_use]
    pub fn with_influence_editor(mut self, sauron_id: i64) -> Self {
        self.influence_editor = Some(sauron_id);
        self
    }

    /// Adds the locations the current step lets the player click.
    #[must_use]
    pub fn with_targets(mut self, targets: Vec<MapTarget>) -> Self {
        self.map_targets = targets;
        self
    }

    /// Hides the hand in the tray, for screens showing a selectable hand.
    #[must_use]
    pub fn without_hand(mut self) -> Self {
        self.show_hand = false;
        self
    }
}

fn hero_tray(game: &Game, data: &GameData, viewer: RouteActor) -> Option<HeroTray> {
    let RouteActor::Hero(hero_id) = viewer else {
        return None;
    };
    let hero = game.heroes.iter().find(|h| h.id == Some(hero_id))?;
    Some(HeroTray {
        info: HeroInfo::new(hero, data),
        hand: hero_hand(data, &hero.code, &hero.hand),
        sheet_pic: image(&format!("heroes/{}.jpg", hero.code)),
        corruptions: hero
            .corruption_cards
            .iter()
            .map(|c| corruption_card(*c))
            .collect(),
    })
}

fn sauron_tray(game: &Game) -> SauronTray {
    SauronTray {
        shadow_hand: game
            .sauron
            .shadow_hand
            .iter()
            .map(|c| shadow_card(*c))
            .collect(),
        plot_hand: game
            .sauron
            .plot_hand
            .iter()
            .map(|c| plot_card(*c))
            .collect(),
        plots: game.board.plots.values().map(|c| plot_card(*c)).collect(),
    }
}
