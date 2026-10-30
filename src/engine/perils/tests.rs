//! Peril cards (§ 6.5).

use crate::engine::board_state::BoardState;
use crate::engine::choices::{PASS, target};
use crate::engine::combat::flow::CombatEnd;
use crate::engine::hero::damage::hand_target;
use crate::engine::perils::effects::option;
use crate::engine::perils::peril_deck::PICK_PERIL;
use crate::engine::test_support::{data, game, mob, rng};
use crate::engine::threats::Threat;
use crate::model::choice::{ChoiceSource, Chooser};
use crate::model::game::Game;

const HOUNDS: u32 = 0;
const ILL_MET: u32 = 4;
const HILL_OF_BLACK_MAGIC: u32 = 6;
const DARK_TOWER: u32 = 7;
const KHAZAD_DUM: u32 = 8;
const ENDURING_EVIL: u32 = 9;
const WEAKNESS_OF_MEN: u32 = 11;
const UNDYING_REGRET: u32 = 13;
/// Wretched corruption card.
const WRETCHED: u32 = 4;

/// The hero stands on `location`, Sauron's Peril deck starts with `deck`.
fn peril_game(location: &str, deck: &[u32]) -> Game {
    let mut g = game();
    g.board.state = BoardState::HeroMovementScreen;
    g.heroes[0].location = location.to_owned();
    g.board.decks.peril.draw = deck.to_vec();
    g
}

/// Sauron draws and picks `card`.
fn suffer(g: &mut Game, card: u32) {
    assert_eq!(
        g.draw_perils(1, data(), &mut rng()).unwrap(),
        Threat::Choice
    );
    g.answer_choice(PICK_PERIL, &[target("peril", card)], data(), &mut rng())
        .unwrap();
}

fn answer(g: &mut Game, option: &str, targets: &[String]) {
    g.answer_choice(option, targets, data(), &mut rng())
        .unwrap();
}

fn chooser(g: &Game) -> Chooser {
    g.board.pending_choice.as_ref().unwrap().chooser
}

// § 6.5: Sauron may only pick a card affecting the hero's location; when none does,
// all three are discarded without effect.
#[test]
fn cards_must_affect_the_hero_location() {
    let mut g = peril_game("bree", &[KHAZAD_DUM, DARK_TOWER, UNDYING_REGRET, HOUNDS]);
    assert_eq!(g.draw_perils(1, data(), &mut rng()).unwrap(), Threat::None);
    assert_eq!(g.board.decks.peril.discard.len(), 3);

    let mut g = peril_game("bree", &[KHAZAD_DUM, HOUNDS, DARK_TOWER]);
    g.draw_perils(1, data(), &mut rng()).unwrap();
    let options = &g.board.pending_choice.as_ref().unwrap().options;
    assert_eq!(options[0].targets, [target("peril", HOUNDS)]);
    assert!(
        g.answer_choice(
            PICK_PERIL,
            &[target("peril", KHAZAD_DUM)],
            data(),
            &mut rng()
        )
        .is_err()
    );
}

// Reckless: Sauron draws one more Peril card.
#[test]
fn reckless_hero_faces_four_cards() {
    let mut g = peril_game("bree", &[KHAZAD_DUM, DARK_TOWER, UNDYING_REGRET, HOUNDS]);
    g.heroes[0].corruption_cards = vec![13];
    g.draw_perils(1, data(), &mut rng()).unwrap();
    let ChoiceSource::PerilDraw { cards, .. } = &g.board.pending_choice.as_ref().unwrap().source
    else {
        panic!("Sauron should pick a peril");
    };
    assert_eq!(cards.len(), 4);
}

// "The Weakness of Men": the hero chooses damage or the black marker advance.
#[test]
fn weakness_of_men_lets_the_hero_choose() {
    let mut g = peril_game("edoras", &[WEAKNESS_OF_MEN]);
    suffer(&mut g, WEAKNESS_OF_MEN);
    assert_eq!(chooser(&g), Chooser::Hero(1));
    answer(&mut g, option::ADVANCE_CORRUPTION, &[]);
    assert_eq!(g.board.story.corruption, 1);
    assert_eq!(g.board.state, BoardState::HeroExploration);
}

// "The Hounds of Sauron": shields discarded reduce the 6 damage; at 0 the hero gains
// 1 favor or training.
#[test]
fn hounds_of_sauron_damage_is_reduced_by_shields() {
    let mut g = peril_game("bree", &[HOUNDS]);
    // Argalad card 6 (Parry) has 1 defense, card 5 (Volley) has 2.
    g.heroes[0].hand = vec![5, 5, 5, 6];
    suffer(&mut g, HOUNDS);
    let all: Vec<String> = (0..3).map(|i| hand_target(1, i)).collect();
    answer(&mut g, option::DISCARD_SHIELDS, &all);
    assert_eq!(g.heroes[0].hand, vec![6]);
    answer(&mut g, option::FAVOR, &[]);
    assert_eq!(g.heroes[0].favor, 1);
    assert_eq!(g.board.state, BoardState::HeroExploration);

    let mut g = peril_game("bree", &[HOUNDS]);
    g.heroes[0].hand = vec![6, 6];
    suffer(&mut g, HOUNDS);
    answer(&mut g, option::DISCARD_SHIELDS, &[hand_target(1, 0)]);
    let ChoiceSource::Damage { amount, .. } = g.board.pending_choice.as_ref().unwrap().source
    else {
        panic!("the hero should pay the damage");
    };
    assert_eq!(amount, 5);
}

// "The Hill of Black Magic": wisdom ≤ plots in play: 5 damage and the turn ends.
#[test]
fn hill_of_black_magic_ends_the_turn() {
    let mut g = peril_game("dol_guldur", &[HILL_OF_BLACK_MAGIC]);
    g.heroes[0].wisdom = 1;
    g.board.plots.insert(1, 3);
    g.heroes[0].hand.clear();
    let life = g.heroes[0].life_pool.len();
    suffer(&mut g, HILL_OF_BLACK_MAGIC);
    assert_eq!(g.heroes[0].life_pool.len(), life - 5);
    assert!(g.heroes[0].turn_finished || g.heroes[0].turn == 2);
    assert_ne!(g.board.state, BoardState::HeroExploration);
}

// "Ill-met Company": Sauron picks one of two Corruption cards for the hero.
#[test]
fn ill_met_company_lets_sauron_pick_the_corruption() {
    let mut g = peril_game("bree", &[ILL_MET]);
    g.board.decks.corruption.draw = vec![2, 14];
    suffer(&mut g, ILL_MET);
    let options = &g.board.pending_choice.as_ref().unwrap().options;
    assert_eq!(options.len(), 1, "no favor to lose");
    answer(&mut g, option::CORRUPTION, &[]);
    assert_eq!(chooser(&g), Chooser::Sauron);
    answer(&mut g, option::GIVE_CORRUPTION, &[target("corruption", 14)]);
    assert_eq!(g.heroes[0].corruption_cards, vec![14]);
    assert_eq!(g.board.decks.corruption.discard, vec![2]);
}

// "Khazad-dûm": the hero fights a Balrog; defeating it gives 2 favor and training.
#[test]
fn khazad_dum_rewards_defeating_the_balrog() {
    let mut g = peril_game("moria", &[KHAZAD_DUM]);
    suffer(&mut g, KHAZAD_DUM);
    let balrog = g.mobs.iter_mut().find(|m| m.code == "balrog").unwrap();
    balrog.id = Some(7);
    answer(&mut g, option::FIGHT, &[]);
    assert_eq!(g.board.state, BoardState::CombatSetupScreenBoardCombats);
    g.board.state = BoardState::PlayCombatCardScreenBoardCombats;
    g.mobs[0].life = 0;
    let end = g.terminate_combat(data(), &mut rng()).unwrap();
    assert_eq!(end, CombatEnd::Choice);
    assert_eq!(g.heroes[0].favor, 2);
    assert!(g.mobs.is_empty(), "the Balrog was no monster token");
    assert!(g.board.monster_discards.is_empty());
}

// "The Dark Tower": the minion fights; the turn goes on even when it survives.
#[test]
fn dark_tower_combat_lets_the_turn_go_on() {
    let mut g = peril_game("barad_dur", &[DARK_TOWER]);
    let mut minion = mob("mouth_of_sauron");
    minion.id = Some(3);
    minion.location = "barad_dur".to_owned();
    g.mobs.push(minion);
    suffer(&mut g, DARK_TOWER);
    assert_eq!(g.combat.as_ref().unwrap().mob_id, 3);
    g.board.state = BoardState::PlayCombatCardScreenBoardCombats;
    let combat = g.combat.as_mut().unwrap();
    combat.hero_exhausted = true;
    combat.mob_exhausted = true;
    let end = g.terminate_combat(data(), &mut rng()).unwrap();
    assert_eq!(end, CombatEnd::Exploration);
}

// "Undying Regret": Sauron places 1 influence, then 1 monster token, in the region.
#[test]
fn undying_regret_places_influence_then_a_monster() {
    let mut g = peril_game("gladden_fields", &[UNDYING_REGRET]);
    let pool = g.board.shadow_pool;
    suffer(&mut g, UNDYING_REGRET);
    assert_eq!(g.board.shadow_pool, pool + 2);
    let spot = g.board.pending_choice.as_ref().unwrap().options[0].targets[0].clone();
    answer(&mut g, option::PLACE_INFLUENCE, std::slice::from_ref(&spot));
    let location = spot.strip_prefix("location:").unwrap();
    assert_eq!(g.board.influence.get(location), Some(&1));
    assert_eq!(chooser(&g), Chooser::Sauron);
    answer(&mut g, option::PLACE_MONSTER, &[]);
    assert_eq!(g.board.state, BoardState::HeroExploration);
}

// "Wretched": the hero's turn ends; Sauron may send him to the nearest Haven.
#[test]
fn wretched_corruption_ends_the_turn() {
    let mut g = peril_game("the_ruins_of_angmar", &[ENDURING_EVIL]);
    g.board
        .influence
        .insert("the_ruins_of_angmar".to_owned(), 1);
    g.board.decks.corruption.draw = vec![WRETCHED];
    suffer(&mut g, ENDURING_EVIL);
    assert!(matches!(
        g.board.pending_choice.as_ref().unwrap().source,
        ChoiceSource::Wretched { .. }
    ));
    assert!(g.heroes[0].corruption_cards.is_empty());
    answer(&mut g, PASS, &[]);
    assert_ne!(g.board.state, BoardState::HeroExploration);
    assert_eq!(g.board.decks.corruption.draw, vec![WRETCHED]);
}
