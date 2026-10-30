//! Sauron drawing and keeping plot and shadow cards.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::board_state::BoardState;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::model::actor_ref::ActorRef;
use crate::model::board::Pile;
use crate::model::game::Game;

/// Which of Sauron's decks is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SauronDeck {
    /// Plot cards: cards not kept go back under the deck.
    Plot,
    /// Shadow cards: cards not kept are discarded.
    Shadow,
}

impl SauronDeck {
    fn name(self) -> &'static str {
        match self {
            Self::Plot => "plot",
            Self::Shadow => "shadow",
        }
    }
}

impl Game {
    /// Draws `count` cards into Sauron's drawn area; a depleted deck is rebuilt from
    /// its discard pile (§ 10.2).
    ///
    /// # Arguments
    /// * `deck` - which deck.
    /// * `count` - number of cards.
    /// * `rng` - random source, for refilling the deck.
    #[tracing::instrument(level = "debug", skip_all, fields(count))]
    pub fn draw_sauron_cards(&mut self, deck: SauronDeck, count: usize, rng: &mut StdRng) {
        let (pile, drawn) = self.sauron_piles(deck);
        if count > pile.draw.len() {
            pile.draw.append(&mut pile.discard);
            pile.draw.shuffle(rng);
            tracing::trace!(pile = "pile.draw", order = ?pile.draw, "shuffled");
        }
        let count = count.min(pile.draw.len());
        let cards: Vec<CardNumber> = pile.draw.drain(..count).collect();
        drawn.extend_from_slice(&cards);
        let action = format!("{}_cards.draw", deck.name());
        self.log(ActorRef::Sauron, &action, &[("count", json!(cards.len()))]);
    }

    /// Sauron draws cards straight into his hand (card effects).
    #[tracing::instrument(level = "debug", skip_all, fields(count))]
    pub fn draw_to_sauron_hand(&mut self, deck: SauronDeck, count: usize, rng: &mut StdRng) {
        let (_, drawn) = self.sauron_piles(deck);
        let waiting = drawn.len();
        self.draw_sauron_cards(deck, count, rng);
        let (hand, drawn) = match deck {
            SauronDeck::Plot => (&mut self.sauron.plot_hand, &mut self.sauron.drawn_plot),
            SauronDeck::Shadow => (&mut self.sauron.shadow_hand, &mut self.sauron.drawn_shadow),
        };
        hand.extend(drawn.split_off(waiting));
    }

    /// Number of drawn cards Sauron must keep: what the starting plot says at setup
    /// (§ 3), one plot card in a Draw Shadow and Plot Cards action (§ 5.7).
    pub fn cards_to_keep(&self, deck: SauronDeck, data: &GameData) -> usize {
        let drawn = match deck {
            SauronDeck::Plot => self.sauron.drawn_plot.len(),
            SauronDeck::Shadow => self.sauron.drawn_shadow.len(),
        };
        let expected = if self.board.state == BoardState::SauronSetupScreen {
            let starting = self
                .board
                .plots
                .values()
                .next()
                .and_then(|card| data.plot(*card).ok());
            let keep = starting.and_then(|plot| match deck {
                SauronDeck::Plot => plot.plot_cards,
                SauronDeck::Shadow => plot.shadow_cards,
            });
            keep.map_or(0, |k| usize::try_from(k.keep).unwrap_or(0))
        } else {
            1
        };
        expected.min(drawn)
    }

    /// Moves the selected drawn cards to Sauron's hand; the others are returned under
    /// the deck (plots) or discarded (shadows).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the number of selected cards isn't the number to
    /// keep, [`EngineError::CardNotAvailable`] when a selected card wasn't drawn.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn keep_sauron_cards(
        &mut self,
        deck: SauronDeck,
        selected: &[CardNumber],
        data: &GameData,
    ) -> Result<(), EngineError> {
        let to_keep = self.cards_to_keep(deck, data);
        if selected.len() != to_keep {
            return Err(EngineError::Refused(format!("keep {to_keep} card(s)")));
        }
        let (pile, drawn) = self.sauron_piles(deck);
        if let Some(card) = selected.iter().find(|c| !drawn.contains(c)) {
            return Err(EngineError::CardNotAvailable {
                card: *card,
                pile: "drawn cards",
            });
        }
        drawn.retain(|c| !selected.contains(c));
        let rest: Vec<CardNumber> = std::mem::take(drawn);
        let rest_count = rest.len();
        match deck {
            SauronDeck::Plot => pile.draw.extend(rest),
            SauronDeck::Shadow => pile.discard.extend(rest),
        }
        let hand = match deck {
            SauronDeck::Plot => &mut self.sauron.plot_hand,
            SauronDeck::Shadow => &mut self.sauron.shadow_hand,
        };
        hand.extend_from_slice(selected);

        let name = deck.name();
        let returned = match deck {
            SauronDeck::Plot => "back_to_bottom",
            SauronDeck::Shadow => "discard",
        };
        self.log(
            ActorRef::Sauron,
            &format!("{name}_cards.{returned}"),
            &[("count", json!(rest_count))],
        );
        self.log(
            ActorRef::Sauron,
            &format!("{name}_cards.keep"),
            &[("count", json!(selected.len()))],
        );
        Ok(())
    }

    fn sauron_piles(&mut self, deck: SauronDeck) -> (&mut Pile, &mut Vec<CardNumber>) {
        match deck {
            SauronDeck::Plot => (&mut self.board.decks.plot, &mut self.sauron.drawn_plot),
            SauronDeck::Shadow => (&mut self.board.decks.shadow, &mut self.sauron.drawn_shadow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::{data, game, rng};

    // Port of game_engine_deck_test.rb.
    #[test]
    fn refills_shadow_deck_when_near_empty() {
        let mut g = game();
        g.board.decks.shadow = Pile {
            draw: vec![4],
            discard: vec![10, 9, 8, 7, 6, 5],
        };
        g.sauron.drawn_shadow = vec![];
        g.draw_sauron_cards(SauronDeck::Shadow, 3, &mut rng());

        let all = [10, 9, 8, 7, 6, 5, 4];
        assert_eq!(g.sauron.drawn_shadow.len(), 3);
        assert!(g.sauron.drawn_shadow.iter().all(|c| all.contains(c)));
        assert_eq!(g.board.decks.shadow.draw.len(), 4);
        assert!(g.board.decks.shadow.discard.is_empty());
    }

    #[test]
    fn kept_plot_cards_go_to_hand_and_others_back_under_the_deck() {
        let mut g = game();
        g.sauron.plot_hand = vec![];
        g.sauron.drawn_plot = vec![];
        g.board.decks.plot.draw = vec![3, 4, 5, 6];
        g.draw_sauron_cards(SauronDeck::Plot, 3, &mut rng());
        g.board.state = BoardState::KeepScreenSauronPlotCards;
        g.keep_sauron_cards(SauronDeck::Plot, &[4], data()).unwrap();
        assert_eq!(g.sauron.plot_hand, vec![4]);
        assert!(g.sauron.drawn_plot.is_empty());
        assert_eq!(g.board.decks.plot.draw, vec![6, 3, 5]);
    }

    #[test]
    fn refuses_cards_not_drawn() {
        let mut g = game();
        g.sauron.drawn_plot = vec![3];
        assert!(g.keep_sauron_cards(SauronDeck::Plot, &[9], data()).is_err());
        assert!(g.keep_sauron_cards(SauronDeck::Plot, &[], data()).is_err());
    }
}
