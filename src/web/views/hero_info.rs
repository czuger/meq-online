//! Hero sheet counters.

use crate::game_data::GameData;
use crate::model::hero::Hero;

/// Counters shown on a hero's sheet.
#[derive(Debug, Clone)]
pub struct HeroInfo {
    /// Display name.
    pub name: String,
    /// Location display name.
    pub location: String,
    /// Favor.
    pub favor: i32,
    /// Cards in hand.
    pub hand: usize,
    /// Cards in the life pool.
    pub life: usize,
    /// Cards in the rest pool.
    pub rest: usize,
    /// Cards in the damage pool.
    pub damage: usize,
    /// Fortitude.
    pub fortitude: i32,
    /// Strength.
    pub strength: i32,
    /// Agility.
    pub agility: i32,
    /// Wisdom.
    pub wisdom: i32,
}

impl HeroInfo {
    /// Counters of a hero.
    pub fn new(hero: &Hero, data: &GameData) -> Self {
        Self {
            name: data
                .heroes
                .get(&hero.code)
                .map_or_else(|| hero.code.clone(), |d| d.name.clone()),
            location: data.locations.name(&hero.location).to_owned(),
            favor: hero.favor,
            hand: hero.hand.len(),
            life: hero.life_pool.len(),
            rest: hero.rest_pool.len(),
            damage: hero.damage_pool.len(),
            fortitude: hero.fortitude,
            strength: hero.strength,
            agility: hero.agility,
            wisdom: hero.wisdom,
        }
    }
}
