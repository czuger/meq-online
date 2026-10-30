//! Character tokens on the map.

use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;

impl Game {
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
}
