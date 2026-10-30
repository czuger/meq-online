//! Story track: markers, stages and dominance.

use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::plot::StoryType;
use crate::model::actor_ref::ActorRef;
use crate::model::board::StoryMarkers;
use crate::model::game::Game;

/// The Finale space, at the end of the story track.
pub const FINAL_SPACE: i32 = 18;

/// The Shadow Falls space: the Finale also begins when Sauron's three markers reached it.
const SHADOW_FALLS_SPACE: i32 = 10;

/// Spaces in each stage of the story track.
const STAGE_SPACES: i32 = 6;

/// Spaces the heroes' marker advances at each Story step (§ 5.3).
pub(crate) const HEROES_STORY_ADVANCE: i32 = 2;

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
    /// Spaces left before the heroes' marker reaches the Finale space.
    pub heroes_to_final: i32,
    /// Spaces left before Sauron's closest marker reaches the Finale space.
    pub sauron_to_final: i32,
    /// Spaces Sauron's markers still need, in all, to reach The Shadow Falls.
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
        stage_of(highest)
    }

    /// Position of one of Sauron's markers.
    pub fn sauron_marker(&self, track: StoryType) -> i32 {
        match track {
            StoryType::Ring => self.ring,
            StoryType::Conquest => self.conquest,
            StoryType::Corruption => self.corruption,
        }
    }

    /// Distances to the end of the game.
    pub fn data(&self) -> StoryData {
        let heroes_to_final = FINAL_SPACE - self.heroes;
        let sauron_to_final = FINAL_SPACE - self.ring.max(self.conquest).max(self.corruption);
        let sauron_to_shadowfall = [self.ring, self.conquest, self.corruption]
            .into_iter()
            .map(|position| (SHADOW_FALLS_SPACE - position).max(0))
            .sum();
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

    /// Advances the lowest of Sauron's markers by `spaces` (§ 6.1, § 8.9).
    ///
    /// # Returns
    /// `false` when several markers are tied: nothing moves and the hero chooses.
    #[tracing::instrument(level = "debug", skip_all, fields(spaces))]
    pub fn advance_lowest(&mut self, spaces: i32) -> bool {
        match self.lowest_sauron_markers().as_slice() {
            [track] => {
                self.advance(*track, spaces);
                true
            }
            _ => false,
        }
    }

    /// Advances one of Sauron's markers.
    #[tracing::instrument(level = "debug", skip_all, fields(?track, spaces))]
    pub fn advance(&mut self, track: StoryType, spaces: i32) {
        match track {
            StoryType::Ring => self.ring += spaces,
            StoryType::Conquest => self.conquest += spaces,
            StoryType::Corruption => self.corruption += spaces,
        }
    }
}

impl Game {
    /// End-of-round story advance: heroes +2 (less after "Grievous Loss"), then each
    /// plot in play advances its track.
    ///
    /// # Errors
    /// Fails when a plot in play is unknown.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn advance_stories_markers(&mut self, data: &GameData) -> Result<(), EngineError> {
        // "Grievous Loss" holds the green marker back.
        let malus = std::mem::take(&mut self.board.turn_effects.heroes_story_malus);
        let spaces = (HEROES_STORY_ADVANCE - malus).max(0);
        self.board.story.heroes += spaces;
        self.log(
            ActorRef::System,
            "story.advance",
            &[("marker_name", json!("heroes")), ("count", json!(spaces))],
        );

        let plots: Vec<u32> = self.board.plots.values().copied().collect();
        for card in plots {
            tracing::trace!(card, "plot advances story");
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
        self.advance_current_event_story(data)
    }
}

/// Stage (1 to 3) of a position on the story track.
pub fn stage_of(position: i32) -> i32 {
    ((position - 1).max(0) / STAGE_SPACES + 1).min(3)
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
        assert!(m.advance_lowest(1));
        assert_eq!(m, markers(0, 2, 2, 3));
    }

    // § 6.1: on a tie the hero chooses which marker advances.
    #[test]
    fn tied_markers_need_a_choice() {
        let mut m = markers(0, 1, 1, 3);
        assert!(!m.advance_lowest(1));
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
