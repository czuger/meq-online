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

/// Which side a state's screen is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenSide {
    /// Only Sauron acts on this screen.
    Sauron,
    /// Only a hero acts on this screen.
    Hero,
    /// Either side may act (combat cards, choices).
    Both,
    /// No screen: the state is transient or the game hasn't started or is over.
    None,
}

impl ScreenSide {
    /// Whether `actor` has something to do on this screen.
    pub fn includes(self, actor: RouteActor) -> bool {
        matches!(
            (self, actor),
            (Self::Both, _)
                | (Self::Sauron, RouteActor::Sauron(_))
                | (Self::Hero, RouteActor::Hero(_))
        )
    }
}

/// The side the screen of `state` is for.
pub fn screen_side(state: BoardState) -> ScreenSide {
    use BoardState as S;
    match state {
        S::SauronSetupScreen
        | S::EditSauronSauronActions
        | S::PlaceInfluenceScreenSauronActions
        | S::KeepScreenSauronPlotCards
        | S::CommandScreenSauronMobs
        | S::PlayScreenSauronPlotCards
        | S::LookForGollumCardsSauronPlotCards
        | S::SauronHavenScreen => ScreenSide::Sauron,
        S::HeroDrawCardsScreen
        | S::HeroRestScreen
        | S::HeroMovementScreen
        | S::HeroExploration
        | S::HeroDefeatScreen
        | S::AfterRestAdvanceStoryMarker
        | S::CombatSetupScreenBoardCombats
        | S::CardsLossScreenBoardCombats => ScreenSide::Hero,
        S::PlayCombatCardScreenBoardCombats | S::ChoiceScreenBoardChoices => ScreenSide::Both,
        S::Created
        | S::WaitingForPlayers
        | S::FinishHeroTurn
        | S::FinishSauronTurn
        | S::Finished => ScreenSide::None,
    }
}

/// Path of the GET route displaying `state` to `actor`.
///
/// Each screen state is named after its route (`play_screen_sauron_plot_cards` is
/// `/sauron/:sauron_id/plot_cards/play_screen`). Transient states have no screen and
/// send the player to the board's play page, which follows the game to whoever acts.
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
        BoardState::PlaceInfluenceScreenSauronActions => {
            format!("/sauron/{id}/sauron_actions/place_influence_screen")
        }
        BoardState::KeepScreenSauronPlotCards => format!("/sauron/{id}/plot_cards/keep_screen"),
        BoardState::CommandScreenSauronMobs => format!("/sauron/{id}/sauron_mobs/command_screen"),
        BoardState::PlayScreenSauronPlotCards => format!("/sauron/{id}/plot_cards/play_screen"),
        BoardState::LookForGollumCardsSauronPlotCards => {
            format!("/sauron/{id}/plot_cards/look_for_gollum_cards")
        }
        BoardState::HeroDrawCardsScreen => format!("/heroes/{id}/draw_cards_screen"),
        BoardState::HeroRestScreen => format!("/heroes/{id}/rest_screen"),
        BoardState::HeroMovementScreen => format!("/heroes/{id}/movement_screen"),
        BoardState::HeroExploration => format!("/heroes/{id}/exploration"),
        BoardState::HeroDefeatScreen => format!("/heroes/{id}/defeat_screen"),
        BoardState::SauronHavenScreen => format!("/sauron/{id}/haven_screen"),
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
        BoardState::ChoiceScreenBoardChoices => {
            format!("/boards/{board_id}/choices/choice_screen")
        }
        BoardState::CardsLossScreenBoardCombats => {
            format!("/boards/{board_id}/combats/cards_loss_screen")
        }
        BoardState::Created
        | BoardState::WaitingForPlayers
        | BoardState::FinishHeroTurn
        | BoardState::FinishSauronTurn
        | BoardState::Finished => format!("/boards/{board_id}/play"),
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
    fn transient_states_go_to_the_play_page() {
        assert_eq!(
            route_for(BoardState::FinishHeroTurn, 3, RouteActor::Hero(2)),
            "/boards/3/play"
        );
    }

    #[test]
    fn hero_screens_leave_sauron_waiting() {
        let side = screen_side(BoardState::CombatSetupScreenBoardCombats);
        assert!(side.includes(RouteActor::Hero(2)));
        assert!(!side.includes(RouteActor::Sauron(1)));
        assert!(
            screen_side(BoardState::PlayCombatCardScreenBoardCombats)
                .includes(RouteActor::Sauron(1))
        );
    }
}
