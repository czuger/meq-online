//! Sauron's turn: setup, action markers, influence and mobs.

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::engine::influence_paths::InfluencePaths;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;

/// Life a minion recovers when healed.
const MINION_HEAL: i32 = 4;

/// Shadow pool capacity per story stage.
const SHADOW_POOL_PER_STAGE: i32 = 4;

/// Why influence couldn't be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfluenceRefusal {
    /// The location isn't linked to a stronghold.
    NoConnection,
    /// The requested value exceeds what the strongholds support.
    MaxReached,
}

impl Game {
    /// Sauron accepts the starting setup: objectives, first event, then his actions.
    ///
    /// # Errors
    /// Transition and event errors.
    pub fn sauron_setup_finished(
        &mut self,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        self.set_objectives(rng);
        self.draw_next_event_card(data)?;
        self.fire(BoardEvent::NextToEditSauronSauronActions)?;
        self.sauron.active = true;
        Ok(())
    }

    /// Replaces Sauron's action markers.
    ///
    /// # Returns
    /// How many times the actions were updated this turn.
    pub fn update_sauron_actions(&mut self, actions: Vec<String>) -> i32 {
        let added: Vec<String> = actions
            .iter()
            .filter(|a| !self.board.sauron_actions.contains(a))
            .cloned()
            .collect();
        let removed: Vec<String> = self
            .board
            .sauron_actions
            .iter()
            .filter(|a| !actions.contains(a))
            .cloned()
            .collect();
        for action in added {
            self.log(
                ActorRef::Sauron,
                &format!("sauron_actions.place.{action}"),
                &[],
            );
        }
        for action in removed {
            self.log(
                ActorRef::Sauron,
                &format!("sauron_actions.remove.{action}"),
                &[],
            );
        }
        self.board.sauron_actions = actions;
        self.board.sauron_actions_count += 1;
        self.board.sauron_actions_count
    }

    /// Sauron sets the influence of a location, within what his strongholds support.
    /// Unknown locations are ignored.
    ///
    /// # Returns
    /// Why the influence was refused, if it was.
    pub fn set_influence(
        &mut self,
        location: &str,
        value: i32,
        data: &GameData,
    ) -> Option<InfluenceRefusal> {
        if !data.locations.exists(location) {
            return None;
        }
        let name = data.locations.name(location).to_owned();
        let max =
            InfluencePaths::new(&self.board.influence, &data.paths).max_for_new_token(location);
        if max == 0 {
            self.log(
                ActorRef::Sauron,
                "place_influence.no_connection",
                &[("location", json!(name))],
            );
            return Some(InfluenceRefusal::NoConnection);
        }
        if max < value {
            self.log(
                ActorRef::Sauron,
                "place_influence.max_reached",
                &[("location", json!(name)), ("val", json!(value))],
            );
            return Some(InfluenceRefusal::MaxReached);
        }
        if self.board.influence.get(location) != Some(&value) {
            self.board.influence.insert(location.to_owned(), value);
            self.log(
                ActorRef::Sauron,
                "place_influence.success",
                &[("location", json!(name)), ("val", json!(value))],
            );
        }
        None
    }

    /// Adds one influence to the shadow pool (`current_val` 0) or removes one,
    /// capped by the story stage.
    ///
    /// # Returns
    /// The new shadow pool.
    pub fn update_shadow_pool_from_map(&mut self, current_val: i32) -> i32 {
        let old = self.board.shadow_pool;
        let wanted = if current_val == 0 { old + 1 } else { old - 1 };
        let new = wanted.min(self.board.story.stage() * SHADOW_POOL_PER_STAGE);
        if new != old {
            self.log(
                ActorRef::Sauron,
                "change_shadow_pool",
                &[("old_value", json!(old)), ("new_value", json!(new))],
            );
            self.board.shadow_pool = new;
        }
        new
    }

    /// Sauron ends his turn; every hero draws cards.
    ///
    /// # Errors
    /// Transition errors.
    pub fn terminate_sauron_turn(&mut self) -> Result<(), EngineError> {
        self.set_heroes_active(true);
        self.fire(BoardEvent::NextToFinishSauronTurn)?;
        self.fire(BoardEvent::NextToHeroDrawCardsScreen)?;
        self.board.sauron_actions_count = 0;
        Ok(())
    }

    /// Locations where Sauron may place a new monster: influenced locations and those
    /// next to the Black Serpent, minus the heroes' locations.
    pub fn monster_placement_locations(&self, data: &GameData) -> Vec<String> {
        let mut locations: Vec<String> = self
            .board
            .influence
            .iter()
            .filter(|(_, v)| **v >= 1)
            .map(|(k, _)| k.clone())
            .collect();
        locations.sort();
        if let Some(serpent) = self.mobs.iter().find(|m| m.code == "black_serpent") {
            locations.extend(
                data.paths
                    .connected(&serpent.location)
                    .into_iter()
                    .map(str::to_owned),
            );
        }
        locations.retain(|l| !self.heroes.iter().any(|h| h.location == *l));
        locations
    }

    /// Locations a mob may move to; monsters only move onto influenced locations.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the mob is unknown.
    pub fn mob_destinations(
        &self,
        mob_id: i64,
        data: &GameData,
    ) -> Result<Vec<String>, EngineError> {
        let mob = self
            .mobs
            .get(self.mob_index(mob_id)?)
            .ok_or(EngineError::NotFound {
                kind: "mob",
                id: mob_id,
            })?;
        let mut destinations: Vec<String> = data
            .paths
            .connected(&mob.location)
            .into_iter()
            .map(str::to_owned)
            .collect();
        if mob.kind == MobKind::Monster {
            destinations.retain(|l| self.board.influence.get(l).copied().unwrap_or(0) >= 1);
        }
        Ok(destinations)
    }

    /// Moves a mob.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the mob is unknown, [`EngineError::GameData`]
    /// when the location is unknown.
    pub fn move_mob(
        &mut self,
        mob_id: i64,
        location: &str,
        data: &GameData,
    ) -> Result<(), EngineError> {
        data.locations.get(location)?;
        self.mob_mut(mob_id)?.location = location.to_owned();
        Ok(())
    }

    /// Heals a minion by 4, up to its starting life.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the mob is unknown,
    /// [`EngineError::Refused`] when it isn't a minion.
    pub fn heal_minion(&mut self, mob_id: i64) -> Result<(), EngineError> {
        let mob = self.mob_mut(mob_id)?;
        if mob.kind != MobKind::Minion {
            return Err(EngineError::Refused("only minions can heal".to_owned()));
        }
        mob.life = (mob.life + MINION_HEAL).min(mob.max_life);
        Ok(())
    }
}
