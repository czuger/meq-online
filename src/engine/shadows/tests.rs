//! Shadow cards (§ 5.9).

use crate::engine::board_state::BoardState;
use crate::engine::choices::{PASS, target};
use crate::engine::combat::flow::CombatEnd;
use crate::engine::defeat::DefeatChoice;
use crate::engine::movement::MoveOutcome;
use crate::engine::shadows::effects::option;
use crate::engine::shadows::shadow_window::{PLAY_SHADOW, ShadowWindow};
use crate::engine::test_support::{data, game, hero, mob, rng};
use crate::engine::threats::Threat;
use crate::game_data::shadow::ShadowTiming;
use crate::model::choice::{ChoiceSource, Chooser, Resume};
use crate::model::combat::CombatOrigin;
use crate::model::game::Game;

const FLOCKS_OF_CREBAIN: u32 = 0;
const ARM_GROWN_LONG: u32 = 1;
const SARUMANS_LIES: u32 = 2;
const EVIL_FOG: u32 = 3;
const LIDLESS_EYE: u32 = 4;
const BLACK_BREATH: u32 = 5;
const THEY_ARE_TERRIBLE: u32 = 6;
const ITHIL_STONE: u32 = 8;
const DARK_PROMISES: u32 = 9;
const STORMS_OF_MORDOR: u32 = 10;
const NEW_POWER: u32 = 11;
const LORD_OF_THE_RINGS: u32 = 13;
const SHIRE_BAGGINS: u32 = 14;
const DO_NOT_TEMPT_ME: u32 = 15;
const GRIEVOUS_LOSS: u32 = 17;
const EVIL_WOUND: u32 = 18;
const WINTER_STORM: u32 = 19;
const BETRAYED: u32 = 22;
/// Corruption cards without side effects here (Indifferent, Reckless).
const PLAIN_CORRUPTION: [u32; 2] = [15, 13];
/// Corruption card "Weak" (-1 strength).
const WEAK: u32 = 14;
/// Corruption card "Greedy".
const GREEDY: u32 = 6;

/// Sauron holds `card` with a full Shadow Pool; the hero stands on Bree.
fn shadow_game(card: u32, state: BoardState) -> Game {
    let mut g = game();
    g.board.state = state;
    g.board.shadow_pool = 12;
    g.board.current_hero_id = Some(1);
    g.sauron.shadow_hand = vec![card];
    g.heroes[0].location = "bree".to_owned();
    g.board.decks.corruption.draw = vec![15, 13, 12];
    g
}

fn answer(g: &mut Game, option: &str, targets: &[String]) {
    g.answer_choice(option, targets, data(), &mut rng())
        .unwrap();
}

fn play(g: &mut Game, card: u32) {
    answer(g, PLAY_SHADOW, &[target("shadow", card)]);
}

fn source(g: &Game) -> ChoiceSource {
    g.board.pending_choice.as_ref().unwrap().source.clone()
}

fn chooser(g: &Game) -> Chooser {
    g.board.pending_choice.as_ref().unwrap().chooser
}

fn window(timing: ShadowTiming) -> ShadowWindow {
    ShadowWindow { timing, hero_id: 1 }
}

/// An orc (id 7) on the hero's location.
fn orc_here(g: &mut Game) {
    let mut orc = mob("orc");
    orc.id = Some(7);
    orc.location = "bree".to_owned();
    g.mobs.push(orc);
}

/// The second hero of the game, Eleanor (id 2).
fn add_eleanor(g: &mut Game) {
    let mut eleanor = hero("eleanor");
    eleanor.id = Some(2);
    eleanor.playing_order = 1;
    g.heroes.push(eleanor);
}

/// The monster a Shadow card brought gets the id the database would give it.
fn save_new_mob(g: &mut Game) {
    g.mobs.last_mut().unwrap().id = Some(9);
}

// § 5.9: without the Shadow Pool requirement, or after a card this turn, no window.
#[test]
fn window_needs_requirement_and_one_card_per_turn() {
    let mut g = shadow_game(WINTER_STORM, BoardState::HeroDrawCardsScreen);
    g.board.shadow_pool = 9;
    let timing = ShadowTiming::HeroTurnStart;
    assert!(
        !g.open_shadow_window(window(timing), Resume::Ambush, data())
            .unwrap()
    );
    g.board.shadow_pool = 10;
    assert_eq!(
        g.playable_shadow_cards(timing, Some(1), data()).unwrap(),
        vec![WINTER_STORM]
    );
    g.board.turn_effects.shadow_played = true;
    assert!(
        g.playable_shadow_cards(timing, Some(1), data())
            .unwrap()
            .is_empty()
    );
    g.board.turn_effects.start_turn();
    assert!(!g.board.turn_effects.shadow_played);
    assert!(
        g.playable_shadow_cards(ShadowTiming::CombatStart, Some(1), data())
            .unwrap()
            .is_empty(),
        "the card's timing must match"
    );
}

// § 5.9: at the start of the hero's turn Sauron plays or passes; passing goes on with
// the Ambush step.
#[test]
fn sauron_passes_at_the_start_of_a_turn() {
    let mut g = shadow_game(FLOCKS_OF_CREBAIN, BoardState::HeroDrawCardsScreen);
    g.draw_cards_finished(1, data(), &mut rng()).unwrap();
    assert_eq!(chooser(&g), Chooser::Sauron);
    assert!(matches!(source(&g), ChoiceSource::ShadowWindow { .. }));
    answer(&mut g, PASS, &[]);
    assert_eq!(g.board.state, BoardState::HeroRestScreen);
    assert_eq!(g.sauron.shadow_hand, vec![FLOCKS_OF_CREBAIN]);
}

// "Flocks of Crebain": the hero fights a Crebain without agility and skips his Ambush
// step; after the combat he goes to his Rest step (here movement: the orc stays).
#[test]
fn flocks_of_crebain_replace_the_ambush() {
    let mut g = shadow_game(FLOCKS_OF_CREBAIN, BoardState::HeroDrawCardsScreen);
    orc_here(&mut g);
    g.draw_cards_finished(1, data(), &mut rng()).unwrap();
    play(&mut g, FLOCKS_OF_CREBAIN);
    assert_eq!(chooser(&g), Chooser::Hero(1));
    save_new_mob(&mut g);
    answer(&mut g, option::FIGHT, &[]);
    let combat = g.combat.as_ref().unwrap();
    assert_eq!(combat.mob_id, 9);
    assert_eq!(
        combat.origin,
        CombatOrigin::Shadow {
            card: FLOCKS_OF_CREBAIN
        }
    );
    assert!(
        g.combat_setup(1, data(), &mut rng()).is_err(),
        "agility is 0"
    );
    g.combat_setup(0, data(), &mut rng()).unwrap();
    g.mobs.last_mut().unwrap().life = 0;
    let end = g.terminate_combat(data(), &mut rng()).unwrap();
    assert_eq!(end, CombatEnd::Rest);
    assert_eq!(g.board.state, BoardState::HeroMovementScreen);
    assert!(
        g.mobs.iter().any(|m| m.id == Some(7)),
        "the orc never ambushed"
    );
    assert_eq!(g.board.decks.shadow.discard, vec![FLOCKS_OF_CREBAIN]);
}

// "Winter Storm": one move only during the Travel step.
#[test]
fn winter_storm_allows_a_single_move() {
    let mut g = shadow_game(WINTER_STORM, BoardState::HeroDrawCardsScreen);
    g.draw_cards_finished(1, data(), &mut rng()).unwrap();
    play(&mut g, WINTER_STORM);
    assert!(g.board.turn_effects.single_move);
    assert_eq!(g.board.state, BoardState::HeroRestScreen);
    g.heroes[0].turn_record.travels = 1;
    let moved = g
        .move_hero(1, "the_shire", &[], data(), &mut rng())
        .unwrap();
    assert_eq!(moved, MoveOutcome::SingleMove);
}

// "His Arm Has Grown Long": the location just entered is perilous this turn.
#[test]
fn his_arm_makes_the_location_perilous() {
    let mut g = shadow_game(ARM_GROWN_LONG, BoardState::HeroMovementScreen);
    assert!(!g.is_perilous(1, data()).unwrap());
    let timing = ShadowTiming::EnteredLocation;
    assert!(
        g.open_shadow_window(window(timing), Resume::TravelThreats, data())
            .unwrap()
    );
    play(&mut g, ARM_GROWN_LONG);
    assert!(g.is_perilous(1, data()).unwrap());
    assert!(g.new_logs.iter().any(|l| l.action == "peril.draw"));
    g.board.turn_effects.start_turn();
    assert!(!g.is_perilous(1, data()).unwrap());
}

// "Dark Promises": before the combat, the hero takes 5 damage or a Corruption card.
#[test]
fn dark_promises_before_the_combat() {
    let mut g = shadow_game(DARK_PROMISES, BoardState::HeroMovementScreen);
    orc_here(&mut g);
    let threat = g.attack_hero(1, 7, CombatOrigin::Travel, data(), &mut rng());
    assert_eq!(threat.unwrap(), Threat::Choice);
    play(&mut g, DARK_PROMISES);
    assert_eq!(chooser(&g), Chooser::Hero(1));
    answer(&mut g, option::DAMAGE, &[]);
    assert!(matches!(source(&g), ChoiceSource::Damage { amount: 5, .. }));
    answer(&mut g, "pay_damage", &[]);
    assert_eq!(g.heroes[0].damage_pool.len(), 5);
    assert_eq!(g.board.state, BoardState::CombatSetupScreenBoardCombats);
}

// "Betrayed": 2 random cards discarded, -1 strength per Corruption card.
#[test]
fn betrayed_weakens_the_hero() {
    let mut g = shadow_game(BETRAYED, BoardState::HeroMovementScreen);
    g.heroes[0].corruption_cards = PLAIN_CORRUPTION.to_vec();
    orc_here(&mut g);
    let hand = g.heroes[0].hand.len();
    g.attack_hero(1, 7, CombatOrigin::Travel, data(), &mut rng())
        .unwrap();
    play(&mut g, BETRAYED);
    assert_eq!(g.heroes[0].hand.len(), hand - 2);
    assert_eq!(g.board.state, BoardState::CombatSetupScreenBoardCombats);
    g.combat_setup(0, data(), &mut rng()).unwrap();
    let hero = &g.heroes[0];
    let expected = hero.strength + hero.agility - 2;
    assert_eq!(g.combat.as_ref().unwrap().hero_strength, Some(expected));
}

// "The Black Breath" is only played against a minion.
#[test]
fn black_breath_needs_a_minion() {
    let mut g = shadow_game(BLACK_BREATH, BoardState::HeroMovementScreen);
    orc_here(&mut g);
    g.attack_hero(1, 7, CombatOrigin::Travel, data(), &mut rng())
        .unwrap();
    assert!(g.board.pending_choice.is_none());

    let mut g = shadow_game(BLACK_BREATH, BoardState::HeroMovementScreen);
    let mut mouth = mob("mouth_of_sauron");
    mouth.location = "bree".to_owned();
    g.mobs.push(mouth);
    g.attack_hero(1, 1, CombatOrigin::Travel, data(), &mut rng())
        .unwrap();
    play(&mut g, BLACK_BREATH);
    assert_eq!(g.heroes[0].corruption_cards.len(), 1);
    let modifiers = g.combat.as_ref().unwrap().modifiers;
    assert_eq!((modifiers.strength, modifiers.agility), (-1, -1));
}

/// The hero, defeated on Bree, recovers in Fornost.
fn defeat(g: &mut Game) {
    g.board.state = BoardState::HeroDefeatScreen;
    g.board.story.ring = 1;
    g.board.story.conquest = 1;
    g.heroes[0].turn = 2;
    g.resolve_hero_defeat(1, DefeatChoice::default(), data(), &mut rng())
        .unwrap();
}

// "Evil Wound": after recovering, 4 damage plus 2 per Corruption card.
#[test]
fn evil_wound_after_a_defeat() {
    let mut g = shadow_game(EVIL_WOUND, BoardState::HeroDefeatScreen);
    g.heroes[0].corruption_cards = vec![PLAIN_CORRUPTION[0]];
    defeat(&mut g);
    assert_eq!(g.heroes[0].location, "fornost");
    play(&mut g, EVIL_WOUND);
    assert!(matches!(source(&g), ChoiceSource::Damage { amount: 6, .. }));
    answer(&mut g, "pay_damage", &[]);
    assert_eq!(g.heroes[0].damage_pool.len(), 6);
    assert_ne!(g.board.state, BoardState::ChoiceScreenBoardChoices);
}

// "Grievous Loss": influence per Corruption card, the green marker held back once.
#[test]
fn grievous_loss_holds_the_heroes_back() {
    let mut g = shadow_game(GRIEVOUS_LOSS, BoardState::HeroDefeatScreen);
    g.heroes[0].corruption_cards = PLAIN_CORRUPTION.to_vec();
    let heroes = g.board.story.heroes;
    defeat(&mut g);
    let budget = g.board.influence_budget.clone();
    play(&mut g, GRIEVOUS_LOSS);
    assert_eq!(g.board.influence_budget.tokens, budget.tokens + 2);
    assert_eq!(
        g.board.influence_budget.shadow_pool_max,
        budget.shadow_pool_max + 2
    );
    // The defeat ended the round: its Story step already took the malus.
    assert_eq!(g.board.story.heroes, heroes + 1);
    g.advance_stories_markers(data()).unwrap();
    assert_eq!(g.board.story.heroes, heroes + 3);
}

// Greedy: a Shadow card played on the hero's turn lets Sauron draw a new one.
#[test]
fn greedy_hero_feeds_the_shadow_hand() {
    let mut g = shadow_game(WINTER_STORM, BoardState::HeroDrawCardsScreen);
    g.heroes[0].corruption_cards = vec![GREEDY];
    g.board.decks.shadow.draw = vec![SARUMANS_LIES];
    g.draw_cards_finished(1, data(), &mut rng()).unwrap();
    play(&mut g, WINTER_STORM);
    assert_eq!(g.sauron.shadow_hand, vec![SARUMANS_LIES]);
}

/// Sauron's Action step, between two actions.
fn action_game(card: u32) -> Game {
    let mut g = shadow_game(card, BoardState::EditSauronSauronActions);
    g.board.current_hero_id = None;
    g.sauron.active = true;
    g
}

fn play_action(g: &mut Game, card: u32) {
    g.play_action_shadow(card, data(), &mut rng()).unwrap();
}

// § 5.9: Action step cards are played between two actions, only then.
#[test]
fn action_step_cards_only_between_actions() {
    let mut g = action_game(SHIRE_BAGGINS);
    g.board.state = BoardState::HeroMovementScreen;
    assert!(
        g.play_action_shadow(SHIRE_BAGGINS, data(), &mut rng())
            .is_err()
    );
    let mut g = action_game(FLOCKS_OF_CREBAIN);
    assert!(
        g.play_action_shadow(FLOCKS_OF_CREBAIN, data(), &mut rng())
            .is_err()
    );
}

// "Shire, Baggins": the yellow marker advances 2 with 3 influence in the Shire.
#[test]
fn shire_baggins_advances_the_ring() {
    let mut g = action_game(SHIRE_BAGGINS);
    g.board.influence.insert("the_shire".to_owned(), 2);
    play_action(&mut g, SHIRE_BAGGINS);
    assert_eq!(g.board.influence["the_shire"], 3);
    assert_eq!(g.board.story.ring, 2);
    assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
    assert!(g.board.turn_effects.shadow_played);
}

// "Storms of Mordor": each hero in turn discards down to 5 cards, then a random card
// per Corruption card.
#[test]
fn storms_of_mordor_reach_every_hero() {
    let mut g = action_game(STORMS_OF_MORDOR);
    add_eleanor(&mut g);
    g.heroes[1].corruption_cards = vec![PLAIN_CORRUPTION[0]];
    let excess = g.heroes[0].hand.len() - 5;
    play_action(&mut g, STORMS_OF_MORDOR);
    assert_eq!(chooser(&g), Chooser::Hero(1));
    let picked: Vec<String> = (0..excess)
        .map(|i| target("hand", format!("1:{i}")))
        .collect();
    answer(&mut g, option::DISCARD_CARDS, &picked);
    assert_eq!(g.heroes[0].hand.len(), 5);
    assert_eq!(chooser(&g), Chooser::Hero(2));
    let excess = g.heroes[1].hand.len() - 5;
    let picked: Vec<String> = (0..excess)
        .map(|i| target("hand", format!("2:{i}")))
        .collect();
    answer(&mut g, option::DISCARD_CARDS, &picked);
    assert_eq!(g.heroes[1].hand.len(), 4);
    assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
}

// "Saruman's Lies": the top plot goes under the deck; a hero with as many Corruption
// cards as his wisdom advances the black marker.
#[test]
fn sarumans_lies_reorder_the_plot_deck() {
    let mut g = action_game(SARUMANS_LIES);
    g.board.decks.plot.draw = vec![3, 4, 5];
    g.heroes[0].corruption_cards = PLAIN_CORRUPTION.to_vec();
    play_action(&mut g, SARUMANS_LIES);
    answer(&mut g, option::PLOT_BOTTOM, &[]);
    assert_eq!(g.board.decks.plot.draw, vec![4, 5, 3]);
    assert_eq!(g.board.story.corruption, 1);
    assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
}

// "The Lidless Eye": Sauron splits two heroes' Corruption cards (with their malus),
// then gives one a new card.
#[test]
fn lidless_eye_redistributes_corruption() {
    let mut g = action_game(LIDLESS_EYE);
    add_eleanor(&mut g);
    g.heroes[0].corruption_cards = vec![PLAIN_CORRUPTION[0], WEAK];
    g.heroes[0].strength -= 1;
    let (argalad, eleanor) = (g.heroes[0].strength, g.heroes[1].strength);
    play_action(&mut g, LIDLESS_EYE);
    answer(
        &mut g,
        option::CHOOSE_HEROES,
        &[target("hero", 1), target("hero", 2)],
    );
    answer(
        &mut g,
        option::REDISTRIBUTE,
        &[target("corruption", PLAIN_CORRUPTION[0])],
    );
    assert_eq!(g.heroes[0].corruption_cards, vec![PLAIN_CORRUPTION[0]]);
    assert_eq!(g.heroes[1].corruption_cards, vec![WEAK]);
    assert_eq!(g.heroes[0].strength, argalad + 1);
    assert_eq!(g.heroes[1].strength, eleanor - 1);
    answer(&mut g, option::CORRUPT_HERO, &[target("hero", 2)]);
    assert_eq!(g.heroes[1].corruption_cards.len(), 2);
    assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
}

// "They Are Terrible": a defeated minion comes back on an influenced location.
#[test]
fn they_are_terrible_bring_back_a_minion() {
    let mut g = action_game(THEY_ARE_TERRIBLE);
    play_action(&mut g, THEY_ARE_TERRIBLE);
    answer(
        &mut g,
        option::PICK_MINION,
        &[target("defeated_minion", "black_serpent")],
    );
    answer(
        &mut g,
        option::PLACE_MINION,
        &[target("location", "barad_dur")],
    );
    let serpent = g.mobs.iter().find(|m| m.code == "black_serpent").unwrap();
    assert_eq!(serpent.location, "barad_dur");
    assert_eq!(serpent.life, serpent.max_life);
}

// "The Ithil-stone": Saruman Falls to Corruption is found and played at once.
#[test]
fn ithil_stone_plays_saruman_falls() {
    let mut g = action_game(ITHIL_STONE);
    g.board.decks.plot.discard = vec![6];
    g.board.plots.clear();
    play_action(&mut g, ITHIL_STONE);
    answer(&mut g, option::PLAY_PLOT, &[target("plot_card", 6)]);
    assert_eq!(g.board.plots.get(&1), Some(&6));
    assert!(g.board.decks.plot.discard.is_empty());
}

// "Lord of the Rings": a discarded card back to hand; "A Dark Messenger" advances the
// yellow marker.
#[test]
fn lord_of_the_rings_takes_back_a_card() {
    let mut g = action_game(LORD_OF_THE_RINGS);
    g.board.decks.shadow.discard = vec![BETRAYED];
    g.board.plots.insert(2, 3);
    play_action(&mut g, LORD_OF_THE_RINGS);
    answer(&mut g, option::TAKE_CARD, &[target("shadow", BETRAYED)]);
    assert_eq!(g.sauron.shadow_hand, vec![BETRAYED]);
    assert_eq!(g.board.decks.shadow.discard, vec![LORD_OF_THE_RINGS]);
    assert_eq!(g.board.story.ring, 1);
}

// "Do Not Tempt Me!": influence on the hero's location, a favor per Corruption card.
#[test]
fn do_not_tempt_me_costs_favor() {
    let mut g = action_game(DO_NOT_TEMPT_ME);
    g.heroes[0].favor = 3;
    g.heroes[0].corruption_cards = PLAIN_CORRUPTION.to_vec();
    play_action(&mut g, DO_NOT_TEMPT_ME);
    answer(&mut g, option::INFLUENCE_ON_HERO, &[target("hero", 1)]);
    assert_eq!(g.board.influence["bree"], 1);
    assert_eq!(g.heroes[0].favor, 1);
}

// "An Evil Fog": the hero is moved, then discards a card per Corruption card.
#[test]
fn evil_fog_moves_a_hero() {
    let mut g = action_game(EVIL_FOG);
    g.heroes[0].corruption_cards = vec![PLAIN_CORRUPTION[0]];
    let hand = g.heroes[0].hand.len();
    let to = data().paths.connected("bree")[0].to_owned();
    play_action(&mut g, EVIL_FOG);
    answer(&mut g, option::MOVE_HERO, &[format!("hero_to:1:{to}")]);
    assert_eq!(g.heroes[0].location, to);
    assert_eq!(g.heroes[0].hand.len(), hand - 1);
}

// "A New Power Is Rising": 2 influence on each location of Mordor.
#[test]
fn new_power_rises_in_mordor() {
    let mut g = action_game(NEW_POWER);
    let mordor = |g: &Game| -> i32 {
        data()
            .locations
            .iter()
            .filter(|(_, l)| l.region_code == "mordor")
            .map(|(code, _)| g.board.influence.get(code).copied().unwrap_or(0))
            .sum()
    };
    let before = mordor(&g);
    play_action(&mut g, NEW_POWER);
    answer(&mut g, option::MORDOR_INFLUENCE, &[]);
    assert!(mordor(&g) > before);
    assert_eq!(g.board.state, BoardState::EditSauronSauronActions);
}
