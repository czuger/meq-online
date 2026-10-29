//! Multi-step flows: hero turns, rounds and combat endings.

use crate::engine::board_state::BoardState;
use crate::engine::combat::flow::{CardPlayed, CombatEnd, CombatPreparation, CombatSide};
use crate::engine::test_support::{data, game, hero, mob, rng};
use crate::engine::turn::TurnEnd;
use crate::model::game::Game;

fn two_heroes() -> Game {
    let mut g = game();
    let mut second = hero("beravor");
    second.id = Some(2);
    second.playing_order = 1;
    second.location = "bree".to_owned();
    g.heroes.push(second);
    g
}

/// Plays one hero turn from the draw screen to the end of exploration.
fn play_turn(g: &mut Game, hero_id: i64) -> TurnEnd {
    g.draw_cards_finished(hero_id, data(), &mut rng()).unwrap();
    assert_eq!(g.current_hero_id().unwrap(), hero_id);
    if g.board.state == BoardState::HeroRestScreen {
        g.skip_rest().unwrap();
    }
    g.board.state = BoardState::HeroExploration;
    g.finish_heroes_turn(hero_id, data()).unwrap()
}

#[test]
fn single_hero_plays_twice_then_sauron() {
    let mut g = game();
    g.board.state = BoardState::HeroDrawCardsScreen;
    assert_eq!(play_turn(&mut g, 1), TurnEnd::HeroDrawCards);
    assert_eq!(g.board.state, BoardState::HeroDrawCardsScreen);
    assert_eq!(play_turn(&mut g, 1), TurnEnd::Boards);
    assert_eq!(g.board.state, BoardState::PlayScreenSauronPlotCards);
    assert!(g.sauron.active);
    assert_eq!(g.board.turn, 2);
    assert_eq!(g.board.story.heroes, 2);
}

#[test]
fn heroes_play_in_order_then_sauron() {
    let mut g = two_heroes();
    g.board.state = BoardState::EditSauronSauronActions;
    g.terminate_sauron_turn().unwrap();
    assert!(g.heroes.iter().all(|h| h.active));

    // Start of round: both heroes draw, then the first one plays.
    g.draw_cards_finished(2, data(), &mut rng()).unwrap();
    assert_eq!(play_turn(&mut g, 1), TurnEnd::Boards);
    assert_eq!(g.current_hero_id().unwrap(), 2);
    assert_eq!(g.board.state, BoardState::HeroDrawCardsScreen);

    assert_eq!(play_turn(&mut g, 2), TurnEnd::Boards);
    assert_eq!(g.board.state, BoardState::PlayScreenSauronPlotCards);
    assert!(g.sauron.active);
}

#[test]
fn story_stage_two_brings_minions() {
    let mut g = game();
    g.board.state = BoardState::HeroExploration;
    g.heroes[0].turn = 2;
    g.board.story.ring = 6;
    g.board.plots.insert(1, 0); // ring +1
    g.finish_heroes_turn(1, data()).unwrap();
    assert!(g.mobs.iter().any(|m| m.code == "gothmog"));
    assert!(g.mobs.iter().any(|m| m.code == "ringwraiths"));
}

fn fight(mob_code: &str) -> Game {
    let mut g = game();
    g.board.state = BoardState::HeroMovementScreen;
    let mut m = mob(mob_code);
    m.location = "the_shire".to_owned();
    g.mobs.push(m);
    let encounter = g.check_location_encounters(1, data(), &mut rng()).unwrap();
    assert_eq!(encounter, crate::engine::encounters::Encounter::Monster);
    g.board.state = BoardState::CombatSetupScreenBoardCombats;
    g.combat_setup(CombatPreparation::GainStrength, data(), &mut rng())
        .unwrap();
    g
}

#[test]
fn combat_round_resolves_when_both_sides_played() {
    let mut g = fight("orc");
    assert_eq!(g.combat.as_ref().unwrap().hero_strength, Some(8));
    let hero_card = g.heroes[0].hand[0];
    let mob_card = g.mobs[0].hand[0];
    assert_eq!(
        g.play_combat_card(CombatSide::Hero, hero_card, data())
            .unwrap(),
        CardPlayed::Waiting
    );
    let played = g
        .play_combat_card(CombatSide::Mob, mob_card, data())
        .unwrap();
    assert_ne!(played, CardPlayed::Waiting);
    assert_eq!(g.combat.as_ref().unwrap().hero_cards.len(), 1);
}

#[test]
fn defeated_monster_leaves_and_hero_moves_on() {
    let mut g = fight("orc");
    g.mobs[0].life = 0;
    g.combat
        .as_mut()
        .unwrap()
        .hero_cards
        .push(crate::model::played_card::PlayedCard::exhausted());
    assert_eq!(g.terminate_combat(data()).unwrap(), CombatEnd::Movement);
    assert!(g.mobs.is_empty());
    assert!(g.combat.is_none());
    assert_eq!(g.board.state, BoardState::HeroMovementScreen);
    assert!(
        g.heroes[0].rest_pool.is_empty(),
        "exhausted placeholders are not cards"
    );
}

#[test]
fn combat_cannot_end_early() {
    let mut g = fight("orc");
    assert!(g.terminate_combat(data()).is_err());
}
