//! A small AI playing the computer hero seats: it tries a few moves per screen,
//! in random order where a real strategy would get complex.

mod candidates;
mod hero_ai;
pub mod hero_move;
pub mod waiting_hero;

#[cfg(test)]
mod tests;
