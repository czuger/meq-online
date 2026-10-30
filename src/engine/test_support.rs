//! Factories shared by engine tests, mirroring the Rails FactoryBot definitions.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::engine::board_state::BoardState;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::game_data::monster_pool_color::MonsterPoolColor;
use crate::model::board::{Board, Decks, Pile, StoryMarkers};
use crate::model::game::Game;
use crate::model::game_options::GameOptions;
use crate::model::hero::Hero;
use crate::model::mob::Mob;
use crate::model::sauron::Sauron;
use crate::model::turn_effects::TurnEffects;

pub fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(|| {
        GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    })
}

pub fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

/// test/factories/heros.rb
pub fn hero(code: &str) -> Hero {
    let deck = data().hero(code).unwrap().starting_deck.clone();
    Hero {
        id: Some(1),
        user_id: 1,
        code: code.to_owned(),
        location: "the_shire".to_owned(),
        fortitude: 5,
        strength: 5,
        agility: 3,
        wisdom: 2,
        favor: 0,
        quest: Some(1),
        advanced_quest: Some(0),
        playing_order: 0,
        turn: 1,
        active: true,
        turn_finished: false,
        damages_taken_this_turn: 0,
        temporary_damages: 0,
        cards_drawn_this_round: 0,
        life_pool: deck.clone(),
        hand: deck,
        rest_pool: vec![],
        damage_pool: vec![],
        items: Default::default(),
        levels: Default::default(),
        turn_record: Default::default(),
        corruption_cards: vec![],
    }
}

/// test/factories/monsters.rb and minion.rb: `(deck, fortitude, strength, life, kind)`.
pub fn mob(code: &str) -> Mob {
    let (deck, fortitude, strength, life, kind) = match code {
        "agent" => ("zealot", 5, 3, 3, MobKind::Monster),
        "orc" => ("zealot", 5, 4, 3, MobKind::Monster),
        "southron" => ("ravager", 6, 3, 4, MobKind::Monster),
        "snaga" => ("zealot", 6, 3, 3, MobKind::Monster),
        "huorn" => ("behemoth", 6, 7, 7, MobKind::Monster),
        "cave_troll" => ("behemoth", 8, 8, 8, MobKind::Monster),
        "mouth_of_sauron" => ("zealot", 6, 6, 9, MobKind::Minion),
        "black_serpent" => ("ravager", 9, 8, 11, MobKind::Minion),
        other => panic!("no factory for {other}"),
    };
    Mob {
        id: Some(1),
        kind,
        code: code.to_owned(),
        name: code.to_owned(),
        location: "the_nindale".to_owned(),
        pool: (kind == MobKind::Monster).then_some(MonsterPoolColor::Orange),
        fortitude,
        strength,
        life,
        max_life: life,
        attack_deck: deck.to_owned(),
        hand: vec![],
        damages_taken_this_turn: 0,
        commanded: false,
    }
}

/// test/factories/boards.rb, with one hero and Sauron.
pub fn game() -> Game {
    let init = data().plot(0).unwrap().influence.clone().unwrap();
    let pools = BTreeMap::from([
        (
            MonsterPoolColor::Orange,
            codes(&[
                "orc", "nothing", "orc", "snaga", "snaga", "nothing", "snaga", "nothing", "snaga",
                "nothing",
            ]),
        ),
        (
            MonsterPoolColor::Purple,
            codes(&[
                "nothing",
                "nothing",
                "nothing",
                "orc",
                "nothing",
                "snaga",
                "orc",
                "snaga",
                "cave_troll",
                "snaga",
            ]),
        ),
        (
            MonsterPoolColor::DarkBlue,
            codes(&[
                "southron", "nothing", "nothing", "nothing", "southron", "snaga", "orc", "nothing",
                "snaga", "southron",
            ]),
        ),
        (
            MonsterPoolColor::Brown,
            codes(&[
                "nothing",
                "cave_troll",
                "cave_troll",
                "orc",
                "orc",
                "nothing",
                "nothing",
                "cave_troll",
                "nothing",
                "cave_troll",
            ]),
        ),
        (
            MonsterPoolColor::DarkGreen,
            codes(&[
                "nothing", "nothing", "huorn", "nothing", "huorn", "snaga", "huorn", "snaga",
                "nothing", "orc",
            ]),
        ),
    ]);
    let board = Board {
        id: Some(1),
        state: BoardState::Created,
        max_heroes: 2,
        waiting_heroes: vec![],
        turn: 1,
        current_hero_id: Some(1),
        winner: None,
        story: StoryMarkers::default(),
        shadow_pool: init.shadow_pool,
        influence: init.init,
        characters: BTreeMap::new(),
        favors: vec![],
        plots: BTreeMap::new(),
        sauron_actions: vec![],
        sauron_actions_count: 0,
        current_action: None,
        decks: Decks {
            plot: Pile {
                draw: (3..=17).collect(),
                discard: vec![],
            },
            shadow: Pile {
                draw: (0..=23).collect(),
                discard: vec![],
            },
            event: Pile {
                draw: (0..=13).collect(),
                discard: vec![],
            },
            event_ii: Pile {
                draw: (0..=13).collect(),
                discard: vec![],
            },
            event_iii: Pile {
                draw: (0..=13).collect(),
                discard: vec![],
            },
            skill: Pile {
                draw: (100..=115).collect(),
                discard: vec![],
            },
            corruption: Pile::default(),
            peril: Pile {
                draw: (0..=14).collect(),
                discard: vec![],
            },
        },
        monster_pools: pools,
        monster_discards: BTreeMap::new(),
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
    };
    let sauron = Sauron {
        id: Some(1),
        user_id: 1,
        active: false,
        plot_hand: vec![0, 1, 2],
        shadow_hand: vec![0, 1, 2],
        drawn_plot: vec![0, 1, 2],
        drawn_shadow: vec![0, 1, 2],
    };
    Game {
        board,
        sauron,
        heroes: vec![hero("argalad")],
        mobs: vec![],
        combat: None,
        new_logs: vec![],
    }
}

fn codes(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}
