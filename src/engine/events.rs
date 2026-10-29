//! Event cards, characters and objectives.

use rand::RngExt;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;

/// Number of objective cards for each side.
const OBJECTIVE_CARDS: u32 = 5;

impl Game {
    /// Draws the next event: it may place a character and favor tokens.
    ///
    /// Drawn events go to the discard, which is shuffled back when the deck runs out.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when no event is left at all; lookup errors.
    pub fn draw_next_event_card(&mut self, data: &GameData) -> Result<(), EngineError> {
        let pile = &mut self.board.decks.event;
        if pile.draw.is_empty() {
            pile.draw.append(&mut pile.discard);
        }
        if pile.draw.is_empty() {
            return Err(EngineError::Refused("event deck is empty".to_owned()));
        }
        let card = pile.draw.remove(0);
        pile.discard.push(card);
        self.board.last_event_card = Some(card);
        self.log(
            ActorRef::System,
            "event.choose_card",
            &[("event_card", json!(format!("I/{card}.png")))],
        );

        let event = data.event(card)?;
        if let Some(character) = &event.character {
            self.place_character(ActorRef::System, &character.name, &character.location, data)?;
        }
        for favor in &event.favors {
            self.board.favors.push(favor.clone());
            self.log(
                ActorRef::System,
                "favor.place",
                &[("location", json!(data.locations.name(favor)))],
            );
        }
        Ok(())
    }

    /// Places (or moves) a character on the map.
    ///
    /// # Errors
    /// [`EngineError::GameData`] when the character or location is unknown.
    pub fn place_character(
        &mut self,
        actor: ActorRef,
        code: &str,
        location: &str,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let name = data.character(code)?.name.clone();
        let location_name = data.locations.get(location)?.name.clone();
        self.board
            .characters
            .insert(code.to_owned(), location.to_owned());
        self.log(
            actor,
            "character.place",
            &[("name", json!(name)), ("location", json!(location_name))],
        );
        Ok(())
    }

    /// Removes a character from the map.
    ///
    /// # Errors
    /// [`EngineError::GameData`] when the character is unknown.
    pub fn remove_character(
        &mut self,
        actor: ActorRef,
        code: &str,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let name = data.character(code)?.name.clone();
        if self.board.characters.remove(code).is_some() {
            self.log(actor, "character.remove", &[("name", json!(name))]);
        }
        Ok(())
    }

    /// Deals the secret objectives of both sides.
    pub fn set_objectives(&mut self, rng: &mut StdRng) {
        self.board.heroes_objective = Some(rng.random_range(0..OBJECTIVE_CARDS));
        self.board.sauron_objective = Some(rng.random_range(0..OBJECTIVE_CARDS));
    }
}

/// Base event cards in a random order.
pub fn starting_event_deck(data: &GameData, rng: &mut StdRng) -> Vec<u32> {
    let mut deck = data.base_event_cards();
    deck.shuffle(rng);
    deck
}

/// Picture of an objective card.
pub fn objective_pic_path(side: &str, card: u32) -> String {
    format!("objectives/original/{side}/{side}_{card}.png")
}
