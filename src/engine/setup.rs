//! Creation of a new game.

use std::collections::BTreeMap;

use rand::RngExt;
use rand::rngs::StdRng;
use rand::seq::{IndexedRandom, SliceRandom};
use serde_json::json;

use crate::engine::board_state::{BoardEvent, BoardState};
use crate::engine::deck::SauronDeck;
use crate::engine::engine_error::EngineError;
use crate::engine::events::event_step::starting_event_decks;
use crate::game_data::GameData;
use crate::game_data::GameDataError;
use crate::game_data::hero_quests::QuestSetupKind;
use crate::game_data::monster_pool_color::MonsterPoolColor;
use crate::model::actor_ref::ActorRef;
use crate::model::board::{Board, Decks, Pile, StoryMarkers};
use crate::model::game::Game;
use crate::model::game_options::GameOptions;
use crate::model::hero::{Hero, HeroItems};
use crate::model::sauron::Sauron;
use crate::model::turn_effects::TurnEffects;

/// Most heroes in a game (§ 1: one Sauron and up to three heroes).
pub const MAX_HEROES: usize = 3;

/// Minions on the map from the start: `(mob, location)`.
pub(crate) const STARTING_MINIONS: [(&str, &str); 2] = [
    ("mouth_of_sauron", "dol_guldur"),
    ("black_serpent", "near_harad"),
];

/// A hero seat of a new game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroSeat<'a> {
    /// Hero code.
    pub code: &'a str,
    /// User holding the seat (a computer seat until a player takes an open seat).
    pub user_id: i64,
    /// The seat waits for a player to join.
    pub open: bool,
}

/// Builds a new game (§ 3): Sauron for `sauron_user_id`, the chosen heroes, the
/// starting plot, decks, minions and quests. The game waits for players while a seat
/// is open, and otherwise opens on Sauron's setup.
///
/// # Arguments
/// * `sauron_user_id` - user playing Sauron (the creator).
/// * `seats` - one to three different heroes, in playing order.
/// * `data` - static data.
/// * `rng` - random source.
///
/// # Errors
/// [`EngineError::Refused`] for a wrong number of heroes or a hero chosen twice;
/// fails when the static data misses a hero, plot or minion.
#[tracing::instrument(level = "debug", skip_all, fields(sauron_user_id, ?seats), err(level = "debug"))]
pub fn new_game(
    sauron_user_id: i64,
    seats: &[HeroSeat<'_>],
    data: &GameData,
    rng: &mut StdRng,
) -> Result<Game, EngineError> {
    check_seats(seats)?;
    let starting_plot = rng.random_range(0..=2);
    tracing::debug!(starting_plot, "starting plot drawn");
    let mut board = new_board(starting_plot, data, rng)?;
    board.max_heroes = i32::try_from(seats.len()).unwrap_or(i32::MAX);
    board.waiting_heroes = seats
        .iter()
        .filter(|s| s.open)
        .map(|s| s.code.to_owned())
        .collect();
    let sauron = Sauron {
        user_id: sauron_user_id,
        ..Sauron::default()
    };
    let mut game = Game {
        board,
        sauron,
        heroes: vec![],
        mobs: vec![],
        combat: None,
        new_logs: vec![],
    };

    for (order, seat) in seats.iter().enumerate() {
        tracing::trace!(
            order,
            hero = seat.code,
            user_id = seat.user_id,
            "seating hero"
        );
        let hero = new_hero(seat.code, seat.user_id, order, data, rng)?;
        game.apply_quest_setup(seat.code, hero.quest, data)?;
        game.heroes.push(hero);
    }
    if game.board.waiting_heroes.is_empty() {
        game.start_game()?;
    } else {
        game.fire(BoardEvent::WaitForPlayers)?;
    }

    for (mob, location) in STARTING_MINIONS {
        tracing::trace!(mob, location, "placing starting minion");
        game.create_mob(mob, location, None, data)?;
    }
    game.log_setup(starting_plot, data)?;
    game.board.plots.insert(1, starting_plot);
    game.starting_plot_instructions(starting_plot, data, rng)?;
    Ok(game)
}

fn check_seats(seats: &[HeroSeat<'_>]) -> Result<(), EngineError> {
    if seats.is_empty() || seats.len() > MAX_HEROES {
        return Err(EngineError::Refused(format!(
            "a game has 1 to {MAX_HEROES} heroes"
        )));
    }
    let mut codes: Vec<&str> = seats.iter().map(|s| s.code).collect();
    codes.sort_unstable();
    codes.dedup();
    if codes.len() != seats.len() {
        return Err(EngineError::Refused("each hero is chosen once".to_owned()));
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all, fields(starting_plot), err(level = "debug"))]
fn new_board(starting_plot: u32, data: &GameData, rng: &mut StdRng) -> Result<Board, EngineError> {
    let influence = data
        .plot(starting_plot)?
        .influence
        .clone()
        .ok_or_else(|| GameDataError::unknown("starting influence of plot", starting_plot))?;

    Ok(Board {
        id: None,
        state: BoardState::Created,
        max_heroes: 0,
        turn: 1,
        current_hero_id: None,
        winner: None,
        story: StoryMarkers::default(),
        shadow_pool: influence.shadow_pool,
        influence: influence.init,
        characters: BTreeMap::new(),
        favors: vec![],
        plots: BTreeMap::new(),
        sauron_actions: vec![],
        sauron_actions_count: 0,
        current_action: None,
        decks: shuffled_decks(data, rng),
        monster_pools: shuffled_monster_pools(data, rng),
        monster_discards: BTreeMap::new(),
        waiting_heroes: vec![],
        ringwraiths_return: false,
        influence_budget: Default::default(),
        last_event: None,
        current_event: None,
        held_event: None,
        pending_choice: None,
        turn_effects: TurnEffects::default(),
        heroes_objective: None,
        sauron_objective: None,
        options: GameOptions::default(),
    })
}

#[tracing::instrument(level = "debug", skip_all)]
fn shuffled_decks(data: &GameData, rng: &mut StdRng) -> Decks {
    // Plots 0 to 2 are the starting plots.
    let mut plot: Vec<u32> = data.plots.keys().copied().filter(|c| *c >= 3).collect();
    plot.shuffle(rng);
    tracing::trace!(pile = "plot", order = ?plot, "shuffled");
    let mut shadow: Vec<u32> = data.shadows.keys().copied().collect();
    shadow.shuffle(rng);
    tracing::trace!(pile = "shadow", order = ?shadow, "shuffled");
    let mut corruption = data.corruption.deck.clone();
    corruption.shuffle(rng);
    tracing::trace!(pile = "corruption", order = ?corruption, "shuffled");
    let mut skills: Vec<u32> = data.skills.keys().copied().collect();
    skills.shuffle(rng);
    tracing::trace!(pile = "skills", order = ?skills, "shuffled");
    let mut peril: Vec<u32> = data.perils.keys().copied().collect();
    peril.shuffle(rng);
    tracing::trace!(pile = "peril", order = ?peril, "shuffled");
    let [event, event_ii, event_iii] = starting_event_decks(data, rng);
    Decks {
        plot: Pile {
            draw: plot,
            discard: vec![],
        },
        shadow: Pile {
            draw: shadow,
            discard: vec![],
        },
        event: Pile {
            draw: event,
            discard: vec![],
        },
        event_ii: Pile {
            draw: event_ii,
            discard: vec![],
        },
        event_iii: Pile {
            draw: event_iii,
            discard: vec![],
        },
        skill: Pile {
            draw: skills,
            discard: vec![],
        },
        corruption: Pile {
            draw: corruption,
            discard: vec![],
        },
        peril: Pile {
            draw: peril,
            discard: vec![],
        },
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn shuffled_monster_pools(
    data: &GameData,
    rng: &mut StdRng,
) -> BTreeMap<MonsterPoolColor, Vec<String>> {
    let mut pools = data.monster_pools.clone();
    for pool in pools.values_mut() {
        tracing::trace!(cards = pool.len(), "shuffling monster pool");
        pool.shuffle(rng);
        tracing::trace!(pile = "pool", order = ?pool, "shuffled");
    }
    pools
}

#[tracing::instrument(level = "debug", skip_all, fields(%code, user_id, order), err(level = "debug"))]
fn new_hero(
    code: &str,
    user_id: i64,
    order: usize,
    data: &GameData,
    rng: &mut StdRng,
) -> Result<Hero, EngineError> {
    let def = data.hero(code)?;
    let mut life_pool = def.starting_deck.clone();
    life_pool.shuffle(rng);
    tracing::trace!(pile = "life_pool", order = ?life_pool, "shuffled");
    let quests = data.hero_quest(code)?;
    let quest = quests.starting_quests.choose(rng).copied();
    tracing::debug!(hero = code, ?quest, "starting quest drawn");
    Ok(Hero {
        id: None,
        user_id,
        code: code.to_owned(),
        location: def.start_location.clone(),
        fortitude: def.fortitude,
        strength: def.strength,
        agility: def.agility,
        wisdom: def.wisdom,
        favor: 0,
        quest,
        advanced_quest: quests.advanced_quests.first().copied(),
        playing_order: i32::try_from(order).unwrap_or(i32::MAX),
        turn: 1,
        active: false,
        turn_finished: false,
        damages_taken_this_turn: 0,
        temporary_damages: 0,
        cards_drawn_this_round: 0,
        life_pool,
        hand: vec![],
        rest_pool: vec![],
        damage_pool: vec![],
        items: HeroItems {
            horse: code == "eometh",
            boat: false,
        },
        levels: Default::default(),
        turn_record: Default::default(),
        corruption_cards: vec![],
    })
}

impl Game {
    /// A player takes an open hero seat; the game starts when no seat is left open.
    ///
    /// # Returns
    /// `true` when the game started.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the hero's seat isn't open; transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(%code, user_id), err(level = "debug"))]
    pub fn join_hero_seat(&mut self, code: &str, user_id: i64) -> Result<bool, EngineError> {
        let position = self
            .board
            .waiting_heroes
            .iter()
            .position(|c| c == code)
            .ok_or_else(|| EngineError::Refused(format!("the seat of {code} is not open")))?;
        self.board.waiting_heroes.remove(position);
        let hero = self
            .heroes
            .iter_mut()
            .find(|h| h.code == code)
            .ok_or_else(|| EngineError::Refused(format!("{code} is not in this game")))?;
        hero.user_id = user_id;
        if !self.board.waiting_heroes.is_empty() {
            return Ok(false);
        }
        self.start_game()?;
        Ok(true)
    }

    /// Every seat is taken: Sauron plays first, starting with his setup.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    fn start_game(&mut self) -> Result<(), EngineError> {
        self.fire(BoardEvent::NextToSauronSetupScreen)?;
        self.sauron.active = true;
        Ok(())
    }

    /// Follows the "Setup" instructions of the hero's Starting Quest (§ 3).
    #[tracing::instrument(level = "debug", skip_all, fields(%hero, ?quest), err(level = "debug"))]
    fn apply_quest_setup(
        &mut self,
        hero: &str,
        quest: Option<u32>,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let quests = data.hero_quest(hero)?;
        let Some(quest) = quest else {
            return Ok(());
        };
        for action in quests.setup.get(&quest).into_iter().flatten() {
            tracing::trace!(?action, "quest setup action");
            match action.action {
                QuestSetupKind::Place => {
                    self.board
                        .characters
                        .insert(action.character.clone(), action.location.clone());
                }
            }
        }
        Ok(())
    }

    /// Sauron follows his starting plot (§ 3): influence to place and plot cards to
    /// draw, some of which he keeps.
    #[tracing::instrument(level = "debug", skip_all, fields(starting_plot), err(level = "debug"))]
    fn starting_plot_instructions(
        &mut self,
        starting_plot: u32,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let plot = data.plot(starting_plot)?;
        if let Some(influence) = &plot.influence {
            self.grant_influence(influence.add, 0);
        }
        if let Some(cards) = plot.plot_cards {
            let count = usize::try_from(cards.draw).unwrap_or(0);
            self.draw_sauron_cards(SauronDeck::Plot, count, rng);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all, fields(starting_plot), err(level = "debug"))]
    fn log_setup(&mut self, starting_plot: u32, data: &GameData) -> Result<(), EngineError> {
        self.log(
            ActorRef::System,
            "setup.set_starting_plot",
            &[("plot_card", json!(starting_plot))],
        );
        let shadow_pool = self.board.shadow_pool;
        self.log(
            ActorRef::System,
            "setup.set_influence_in_shadow_pool",
            &[("count", json!(shadow_pool))],
        );
        let init = data
            .plot(starting_plot)?
            .influence
            .as_ref()
            .map(|i| i.init.clone())
            .unwrap_or_default();
        for (location, count) in init {
            tracing::trace!(%location, count, "initial influence");
            let params = [
                ("count", json!(count)),
                ("location", json!(data.locations.name(&location))),
            ];
            self.log(ActorRef::System, "setup.set_influence_in_location", &params);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::{data, rng};

    fn seats<'a>(codes: &[&'a str], open: bool) -> Vec<HeroSeat<'a>> {
        codes
            .iter()
            .enumerate()
            .map(|(i, code)| HeroSeat {
                code,
                user_id: i64::try_from(i).unwrap() + 2,
                open,
            })
            .collect()
    }

    fn three_heroes() -> Vec<HeroSeat<'static>> {
        seats(&["argalad", "eleanor", "eometh"], false)
    }

    // Port of boards_controller_test.rb "should create a board" and board_test.rb.
    #[test]
    fn creates_a_board() {
        let game = new_game(1, &three_heroes(), data(), &mut rng()).unwrap();
        assert_eq!(game.heroes.len(), 3);
        assert!(game.sauron.active);
        assert_eq!(game.board.state, BoardState::SauronSetupScreen);
        assert_eq!(game.board.plots.len(), 1);
        assert_eq!(game.mobs.len(), 2);
        let plots = game.board.decks.plot.draw.len() + game.sauron.drawn_plot.len();
        assert_eq!(plots, 15);
        assert!(
            !game.sauron.drawn_plot.is_empty(),
            "§ 3: the starting plot draws"
        );
        assert!(game.board.influence_budget.tokens > 0);
        assert_eq!(game.board.decks.shadow.draw.len(), 24);
        assert!(
            game.heroes
                .iter()
                .all(|h| h.hand.is_empty() && !h.life_pool.is_empty())
        );
        assert!(game.heroes.iter().any(|h| h.items.horse));
    }

    // § 3: the characters placed match the Starting Quest kept by each hero, and each
    // hero receives its Advanced Quest.
    #[test]
    fn quest_setup_matches_the_hero_quest() {
        for seed in 0..20 {
            let mut rng = <StdRng as rand::SeedableRng>::seed_from_u64(seed);
            let game = new_game(1, &three_heroes(), data(), &mut rng).unwrap();
            for hero in &game.heroes {
                let quests = data().hero_quest(&hero.code).unwrap();
                let quest = hero.quest.unwrap();
                for action in quests.setup.get(&quest).into_iter().flatten() {
                    assert_eq!(
                        game.board.characters.get(&action.character),
                        Some(&action.location)
                    );
                }
                assert_eq!(hero.advanced_quest, Some(0));
            }
            let expected: usize = game
                .heroes
                .iter()
                .map(|h| {
                    let quests = data().hero_quest(&h.code).unwrap();
                    quests.setup.get(&h.quest.unwrap()).map_or(0, Vec::len)
                })
                .sum();
            assert_eq!(game.board.characters.len(), expected, "seed {seed}");
        }
    }

    // § 1 and § 3: one to three different heroes, Argalad included.
    #[test]
    fn one_to_three_different_heroes() {
        let refused =
            |codes: &[&str]| new_game(1, &seats(codes, false), data(), &mut rng()).is_err();
        assert!(refused(&[]));
        assert!(refused(&["argalad", "beravor", "eleanor", "thalin"]));
        assert!(refused(&["argalad", "argalad"]));
        let game = new_game(1, &seats(&["argalad"], false), data(), &mut rng()).unwrap();
        assert_eq!(game.heroes[0].location, "woodland_realm");
    }

    #[test]
    fn players_take_open_seats_then_the_game_starts() {
        let mut game =
            new_game(1, &seats(&["thalin", "beravor"], true), data(), &mut rng()).unwrap();
        assert_eq!(game.board.state, BoardState::WaitingForPlayers);
        assert!(!game.sauron.active);
        assert!(!game.join_hero_seat("thalin", 9).unwrap());
        assert!(game.join_hero_seat("thalin", 9).is_err());
        assert!(game.join_hero_seat("beravor", 10).unwrap());
        assert_eq!(game.board.state, BoardState::SauronSetupScreen);
        assert!(game.sauron.active);
        assert_eq!(game.heroes[0].user_id, 9);
    }
}
