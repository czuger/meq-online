//! Multi-step flows: hero turns, rounds and combat endings.

use crate::engine::board_state::BoardState;
use crate::engine::combat::flow::{CardPlayed, CombatEnd, CombatSide};
use crate::engine::test_support::{data, game, hero, mob, rng};
use crate::engine::turn::TurnEnd;
use crate::game_data::monster_pool_color::MonsterPoolColor;
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
    g.board.sauron_actions_count = 2;
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
    fight_at(mob_code, false)
}

fn fight_at(mob_code: &str, ambush: bool) -> Game {
    let mut g = game();
    g.board.state = BoardState::HeroMovementScreen;
    let mut m = mob(mob_code);
    m.location = "the_shire".to_owned();
    g.mobs.push(m);
    let encounter = g
        .check_location_encounters(1, ambush, data(), &mut rng())
        .unwrap();
    assert_eq!(encounter, crate::engine::encounters::Encounter::Monster);
    g.board.state = BoardState::CombatSetupScreenBoardCombats;
    g.combat_setup(0, data(), &mut rng()).unwrap();
    g
}

// § 8.3 Preparation: spent agility draws cards, the rest adds strength.
#[test]
fn preparation_splits_agility() {
    let mut g = game();
    g.board.state = BoardState::HeroMovementScreen;
    let mut m = mob("orc");
    m.location = "the_shire".to_owned();
    g.mobs.push(m);
    g.check_location_encounters(1, false, data(), &mut rng())
        .unwrap();
    g.board.state = BoardState::CombatSetupScreenBoardCombats;
    let hand = g.heroes[0].hand.len();
    assert!(g.combat_setup(4, data(), &mut rng()).is_err());
    g.combat_setup(2, data(), &mut rng()).unwrap();
    assert_eq!(g.heroes[0].hand.len(), hand + 2);
    assert_eq!(g.combat.as_ref().unwrap().hero_strength, Some(6));
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

// § 8.7: a combatant may declare itself exhausted before a round, the hero first.
#[test]
fn voluntary_exhaustion_hero_first() {
    let mut g = fight("orc");
    assert!(g.declare_exhaustion(CombatSide::Mob, data()).is_err());
    assert_eq!(
        g.declare_exhaustion(CombatSide::Hero, data()).unwrap(),
        CardPlayed::Waiting
    );
    assert!(g.combat.as_ref().unwrap().hero_exhausted);
    assert!(g.declare_exhaustion(CombatSide::Hero, data()).is_err());
    let mob_card = g.mobs[0].hand[0];
    g.play_combat_card(CombatSide::Mob, mob_card, data())
        .unwrap();
    assert_eq!(g.combat.as_ref().unwrap().mob_cards.len(), 1);
    assert!(
        g.combat.as_ref().unwrap().hero_cards[0].card.is_none(),
        "an exhausted hero reveals nothing"
    );
}

#[test]
fn both_sides_declaring_ends_the_combat() {
    let mut g = fight("orc");
    let hero_card = g.heroes[0].hand[0];
    g.play_combat_card(CombatSide::Hero, hero_card, data())
        .unwrap();
    g.declare_exhaustion(CombatSide::Mob, data()).unwrap();
    assert_eq!(g.combat.as_ref().unwrap().hero_cards.len(), 1);
    g.declare_exhaustion(CombatSide::Hero, data()).unwrap();
    g.declare_exhaustion(CombatSide::Mob, data()).unwrap_err();
    let result = g.refresh_combat_result().unwrap();
    assert!(result.hero_exhausted && result.mob_exhausted);
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
    assert_eq!(
        g.terminate_combat(data(), &mut rng()).unwrap(),
        CombatEnd::Movement
    );
    assert!(g.mobs.is_empty());
    assert_eq!(
        g.board.monster_discards[&MonsterPoolColor::Orange],
        vec!["orc"],
        "§ 8.3: the monster token is discarded"
    );
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
    assert!(g.terminate_combat(data(), &mut rng()).is_err());
}

fn both_exhausted(g: &mut Game) {
    let combat = g.combat.as_mut().unwrap();
    combat.hero_exhausted = true;
    combat.mob_exhausted = true;
}

// § 6.5: a monster met while travelling that isn't defeated ends the hero's turn, and
// § 8.3: Sauron may place influence equal to its wisdom.
#[test]
fn surviving_mob_ends_the_turn_when_travelling() {
    let mut g = fight("orc");
    g.heroes[0].turn = 2;
    both_exhausted(&mut g);
    let end = g.terminate_combat(data(), &mut rng()).unwrap();
    assert!(matches!(end, CombatEnd::TurnEnded(_)));
    let wisdom = data().mob("orc").unwrap().wisdom;
    assert_eq!(g.board.influence_budget.tokens, wisdom);
    assert!(g.mobs.is_empty(), "the token is discarded anyway");
}

// § 6.4: after an ambush the turn goes on unless the hero is defeated.
#[test]
fn surviving_mob_doesnt_end_the_turn_after_an_ambush() {
    let mut g = fight_at("orc", true);
    both_exhausted(&mut g);
    assert_eq!(
        g.terminate_combat(data(), &mut rng()).unwrap(),
        CombatEnd::Movement
    );
    assert_eq!(g.board.state, BoardState::HeroMovementScreen);
}

// § 8.8: defeated Ringwraiths leave the board and come back on Minas Morgul, at full
// life, at the start of Sauron's next Action step.
#[test]
fn ringwraiths_come_back_at_the_next_action_step() {
    let mut g = game();
    g.board.state = BoardState::HeroMovementScreen;
    g.create_mob("ringwraiths", "the_shire", None, data())
        .unwrap();
    g.mobs[0].id = Some(1);
    g.check_location_encounters(1, false, data(), &mut rng())
        .unwrap();
    g.board.state = BoardState::PlayCombatCardScreenBoardCombats;
    g.combat.as_mut().unwrap().hero_strength = Some(5);
    g.mobs[0].life = 0;
    g.heroes[0].turn = 2;
    assert_eq!(
        g.terminate_combat(data(), &mut rng()).unwrap(),
        CombatEnd::Movement
    );
    assert!(g.mobs.is_empty());
    assert!(g.board.ringwraiths_return);

    g.board.state = BoardState::PlayScreenSauronPlotCards;
    g.plot_play_finished(data(), &mut rng()).unwrap();
    let wraiths = g.mobs.iter().find(|m| m.code == "ringwraiths").unwrap();
    assert_eq!(wraiths.location, "minas_morgul");
    assert_eq!(wraiths.life, wraiths.max_life);
    assert!(!g.board.ringwraiths_return);
}

// § 8.9: a defeated hero goes through the defeat steps and its turn ends.
#[test]
fn defeated_hero_recovers_in_a_haven() {
    let mut g = fight("orc");
    g.heroes[0].turn = 2;
    g.heroes[0].favor = 1;
    g.board.story.ring = 1;
    g.board.story.conquest = 1;
    g.heroes[0].hand.clear();
    g.heroes[0].damage_pool = std::mem::take(&mut g.heroes[0].life_pool);
    let end = g.terminate_combat(data(), &mut rng()).unwrap();
    assert!(matches!(end, CombatEnd::TurnEnded(_)), "{end:?}");
    assert_eq!(g.board.story.corruption, 1);
    assert_eq!(g.heroes[0].favor, 0);
    assert_eq!(g.heroes[0].location, "fornost");
    assert!(!g.heroes[0].life_pool.is_empty());
}

// § 10.1: in a two-player game, a special Hero Rally step comes between the hero's turns.
#[test]
fn two_player_game_rallies_between_hero_turns() {
    let mut g = game();
    g.board.state = BoardState::HeroExploration;
    let location = g.heroes[0].location.clone();
    g.board.influence.insert(location.clone(), 1);
    g.heroes[0].cards_drawn_this_round = 5;
    assert_eq!(
        g.finish_heroes_turn(1, data()).unwrap(),
        TurnEnd::HeroDrawCards
    );
    assert!(!g.board.influence.contains_key(&location));
    assert_eq!(g.heroes[0].cards_drawn_this_round, 0);
}

// § 5.2: a hero on a Shadow Stronghold doesn't remove its influence.
#[test]
fn no_rally_on_a_shadow_stronghold() {
    let mut g = game();
    g.board.state = BoardState::HeroExploration;
    g.heroes[0].location = "dol_guldur".to_owned();
    g.board.influence.insert("dol_guldur".to_owned(), 2);
    g.finish_heroes_turn(1, data()).unwrap();
    assert_eq!(g.board.influence.get("dol_guldur"), Some(&2));
}
