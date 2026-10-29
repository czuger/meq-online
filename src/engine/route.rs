//! Screen to open for the current phase of a board.

use crate::engine::board_state::BoardState;

/// The player asking where to go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteActor {
    /// A hero, by id.
    Hero(i64),
    /// The Sauron player, by id.
    Sauron(i64),
}

impl RouteActor {
    fn id(self) -> i64 {
        match self {
            Self::Hero(id) | Self::Sauron(id) => id,
        }
    }

    /// Path segment naming the side in combat routes.
    pub fn side(self) -> &'static str {
        match self {
            Self::Hero(_) => "hero",
            Self::Sauron(_) => "sauron",
        }
    }
}

/// Path of the GET route displaying `state` to `actor`.
///
/// Each screen state is named after its route (`play_screen_sauron_plot_cards` is
/// `/sauron/:sauron_id/plot_cards/play_screen`). Transient states have no screen and
/// send the player back to the board list.
///
/// # Arguments
/// * `state` - current board state.
/// * `board_id` - the board.
/// * `actor` - who asks.
pub fn route_for(state: BoardState, board_id: i64, actor: RouteActor) -> String {
    let id = actor.id();
    match state {
        BoardState::SauronSetupScreen => format!("/sauron/{id}/setup_screen"),
        BoardState::EditSauronSauronActions => format!("/sauron/{id}/sauron_actions/edit"),
        BoardState::PlayScreenSauronPlotCards => format!("/sauron/{id}/plot_cards/play_screen"),
        BoardState::LookForGollumCardsSauronPlotCards => {
            format!("/sauron/{id}/plot_cards/look_for_gollum_cards")
        }
        BoardState::HeroDrawCardsScreen => format!("/heroes/{id}/draw_cards_screen"),
        BoardState::HeroRestScreen => format!("/heroes/{id}/rest_screen"),
        BoardState::HeroMovementScreen => format!("/heroes/{id}/movement_screen"),
        BoardState::HeroExploration => format!("/heroes/{id}/exploration"),
        BoardState::AfterRestAdvanceStoryMarker => {
            format!("/heroes/{id}/after_rest_advance_story_marker_screen")
        }
        BoardState::CombatSetupScreenBoardCombats => {
            format!("/boards/{board_id}/combats/combat_setup_screen")
        }
        BoardState::PlayCombatCardScreenBoardCombats => {
            format!(
                "/boards/{board_id}/combats/{}/play_combat_card_screen",
                actor.side()
            )
        }
        BoardState::CardsLossScreenBoardCombats => {
            format!("/boards/{board_id}/combats/cards_loss_screen")
        }
        BoardState::Created
        | BoardState::WaitingForPlayers
        | BoardState::FinishHeroTurn
        | BoardState::FinishSauronTurn
        | BoardState::AfterDefeatAdvanceStoryMarker
        | BoardState::Finished => "/boards".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_states_use_their_route() {
        let route = route_for(
            BoardState::PlayScreenSauronPlotCards,
            3,
            RouteActor::Sauron(7),
        );
        assert_eq!(route, "/sauron/7/plot_cards/play_screen");
        let route = route_for(
            BoardState::PlayCombatCardScreenBoardCombats,
            3,
            RouteActor::Hero(2),
        );
        assert_eq!(route, "/boards/3/combats/hero/play_combat_card_screen");
    }

    #[test]
    fn transient_states_go_back_to_boards() {
        assert_eq!(
            route_for(BoardState::FinishHeroTurn, 3, RouteActor::Hero(2)),
            "/boards"
        );
    }
}
