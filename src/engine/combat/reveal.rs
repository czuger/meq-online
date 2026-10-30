//! Resolution of a combat round once both secret cards are chosen.

use crate::engine::combat::card_power::{Phase, apply_card_power};
use crate::engine::combat::combatant::Combatant;
use crate::engine::combat::mob_power::{
    MobPowerEffect, apply_mob_power, mob_damage_bonus, mob_strength_discount,
};
use crate::engine::combat::side::Side;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::combat::Combat;
use crate::model::hero::Hero;
use crate::model::mob::Mob;
use crate::model::played_card::PlayedCard;

/// Reveals both secret cards and resolves the round: exhaustion, powers and damage.
///
/// Damage to the hero is added to `hero.temporary_damages`; damage to the mob is
/// removed from its life. Removing the played cards from hands is the caller's job.
///
/// # Arguments
/// * `combat` - combat in progress; its played-card history grows by one card per side.
/// * `hero` - fighting hero; reactivated unless exhausted.
/// * `mob` - fighting mob.
/// * `data` - static data, to read the played cards.
///
/// # Errors
/// [`EngineError::CombatNotReady`] when the hero strength or a secret card is missing,
/// [`EngineError::GameData`] when a played card is unknown.
pub fn reveal_secretly_played_cards(
    combat: &mut Combat,
    hero: &mut Hero,
    mob: &mut Mob,
    data: &GameData,
) -> Result<(), EngineError> {
    hero.damages_taken_this_turn = 0;
    mob.damages_taken_this_turn = 0;

    let (hero_card, mob_card) = current_cards(combat, hero, mob, data)?;
    let mut hero_side = Side::new(hero_card, combat.hero_cards.last().cloned());
    let mut mob_side = Side::new(mob_card, combat.mob_cards.last().cloned());

    check_exhaustion(combat, hero, mob, &mut hero_side, &mut mob_side)?;
    resolve_powers_and_damage(hero, mob, &mut hero_side, &mut mob_side);

    if !combat.hero_exhausted {
        hero.active = true;
    }
    combat.hero_cards.push(hero_side.current);
    combat.mob_cards.push(mob_side.current);
    Ok(())
}

/// Snapshots the cards revealed this round; exhausted sides play a placeholder.
fn current_cards(
    combat: &Combat,
    hero: &Hero,
    mob: &Mob,
    data: &GameData,
) -> Result<(PlayedCard, PlayedCard), EngineError> {
    let hero_card = if combat.hero_exhausted {
        PlayedCard::exhausted()
    } else {
        let card = combat
            .hero_secret_card
            .ok_or(EngineError::CombatNotReady("hero card missing"))?;
        PlayedCard::from_def(card, data.hero_card(&hero.code, card)?)
    };
    let mob_card = if combat.mob_exhausted {
        PlayedCard::exhausted()
    } else {
        let card = combat
            .mob_secret_card
            .ok_or(EngineError::CombatNotReady("mob card missing"))?;
        PlayedCard::from_def(card, data.mob_card(&mob.attack_deck, card)?)
    };
    Ok((hero_card, mob_card))
}

/// Spends strength; a side whose spent strength exceeds its strength is exhausted and
/// its card is cancelled (§ 8.3, § 8.7).
fn check_exhaustion(
    combat: &mut Combat,
    hero: &mut Hero,
    mob: &Mob,
    hero_side: &mut Side,
    mob_side: &mut Side,
) -> Result<(), EngineError> {
    let hero_strength = combat
        .hero_strength
        .ok_or(EngineError::CombatNotReady("hero strength not set"))?;
    combat.hero_strength_used += hero_side.current.strength_cost;
    if combat.hero_strength_used > hero_strength {
        hero_side.current.cancelled = true;
        combat.hero_exhausted = true;
        hero.active = false;
    }

    let mob_cost = (mob_side.current.strength_cost - mob_strength_discount(&mob.code)).max(0);
    combat.mob_strength_used += mob_cost;
    if combat.mob_strength_used > mob.strength {
        mob_side.current.cancelled = true;
        combat.mob_exhausted = true;
    }
    Ok(())
}

fn resolve_powers_and_damage(
    hero: &mut Hero,
    mob: &mut Mob,
    hero_side: &mut Side,
    mob_side: &mut Side,
) {
    if let MobPowerEffect::DamageHero(amount) = apply_mob_power(&mob.code, mob_side, hero_side) {
        hero.deal_damages(amount);
    }

    apply_card_power(Phase::Previous, hero_side, mob_side, hero);
    apply_card_power(Phase::Previous, mob_side, hero_side, mob);
    apply_card_power(Phase::Current, hero_side, mob_side, hero);
    apply_card_power(Phase::Current, mob_side, hero_side, mob);

    hero_side.final_attack += hero_side.current.attack;
    hero_side.final_defense += hero_side.current.defense;
    mob_side.final_attack += mob_side.current.attack;
    mob_side.final_defense += mob_side.current.defense;

    deal_damages(hero, mob, hero_side, mob_side);

    apply_card_power(Phase::After, hero_side, mob_side, hero);
    apply_card_power(Phase::After, mob_side, hero_side, mob);
}

/// Each side deals its attack minus the opposing defense; cancelled cards count as 0.
fn deal_damages(hero: &mut Hero, mob: &mut Mob, hero_side: &Side, mob_side: &Side) {
    let totals = |side: &Side| {
        if side.current.cancelled {
            (0, 0)
        } else {
            (side.final_attack, side.final_defense)
        }
    };
    let (hero_attack, hero_defense) = totals(hero_side);
    let (mob_attack, mob_defense) = totals(mob_side);

    let mut to_hero = mob_attack - hero_defense;
    if !mob_side.current.cancelled {
        to_hero += mob_damage_bonus(&mob.code, &mob_side.current);
    }
    let to_mob = hero_attack - mob_defense;

    if !mob_side.current.cancelled {
        hero.deal_damages(to_hero.max(0));
    }
    if !hero_side.current.cancelled {
        mob.deal_damages(to_mob.max(0));
    }
}
