//! What a hero meets on its location.

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::combat::start::start_combat;
use crate::engine::engine_error::EngineError;
use crate::engine::tokens::{TokenKind, tokens_by_location};
use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::mob::NOTHING_CODE;

/// Result of checking a location for encounters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encounter {
    /// Nothing to see (or a blank monster token was revealed and removed).
    Empty,
    /// A combat started.
    Monster,
    /// There are characters, favors or plots to explore.
    Exploration,
}

impl Game {
    /// Checks the hero's location; the strongest mob there (life, then strength) fights.
    /// `ambush` tells whether this is the Ambush step (otherwise the Travel step).
    ///
    /// # Errors
    /// [`EngineError::CombatInProgress`] when a combat is already running; lookup errors.
    pub fn check_location_encounters(
        &mut self,
        hero_id: i64,
        ambush: bool,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Encounter, EngineError> {
        let location = self.hero(hero_id)?.location.clone();
        let tokens = tokens_by_location(self, data);
        let has_tokens = tokens
            .get(&location)
            .is_some_and(|list| list.iter().any(|t| t.kind != TokenKind::Hero));
        if !has_tokens {
            return Ok(Encounter::Empty);
        }

        let strongest = self
            .mobs
            .iter()
            .enumerate()
            .filter(|(_, m)| m.location == location)
            // `max_by_key` keeps the last maximum; reversing keeps the first, as SQL did.
            .rev()
            .max_by_key(|(_, m)| (m.life, m.strength))
            .map(|(index, _)| index);
        let Some(index) = strongest else {
            return Ok(Encounter::Exploration);
        };

        let (mob_id, code) = self
            .mobs
            .get(index)
            .map(|m| (m.id, m.code.clone()))
            .unwrap_or_default();
        if code == NOTHING_CODE {
            let mob_id = mob_id.ok_or(EngineError::Unpersisted("mob"))?;
            self.discard_monster(mob_id)?;
            let hero_name = data.hero(&self.hero(hero_id)?.code)?.name.clone();
            let params = [("location", json!(location)), ("hero", json!(hero_name))];
            self.log(ActorRef::Hero(hero_id), "combat.encounter_nothing", &params);
            return Ok(Encounter::Empty);
        }

        if self.combat.is_some() {
            return Err(EngineError::CombatInProgress);
        }
        let hero_index = self.hero_index(hero_id)?;
        let (Some(hero), Some(mob)) = (self.heroes.get(hero_index), self.mobs.get_mut(index))
        else {
            return Err(EngineError::NotFound { kind: "mob", id: 0 });
        };
        let mut combat = start_combat(hero, mob, data, rng)?;
        combat.ambush = ambush;
        self.combat = Some(combat);
        Ok(Encounter::Monster)
    }
}
