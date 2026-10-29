//! Port of combat_call_only_test.rb: every card of every deck resolves without error.

use crate::engine::combat::tests::fight::Fight;
use crate::engine::test_support::{data, hero, mob};

fn fight() -> Fight {
    Fight::new(hero("argalad"), mob("agent"), 5)
}

#[test]
fn call_all_mob_decks() {
    for deck in ["ravager", "zealot", "behemoth"] {
        let mut f = fight();
        f.mob.attack_deck = deck.to_owned();
        let first_hero_card = f.hero.hand[0];
        for card in data().attack_deck(deck).unwrap().deck.clone() {
            f.round(first_hero_card, card);
        }
    }
}

#[test]
fn call_all_hero_decks() {
    for code in ["argalad", "beravor", "eleanor", "eometh", "thalin"] {
        let mut f = fight();
        f.hero.code = code.to_owned();
        let first_mob_card = data().attack_deck("zealot").unwrap().deck[0];
        for card in data().hero(code).unwrap().starting_deck.clone() {
            f.round(card, first_mob_card);
        }
    }
}
