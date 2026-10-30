//! Game rules, free of HTTP and database code.

pub mod activation;
pub mod ai;
pub mod board_state;
pub mod characters;
pub mod choices;
pub mod combat;
pub mod commands;
pub mod deck;
pub mod defeat;
pub mod dominance;
pub mod engine_error;
pub mod events;
pub mod exploration;
pub mod finale;
pub mod game_access;
pub mod hero;
pub mod high_stakes;
pub mod influence;
pub mod influence_paths;
pub mod missions;
pub mod mobs;
pub mod movement;
pub mod perils;
pub mod plot_locations;
pub mod plots;
pub mod rest;
pub mod route;
pub mod sauron_actions;
pub mod sauron_turn;
pub mod setup;
pub mod shadows;
pub mod story;
pub mod threats;
pub mod tokens;
pub mod turn;

#[cfg(test)]
pub(crate) mod test_support;

pub use crate::engine::engine_error::EngineError;

#[cfg(test)]
mod flow_tests;
