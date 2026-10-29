//! French description of game phases.

use crate::engine::board_state::BoardState;

/// What the game is waiting for, in French.
pub fn state_label(state: BoardState) -> &'static str {
    match state {
        BoardState::Created => "Partie créée",
        BoardState::WaitingForPlayers => "En attente de joueurs",
        BoardState::SauronSetupScreen => "Mise en place de Sauron",
        BoardState::EditSauronSauronActions => "Actions de Sauron",
        BoardState::HeroDrawCardsScreen => "Pioche des héros",
        BoardState::HeroRestScreen => "Repos du héros",
        BoardState::FinishHeroTurn => "Fin du tour du héros",
        BoardState::FinishSauronTurn => "Fin du tour de Sauron",
        BoardState::AfterDefeatAdvanceStoryMarker => "Défaite du héros",
        BoardState::HeroMovementScreen => "Déplacement du héros",
        BoardState::HeroExploration => "Exploration du héros",
        BoardState::PlayScreenSauronPlotCards => "Sauron joue une intrigue",
        BoardState::AfterRestAdvanceStoryMarker => "Choix du marqueur d'histoire",
        BoardState::CombatSetupScreenBoardCombats => "Préparation du combat",
        BoardState::PlayCombatCardScreenBoardCombats => "Combat",
        BoardState::LookForGollumCardsSauronPlotCards => "Recherche d'une carte Gollum",
        BoardState::Finished => "Partie terminée",
        BoardState::CardsLossScreenBoardCombats => "Pertes du héros",
    }
}
