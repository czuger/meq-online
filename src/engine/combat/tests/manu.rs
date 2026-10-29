//! Port of combat_manu_test.rb.

use crate::engine::combat::tests::fight::Fight;
use crate::engine::test_support::{data, hero, mob};

fn fight() -> Fight {
    let mut monster = mob("agent");
    monster.strength = 50;
    monster.life = 50;
    Fight::new(hero("argalad"), monster, 50)
}

fn switch_hero(f: &mut Fight, code: &str) {
    f.hero.code = code.to_owned();
    f.hero.hand = data().hero(code).unwrap().starting_deck.clone();
}

#[test]
fn combat_4() {
    let mut f = Fight::new(hero("argalad"), mob("snaga"), 5);
    let (precision, hack) = (f.hero_card_by_name("Precision"), f.mob_card_by_name("Hack"));
    assert_eq!(f.round(precision, hack), (0, 2));
}

#[test]
fn combat_3() {
    let mut f = fight();
    switch_hero(&mut f, "beravor");
    assert_eq!(f.round(0, 4), (0, 2));
    assert_eq!(f.round(6, 7), (1, 0));
    assert_eq!(f.round(4, 10), (5, 8));
}

#[test]
fn combat_2() {
    let mut f = fight();
    switch_hero(&mut f, "beravor");
    assert_eq!(f.round(5, 10), (2, 3));
    switch_hero(&mut f, "argalad");
    assert_eq!(f.round(4, 6), (0, 0));
    assert_eq!(f.round(6, 7), (0, 1));
}

#[test]
fn combat_1() {
    let mut f = fight();
    assert_eq!(f.round(8, 10), (5, 9));
    switch_hero(&mut f, "beravor");
    assert_eq!(f.round(1, 10), (0, 0));
    assert_eq!(f.round(0, 9), (2, 0));
}
