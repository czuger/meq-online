//! The Event step of Sauron's turn and the event card in play (§ 5.6).

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::engine::story::Dominance;
use crate::game_data::event::EventAbility;
use crate::game_data::{GameData, event_set};
use crate::model::actor_ref::ActorRef;
use crate::model::board::{EventRef, Pile};
use crate::model::game::Game;

/// Cards drawn when a side is dominant.
const DRAWN_WITH_DOMINANCE: usize = 3;

/// The three Event decks of a new game, shuffled, stage I first.
pub fn starting_event_decks(data: &GameData, rng: &mut StdRng) -> [Vec<u32>; 3] {
    [1, 2, 3].map(|stage| {
        let mut deck = data.event_cards(stage);
        deck.shuffle(rng);
        deck
    })
}

/// Picture of an event card, under `static/images/events/`.
pub fn event_pic_path(event: EventRef) -> String {
    format!("{}/{}.png", event_set(event.stage), event.card)
}

impl Game {
    /// The Event deck of a stage.
    pub fn event_pile(&mut self, stage: u8) -> &mut Pile {
        match stage {
            1 => &mut self.board.decks.event,
            2 => &mut self.board.decks.event_ii,
            _ => &mut self.board.decks.event_iii,
        }
    }

    /// Event step (§ 5.6): draws from the current stage's deck (three cards when a side
    /// is dominant, one otherwise), resolves the lowest priority when the heroes are
    /// dominant or the highest when Sauron is, then goes to the Action step unless the
    /// card waits for a choice.
    ///
    /// # Errors
    /// Lookup, choice and transition errors.
    pub fn event_step(&mut self, data: &GameData, rng: &mut StdRng) -> Result<(), EngineError> {
        let stage = u8::try_from(self.board.story.stage())
            .unwrap_or(3)
            .clamp(1, 3);
        let dominance = self.board.story.data().dominance;
        let count = if dominance == Dominance::Nobody {
            1
        } else {
            DRAWN_WITH_DOMINANCE
        };
        let drawn = self.draw_events(stage, count, rng);
        let chosen = choose_event(stage, &drawn, dominance, data)?;
        let Some(card) = chosen else {
            return self.start_action_step(data);
        };
        let others = drawn.into_iter().filter(|c| *c != card);
        self.event_pile(stage).discard.extend(others);
        self.resolve_event(EventRef { stage, card }, data)?;
        if self.board.pending_choice.is_some() {
            return Ok(());
        }
        self.start_action_step(data)
    }

    fn draw_events(&mut self, stage: u8, count: usize, rng: &mut StdRng) -> Vec<u32> {
        let pile = self.event_pile(stage);
        if pile.draw.len() < count {
            pile.discard.shuffle(rng);
            pile.draw.append(&mut pile.discard);
        }
        let count = count.min(pile.draw.len());
        pile.draw.drain(..count).collect()
    }

    /// Resolves an event: its characters and favor, the card in play or discarded,
    /// then its ability.
    ///
    /// # Errors
    /// Lookup and choice errors.
    pub fn resolve_event(&mut self, event: EventRef, data: &GameData) -> Result<(), EngineError> {
        self.board.last_event = Some(event);
        self.log(
            ActorRef::System,
            "event.choose_card",
            &[("event_card", json!(event_pic_path(event)))],
        );
        self.place_event_tokens(event, data)?;
        let def = data.event(event.stage, event.card)?;
        if def.affect.is_some() {
            if let Some(previous) = self.board.current_event.replace(event) {
                self.discard_event(previous);
            }
        } else if def.ability != EventAbility::TrustInFriendship {
            // "Trust in Friendship" stays with the hero it is given to.
            self.discard_event(event);
        }
        self.start_event_ability(event, data)
    }

    /// Characters and favor of an event; a character already on another location
    /// stays where it is.
    fn place_event_tokens(&mut self, event: EventRef, data: &GameData) -> Result<(), EngineError> {
        let def = data.event(event.stage, event.card)?;
        if let Some(character) = def
            .character
            .as_ref()
            .filter(|c| !self.board.characters.contains_key(&c.name))
        {
            self.place_character(ActorRef::System, &character.name, &character.location, data)?;
        }
        for favor in &def.favors {
            self.board.favors.push(favor.clone());
            self.log(
                ActorRef::System,
                "favor.place",
                &[("location", json!(data.locations.name(favor)))],
            );
        }
        Ok(())
    }

    /// Puts an event card faceup on its deck's discard pile.
    pub fn discard_event(&mut self, event: EventRef) {
        self.event_pile(event.stage).discard.push(event.card);
    }

    /// The card in play leaves the Current Event Card space.
    pub fn discard_current_event(&mut self) {
        if let Some(event) = self.board.current_event.take() {
            self.discard_event(event);
        }
    }

    /// Story marker advance of the event in play, applied every Story step.
    ///
    /// # Errors
    /// Lookup errors.
    pub fn advance_current_event_story(&mut self, data: &GameData) -> Result<(), EngineError> {
        let Some(event) = self.board.current_event else {
            return Ok(());
        };
        if let Some(story) = data.event(event.stage, event.card)?.story {
            self.board.story.advance(story.story_type, story.advance);
            self.log(
                ActorRef::System,
                "story.advance",
                &[
                    ("marker_name", json!(story.story_type)),
                    ("count", json!(story.advance)),
                ],
            );
        }
        Ok(())
    }
}

/// The card resolved among those drawn (§ 5.6).
fn choose_event(
    stage: u8,
    drawn: &[u32],
    dominance: Dominance,
    data: &GameData,
) -> Result<Option<u32>, EngineError> {
    let mut by_priority = drawn
        .iter()
        .map(|card| Ok((data.event(stage, *card)?.priority, *card)))
        .collect::<Result<Vec<(u32, u32)>, EngineError>>()?;
    by_priority.sort_unstable();
    let chosen = match dominance {
        Dominance::Sauron => by_priority.last(),
        Dominance::Heroes | Dominance::Nobody => by_priority.first(),
    };
    Ok(chosen.map(|(_, card)| *card))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};

    fn event_step_game() -> Game {
        let mut g = game();
        g.board.state = BoardState::PlayScreenSauronPlotCards;
        g.board.decks.event.draw = vec![];
        g
    }

    // § 5.6: heroes dominant resolve the lowest priority, Sauron dominant the highest.
    #[test]
    fn dominance_picks_the_priority() {
        // I/0 is priority 1, I/5 priority 14, I/2 priority 4: all placing no hero choice.
        let mut g = event_step_game();
        g.board.story.heroes = 5;
        g.board.decks.event.draw = vec![5, 2, 13];
        g.event_step(data(), &mut rng()).unwrap();
        assert_eq!(g.board.last_event, Some(EventRef { stage: 1, card: 2 }));
        assert_eq!(
            g.board.decks.event.discard.len(),
            3,
            "the two others and the resolved card"
        );

        let mut g = event_step_game();
        g.board.story.ring = 5;
        g.board.decks.event.draw = vec![12, 5, 13];
        g.event_step(data(), &mut rng()).unwrap();
        assert_eq!(g.board.last_event, Some(EventRef { stage: 1, card: 5 }));
        assert_eq!(
            g.board.current_event, g.board.last_event,
            "a green flag card stays"
        );
    }

    // § 5.6: without dominance, the top card; the current stage picks the deck.
    #[test]
    fn stage_picks_the_deck() {
        let mut g = event_step_game();
        g.board.story.heroes = 7;
        g.board.story.ring = 7;
        g.board.decks.event_ii.draw = vec![10, 11];
        g.event_step(data(), &mut rng()).unwrap();
        assert_eq!(g.board.last_event, Some(EventRef { stage: 2, card: 10 }));
        assert_eq!(g.board.characters["aragorn"], "bree");
        assert_eq!(g.board.characters["gandalf"], "bree");
        assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
    }

    // § 5.6: a character already on another location stays where it is.
    #[test]
    fn event_leaves_placed_characters_alone() {
        let mut g = event_step_game();
        g.board.decks.event.draw = vec![13];
        g.board
            .characters
            .insert("aragorn".to_owned(), "the_shire".to_owned());
        g.event_step(data(), &mut rng()).unwrap();
        assert_eq!(g.board.characters["aragorn"], "the_shire");
    }

    // § 5.6: a new green flag card replaces the one in play.
    #[test]
    fn new_card_in_play_replaces_the_old_one() {
        let mut g = event_step_game();
        g.board.current_event = Some(EventRef { stage: 1, card: 4 });
        g.board.decks.event.draw = vec![13];
        g.event_step(data(), &mut rng()).unwrap();
        assert_eq!(g.board.current_event, Some(EventRef { stage: 1, card: 13 }));
        assert_eq!(g.board.decks.event.discard, vec![4]);
    }
}
