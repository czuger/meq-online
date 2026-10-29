//! Port of combat_exhaustion_test.rb.

use crate::engine::combat::tests::fight::Fight;
use crate::engine::test_support::{hero, mob};
use crate::game_data::combat_card_def::CardType;

fn fight(hero_used: i32, mob_used: i32) -> Fight {
    let mut f = Fight::new(hero("argalad"), mob("orc"), 5);
    f.combat.hero_strength_used = hero_used;
    f.combat.mob_strength_used = mob_used;
    f
}

#[test]
fn both_exhausted_aimed_shot_vs_reckless() {
    let mut f = fight(3, 3);
    assert_eq!(f.round(3, 10), (0, 0));
    assert!(f.combat.hero_cards.last().unwrap().cancelled);
}

#[test]
fn hero_only_exhausted_aimed_shot_vs_reckless() {
    let mut f = fight(3, 0);
    f.mob.strength = 50;
    assert_eq!(f.round(3, 10), (5, 0));
    assert!(f.combat.hero_cards.last().unwrap().cancelled);
    assert!(!f.hero.active);

    assert_eq!(f.round(3, 10), (5, 0));
    assert_eq!(
        f.combat.hero_cards.last().unwrap().card_type,
        CardType::Exhausted
    );
}

#[test]
fn mob_only_exhausted_aimed_shot_vs_reckless() {
    let mut f = fight(0, 3);
    assert_eq!(f.round(3, 10), (0, 3));
    assert!(f.combat.mob_cards.last().unwrap().cancelled);
    assert!(f.combat.mob_exhausted);
}
