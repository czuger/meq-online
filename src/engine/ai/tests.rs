//! Tests of the computer heroes.

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::engine::ai::waiting_hero::waiting_computer_hero;
use crate::engine::board_state::BoardState;
use crate::engine::choices::{PASS, plain_option};
use crate::engine::combat::flow::CombatSide;
use crate::engine::defeat::closest_havens;
use crate::engine::route::{ScreenSide, screen_side};
use crate::engine::test_support::{data, game, hero, mob, rng};
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// Rounds the soak test plays for each seed.
const ROUNDS: usize = 3;

fn two_heroes() -> Game {
    let mut g = game();
    let mut second = hero("beravor");
    second.id = Some(2);
    second.playing_order = 1;
    second.location = "bree".to_owned();
    g.heroes.push(second);
    g
}

/// Two heroes with monsters around them and on Bree, for ambushes and travel combats.
fn guarded_heroes() -> Game {
    let mut g = two_heroes();
    let places = [
        "bree",
        "old_forest",
        "hills_of_evendim",
        "tharbad",
        "weathertop",
        "the_grey_havens",
    ];
    for (id, location) in (1..).zip(places) {
        let mut orc = mob(if id % 2 == 0 { "cave_troll" } else { "snaga" });
        orc.id = Some(id);
        orc.location = location.to_owned();
        g.mobs.push(orc);
    }
    g
}

/// Sauron ends his turn at once; the next round's Hero Draw step starts.
fn next_round(g: &mut Game) {
    g.board.state = BoardState::EditSauronSauronActions;
    g.board.sauron_actions_count = 2;
    g.terminate_sauron_turn().unwrap();
}

fn drawing(mut g: Game) -> Game {
    g.board.state = BoardState::HeroDrawCardsScreen;
    g.set_heroes_active(true);
    g
}

fn heroes_choice(g: &mut Game) {
    g.board.state = BoardState::ChoiceScreenBoardChoices;
    g.board.pending_choice = Some(PendingChoice {
        chooser: Chooser::Heroes,
        source: ChoiceSource::Champion,
        options: vec![plain_option(PASS)],
        resume: Resume::EndTurn,
    });
}

#[test]
fn waits_on_the_current_hero_when_computer() {
    let mut g = game();
    g.board.state = BoardState::HeroMovementScreen;
    assert_eq!(waiting_computer_hero(&g, &[1]), Some(1));
    assert_eq!(waiting_computer_hero(&g, &[2]), None);
    g.board.state = BoardState::EditSauronSauronActions;
    assert_eq!(waiting_computer_hero(&g, &[1]), None);
}

#[test]
fn draw_waits_on_active_computer_heroes() {
    let mut g = drawing(two_heroes());
    assert_eq!(waiting_computer_hero(&g, &[2]), Some(2));
    g.heroes[1].active = false;
    assert_eq!(waiting_computer_hero(&g, &[2]), None);
}

#[test]
fn heroes_choice_is_left_to_a_human_hero() {
    let mut g = two_heroes();
    heroes_choice(&mut g);
    assert_eq!(waiting_computer_hero(&g, &[2]), None);
    assert_eq!(waiting_computer_hero(&g, &[1, 2]), Some(1));
}

#[test]
fn a_human_hero_is_not_played() {
    let mut g = game();
    g.board.state = BoardState::HeroMovementScreen;
    let before = g.clone();
    assert_eq!(g.play_computer_heroes(&[], data(), &mut rng()).unwrap(), 0);
    assert_eq!(g, before);
}

#[test]
fn a_computer_hero_plays_its_turn() {
    let mut g = drawing(game());
    let moves = g.play_computer_heroes(&[1], data(), &mut rng()).unwrap();
    assert!(moves > 0);
    assert_eq!(waiting_computer_hero(&g, &[1]), None, "{}", g.board.state);
    assert_eq!(g.board.state, BoardState::PlayScreenSauronPlotCards);
    assert_eq!(g.board.turn, 2, "a solo hero plays twice, then Sauron");
}

/// Computer heroes play whole rounds against a Sauron passing, playing his first
/// combat card and sending defeated heroes to the first Haven; every stop of the AI
/// must be a decision of Sauron.
#[test]
fn computer_heroes_play_rounds_for_many_seeds() {
    let mut combats = 0;
    for seed in 0..100 {
        let mut g = drawing(guarded_heroes());
        let mut rng = StdRng::seed_from_u64(seed);
        let mut rounds = 0;
        for _ in 0..500 {
            g.play_computer_heroes(&[1, 2], data(), &mut rng).unwrap();
            assert_eq!(waiting_computer_hero(&g, &[1, 2]), None, "seed {seed}");
            if g.board.state == BoardState::Finished {
                break;
            }
            if screen_side(g.board.state) == ScreenSide::Sauron
                && g.board.state != BoardState::SauronHavenScreen
            {
                rounds += 1;
                if rounds == ROUNDS {
                    break;
                }
                next_round(&mut g);
                continue;
            }
            combats += usize::from(g.combat.is_some());
            sauron_decides(&mut g, &mut rng, seed);
        }
        assert!(
            rounds == ROUNDS || g.board.state == BoardState::Finished,
            "seed {seed}: stuck in {}",
            g.board.state
        );
    }
    assert!(combats > 0, "no combat was fought");
}

fn sauron_decides(g: &mut Game, rng: &mut StdRng, seed: u64) {
    match g.board.state {
        BoardState::ChoiceScreenBoardChoices => {
            let choice = g.board.pending_choice.clone().unwrap();
            let option = choice
                .options
                .iter()
                .find(|o| o.code == PASS)
                .unwrap_or(&choice.options[0]);
            let targets: Vec<String> = option.targets.iter().take(option.min).cloned().collect();
            g.answer_choice(&option.code, &targets, data(), rng)
                .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        }
        BoardState::PlayCombatCardScreenBoardCombats => {
            let combat = g.combat.as_ref().unwrap();
            let mob = g.mobs.iter().find(|m| m.id == Some(combat.mob_id)).unwrap();
            match mob.hand.first().copied() {
                Some(card) => g.play_combat_card(CombatSide::Mob, card, data()),
                None => g.declare_exhaustion(CombatSide::Mob, data()),
            }
            .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        }
        BoardState::SauronHavenScreen => {
            let hero = g.hero(g.current_hero_id().unwrap()).unwrap();
            let haven = closest_havens(&hero.location, data())[0];
            g.choose_defeat_haven(haven, data(), rng)
                .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        }
        other => panic!("seed {seed}: nothing to do in {other}"),
    }
}
