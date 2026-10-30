//! Command Monsters and Minions (§ 5.7): move, place a monster token, heal a minion.

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::mob::Mob;
use crate::model::sauron_action::{ActionInProgress, SauronAction};

/// Damage a minion recovers when healed.
const MINION_HEAL: i32 = 4;

/// Minion whose ability lets Sauron place monsters next to it.
const BLACK_SERPENT: &str = "black_serpent";

impl Game {
    /// The Command action in progress, when it still has a command.
    ///
    /// # Errors
    /// [`EngineError::Refused`] outside a Command action or without commands left.
    fn command(&mut self) -> Result<&mut ActionInProgress, EngineError> {
        match self.board.current_action.as_mut() {
            Some(action) if action.action == SauronAction::Command && action.commands_left > 0 => {
                Ok(action)
            }
            Some(action) if action.action == SauronAction::Command => {
                Err(EngineError::Refused("no command left".to_owned()))
            }
            _ => Err(EngineError::Refused(
                "only during a Command action".to_owned(),
            )),
        }
    }

    /// The mob, once it is checked it hasn't received a command in this action.
    fn uncommanded_mob(&self, mob_id: i64) -> Result<&Mob, EngineError> {
        let mob = self
            .mobs
            .get(self.mob_index(mob_id)?)
            .ok_or(EngineError::NotFound {
                kind: "mob",
                id: mob_id,
            })?;
        if mob.commanded {
            return Err(EngineError::Refused(
                "each monster or minion gets one command per action".to_owned(),
            ));
        }
        Ok(mob)
    }

    /// Locations where Sauron may place a monster token: influenced locations and those
    /// next to the Black Serpent, never on a hero.
    pub fn monster_placement_locations(&self, data: &GameData) -> Vec<String> {
        let mut locations: Vec<String> = self
            .board
            .influence
            .iter()
            .filter(|(_, v)| **v >= 1)
            .map(|(k, _)| k.clone())
            .collect();
        locations.sort();
        if let Some(serpent) = self.mobs.iter().find(|m| m.code == BLACK_SERPENT) {
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

    /// Locations a mob may move to: adjacent ones, with influence for monster tokens.
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

    /// Command: moves a monster or minion to an adjacent location.
    ///
    /// # Errors
    /// [`EngineError::Refused`] outside a Command action, without commands left, for a
    /// mob already commanded or a forbidden destination; lookup errors.
    pub fn command_move_mob(
        &mut self,
        mob_id: i64,
        location: &str,
        data: &GameData,
    ) -> Result<(), EngineError> {
        self.command()?;
        self.uncommanded_mob(mob_id)?;
        if !self
            .mob_destinations(mob_id, data)?
            .iter()
            .any(|l| l == location)
        {
            return Err(EngineError::Refused(format!("can't move to {location}")));
        }
        let mob = self.mob_mut(mob_id)?;
        mob.location = location.to_owned();
        mob.commanded = true;
        let name = mob.name.clone();
        self.command()?.commands_left -= 1;
        let params = [
            ("name", json!(name)),
            ("location", json!(data.locations.name(location))),
        ];
        self.log(ActorRef::Sauron, "command.move", &params);
        Ok(())
    }

    /// Command: draws a monster token and places it; one per Command action.
    ///
    /// # Errors
    /// [`EngineError::Refused`] outside a Command action, without commands left, after
    /// a placement in this action or on a forbidden location; pool errors.
    pub fn command_place_monster(
        &mut self,
        location: &str,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if self.command()?.monster_placed {
            return Err(EngineError::Refused(
                "one monster token per Command action".to_owned(),
            ));
        }
        if !self
            .monster_placement_locations(data)
            .iter()
            .any(|l| l == location)
        {
            return Err(EngineError::Refused(format!(
                "can't place a monster on {location}"
            )));
        }
        self.place_new_monster(location, data, rng)?;
        if let Some(mob) = self.mobs.last_mut() {
            // A token can't be moved by the action that placed it.
            mob.commanded = true;
        }
        let action = self.command()?;
        action.commands_left -= 1;
        action.monster_placed = true;
        let params = [("location", json!(data.locations.name(location)))];
        self.log(ActorRef::Sauron, "command.place", &params);
        Ok(())
    }

    /// Command: removes up to four damage from a minion that isn't in a hero's location.
    ///
    /// # Errors
    /// [`EngineError::Refused`] outside a Command action, without commands left, for a
    /// mob already commanded, a monster, or a minion in a hero's location.
    pub fn command_heal_minion(&mut self, mob_id: i64) -> Result<(), EngineError> {
        self.command()?;
        let mob = self.uncommanded_mob(mob_id)?;
        if mob.kind != MobKind::Minion {
            return Err(EngineError::Refused("only minions can heal".to_owned()));
        }
        if self.heroes.iter().any(|h| h.location == mob.location) {
            return Err(EngineError::Refused(
                "a minion can't heal in a hero's location".to_owned(),
            ));
        }
        let mob = self.mob_mut(mob_id)?;
        mob.life = (mob.life + MINION_HEAL).min(mob.max_life);
        mob.commanded = true;
        let name = mob.name.clone();
        self.command()?.commands_left -= 1;
        self.log(ActorRef::Sauron, "command.heal", &[("name", json!(name))]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, mob, rng};

    fn commanding(value: i32) -> Game {
        let mut g = game();
        g.board.state = BoardState::CommandScreenSauronMobs;
        g.board.current_action = Some(ActionInProgress {
            action: SauronAction::Command,
            value,
            commands_left: value,
            monster_placed: false,
        });
        let mut serpent = mob("black_serpent");
        serpent.location = "near_harad".to_owned();
        serpent.life = 2;
        g.mobs.push(serpent);
        g
    }

    // § 5.7: a minion can't be healed while in a hero's location.
    #[test]
    fn no_healing_in_a_hero_location() {
        let mut g = commanding(3);
        g.heroes[0].location = "near_harad".to_owned();
        assert!(g.command_heal_minion(1).is_err());
        g.heroes[0].location = "the_shire".to_owned();
        g.command_heal_minion(1).unwrap();
        assert_eq!(g.mobs[0].life, 6);
    }

    // § 5.7: at most one command per monster or minion and per action.
    #[test]
    fn one_command_per_mob() {
        let mut g = commanding(3);
        g.command_move_mob(1, "pelargir", data()).unwrap();
        assert!(g.command_move_mob(1, "near_harad", data()).is_err());
        assert!(g.command_heal_minion(1).is_err());
        assert_eq!(g.board.current_action.unwrap().commands_left, 2);
    }

    #[test]
    fn moves_only_to_adjacent_locations() {
        let mut g = commanding(3);
        assert!(g.command_move_mob(1, "the_shire", data()).is_err());
    }

    // § 5.7: X commands; one monster token per action, never on a hero.
    #[test]
    fn commands_and_placements_are_limited() {
        let mut g = commanding(1);
        let hero_location = g.heroes[0].location.clone();
        g.board.influence.insert(hero_location.clone(), 1);
        assert!(
            g.command_place_monster(&hero_location, data(), &mut rng())
                .is_err()
        );
        g.command_place_monster("pelargir", data(), &mut rng())
            .unwrap();
        assert!(g.mobs.last().unwrap().commanded);
        assert!(g.command_heal_minion(1).is_err(), "no command left");

        let mut g = commanding(3);
        g.command_place_monster("pelargir", data(), &mut rng())
            .unwrap();
        assert!(
            g.command_place_monster("osgiliath", data(), &mut rng())
                .is_err()
        );
    }

    #[test]
    fn no_command_outside_a_command_action() {
        let mut g = commanding(3);
        g.board.current_action = None;
        assert!(g.command_heal_minion(1).is_err());
    }
}
