//! Port of combat_scenarios_test.rb.

use crate::engine::combat::tests::fight::Fight;
use crate::engine::test_support::{hero, mob};

fn fight() -> Fight {
    Fight::new(hero("argalad"), mob("southron"), 5 + 3)
}

fn with_zealot(mut fight: Fight) -> Fight {
    fight.mob.attack_deck = "zealot".to_owned();
    fight
}

#[test]
fn precision_vs_ranged_strike() {
    let mut f = with_zealot(fight());
    assert_eq!(f.round(2, 1), (2, 2));
}

#[test]
fn attack_of_opportunity_vs_charge() {
    let mut f = with_zealot(fight());
    assert_eq!(f.round(8, 8), (2, 7));
}

#[test]
fn charge_vs_aimed_shot() {
    let mut f = fight();
    assert_eq!(f.round(3, 7), (2, 0));
}

#[test]
fn fall_back_then_parry_vs_attack_of_opportunity() {
    let mut f = fight();
    f.mob.strength = 50;
    assert_eq!(f.round(4, 10), (1, 1));
    assert_eq!(f.round(6, 10), (0, 1));
}

#[test]
fn mouth_of_sauron_example_1() {
    let mut f = Fight::new(hero("argalad"), mob("mouth_of_sauron"), 5);
    let (volley, ranged_strike) = (
        f.hero_card_by_name("Volley"),
        f.mob_card_by_name("Ranged Strike"),
    );
    assert_eq!(f.round(volley, ranged_strike), (1, 1));
    let (aimed_shot, reckless) = (
        f.hero_card_by_name("Aimed Shot"),
        f.mob_card_by_name("Reckless"),
    );
    assert_eq!(f.round(aimed_shot, reckless), (5, 0));
}

#[test]
fn combat_example_7() {
    let mut f = Fight::new(hero("argalad"), mob("black_serpent"), 8);
    assert_eq!(f.round(1, 0), (0, 4));
    assert_eq!(f.round(5, 2), (6, 0));
    assert_eq!(f.round(4, 0), (0, 3));
}
