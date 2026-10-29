//! Innate powers of some mobs.

use crate::engine::combat::side::Side;
use crate::model::played_card::PlayedCard;

/// Effect of a mob's innate power, resolved at the start of a round.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobPowerEffect {
    /// Nothing happens.
    None,
    /// The hero takes this much damage.
    DamageHero(i32),
}

/// Triggers the innate power of the mob `code`, before any card power.
///
/// # Arguments
/// * `code` - mob code.
/// * `mob` - mob side of the round.
/// * `hero` - hero side of the round.
///
/// # Returns
/// Damage to apply to the hero, if any.
pub fn apply_mob_power(code: &str, mob: &Side, hero: &mut Side) -> MobPowerEffect {
    match code {
        // Orcs hurt in melee exchanges.
        "orc" if hero.current.is_melee() && mob.current.is_melee() => MobPowerEffect::DamageHero(1),
        // Cave trolls throwing rocks cancel the hero's card.
        "cave_troll" if mob.current.is_ranged() => {
            hero.current.cancelled = true;
            MobPowerEffect::None
        }
        _ => MobPowerEffect::None,
    }
}

/// Extra damage dealt by the mob `code` with an uncancelled `card`.
pub fn mob_damage_bonus(code: &str, card: &PlayedCard) -> i32 {
    match code {
        "snaga" if card.is_ranged() => 1,
        "huorn" if card.is_melee() => 1,
        _ => 0,
    }
}

/// Strength discount the mob `code` gets on each card.
pub fn mob_strength_discount(code: &str) -> i32 {
    match code {
        "southron" => 1,
        _ => 0,
    }
}
