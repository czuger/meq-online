//! The moves a computer hero tries on each screen, best first. Where a real
//! strategy would get complex, the order is random.

use std::collections::{BTreeMap, BTreeSet};

use rand::RngExt;
use rand::rngs::StdRng;
use rand::seq::{IndexedRandom, SliceRandom};

use crate::engine::EngineError;
use crate::engine::ai::hero_move::HeroMove;
use crate::engine::board_state::BoardState;
use crate::engine::choices::PASS;
use crate::engine::defeat::{DefeatChoice, defeat_losses};
use crate::engine::exploration::ExploreChoice;
use crate::engine::movement::path_difficulty;
use crate::engine::rest::Rest;
use crate::engine::tokens::TokenKind;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::location_paths::Destination;
use crate::model::choice::ChoiceOption;
use crate::model::game::Game;
use crate::model::hero::Hero;

/// Moves for `hero_id` on the current screen, in the order to try them.
///
/// # Errors
/// Lookup errors.
pub fn candidates(
    game: &Game,
    hero_id: i64,
    data: &GameData,
    rng: &mut StdRng,
) -> Result<Vec<HeroMove>, EngineError> {
    let hero = game.hero(hero_id)?;
    Ok(match game.board.state {
        BoardState::HeroDrawCardsScreen => draw(hero, data),
        BoardState::HeroRestScreen => rest(hero),
        BoardState::AfterRestAdvanceStoryMarker => markers(game, rng),
        BoardState::HeroMovementScreen => movement(game, hero, data, rng)?,
        BoardState::HeroExploration => exploration(game, hero_id, data)?,
        BoardState::HeroDefeatScreen => defeat(game, hero, rng),
        BoardState::CombatSetupScreenBoardCombats => combat_setup(game, hero, rng),
        BoardState::PlayCombatCardScreenBoardCombats => combat_round(game, hero, data, rng),
        BoardState::CardsLossScreenBoardCombats => cards_loss(hero, rng),
        BoardState::ChoiceScreenBoardChoices => answers(game, rng),
        _ => vec![],
    })
}

/// Draws every card allowed, then ends the draw.
fn draw(hero: &Hero, data: &GameData) -> Vec<HeroMove> {
    let left = usize::try_from(hero.cards_left_to_draw(data)).unwrap_or(0);
    let mut moves = Vec::new();
    if left > 0 {
        moves.push(HeroMove::Draw(left));
    }
    moves.push(HeroMove::DrawFinished);
    moves
}

/// Heals when wounded (refused outside a Haven), rests when the life pool runs low,
/// otherwise skips: resting advances Sauron's story.
fn rest(hero: &Hero) -> Vec<HeroMove> {
    let wounded = !hero.damage_pool.is_empty();
    let tired = hero.life_pool.len() < usize::try_from(hero.fortitude).unwrap_or(0);
    let mut moves = Vec::new();
    if wounded {
        moves.push(HeroMove::Rest(Rest {
            heal: true,
            ..Rest::default()
        }));
    }
    if wounded || tired {
        moves.push(HeroMove::Rest(Rest::default()));
    }
    moves.push(HeroMove::SkipRest);
    moves.push(HeroMove::Rest(Rest::default()));
    moves
}

fn markers(game: &Game, rng: &mut StdRng) -> Vec<HeroMove> {
    let mut markers = game.board.story.lowest_sauron_markers();
    markers.shuffle(rng);
    markers.into_iter().map(HeroMove::AdvanceMarker).collect()
}

/// Travels toward the closest favor or plot, ties broken at random. On such a
/// location the hero stays half of the time, to explore it.
fn movement(
    game: &Game,
    hero: &Hero,
    data: &GameData,
    rng: &mut StdRng,
) -> Result<Vec<HeroMove>, EngineError> {
    let goals = goals(game, data);
    if goals.contains(&hero.location) && rng.random_bool(0.5) {
        return Ok(vec![HeroMove::MovementFinished]);
    }
    let distances = goal_distances(&goals, data);
    let mut paths: Vec<&Destination> = data.paths.destinations(&hero.location).iter().collect();
    paths.shuffle(rng);
    paths.sort_by_key(|p| distances.get(&p.dest).copied().unwrap_or(u32::MAX));
    let mut moves = Vec::new();
    for path in paths {
        tracing::trace!(destination = %path.dest, "computer hero destination");
        for cards in payments(hero, path, data, rng)? {
            let to = path.dest.clone();
            moves.push(HeroMove::Move { to, cards });
        }
    }
    moves.push(HeroMove::MovementFinished);
    Ok(moves)
}

/// Locations holding a favor or an active plot.
fn goals(game: &Game, data: &GameData) -> BTreeSet<String> {
    let mut goals: BTreeSet<String> = game.board.favors.iter().cloned().collect();
    let plots = game.board.plots.keys();
    goals.extend(plots.filter_map(|slot| game.plot_location(*slot, data)));
    goals
}

/// Distance from every location to the closest goal.
fn goal_distances(goals: &BTreeSet<String>, data: &GameData) -> BTreeMap<String, u32> {
    let mut closest: BTreeMap<String, u32> = BTreeMap::new();
    for goal in goals {
        tracing::trace!(%goal, "computer hero goal");
        for (location, distance) in data.paths.distances_from(goal) {
            let entry = closest.entry(location.to_owned()).or_insert(distance);
            *entry = (*entry).min(distance);
        }
    }
    closest
}

/// Ways to pay for `path`: one random card of its terrain, and as many random cards
/// as its difficulty.
fn payments(
    hero: &Hero,
    path: &Destination,
    data: &GameData,
    rng: &mut StdRng,
) -> Result<Vec<Vec<CardNumber>>, EngineError> {
    let mut hand = hero.hand.clone();
    hand.shuffle(rng);
    let mut payments = Vec::new();
    for card in &hand {
        if data.hero_card(&hero.code, *card)?.movement_type == Some(path.path_type) {
            payments.push(vec![*card]);
            break;
        }
    }
    let needed = usize::try_from(path_difficulty(path, hero)).unwrap_or(usize::MAX);
    if needed <= hand.len() {
        payments.push(hand[..needed].to_vec());
    }
    Ok(payments)
}

/// Takes everything on the location (favors, plot, event, characters for favor),
/// then less when the rules refuse it, then ends the turn. Never a Dark Path.
fn exploration(game: &Game, hero_id: i64, data: &GameData) -> Result<Vec<HeroMove>, EngineError> {
    let hero = game.hero(hero_id)?;
    // Listing the tokens logs hidden characters: list them on a copy.
    let tokens = game.clone().exploration_tokens(hero_id, data)?;
    let has = |kind: TokenKind| tokens.iter().any(|t| t.kind == kind);
    let favors = game.board.favors.iter().filter(|f| **f == hero.location);
    let everything = ExploreChoice {
        characters: tokens
            .iter()
            .filter(|t| t.kind == TokenKind::Character)
            .map(|t| t.code.clone())
            .collect(),
        favors: favors.count(),
        plot: has(TokenKind::Plot),
        event: has(TokenKind::Event),
        ..ExploreChoice::default()
    };
    let without_plot = ExploreChoice {
        plot: false,
        ..everything.clone()
    };
    let favors_only = ExploreChoice {
        favors: everything.favors,
        ..ExploreChoice::default()
    };
    let mut moves: Vec<HeroMove> = [everything, without_plot, favors_only]
        .into_iter()
        .filter(|choice| *choice != ExploreChoice::default())
        .map(HeroMove::Explore)
        .collect();
    moves.dedup();
    moves.push(HeroMove::EndTurn);
    Ok(moves)
}

fn defeat(game: &Game, hero: &Hero, rng: &mut StdRng) -> Vec<HeroMove> {
    let marker = game.defeat_marker_choices().choose(rng).copied();
    let losses = defeat_losses(hero);
    let loss = (losses.len() > 1)
        .then(|| losses.choose(rng).copied())
        .flatten();
    vec![
        HeroMove::Defeat(DefeatChoice { marker, loss }),
        HeroMove::Defeat(DefeatChoice::default()),
    ]
}

/// A random split of agility between cards and strength.
fn combat_setup(game: &Game, hero: &Hero, rng: &mut StdRng) -> Vec<HeroMove> {
    let agility = game
        .combat
        .as_ref()
        .map_or(0, |c| c.modifiers.agility(hero.agility));
    vec![
        HeroMove::CombatSetup(rng.random_range(0..=agility)),
        HeroMove::CombatSetup(0),
    ]
}

/// Ends a combat that is over, otherwise plays a random card the remaining strength
/// pays for (strength is only checked on reveal), otherwise declares exhaustion.
fn combat_round(game: &Game, hero: &Hero, data: &GameData, rng: &mut StdRng) -> Vec<HeroMove> {
    let mut moves = vec![HeroMove::Terminate];
    let Some(combat) = game.combat.as_ref() else {
        return moves;
    };
    let strength_left = combat.hero_strength.unwrap_or(0) - combat.hero_strength_used;
    let mut hand = hero.hand.clone();
    hand.shuffle(rng);
    let (affordable, costly): (Vec<CardNumber>, Vec<CardNumber>) =
        hand.into_iter().partition(|card| {
            data.hero_card(&hero.code, *card)
                .is_ok_and(|def| def.strength_cost <= strength_left)
        });
    moves.extend(affordable.into_iter().map(HeroMove::CombatCard));
    moves.push(HeroMove::Exhaustion);
    moves.extend(costly.into_iter().map(HeroMove::CombatCard));
    moves
}

/// Pays damage with a random number of random hand cards, the rest from the life pool.
fn cards_loss(hero: &Hero, rng: &mut StdRng) -> Vec<HeroMove> {
    let mut hand = hero.hand.clone();
    hand.shuffle(rng);
    let most = usize::try_from(hero.temporary_damages)
        .unwrap_or(0)
        .min(hand.len());
    let mut counts: Vec<usize> = (0..=most).collect();
    counts.shuffle(rng);
    counts
        .into_iter()
        .map(|count| HeroMove::CardsLoss(hand[..count].to_vec()))
        .collect()
}

/// Every option in random order, passing first half of the time, each with random
/// targets and then with its first ones.
fn answers(game: &Game, rng: &mut StdRng) -> Vec<HeroMove> {
    let Some(choice) = game.board.pending_choice.as_ref() else {
        return vec![];
    };
    let mut options: Vec<&ChoiceOption> = choice.options.iter().collect();
    options.shuffle(rng);
    if rng.random_bool(0.5) {
        options.sort_by_key(|o| o.code != PASS);
    }
    let mut moves = Vec::new();
    for option in options {
        tracing::trace!(option = %option.code, "computer hero option");
        moves.push(random_answer(option, rng));
        let targets = option.targets.iter().take(option.min).cloned().collect();
        let code = option.code.clone();
        moves.push(HeroMove::Answer {
            option: code,
            targets,
        });
    }
    moves
}

fn random_answer(option: &ChoiceOption, rng: &mut StdRng) -> HeroMove {
    let max = option.max.min(option.targets.len());
    let count = if option.min < max {
        rng.random_range(option.min..=max)
    } else {
        option.min
    };
    HeroMove::Answer {
        option: option.code.clone(),
        targets: option.targets.sample(rng, count).cloned().collect(),
    }
}
