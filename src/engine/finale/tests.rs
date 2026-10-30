use super::*;
use crate::engine::board_state::BoardState;
use crate::engine::combat::flow::CombatEnd;
use crate::engine::test_support::{data, game, hero, rng};
use crate::model::board::StoryMarkers;
use crate::model::mission::{HEROES_MISSIONS, HeroesMission, SAURON_MISSIONS, SauronMission};

const SHIRE: &str = "the_shire";

fn card<T: PartialEq>(cards: &[T], mission: &T) -> Option<u32> {
    cards
        .iter()
        .position(|m| m == mission)
        .and_then(|i| u32::try_from(i).ok())
}

/// A single-hero game at the end of its round, the Story step about to place the
/// markers on `story` (the heroes' marker moves 2 more spaces).
fn round_end(story: StoryMarkers, sauron: SauronMission, heroes: HeroesMission) -> Game {
    let mut g = game();
    g.board.state = BoardState::HeroExploration;
    g.heroes[0].turn = 2;
    g.board.story = StoryMarkers {
        heroes: story.heroes - 2,
        ..story
    };
    g.board.sauron_objective = card(&SAURON_MISSIONS, &sauron);
    g.board.heroes_objective = card(&HEROES_MISSIONS, &heroes);
    g
}

fn markers(heroes: i32, ring: i32, conquest: i32, corruption: i32) -> StoryMarkers {
    StoryMarkers {
        heroes,
        ring,
        conquest,
        corruption,
    }
}

fn end_round(g: &mut Game) {
    g.finish_heroes_turn(1, data(), &mut rng()).unwrap();
}

fn ringwraiths(g: &Game) -> &Mob {
    g.mobs.iter().find(|m| m.code == RINGWRAITHS).unwrap()
}

// § 5.3, § 9: the Finale begins at the Finale space or with all Sauron's markers on
// The Shadow Falls.
#[test]
fn finale_triggers() {
    let mut g = game();
    g.board.story = markers(17, 12, 0, 0);
    assert!(!g.finale_triggered());
    g.board.story = markers(18, 0, 0, 0);
    assert!(g.finale_triggered());
    g.board.story = markers(0, 10, 10, 9);
    assert!(!g.finale_triggered());
    g.board.story = markers(0, 10, 11, 10);
    assert!(g.finale_triggered());
}

// § 9 step 1: the dominant side wins at once by its fulfilled Mission.
#[test]
fn dominant_side_wins_by_its_mission() {
    let story = markers(14, 18, 0, 0);
    let mut g = round_end(
        story,
        SauronMission::InDarknessBindThem,
        HeroesMission::NobleBlood,
    );
    end_round(&mut g);
    assert_eq!(g.board.state, BoardState::Finished);
    assert_eq!(g.board.winner, Some(Winner::Sauron));
    assert!(!g.sauron.active && !g.heroes_active());
}

// § 9 step 1: the dominant side's Mission alone counts; the other's isn't checked.
#[test]
fn only_the_dominant_mission_counts() {
    let story = markers(18, 2, 0, 0);
    let mut g = round_end(
        story,
        SauronMission::HisDarkThrone,
        HeroesMission::SpearOfTheWest,
    );
    g.board.plots = [(1, 0), (2, 3), (3, 4)].into();
    end_round(&mut g);
    assert_eq!(g.board.winner, None);
    assert_eq!(g.board.state, BoardState::ChoiceScreenBoardChoices);
}

// § 9 step 1: with no dominant side, a single fulfilled Mission wins; two go on.
#[test]
fn no_dominance_needs_a_single_mission() {
    let story = markers(18, 18, 0, 0);
    let mut g = round_end(
        story,
        SauronMission::HisDarkThrone,
        HeroesMission::NobleBlood,
    );
    end_round(&mut g);
    assert_eq!(g.board.winner, Some(Winner::Heroes));

    let mut g = round_end(
        story,
        SauronMission::InDarknessBindThem,
        HeroesMission::NobleBlood,
    );
    end_round(&mut g);
    assert_eq!(g.board.winner, None);
    assert_eq!(g.board.state, BoardState::ChoiceScreenBoardChoices);
}

// § 9 step 2: fulfilled Missions advance their markers, not past the Finale space.
#[test]
fn missions_advance_markers() {
    let story = markers(18, 13, 4, 18);
    let mut g = round_end(
        story,
        SauronMission::WhereTheShadowsLie,
        HeroesMission::NobleBlood,
    );
    end_round(&mut g);
    assert_eq!(g.board.winner, None);
    assert_eq!(g.board.story, markers(18, 14, 5, 18));
}

// § 9 step 3: the Ringwraiths are healed, the heroes recover and draw their fortitude.
#[test]
fn preparation() {
    let story = markers(18, 0, 0, 0);
    let mut g = round_end(
        story,
        SauronMission::HisDarkThrone,
        HeroesMission::SpearOfTheWest,
    );
    g.create_mob(RINGWRAITHS, SHIRE, None, data()).unwrap();
    g.mobs[0].life = 2;
    let cards = g.heroes[0].life_pool.len() + g.heroes[0].hand.len();
    let damaged = g.heroes[0].life_pool.split_off(3);
    g.heroes[0].damage_pool = damaged;
    end_round(&mut g);
    let hero = &g.heroes[0];
    assert_eq!(hero.hand.len(), 5);
    assert_eq!(hero.hand.len() + hero.life_pool.len(), cards);
    assert!(hero.damage_pool.is_empty() && hero.rest_pool.is_empty());
    assert_eq!(g.mobs.len(), 1);
    // Heroes 0 spaces from the Finale, Sauron 18: the health can't fall below 1.
    assert_eq!(ringwraiths(&g).life, 1);
}

// § 9 step 3: defeated Ringwraiths come back for the Finale.
#[test]
fn defeated_ringwraiths_come_back() {
    let story = markers(18, 0, 0, 0);
    let mut g = round_end(
        story,
        SauronMission::HisDarkThrone,
        HeroesMission::SpearOfTheWest,
    );
    g.board.ringwraiths_return = true;
    end_round(&mut g);
    assert_eq!(ringwraiths(&g).location, RINGWRAITHS_HOME);
    assert!(!g.board.ringwraiths_return);
}

// § 9 step 4: Sauron dominant, the Ringwraiths gain the heroes' distance to the Finale.
#[test]
fn sauron_dominance_strengthens_the_ringwraiths() {
    let story = markers(12, 18, 3, 3);
    let mut g = round_end(
        story,
        SauronMission::HisDarkThrone,
        HeroesMission::SpearOfTheWest,
    );
    end_round(&mut g);
    let def = data().mob(RINGWRAITHS).unwrap();
    let ringwraiths = ringwraiths(&g);
    assert_eq!(ringwraiths.life, def.life.unwrap() + 6);
    assert_eq!(ringwraiths.max_life, def.life.unwrap() + 6);
    assert_eq!(ringwraiths.fortitude, def.fortitude + 6);
}

// § 9 example: heroes dominant, the Ringwraiths lose the lower of Sauron's distances.
#[test]
fn heroes_dominance_weakens_the_ringwraiths() {
    // Closest marker 3 spaces from the Finale; 2 + 3 + 0 spaces to The Shadow Falls.
    let story = markers(18, 8, 7, 15);
    let mut g = round_end(
        story,
        SauronMission::HisDarkThrone,
        HeroesMission::SpearOfTheWest,
    );
    end_round(&mut g);
    let def = data().mob(RINGWRAITHS).unwrap();
    assert_eq!(ringwraiths(&g).life, def.life.unwrap() - 3);
    assert_eq!(ringwraiths(&g).fortitude, def.fortitude - 3);
}

/// A two-hero Finale waiting for the champion, the Ringwraiths persisted.
fn champion_choice() -> Game {
    let mut g = round_end(
        markers(18, 0, 0, 0),
        SauronMission::HisDarkThrone,
        HeroesMission::SpearOfTheWest,
    );
    let mut second = hero("beravor");
    second.id = Some(2);
    second.playing_order = 1;
    second.turn_finished = true;
    g.heroes.push(second);
    g.finish_heroes_turn(1, data(), &mut rng()).unwrap();
    for (id, mob) in (10..).zip(g.mobs.iter_mut()) {
        mob.id = Some(id);
    }
    g
}

// § 9 steps 5 and 6: any hero picks the champion, who fights the Ringwraiths.
#[test]
fn champion_fights_the_ringwraiths() {
    let mut g = champion_choice();
    let choice = g.board.pending_choice.clone().unwrap();
    assert_eq!(choice.chooser, Chooser::Heroes);
    assert_eq!(choice.options[0].targets, vec!["hero:1", "hero:2"]);
    assert!(g.heroes.iter().all(|h| h.active));

    g.answer_choice(CHAMPION, &["hero:2".to_owned()], data(), &mut rng())
        .unwrap();
    assert_eq!(g.board.state, BoardState::CombatSetupScreenBoardCombats);
    assert_eq!(g.current_hero_id().unwrap(), 2);
    let combat = g.combat.as_ref().unwrap();
    assert_eq!(combat.origin, CombatOrigin::Finale);
    assert_eq!(combat.hero_id, 2);
    assert_eq!(Some(combat.mob_id), ringwraiths(&g).id);
    assert!(g.heroes[1].active && !g.heroes[0].active);
}

fn finale_fight() -> Game {
    let mut g = champion_choice();
    g.answer_choice(CHAMPION, &["hero:1".to_owned()], data(), &mut rng())
        .unwrap();
    g.combat_setup(0, data(), &mut rng()).unwrap();
    g
}

// § 9 step 7: the heroes win when the Ringwraiths are defeated.
#[test]
fn defeating_the_ringwraiths_wins() {
    let mut g = finale_fight();
    g.mobs.iter_mut().for_each(|m| m.life = 0);
    let end = g.terminate_combat(data(), &mut rng()).unwrap();
    assert_eq!(end, CombatEnd::GameOver);
    assert_eq!(g.board.winner, Some(Winner::Heroes));
    assert_eq!(g.board.state, BoardState::Finished);
    assert!(g.combat.is_none());
}

// § 9 step 7: Sauron wins when the champion falls, without the defeat steps.
#[test]
fn a_fallen_champion_loses() {
    let mut g = finale_fight();
    let story = g.board.story;
    let hero = &mut g.heroes[0];
    hero.life_pool.clear();
    hero.hand.clear();
    assert_eq!(
        g.terminate_combat(data(), &mut rng()).unwrap(),
        CombatEnd::GameOver
    );
    assert_eq!(g.board.winner, Some(Winner::Sauron));
    assert_eq!(g.board.story, story);
}

// § 9 step 7: both exhausted, the Ringwraiths were not defeated.
#[test]
fn exhausted_champion_loses() {
    let mut g = finale_fight();
    let combat = g.combat.as_mut().unwrap();
    combat.hero_exhausted = true;
    combat.mob_exhausted = true;
    g.terminate_combat(data(), &mut rng()).unwrap();
    assert_eq!(g.board.winner, Some(Winner::Sauron));
}

// § 5.3: minions of a new stage enter before the Finale begins.
#[test]
fn stage_minions_enter_before_the_finale() {
    // Plot 0 moves the ring marker into stage III and onto The Shadow Falls with the others.
    let story = markers(4, 12, 10, 10);
    let mut g = round_end(
        story,
        SauronMission::HisDarkThrone,
        HeroesMission::SpearOfTheWest,
    );
    g.board.plots.insert(1, 0);
    end_round(&mut g);
    assert_eq!(g.board.story.ring, 13);
    assert!(g.mobs.iter().any(|m| m.code == "witch_king"));
    assert_eq!(g.board.state, BoardState::ChoiceScreenBoardChoices);
}
