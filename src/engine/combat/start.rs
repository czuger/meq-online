//! Creation of a combat.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::combat::Combat;
use crate::model::hero::Hero;
use crate::model::mob::Mob;

/// Starts a combat: the mob draws `fortitude` cards from a freshly shuffled attack deck.
///
/// # Arguments
/// * `hero` - fighting hero.
/// * `mob` - fighting mob; its hand is replaced.
/// * `data` - static data, for the attack deck.
/// * `rng` - random source for the shuffle.
///
/// # Returns
/// The new combat.
///
/// # Errors
/// [`EngineError::Unpersisted`] when the hero or mob has no id yet,
/// [`EngineError::GameData`] when the attack deck is unknown.
pub fn start_combat(
    hero: &Hero,
    mob: &mut Mob,
    data: &GameData,
    rng: &mut StdRng,
) -> Result<Combat, EngineError> {
    let hero_id = hero.id.ok_or(EngineError::Unpersisted("hero"))?;
    let mob_id = mob.id.ok_or(EngineError::Unpersisted("mob"))?;

    let mut deck = data.attack_deck(&mob.attack_deck)?.deck.clone();
    deck.shuffle(rng);
    deck.truncate(usize::try_from(mob.fortitude).unwrap_or(0));
    mob.hand = deck;

    Ok(Combat {
        hero_id,
        mob_id,
        ..Combat::default()
    })
}
