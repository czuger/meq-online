//! Port of combat_mobs_powers_test.rb.

use crate::engine::combat::tests::fight::Fight;
use crate::engine::test_support::{hero, mob};

fn fight(code: &str) -> Fight {
    Fight::new(hero("argalad"), mob(code), 5)
}

#[test]
fn orc_power() {
    assert_eq!(fight("orc").round(6, 9), (1, 0));
}

#[test]
fn snaga_power() {
    assert_eq!(fight("snaga").round(6, 1), (1, 0));
}

#[test]
fn huorn_power() {
    assert_eq!(fight("huorn").round(6, 9), (3, 3));
}

#[test]
fn cave_troll_power() {
    assert_eq!(fight("cave_troll").round(2, 1), (3, 0));
}

#[test]
fn southron_power() {
    let mut f = fight("southron");
    assert_eq!(f.round(2, 10), (1, 2));
    assert_eq!(f.combat.mob_strength_used, 2);
}

#[test]
fn southron_power_assert_zero_strength_cost() {
    let mut f = fight("southron");
    assert_eq!(f.round(2, 0), (0, 2));
    assert_eq!(f.combat.mob_strength_used, 0);
}
