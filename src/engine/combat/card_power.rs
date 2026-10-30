//! Effects of combat card powers.

use crate::engine::combat::combatant::Combatant;
use crate::engine::combat::side::Side;
use crate::game_data::combat_card_def::CardPower;

/// Step of the round resolution in which a card power is triggered.
///
/// The original rules engine also had a "current cancel" step meant for cancelling
/// powers, but its check matched the current step, so cancelling powers (Charge,
/// Evade) actually resolve during [`Phase::Current`]. The step is omitted here and
/// the resulting order is kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Powers of the card played in the previous round.
    Previous,
    /// Powers of the card played this round, before damage.
    Current,
    /// Powers of the card played this round, after damage.
    After,
}

/// Triggers the power of one of `me`'s cards for `phase`.
///
/// In [`Phase::Previous`] the triggering card is `me.previous`; otherwise it is
/// `me.current`. A cancelled card triggers nothing.
///
/// # Arguments
/// * `phase` - resolution step.
/// * `me` - side owning the card.
/// * `op` - opposing side.
/// * `me_fighter` - entity owning the card, for powers that hurt their owner.
#[tracing::instrument(level = "debug", skip_all, fields(?phase))]
pub fn apply_card_power(
    phase: Phase,
    me: &mut Side,
    op: &mut Side,
    me_fighter: &mut impl Combatant,
) {
    let card = match phase {
        Phase::Previous => me.previous.as_ref(),
        Phase::Current | Phase::After => Some(&me.current),
    };
    let Some(card) = card else { return };
    if card.cancelled {
        return;
    }
    match (card.power, phase) {
        (CardPower::Precision | CardPower::Smash, Phase::Current) => op.current.defense = 0,
        (CardPower::DeadlyFinesse, Phase::Current) => deadly_finesse(op),
        (CardPower::Reckless, Phase::After) => {
            if me_fighter.damages_taken_this_turn() >= 1 {
                me_fighter.deal_damages(2);
            }
        }
        (CardPower::Charge, Phase::Current) => {
            if !op.current.cancelled && op.current.is_ranged() {
                op.cancel_current();
            }
        }
        (CardPower::Evade, Phase::Current) => {
            if !op.current.cancelled && op.current.is_melee() {
                op.cancel_current();
            }
        }
        (CardPower::Parry, Phase::Current) => {
            if !op.current.cancelled && op.current.is_melee() {
                me.final_defense += 2;
            }
        }
        _ => apply_bonus_power(card.power, phase, me, op),
    }
}

/// Powers that only add attack or defense, or cancel from the previous round.
#[tracing::instrument(level = "debug", skip_all, fields(?phase))]
fn apply_bonus_power(power: CardPower, phase: Phase, me: &mut Side, op: &mut Side) {
    match (power, phase) {
        (CardPower::Concentrate, Phase::Previous) => {
            me.final_attack += 2;
            me.final_defense += 2;
        }
        (CardPower::FallBack, Phase::Previous) => {
            if !op.current.cancelled && op.current.is_melee() {
                op.cancel_current();
            }
        }
        (CardPower::Volley, Phase::Previous) => {
            if !me.current.cancelled && me.current.is_ranged() {
                me.final_attack += 2;
            }
        }
        (CardPower::AimedShot, Phase::Current) => {
            let previous_intact = op.previous.as_ref().is_none_or(|p| !p.cancelled);
            let same_type = op
                .previous
                .as_ref()
                .is_some_and(|p| p.card_type == op.current.card_type);
            if !op.current.cancelled && previous_intact && same_type {
                me.final_attack += 3;
            }
        }
        (CardPower::Overdraw | CardPower::RangedStrike, Phase::Current) => {
            if !op.current.cancelled && op.current.is_ranged() {
                me.final_attack += 2;
            }
        }
        (CardPower::Hack, Phase::Current) => {
            if !op.current.cancelled && op.current.is_melee() {
                me.final_attack += 1;
            }
        }
        (CardPower::AttackOfOpportunity, Phase::Current)
            if op.current.cancelled || op.current.defense == 0 =>
        {
            me.final_attack += 5;
        }
        _ => {}
    }
}

/// Brings the opponent's printed values down to 1.
///
/// The totals are lowered by the printed value minus 1 *and* the printed value is
/// then set to 1 and added later, as in the original engine.
#[tracing::instrument(level = "debug", skip_all)]
fn deadly_finesse(op: &mut Side) {
    op.final_defense -= op.current.defense - 1;
    op.final_attack -= op.current.attack - 1;
    op.current.defense = 1;
    op.current.attack = 1;
}
