//! Monsters and minions entering or leaving the map.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::engine::setup::STARTING_MINIONS;
use crate::engine::turn::STAGE_SPAWNS;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::game_data::monster_pool_color::MonsterPoolColor;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::mob::{Mob, NOTHING_CODE};

/// Minion that comes back after being defeated.
pub const RINGWRAITHS: &str = "ringwraiths";

/// Where the Ringwraiths come back.
pub(crate) const RINGWRAITHS_HOME: &str = "minas_morgul";

impl Game {
    /// Puts a new mob on the map.
    ///
    /// Minions ignore `pool`. The blank `nothing` token is a monster with no stats.
    ///
    /// # Errors
    /// [`EngineError::GameData`] when the mob is unknown or has no life defined.
    pub fn create_mob(
        &mut self,
        code: &str,
        location: &str,
        pool: Option<MonsterPoolColor>,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let mob = if code == NOTHING_CODE {
            nothing_token(location, pool)
        } else {
            let def = data.mob(code)?;
            let life = def
                .life
                .ok_or_else(|| crate::game_data::GameDataError::unknown("life of mob", code))?;
            Mob {
                id: None,
                kind: def.kind,
                code: code.to_owned(),
                name: def.name.clone(),
                location: location.to_owned(),
                pool: if def.kind == MobKind::Monster {
                    pool
                } else {
                    None
                },
                fortitude: def.fortitude,
                strength: def.strength,
                life,
                max_life: life,
                attack_deck: def.attack_deck.clone(),
                hand: vec![],
                damages_taken_this_turn: 0,
                commanded: false,
            }
        };
        self.mobs.push(mob);
        Ok(())
    }

    /// Draws the top monster of the pool matching the location's color and places it.
    /// An empty pool is first rebuilt from its discarded tokens (§ 10.2).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the pool and its discard are both empty,
    /// [`EngineError::GameData`] when the location is unknown.
    pub fn place_new_monster(
        &mut self,
        location: &str,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let color = data.locations.get(location)?.color_code;
        let pool = MonsterPoolColor::for_location_color(color);
        let pile = self.board.monster_pools.entry(pool).or_default();
        if pile.is_empty() {
            let discard = self.board.monster_discards.entry(pool).or_default();
            pile.append(discard);
            pile.shuffle(rng);
        }
        if pile.is_empty() {
            return Err(EngineError::Refused(format!(
                "monster pool {} is empty",
                pool.as_str()
            )));
        }
        let code = pile.remove(0);
        self.create_mob(&code, location, Some(pool), data)
    }

    /// Takes a monster token off the map and puts it faceup on its pool's discard
    /// pile (§ 8.3, § 8.8).
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the mob isn't on the board.
    pub fn discard_monster(&mut self, mob_id: i64) -> Result<(), EngineError> {
        let index = self.mob_index(mob_id)?;
        let mob = self.mobs.remove(index);
        if let Some(pool) = mob.pool {
            self.board
                .monster_discards
                .entry(pool)
                .or_default()
                .push(mob.code);
        }
        Ok(())
    }

    /// Minions that entered play and were defeated: those of the stages reached (or
    /// starting in play) that aren't on the board.
    pub fn defeated_minions(&self) -> Vec<String> {
        let stage = self.board.story.stage();
        let starting = STARTING_MINIONS.iter().map(|(code, _)| *code);
        let spawned = STAGE_SPAWNS
            .iter()
            .filter(|(at, _, _)| *at <= stage)
            .map(|(_, code, _)| *code);
        starting
            .chain(spawned)
            .filter(|code| !self.mobs.iter().any(|m| m.code == *code))
            .map(str::to_owned)
            .collect()
    }

    /// Defeated Ringwraiths come back on Minas Morgul, at full life, at the start of
    /// Sauron's next Action step (§ 8.8).
    ///
    /// # Errors
    /// [`EngineError::GameData`] when the Ringwraiths are missing from the data.
    pub fn return_ringwraiths(&mut self, data: &GameData) -> Result<(), EngineError> {
        if !self.board.ringwraiths_return {
            return Ok(());
        }
        self.board.ringwraiths_return = false;
        self.create_mob(RINGWRAITHS, RINGWRAITHS_HOME, None, data)?;
        let params = [("location", json!(data.locations.name(RINGWRAITHS_HOME)))];
        self.log(ActorRef::System, "ringwraiths.return", &params);
        Ok(())
    }
}

fn nothing_token(location: &str, pool: Option<MonsterPoolColor>) -> Mob {
    Mob {
        id: None,
        kind: MobKind::Monster,
        code: NOTHING_CODE.to_owned(),
        name: "Nothing".to_owned(),
        location: location.to_owned(),
        pool,
        fortitude: -1,
        strength: -1,
        life: -1,
        max_life: -1,
        attack_deck: "none".to_owned(),
        hand: vec![],
        damages_taken_this_turn: 0,
        commanded: false,
    }
}
