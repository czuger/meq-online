//! The dominant side (§ 5.5), counted in Story steps with the Advanced dominance
//! optional rule (§ 10.4).

use crate::engine::engine_error::EngineError;
use crate::engine::story::{Dominance, FINAL_SPACE, HEROES_STORY_ADVANCE};
use crate::game_data::GameData;
use crate::game_data::plot::PlotStory;
use crate::model::board::StoryMarkers;
use crate::model::game::Game;

impl Game {
    /// The dominant side: by distances to the end of the game (§ 5.5), or by the Story
    /// steps each side needs to reach the Finale with Advanced dominance (§ 10.4).
    ///
    /// # Errors
    /// Fails when an active plot is unknown.
    pub fn dominance(&self, data: &GameData) -> Result<Dominance, EngineError> {
        if !self.board.options.advanced_dominance {
            return Ok(self.board.story.data().dominance);
        }
        let plots = self
            .board
            .plots
            .values()
            .map(|card| data.plot(*card).map(|plot| plot.story))
            .collect::<Result<Vec<PlotStory>, _>>()?;
        Ok(advanced_dominance(self.board.story, &plots))
    }
}

/// Advanced dominance: the side reaching the Finale in fewer simulated Story steps,
/// Sauron's active `plots` staying in play. Sauron never gets there without plots.
pub fn advanced_dominance(story: StoryMarkers, plots: &[PlotStory]) -> Dominance {
    let heroes = heroes_story_steps(story);
    match sauron_story_steps(story, plots) {
        Some(sauron) if sauron < heroes => Dominance::Sauron,
        Some(sauron) if sauron == heroes => Dominance::Nobody,
        _ => Dominance::Heroes,
    }
}

/// Story steps the heroes' marker needs to reach the Finale space.
fn heroes_story_steps(story: StoryMarkers) -> i32 {
    let distance = story.data().heroes_to_final.max(0);
    (distance + HEROES_STORY_ADVANCE - 1) / HEROES_STORY_ADVANCE
}

/// Story steps Sauron's markers need to reach the Finale space or, all three, The
/// Shadow Falls; `None` when his plots never get there.
fn sauron_story_steps(mut story: StoryMarkers, plots: &[PlotStory]) -> Option<i32> {
    // Each step with a plot advances a marker at least one space: past this many
    // steps, one marker is on the Finale space.
    for steps in 0..=FINAL_SPACE {
        let data = story.data();
        if data.sauron_to_final <= 0 || data.sauron_to_shadowfall <= 0 {
            return Some(steps);
        }
        for plot in plots {
            story.advance(plot.story_type, plot.advance);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::test_support::{data, game};
    use crate::game_data::plot::StoryType;

    fn markers(heroes: i32, ring: i32, conquest: i32, corruption: i32) -> StoryMarkers {
        StoryMarkers {
            heroes,
            ring,
            conquest,
            corruption,
        }
    }

    fn plot(story_type: StoryType, advance: i32) -> PlotStory {
        PlotStory {
            story_type,
            advance,
        }
    }

    // § 10.4 example: heroes 5 spaces away need 3 Story steps; black and red advancing
    // 2 spaces each need 4: the heroes are dominant.
    #[test]
    fn rules_example_heroes_dominant() {
        let story = markers(13, 10, 3, 3);
        let plots = [plot(StoryType::Corruption, 2), plot(StoryType::Conquest, 2)];
        assert_eq!(heroes_story_steps(story), 3);
        assert_eq!(sauron_story_steps(story, &plots), Some(4));
        assert_eq!(advanced_dominance(story, &plots), Dominance::Heroes);
    }

    // § 10.4: fast plots make Sauron dominant although his marker is farther away.
    #[test]
    fn fast_plots_give_sauron_dominance() {
        let story = markers(10, 0, 0, 9);
        assert_eq!(story.data().dominance, Dominance::Heroes);
        let plots = [plot(StoryType::Corruption, 3)];
        assert_eq!(advanced_dominance(story, &plots), Dominance::Sauron);
    }

    // § 10.4: as many Story steps on both sides, nobody is dominant; without plots
    // Sauron never reaches the Finale.
    #[test]
    fn ties_and_no_plots() {
        let story = markers(12, 0, 0, 10);
        assert_eq!(
            advanced_dominance(story, &[plot(StoryType::Corruption, 3)]),
            Dominance::Nobody
        );
        assert_eq!(advanced_dominance(story, &[]), Dominance::Heroes);
    }

    // § 10.4: the option switches the board from distances to Story steps; plot 8
    // advances the corruption marker 3 spaces.
    #[test]
    fn board_option_selects_the_rule() {
        let mut g = game();
        g.board.story = markers(10, 0, 0, 9);
        g.board.plots.insert(1, 8);
        assert_eq!(g.dominance(data()).unwrap(), Dominance::Heroes);
        g.board.options.advanced_dominance = true;
        assert_eq!(g.dominance(data()).unwrap(), Dominance::Sauron);
    }
}
