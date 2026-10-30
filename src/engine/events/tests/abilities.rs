//! Abilities of the event cards resolved during the Event step.

use crate::engine::board_state::BoardState;
use crate::engine::choices::{PASS, parse_target};
use crate::engine::engine_error::EngineError;
use crate::engine::events::abilities::area_locations;
use crate::engine::test_support::{data, game, hero, mob, rng};
use crate::game_data::event::Area;
use crate::model::board::EventRef;
use crate::model::choice::Chooser;
use crate::model::game::Game;

/// A game with two heroes, Beravor (id 1) then Eleanor (id 2), at the Plot step.
fn two_heroes() -> Game {
    let mut g = game();
    let mut eleanor = hero("eleanor");
    eleanor.id = Some(2);
    eleanor.playing_order = 1;
    g.heroes.push(eleanor);
    g.board.state = BoardState::PlayScreenSauronPlotCards;
    g
}

fn resolve(g: &mut Game, stage: u8, card: u32) {
    g.resolve_event(EventRef { stage, card }, data()).unwrap();
}

fn answer(g: &mut Game, option: &str, targets: &[&str]) -> Result<(), EngineError> {
    let targets: Vec<String> = targets.iter().map(|t| (*t).to_owned()).collect();
    g.answer_choice(option, &targets, data(), &mut rng())
}

fn chooser(g: &Game) -> Chooser {
    g.board.pending_choice.as_ref().unwrap().chooser
}

fn region_location(region: &str) -> String {
    area_locations(&Area::Region(region.to_owned()), data())
        .into_iter()
        .next()
        .unwrap()
}

// "Messengers of the Stormcrow": each hero with favor, in order, spends 1 favor or
// receives a Corruption card; then Sauron's Action step.
#[test]
fn favor_or_corruption_asks_each_hero_with_favor() {
    let mut g = two_heroes();
    g.heroes[0].favor = 2;
    g.heroes[1].favor = 1;
    g.board.decks.corruption.draw = vec![2];
    resolve(&mut g, 1, 0);
    assert_eq!(chooser(&g), Chooser::Hero(1));
    answer(&mut g, "spend_favor", &[]).unwrap();
    assert_eq!(g.heroes[0].favor, 1);
    assert_eq!(chooser(&g), Chooser::Hero(2));
    answer(&mut g, "corruption", &[]).unwrap();
    assert_eq!(g.heroes[1].corruption_cards, vec![2]);
    assert!(g.board.pending_choice.is_none());
    assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
    assert!(g.sauron.active);
}

// Heroes without favor are not asked.
#[test]
fn favor_or_corruption_skips_heroes_without_favor() {
    let mut g = two_heroes();
    resolve(&mut g, 1, 0);
    assert!(g.board.pending_choice.is_none());
}

// "Return to the Kingdom" places its characters, Éowyn places Théoden and clears
// 1 influence around Edoras: no choice.
#[test]
fn placing_abilities_need_no_choice() {
    let mut g = two_heroes();
    resolve(&mut g, 2, 10);
    assert_eq!(g.board.characters["aragorn"], "bree");
    assert_eq!(g.board.characters["gandalf"], "bree");

    let near = data().paths.connected("edoras")[0].to_owned();
    g.board.influence.insert(near.clone(), 2);
    resolve(&mut g, 3, 2);
    assert_eq!(g.board.characters["theoden"], "edoras");
    assert!(g.board.influence.get(&near).copied().unwrap_or(0) < 2);
    assert!(g.board.pending_choice.is_none());
}

// Radagast: heroes with 4 favor or more lose 1; Gandalf, if on the map, goes to Isengard.
#[test]
fn radagast_takes_favor_and_moves_gandalf() {
    let mut g = two_heroes();
    g.heroes[0].favor = 4;
    g.heroes[1].favor = 3;
    g.board
        .characters
        .insert("gandalf".to_owned(), "bree".to_owned());
    resolve(&mut g, 3, 0);
    assert_eq!((g.heroes[0].favor, g.heroes[1].favor), (3, 3));
    assert_eq!(g.board.characters["gandalf"], "isengard");
}

// Boromir's Dream: heroes without favor gain 1; the Ring marker advances when lowest.
#[test]
fn boromirs_dream_gives_favor_and_advances_the_ring() {
    let mut g = two_heroes();
    g.heroes[1].favor = 2;
    resolve(&mut g, 3, 1);
    assert_eq!((g.heroes[0].favor, g.heroes[1].favor), (1, 2));
    assert_eq!(g.board.story.ring, 1);
}

// Clear tokens or place a favor: the heroes choose, one token of each kind at most
// when counted per kind.
#[test]
fn clear_tokens_or_favor() {
    let mut g = two_heroes();
    resolve(&mut g, 1, 8);
    assert_eq!(chooser(&g), Chooser::Heroes);
    answer(&mut g, "place_favor", &[]).unwrap();
    assert!(g.board.favors.contains(&"the_shire".to_owned()));

    let mut g = two_heroes();
    g.board.influence.insert("edoras".to_owned(), 2);
    resolve(&mut g, 2, 6);
    let refused = answer(
        &mut g,
        "remove_tokens",
        &["influence:edoras:1", "influence:edoras:2"],
    );
    assert!(refused.is_err());
}

// Sauron discards a character or a favor of the region; nothing to ask without any.
#[test]
fn sauron_discards_a_favor() {
    let mut g = two_heroes();
    resolve(&mut g, 1, 11);
    assert!(g.board.pending_choice.is_none());

    let location = region_location("rhoan");
    g.board.favors.push(location.clone());
    resolve(&mut g, 1, 11);
    assert_eq!(chooser(&g), Chooser::Sauron);
    // The card places its own favors too.
    let count = |g: &Game| g.board.favors.iter().filter(|f| **f == location).count();
    let before = count(&g);
    answer(&mut g, "discard_favor", &[&format!("favor:{location}:1")]).unwrap();
    assert_eq!(count(&g), before - 1);
}

// Sauron must move a character to the Mouth of the Greyflood.
#[test]
fn sauron_moves_a_character() {
    let mut g = two_heroes();
    resolve(&mut g, 1, 2);
    let options = &g.board.pending_choice.as_ref().unwrap().options;
    assert!(options.iter().all(|o| o.code != PASS), "Sauron must choose");
    answer(&mut g, "place", &["character:boromir"]).unwrap();
    assert_eq!(g.board.characters["boromir"], "mouth_of_the_greyflood");
}

// Temptation: Sauron picks a hero, who accepts or lets Sauron draw a card.
#[test]
fn temptation_refused_gives_sauron_a_card() {
    let mut g = two_heroes();
    resolve(&mut g, 1, 6);
    assert_eq!(chooser(&g), Chooser::Sauron);
    answer(&mut g, "choose_hero", &["hero:2"]).unwrap();
    assert_eq!(chooser(&g), Chooser::Hero(2));
    let hand = g.sauron.plot_hand.len();
    answer(&mut g, "refuse", &[]).unwrap();
    assert_eq!(g.sauron.plot_hand.len(), hand + 1);
    assert!(g.board.pending_choice.is_none());
}

// Trust in Friendship: each hero with favor gives 1 to the card, which Sauron gives
// to a hero.
#[test]
fn trust_in_friendship_is_given_to_a_hero() {
    let mut g = two_heroes();
    g.heroes[0].favor = 2;
    resolve(&mut g, 1, 3);
    assert_eq!(g.heroes[0].favor, 1);
    answer(&mut g, "give", &["hero:2"]).unwrap();
    let held = g.board.held_event.as_ref().unwrap();
    assert_eq!((held.hero_id, held.favor), (2, 1));
}

// The heroes may hurt a minion of the region.
#[test]
fn heroes_hurt_a_minion() {
    let mut g = two_heroes();
    let mut minion = mob("mouth_of_sauron");
    minion.location = region_location("mirkwood");
    g.mobs.push(minion);
    resolve(&mut g, 1, 1);
    answer(&mut g, "hurt_minion", &["minion:1"]).unwrap();
    assert_eq!(g.mobs[0].life, 9 - 2);
}

// A character moves to an adjacent location and clears its influence.
#[test]
fn character_moves_and_clears() {
    let mut g = two_heroes();
    g.board
        .characters
        .insert("aragorn".to_owned(), "bree".to_owned());
    resolve(&mut g, 2, 4);
    let target = g.board.pending_choice.as_ref().unwrap().options[0].targets[0].clone();
    let to = parse_target(&target).1[1].to_owned();
    g.board.influence.insert(to.clone(), 1);
    answer(&mut g, "move_character", &[&target]).unwrap();
    assert_eq!(g.board.characters["aragorn"], to);
    assert_eq!(g.board.influence.get(&to).copied().unwrap_or(0), 0);
}

// He Has at Last Heard: Sauron may move a minion to the Shire.
#[test]
fn minion_goes_to_the_shire() {
    let mut g = two_heroes();
    g.mobs.push(mob("mouth_of_sauron"));
    resolve(&mut g, 3, 13);
    answer(&mut g, "move_minion", &["minion:1"]).unwrap();
    assert_eq!(g.mobs[0].location, "the_shire");
}

// Warrior of the Lidless Eye does nothing while Gothmog is not on the map.
#[test]
fn warrior_needs_gothmog() {
    let mut g = two_heroes();
    resolve(&mut g, 3, 3);
    assert!(g.board.pending_choice.is_none());
}
