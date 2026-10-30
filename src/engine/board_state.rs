//! Game phases and the transitions between them.

use std::fmt;
use std::str::FromStr;

use crate::engine::engine_error::EngineError;

/// Phase of a board.
///
/// Each screen phase is named after the GET route that displays it: a state and its
/// route must be renamed together (see [`crate::engine::route`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoardState {
    /// Just created.
    Created,
    /// Waiting for players to join.
    WaitingForPlayers,
    /// Sauron reviews the starting plot.
    SauronSetupScreen,
    /// Sauron chooses his next action on the action tracks.
    EditSauronSauronActions,
    /// Sauron places influence (Place Influence action).
    PlaceInfluenceScreenSauronActions,
    /// Sauron keeps one of the drawn plot cards (Draw Shadow and Plot Cards action).
    KeepScreenSauronPlotCards,
    /// Sauron commands monsters and minions (Command action).
    CommandScreenSauronMobs,
    /// Heroes draw their cards.
    HeroDrawCardsScreen,
    /// The current hero rests or heals.
    HeroRestScreen,
    /// Transient: the current hero's turn is over.
    FinishHeroTurn,
    /// Transient: Sauron's turn is over.
    FinishSauronTurn,
    /// A defeated hero picks the story marker to advance and what to lose.
    HeroDefeatScreen,
    /// Sauron picks the Haven a defeated hero goes to, between equally close ones.
    SauronHavenScreen,
    /// The current hero moves.
    HeroMovementScreen,
    /// The current hero explores its location.
    HeroExploration,
    /// Sauron plays a plot card.
    PlayScreenSauronPlotCards,
    /// The hero picks which tied story marker advances after resting.
    AfterRestAdvanceStoryMarker,
    /// The hero chooses between drawing cards and gaining strength.
    CombatSetupScreenBoardCombats,
    /// Both sides pick a combat card.
    PlayCombatCardScreenBoardCombats,
    /// Sauron fetches a Gollum plot card.
    LookForGollumCardsSauronPlotCards,
    /// The game is over.
    Finished,
    /// The hero chooses cards to lose to damage.
    CardsLossScreenBoardCombats,
    /// A player answers a card's choice.
    ChoiceScreenBoardChoices,
}

/// Every state, for exhaustive checks.
pub const ALL_STATES: [BoardState; 23] = [
    BoardState::Created,
    BoardState::WaitingForPlayers,
    BoardState::SauronSetupScreen,
    BoardState::EditSauronSauronActions,
    BoardState::PlaceInfluenceScreenSauronActions,
    BoardState::KeepScreenSauronPlotCards,
    BoardState::CommandScreenSauronMobs,
    BoardState::HeroDrawCardsScreen,
    BoardState::HeroRestScreen,
    BoardState::FinishHeroTurn,
    BoardState::FinishSauronTurn,
    BoardState::HeroDefeatScreen,
    BoardState::SauronHavenScreen,
    BoardState::HeroMovementScreen,
    BoardState::HeroExploration,
    BoardState::PlayScreenSauronPlotCards,
    BoardState::AfterRestAdvanceStoryMarker,
    BoardState::CombatSetupScreenBoardCombats,
    BoardState::PlayCombatCardScreenBoardCombats,
    BoardState::LookForGollumCardsSauronPlotCards,
    BoardState::Finished,
    BoardState::CardsLossScreenBoardCombats,
    BoardState::ChoiceScreenBoardChoices,
];

impl BoardState {
    /// Snake-case name, stored in the database and matching the GET route name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::WaitingForPlayers => "waiting_for_players",
            Self::SauronSetupScreen => "sauron_setup_screen",
            Self::EditSauronSauronActions => "edit_sauron_sauron_actions",
            Self::PlaceInfluenceScreenSauronActions => "place_influence_screen_sauron_actions",
            Self::KeepScreenSauronPlotCards => "keep_screen_sauron_plot_cards",
            Self::CommandScreenSauronMobs => "command_screen_sauron_mobs",
            Self::HeroDrawCardsScreen => "hero_draw_cards_screen",
            Self::HeroRestScreen => "hero_rest_screen",
            Self::FinishHeroTurn => "finish_hero_turn",
            Self::FinishSauronTurn => "finish_sauron_turn",
            Self::HeroDefeatScreen => "hero_defeat_screen",
            Self::SauronHavenScreen => "sauron_haven_screen",
            Self::HeroMovementScreen => "hero_movement_screen",
            Self::HeroExploration => "hero_exploration",
            Self::PlayScreenSauronPlotCards => "play_screen_sauron_plot_cards",
            Self::AfterRestAdvanceStoryMarker => "after_rest_advance_story_marker",
            Self::CombatSetupScreenBoardCombats => "combat_setup_screen_board_combats",
            Self::PlayCombatCardScreenBoardCombats => "play_combat_card_screen_board_combats",
            Self::LookForGollumCardsSauronPlotCards => "look_for_gollum_cards_sauron_plot_cards",
            Self::Finished => "finished",
            Self::CardsLossScreenBoardCombats => "cards_loss_screen_board_combats",
            Self::ChoiceScreenBoardChoices => "choice_screen_board_choices",
        }
    }

    /// Applies `event` to this state.
    ///
    /// # Returns
    /// The new state.
    ///
    /// # Errors
    /// [`EngineError::InvalidTransition`] when `event` isn't allowed from this state.
    pub fn transition(self, event: BoardEvent) -> Result<Self, EngineError> {
        let (from, to) = event.rule();
        if from.contains(&self) {
            Ok(to)
        } else {
            Err(EngineError::InvalidTransition {
                from: self.as_str(),
                event: event.as_str(),
            })
        }
    }
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned when parsing an unknown state name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown board state '{0}'")]
pub struct UnknownBoardState(pub String);

impl FromStr for BoardState {
    type Err = UnknownBoardState;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ALL_STATES
            .into_iter()
            .find(|state| state.as_str() == s)
            .ok_or_else(|| UnknownBoardState(s.to_owned()))
    }
}

/// Something that moves a board to another phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardEvent {
    /// Wait for players.
    WaitForPlayers,
    /// Go to Sauron's setup.
    NextToSauronSetupScreen,
    /// Go to Sauron's actions.
    NextToEditSauronSauronActions,
    /// Start a Place Influence action.
    NextToPlaceInfluenceScreenSauronActions,
    /// Start a Draw Shadow and Plot Cards action.
    NextToKeepScreenSauronPlotCards,
    /// Start a Command action.
    NextToCommandScreenSauronMobs,
    /// Go to the heroes' draw.
    NextToHeroDrawCardsScreen,
    /// Go to the rest step.
    NextToHeroRestScreen,
    /// Go to the story marker choice after resting.
    NextToAfterRestAdvanceStoryMarker,
    /// Go to the movement step.
    NextToHeroMovementScreen,
    /// Go to combat setup.
    NextToCombatSetupScreenBoardCombats,
    /// Go to the combat card choice.
    NextToPlayCombatCardScreenBoardCombats,
    /// Go to the card loss step.
    NextToCardsLossScreenBoardCombats,
    /// A hero was defeated.
    NextToHeroDefeatScreen,
    /// Sauron chooses the defeated hero's Haven.
    NextToSauronHavenScreen,
    /// Go to exploration.
    NextToHeroExploration,
    /// Go to the Gollum card search.
    NextToLookForGollumCardsSauronPlotCards,
    /// Go to Sauron's plot step.
    NextToPlayScreenSauronPlotCards,
    /// End the hero's turn.
    NextToFinishHeroTurn,
    /// End Sauron's turn.
    NextToFinishSauronTurn,
    /// End the game.
    FinishGame,
    /// Wait for a player's choice.
    NextToChoiceScreenBoardChoices,
}

impl BoardEvent {
    /// States the event may be fired from, and the state it leads to.
    fn rule(self) -> (&'static [BoardState], BoardState) {
        use BoardState as S;
        match self {
            Self::WaitForPlayers => (&[S::Created], S::WaitingForPlayers),
            Self::NextToSauronSetupScreen => {
                (&[S::Created, S::WaitingForPlayers], S::SauronSetupScreen)
            }
            Self::NextToEditSauronSauronActions => (
                &[
                    S::PlayScreenSauronPlotCards,
                    S::SauronSetupScreen,
                    S::PlaceInfluenceScreenSauronActions,
                    S::KeepScreenSauronPlotCards,
                    S::CommandScreenSauronMobs,
                    S::ChoiceScreenBoardChoices,
                ],
                S::EditSauronSauronActions,
            ),
            Self::NextToPlaceInfluenceScreenSauronActions => (
                &[S::EditSauronSauronActions],
                S::PlaceInfluenceScreenSauronActions,
            ),
            Self::NextToKeepScreenSauronPlotCards => {
                (&[S::EditSauronSauronActions], S::KeepScreenSauronPlotCards)
            }
            Self::NextToCommandScreenSauronMobs => {
                (&[S::EditSauronSauronActions], S::CommandScreenSauronMobs)
            }
            Self::NextToHeroDrawCardsScreen => (
                &[S::FinishHeroTurn, S::FinishSauronTurn],
                S::HeroDrawCardsScreen,
            ),
            Self::NextToHeroRestScreen => (
                &[
                    S::PlayCombatCardScreenBoardCombats,
                    S::HeroDrawCardsScreen,
                    S::HeroExploration,
                    S::AfterRestAdvanceStoryMarker,
                    S::ChoiceScreenBoardChoices,
                ],
                S::HeroRestScreen,
            ),
            Self::NextToAfterRestAdvanceStoryMarker => (
                &[S::HeroRestScreen, S::ChoiceScreenBoardChoices],
                S::AfterRestAdvanceStoryMarker,
            ),
            Self::NextToHeroMovementScreen => (
                &[
                    S::PlayCombatCardScreenBoardCombats,
                    S::HeroExploration,
                    S::HeroRestScreen,
                    S::AfterRestAdvanceStoryMarker,
                    S::ChoiceScreenBoardChoices,
                    S::HeroDrawCardsScreen,
                ],
                S::HeroMovementScreen,
            ),
            Self::NextToCombatSetupScreenBoardCombats => (
                &[
                    S::HeroMovementScreen,
                    S::HeroDrawCardsScreen,
                    S::ChoiceScreenBoardChoices,
                ],
                S::CombatSetupScreenBoardCombats,
            ),
            Self::NextToPlayCombatCardScreenBoardCombats => (
                &[
                    S::CardsLossScreenBoardCombats,
                    S::CombatSetupScreenBoardCombats,
                ],
                S::PlayCombatCardScreenBoardCombats,
            ),
            Self::NextToCardsLossScreenBoardCombats => (
                &[S::PlayCombatCardScreenBoardCombats],
                S::CardsLossScreenBoardCombats,
            ),
            Self::NextToHeroDefeatScreen => (
                &[
                    S::PlayCombatCardScreenBoardCombats,
                    S::ChoiceScreenBoardChoices,
                ],
                S::HeroDefeatScreen,
            ),
            Self::NextToSauronHavenScreen => (&[S::HeroDefeatScreen], S::SauronHavenScreen),
            Self::NextToHeroExploration => (
                &[
                    S::HeroMovementScreen,
                    S::ChoiceScreenBoardChoices,
                    S::PlayCombatCardScreenBoardCombats,
                ],
                S::HeroExploration,
            ),
            Self::NextToLookForGollumCardsSauronPlotCards => (
                &[S::PlayScreenSauronPlotCards],
                S::LookForGollumCardsSauronPlotCards,
            ),
            Self::NextToPlayScreenSauronPlotCards => (
                &[S::LookForGollumCardsSauronPlotCards, S::FinishHeroTurn],
                S::PlayScreenSauronPlotCards,
            ),
            Self::NextToFinishHeroTurn => (
                &[
                    S::HeroDefeatScreen,
                    S::SauronHavenScreen,
                    S::HeroExploration,
                    S::PlayCombatCardScreenBoardCombats,
                    S::ChoiceScreenBoardChoices,
                ],
                S::FinishHeroTurn,
            ),
            Self::NextToFinishSauronTurn => (&[S::EditSauronSauronActions], S::FinishSauronTurn),
            Self::FinishGame => (
                &[S::FinishHeroTurn, S::PlayCombatCardScreenBoardCombats],
                S::Finished,
            ),
            Self::NextToChoiceScreenBoardChoices => (
                &[
                    S::PlayScreenSauronPlotCards,
                    S::SauronSetupScreen,
                    S::LookForGollumCardsSauronPlotCards,
                    S::HeroExploration,
                    S::HeroMovementScreen,
                    S::HeroDrawCardsScreen,
                    S::HeroRestScreen,
                    S::PlayCombatCardScreenBoardCombats,
                    S::EditSauronSauronActions,
                    S::HeroDefeatScreen,
                    S::SauronHavenScreen,
                    S::FinishHeroTurn,
                ],
                S::ChoiceScreenBoardChoices,
            ),
        }
    }

    /// Name of the event, as in the original state machine.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WaitForPlayers => "wait_for_players",
            Self::NextToSauronSetupScreen => "next_to_sauron_setup_screen",
            Self::NextToEditSauronSauronActions => "next_to_edit_sauron_sauron_actions",
            Self::NextToPlaceInfluenceScreenSauronActions => {
                "next_to_place_influence_screen_sauron_actions"
            }
            Self::NextToKeepScreenSauronPlotCards => "next_to_keep_screen_sauron_plot_cards",
            Self::NextToCommandScreenSauronMobs => "next_to_command_screen_sauron_mobs",
            Self::NextToHeroDrawCardsScreen => "next_to_hero_draw_cards_screen",
            Self::NextToHeroRestScreen => "next_to_hero_rest_screen",
            Self::NextToAfterRestAdvanceStoryMarker => "next_to_after_rest_advance_story_marker",
            Self::NextToHeroMovementScreen => "next_to_hero_movement_screen",
            Self::NextToCombatSetupScreenBoardCombats => {
                "next_to_combat_setup_screen_board_combats"
            }
            Self::NextToPlayCombatCardScreenBoardCombats => {
                "next_to_play_combat_card_screen_board_combats"
            }
            Self::NextToCardsLossScreenBoardCombats => "next_to_cards_loss_screen_board_combats",
            Self::NextToHeroDefeatScreen => "next_to_hero_defeat_screen",
            Self::NextToSauronHavenScreen => "next_to_sauron_haven_screen",
            Self::NextToHeroExploration => "next_to_hero_exploration",
            Self::NextToLookForGollumCardsSauronPlotCards => {
                "next_to_look_for_gollum_cards_sauron_plot_cards"
            }
            Self::NextToPlayScreenSauronPlotCards => "next_to_play_screen_sauron_plot_cards",
            Self::NextToFinishHeroTurn => "next_to_finish_hero_turn",
            Self::NextToFinishSauronTurn => "next_to_finish_sauron_turn",
            Self::FinishGame => "finish_game",
            Self::NextToChoiceScreenBoardChoices => "next_to_choice_screen_board_choices",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for state in ALL_STATES {
            assert_eq!(state.as_str().parse::<BoardState>().unwrap(), state);
        }
    }

    #[test]
    fn allowed_transition() {
        let next = BoardState::HeroMovementScreen.transition(BoardEvent::NextToHeroExploration);
        assert_eq!(next.unwrap(), BoardState::HeroExploration);
    }

    #[test]
    fn refused_transition() {
        let next = BoardState::Created.transition(BoardEvent::NextToHeroExploration);
        assert!(matches!(next, Err(EngineError::InvalidTransition { .. })));
    }
}
