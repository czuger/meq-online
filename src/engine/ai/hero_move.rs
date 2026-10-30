//! One move of a computer hero: a call to an engine entry point.

use rand::rngs::StdRng;

use crate::engine::EngineError;
use crate::engine::combat::flow::CombatSide;
use crate::engine::defeat::DefeatChoice;
use crate::engine::exploration::ExploreChoice;
use crate::engine::rest::Rest;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::plot::StoryType;
use crate::model::game::Game;

/// A move the computer hero may try, as the matching screen's form would submit it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeroMove {
    /// Draws this many cards in the Hero Draw step.
    Draw(usize),
    /// Ends the hero's Hero Draw step.
    DrawFinished,
    /// Rests (or heals) in the Rest step.
    Rest(Rest),
    /// Skips the Rest step.
    SkipRest,
    /// Picks the tied story marker advancing after a rest.
    AdvanceMarker(StoryType),
    /// Travels to `to`, paying with `cards`.
    Move {
        /// Destination location code.
        to: String,
        /// Hand cards paying for the path.
        cards: Vec<CardNumber>,
    },
    /// Ends the Travel step.
    MovementFinished,
    /// Explores the hero's location.
    Explore(ExploreChoice),
    /// Ends the hero's turn after exploring.
    EndTurn,
    /// Splits agility between drawing cards and strength.
    CombatSetup(i32),
    /// Secretly chooses a combat card.
    CombatCard(CardNumber),
    /// Declares itself exhausted for the round.
    Exhaustion,
    /// Ends a combat that is over.
    Terminate,
    /// Pays the round's damage with these hand cards, the rest from the life pool.
    CardsLoss(Vec<CardNumber>),
    /// Resolves the hero's defeat.
    Defeat(DefeatChoice),
    /// Answers the pending choice.
    Answer {
        /// Option code.
        option: String,
        /// Targets picked within the option.
        targets: Vec<String>,
    },
}

impl HeroMove {
    /// Applies the move for `hero_id`.
    ///
    /// # Errors
    /// The errors of the engine entry point: the rules refused the move.
    pub fn apply(
        &self,
        game: &mut Game,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        match self {
            Self::Draw(count) => {
                game.hero_mut(hero_id)?.draw_step_cards(*count, data);
            }
            Self::DrawFinished => game.draw_cards_finished(hero_id, data, rng)?,
            Self::Rest(rest) => {
                game.rest_hero(hero_id, *rest, data, rng)?;
            }
            Self::SkipRest => game.skip_rest()?,
            Self::AdvanceMarker(marker) => game.advance_chosen_story_marker(*marker, data)?,
            Self::Move { to, cards } => {
                game.move_hero(hero_id, to, cards, data, rng)?;
            }
            Self::MovementFinished => game.movement_finished()?,
            Self::Explore(choice) => {
                game.explore(hero_id, choice, data, rng)?;
            }
            Self::EndTurn => {
                game.finish_heroes_turn(hero_id, data, rng)?;
            }
            Self::CombatSetup(draw) => game.combat_setup(*draw, data, rng)?,
            Self::CombatCard(card) => {
                game.play_combat_card(CombatSide::Hero, *card, data)?;
            }
            Self::Exhaustion => {
                game.declare_exhaustion(CombatSide::Hero, data)?;
            }
            Self::Terminate => {
                game.terminate_combat(data, rng)?;
            }
            Self::CardsLoss(cards) => game.combat_cards_loss(cards)?,
            Self::Defeat(choice) => {
                game.resolve_hero_defeat(hero_id, *choice, data, rng)?;
            }
            Self::Answer { option, targets } => game.answer_choice(option, targets, data, rng)?,
        }
        Ok(())
    }
}
