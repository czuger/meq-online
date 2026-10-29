//! Combat test helper.

use crate::engine::combat::reveal::reveal_secretly_played_cards;
use crate::engine::combat::start::start_combat;
use crate::engine::test_support::{data, rng};
use crate::model::combat::Combat;
use crate::model::hero::Hero;
use crate::model::mob::Mob;

/// A combat between a hero and a mob, as set up by the Rails tests.
pub struct Fight {
    pub combat: Combat,
    pub hero: Hero,
    pub mob: Mob,
}

impl Fight {
    pub fn new(mut hero: Hero, mut mob: Mob, hero_strength: i32) -> Self {
        let mut combat = start_combat(&hero, &mut mob, data(), &mut rng()).unwrap();
        combat.hero_strength = Some(hero_strength);
        hero.active = true;
        Self { combat, hero, mob }
    }

    /// Reveals a round and returns `(hero damage taken, mob life lost)`.
    pub fn round(&mut self, hero_card: u32, mob_card: u32) -> (i32, i32) {
        let damages_before = self.hero.temporary_damages;
        let life_before = self.mob.life;
        self.combat.hero_secret_card = Some(hero_card);
        self.combat.mob_secret_card = Some(mob_card);
        reveal_secretly_played_cards(&mut self.combat, &mut self.hero, &mut self.mob, data())
            .unwrap();
        (
            self.hero.temporary_damages - damages_before,
            life_before - self.mob.life,
        )
    }

    pub fn hero_card_by_name(&self, name: &str) -> u32 {
        card_by_name(&data().hero(&self.hero.code).unwrap().cards, name)
    }

    pub fn mob_card_by_name(&self, name: &str) -> u32 {
        card_by_name(
            &data().attack_deck(&self.mob.attack_deck).unwrap().cards,
            name,
        )
    }
}

fn card_by_name(
    cards: &std::collections::BTreeMap<u32, crate::game_data::combat_card_def::CombatCardDef>,
    name: &str,
) -> u32 {
    cards
        .iter()
        .find(|(_, c)| c.name == name)
        .map(|(n, _)| *n)
        .unwrap_or_else(|| panic!("card {name} not found"))
}
