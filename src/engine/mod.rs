//! Game rules, free of HTTP and database code.

pub mod activation;
pub mod board_state;
pub mod combat;
pub mod deck;
pub mod encounters;
pub mod engine_error;
pub mod events;
pub mod exploration;
pub mod game_access;
pub mod hero;
pub mod influence_paths;
pub mod mobs;
pub mod movement;
pub mod plots;
pub mod rest;
pub mod route;
pub mod sauron_turn;
pub mod setup;
pub mod story;
pub mod tokens;
pub mod turn;

#[cfg(test)]
mod test_support;

pub use crate::engine::engine_error::EngineError;

#[cfg(test)]
mod flow_tests;
