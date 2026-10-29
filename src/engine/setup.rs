//! Creation of a new game.

use std::collections::BTreeMap;

use rand::RngExt;
use rand::rngs::StdRng;
use rand::seq::{IndexedRandom, SliceRandom};
use serde_json::json;

use crate::engine::board_state::{BoardEvent, BoardState};
use crate::engine::engine_error::EngineError;
use crate::engine::events::starting_event_deck;
use crate::game_data::GameData;
use crate::game_data::GameDataError;
use crate::game_data::hero_quests::QuestSetupKind;
use crate::game_data::monster_pool_color::MonsterPoolColor;
use crate::model::actor_ref::ActorRef;
use crate::model::board::{Board, Decks, Pile, StoryMarkers};
use crate::model::game::Game;
use crate::model::hero::{Hero, HeroItems};
use crate::model::sauron::Sauron;

/// Heroes seated at every new game, each controlled by a computer seat.
pub const STARTING_HEROES: [&str; 4] = ["beravor", "eleanor", "thalin", "eometh"];

/// Plot cards left out of the deck (their effects aren't implemented).
const EXCLUDED_PLOTS: [u32; 5] = [10, 11, 14, 15, 17];

/// Shadow cards left out of the deck (played during hero movement, not implemented).
const EXCLUDED_SHADOW_CARDS: [u32; 2] = [1, 21];

/// Minions on the map from the start: `(mob, location)`.
const STARTING_MINIONS: [(&str, &str); 2] = [
    ("mouth_of_sauron", "dol_guldur"),
    ("black_serpent", "near_harad"),
];

/// Builds a new game: Sauron for `sauron_user_id`, the [`STARTING_HEROES`] for the
/// computer seats, the starting plot, decks, minions and quests.
///
/// # Arguments
/// * `sauron_user_id` - user playing Sauron (the creator).
/// * `hero_user_ids` - user of each starting hero, in [`STARTING_HEROES`] order.
/// * `data` - static data.
/// * `rng` - random source.
///
/// # Errors
/// Fails when the static data misses a starting hero, plot or minion.
pub fn new_game(
    sauron_user_id: i64,
    hero_user_ids: [i64; 4],
    data: &GameData,
    rng: &mut StdRng,
) -> Result<Game, EngineError> {
    let starting_plot = rng.random_range(0..=2);
    let board = new_board(starting_plot, data, rng)?;
    // The game opens on Sauron's setup screen, so Sauron plays first.
    let sauron = Sauron {
        user_id: sauron_user_id,
        active: true,
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

    for (order, (code, user_id)) in STARTING_HEROES.iter().zip(hero_user_ids).enumerate() {
        let hero = new_hero(code, user_id, order, data, rng)?;
        game.apply_quest_setup(code, data, rng)?;
        game.heroes.push(hero);
    }
    game.fire(BoardEvent::NextToSauronSetupScreen)?;

    for (mob, location) in STARTING_MINIONS {
        game.create_mob(mob, location, None, data)?;
    }
    game.log_setup(starting_plot, data)?;
    game.board.plots.insert(1, starting_plot);
    Ok(game)
}

fn new_board(starting_plot: u32, data: &GameData, rng: &mut StdRng) -> Result<Board, EngineError> {
    let influence = data
        .plot(starting_plot)?
        .influence
        .clone()
        .ok_or_else(|| GameDataError::unknown("starting influence of plot", starting_plot))?;

    Ok(Board {
        id: None,
        state: BoardState::Created,
        max_heroes: 4,
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
        decks: shuffled_decks(data, rng),
        monster_pools: shuffled_monster_pools(data, rng),
        last_event_card: None,
        heroes_objective: None,
        sauron_objective: None,
    })
}

fn shuffled_decks(data: &GameData, rng: &mut StdRng) -> Decks {
    let mut plot: Vec<u32> = (3..=17).filter(|c| !EXCLUDED_PLOTS.contains(c)).collect();
    plot.shuffle(rng);
    let mut shadow: Vec<u32> = (0..=23)
        .filter(|c| !EXCLUDED_SHADOW_CARDS.contains(c))
        .collect();
    shadow.shuffle(rng);
    let mut corruption = data.corruption.deck.clone();
    corruption.shuffle(rng);
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
            draw: starting_event_deck(data, rng),
            discard: vec![],
        },
        corruption: Pile {
            draw: corruption,
            discard: vec![],
        },
    }
}

fn shuffled_monster_pools(
    data: &GameData,
    rng: &mut StdRng,
) -> BTreeMap<MonsterPoolColor, Vec<String>> {
    let mut pools = data.monster_pools.clone();
    for pool in pools.values_mut() {
        pool.shuffle(rng);
    }
    pools
}

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
    let quest = data.hero_quest(code)?.starting_quests.choose(rng).copied();
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
        playing_order: i32::try_from(order).unwrap_or(i32::MAX),
        turn: 1,
        active: false,
        turn_finished: false,
        damages_taken_this_turn: 0,
        temporary_damages: 0,
        corruption_discarded_this_turn: false,
        life_pool,
        hand: vec![],
        rest_pool: vec![],
        damage_pool: vec![],
        items: HeroItems {
            horse: code == "eometh",
        },
        used_powers: Default::default(),
        corruption_cards: vec![],
    })
}

impl Game {
    /// Places the characters of a hero's starting quest.
    ///
    /// The quest is drawn again here, independently of the quest stored on the hero,
    /// as in the original engine.
    fn apply_quest_setup(
        &mut self,
        hero: &str,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let quests = data.hero_quest(hero)?;
        let Some(quest) = quests.starting_quests.choose(rng) else {
            return Ok(());
        };
        for action in quests.setup.get(quest).into_iter().flatten() {
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

    // Port of boards_controller_test.rb "should create a board" and board_test.rb.
    #[test]
    fn creates_a_board() {
        let game = new_game(1, [2, 3, 4, 5], data(), &mut rng()).unwrap();
        assert_eq!(game.heroes.len(), 4);
        assert_eq!(game.board.state, BoardState::SauronSetupScreen);
        assert_eq!(game.board.plots.len(), 1);
        assert_eq!(game.mobs.len(), 2);
        assert_eq!(game.board.decks.plot.draw.len(), 10);
        assert_eq!(game.board.decks.shadow.draw.len(), 22);
        assert!(
            game.heroes
                .iter()
                .all(|h| h.hand.is_empty() && !h.life_pool.is_empty())
        );
        assert!(game.heroes.iter().any(|h| h.items.horse));
    }
}
