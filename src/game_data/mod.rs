//! Static game data read from `data/*.yaml`.

pub mod attack_deck;
pub mod character;
pub mod code_map;
pub mod combat_card_def;
pub mod corruption_card;
pub mod event;
// Files are named after their primary type, even when it matches the module name.
#[allow(clippy::module_inception)]
pub mod game_data;
pub mod game_data_error;
pub mod hero_def;
pub mod hero_quests;
pub mod location;
pub mod location_paths;
pub mod locations;
pub mod mob_def;
pub mod monster_pool_color;
pub mod peril;
pub mod plot;
pub mod terrain;
pub mod yaml_loader;

pub use crate::game_data::game_data::{GameData, event_set};
pub use crate::game_data::game_data_error::GameDataError;
