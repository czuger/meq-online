//! Monsters and minions entering or leaving the map.

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::game_data::monster_pool_color::MonsterPoolColor;
use crate::model::game::Game;
use crate::model::mob::{Mob, NOTHING_CODE};

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
            }
        };
        self.mobs.push(mob);
        Ok(())
    }

    /// Draws the top monster of the pool matching the location's color and places it.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the pool is empty, [`EngineError::GameData`] when
    /// the location is unknown.
    pub fn place_new_monster(
        &mut self,
        location: &str,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let color = data.locations.get(location)?.color_code;
        let pool = MonsterPoolColor::for_location_color(color);
        let pile = self.board.monster_pools.entry(pool).or_default();
        if pile.is_empty() {
            return Err(EngineError::Refused(format!(
                "monster pool {} is empty",
                pool.as_str()
            )));
        }
        let code = pile.remove(0);
        self.create_mob(&code, location, Some(pool), data)
    }

    /// Returns a monster to the bottom of its pool and removes it from the map.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the mob isn't on the board.
    pub fn place_monster_back_to_pool(&mut self, mob_id: i64) -> Result<(), EngineError> {
        let index = self.mob_index(mob_id)?;
        let mob = self.mobs.remove(index);
        if let Some(pool) = mob.pool {
            self.board
                .monster_pools
                .entry(pool)
                .or_default()
                .push(mob.code);
        }
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
    }
}
