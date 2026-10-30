//! Story track: markers, stages and the end of the game.

use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::plot::StoryType;
use crate::model::actor_ref::ActorRef;
use crate::model::board::{StoryMarkers, Winner};
use crate::model::game::Game;

/// Space of the story track where a side wins.
const FINAL_SPACE: i32 = 18;

/// Side closer to its goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dominance {
    /// Heroes are closer.
    Heroes,
    /// Sauron is closer.
    Sauron,
    /// Both are as close.
    Nobody,
}

/// Distances to the end of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoryData {
    /// Spaces left before the heroes win.
    pub heroes_to_final: i32,
    /// Spaces left before Sauron's best marker wins.
    pub sauron_to_final: i32,
    /// Points left before Shadowfall.
    pub sauron_to_shadowfall: i32,
    /// Who is ahead.
    pub dominance: Dominance,
}

impl StoryMarkers {
    /// Story stage (1 to 3), driven by the most advanced marker.
    pub fn stage(&self) -> i32 {
        let highest = self
            .heroes
            .max(self.ring)
            .max(self.conquest)
            .max(self.corruption);
        (highest - 1).max(0) / 6 + 1
    }

    /// Distances to the end of the game.
    pub fn data(&self) -> StoryData {
        let heroes_to_final = FINAL_SPACE - self.heroes;
        let sauron_to_final = FINAL_SPACE - self.ring.max(self.conquest).max(self.corruption);
        let shadowfall_points = self.ring.min(10) + self.conquest.min(10) + self.corruption.min(10);
        let sauron_to_shadowfall = 30 - shadowfall_points;
        let sauron_best = sauron_to_final.min(sauron_to_shadowfall);
        let dominance = match heroes_to_final.cmp(&sauron_best) {
            std::cmp::Ordering::Less => Dominance::Heroes,
            std::cmp::Ordering::Greater => Dominance::Sauron,
            std::cmp::Ordering::Equal => Dominance::Nobody,
        };
        StoryData {
            heroes_to_final,
            sauron_to_final,
            sauron_to_shadowfall,
            dominance,
        }
    }

    /// Sauron's markers tied for the lowest (leftmost) position.
    pub fn lowest_sauron_markers(&self) -> Vec<StoryType> {
        let min = self.ring.min(self.conquest).min(self.corruption);
        [
            (StoryType::Ring, self.ring),
            (StoryType::Conquest, self.conquest),
            (StoryType::Corruption, self.corruption),
        ]
        .into_iter()
        .filter(|(_, position)| *position == min)
        .map(|(track, _)| track)
        .collect()
    }

    /// Advances the lowest of Sauron's markers by one space (§ 6.1, § 8.9).
    ///
    /// # Returns
    /// `false` when several markers are tied: nothing moves and the hero chooses.
    pub fn advance_lowest(&mut self) -> bool {
        match self.lowest_sauron_markers().as_slice() {
            [track] => {
                self.advance(*track, 1);
                true
            }
            _ => false,
        }
    }

    /// Advances one of Sauron's markers.
    pub fn advance(&mut self, track: StoryType, spaces: i32) {
        match track {
            StoryType::Ring => self.ring += spaces,
            StoryType::Conquest => self.conquest += spaces,
            StoryType::Corruption => self.corruption += spaces,
        }
    }
}

impl Game {
    /// End-of-round story advance: heroes +2, then each plot in play advances its track.
    ///
    /// # Errors
    /// Fails when a plot in play is unknown or the game can't be finished now.
    pub fn advance_stories_markers(&mut self, data: &GameData) -> Result<(), EngineError> {
        self.board.story.heroes += 2;
        self.log(
            ActorRef::System,
            "story.advance",
            &[("marker_name", json!("heroes")), ("count", json!(2))],
        );

        let plots: Vec<u32> = self.board.plots.values().copied().collect();
        for card in plots {
            let story = data.plot(card)?.story;
            self.board.story.advance(story.story_type, story.advance);
            self.log(
                ActorRef::System,
                "story.advance",
                &[
                    ("marker_name", json!(story.story_type)),
                    ("count", json!(story.advance)),
                ],
            );
        }
        self.advance_current_event_story(data)?;
        self.check_game_end()
    }

    /// Ends the game when a side reached its goal.
    ///
    /// # Errors
    /// [`EngineError::InvalidTransition`] when the game ends outside the end of a hero turn.
    pub fn check_game_end(&mut self) -> Result<(), EngineError> {
        let story = self.board.story.data();
        let heroes_won = story.heroes_to_final <= 0;
        let sauron_won = story.sauron_to_final <= 0 || story.sauron_to_shadowfall <= 0;
        let winner = match (heroes_won, sauron_won) {
            (true, true) => Winner::Draw,
            (false, true) => Winner::Sauron,
            (true, false) => Winner::Heroes,
            (false, false) => return Ok(()),
        };
        self.board.winner = Some(winner);
        self.fire(BoardEvent::FinishGame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn markers(heroes: i32, ring: i32, conquest: i32, corruption: i32) -> StoryMarkers {
        StoryMarkers {
            heroes,
            ring,
            conquest,
            corruption,
        }
    }

    #[test]
    fn stage_grows_every_six_spaces() {
        assert_eq!(markers(0, 0, 0, 0).stage(), 1);
        assert_eq!(markers(6, 0, 0, 0).stage(), 1);
        assert_eq!(markers(7, 0, 0, 0).stage(), 2);
        assert_eq!(markers(0, 13, 0, 0).stage(), 3);
    }

    #[test]
    fn lowest_marker_advances() {
        let mut m = markers(0, 2, 1, 3);
        assert!(m.advance_lowest());
        assert_eq!(m, markers(0, 2, 2, 3));
    }

    // § 6.1: on a tie the hero chooses which marker advances.
    #[test]
    fn tied_markers_need_a_choice() {
        let mut m = markers(0, 1, 1, 3);
        assert!(!m.advance_lowest());
        assert_eq!(m, markers(0, 1, 1, 3));
        assert_eq!(
            m.lowest_sauron_markers(),
            vec![StoryType::Ring, StoryType::Conquest]
        );
    }

    #[test]
    fn dominance() {
        assert_eq!(markers(10, 0, 0, 0).data().dominance, Dominance::Heroes);
        assert_eq!(markers(0, 10, 0, 0).data().dominance, Dominance::Sauron);
    }
}
