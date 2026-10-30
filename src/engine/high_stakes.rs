//! The High stakes stage III optional rule (§ 10.4): in stage III, Sauron picks the
//! marker a defeated hero advances while the heroes are dominant, and the heroes'
//! marker advances for each minion they defeat while Sauron is.

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::choices::{parse_target, target, target_option};
use crate::engine::engine_error::EngineError;
use crate::engine::story::Dominance;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::game_data::plot::StoryType;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// Option code of Sauron picking the marker a defeated hero advances.
pub const ADVANCE_MARKER: &str = "advance_marker";

/// Story stage where the rule applies.
const HIGH_STAKES_STAGE: i32 = 3;

/// Sauron's markers, as choice targets.
const SAURON_MARKERS: [StoryType; 3] =
    [StoryType::Ring, StoryType::Conquest, StoryType::Corruption];

impl Game {
    /// Whether Sauron picks the marker the defeated hero advances: the rule is in use,
    /// the story is in stage III, the heroes are dominant and Sauron didn't pick yet.
    ///
    /// # Errors
    /// Fails when an active plot is unknown.
    pub fn sauron_picks_defeat_marker(&self, data: &GameData) -> Result<bool, EngineError> {
        if self.board.turn_effects.defeat_marker.is_some() {
            return Ok(false);
        }
        Ok(self.high_stakes_dominance(data)? == Some(Dominance::Heroes))
    }

    /// Asks Sauron which of his markers the defeated hero advances.
    ///
    /// # Errors
    /// Transition errors.
    pub fn ask_defeat_marker(&mut self, hero_id: i64) -> Result<(), EngineError> {
        let targets = SAURON_MARKERS
            .iter()
            .map(|marker| target("marker", marker_code(*marker)))
            .collect();
        self.ask(PendingChoice {
            chooser: Chooser::Sauron,
            source: ChoiceSource::DefeatMarker { hero_id },
            options: vec![target_option(ADVANCE_MARKER, targets, 1, 1)],
            // The defeat goes on from the answer itself and ends the turn.
            resume: Resume::EndTurn,
        })
    }

    /// Keeps Sauron's marker until the defeat resolves, then goes on with the defeat.
    ///
    /// # Errors
    /// [`EngineError::Refused`] for an unknown marker; errors of the defeat.
    pub fn apply_defeat_marker(
        &mut self,
        hero_id: i64,
        targets: &[String],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let marker = targets
            .first()
            .and_then(|code| {
                let (_, values) = parse_target(code);
                let value = values.first().copied().unwrap_or_default();
                SAURON_MARKERS
                    .into_iter()
                    .find(|m| marker_code(*m) == value)
            })
            .ok_or_else(|| EngineError::Refused("choose one of Sauron's markers".to_owned()))?;
        self.board.turn_effects.defeat_marker = Some(marker);
        self.start_hero_defeat(hero_id, data, rng).map(|_| ())
    }

    /// A hero defeated a mob in combat: with Sauron dominant in stage III, defeating a
    /// minion advances the heroes' marker one space.
    ///
    /// # Errors
    /// Lookup errors.
    pub fn high_stakes_mob_defeated(
        &mut self,
        hero_id: i64,
        mob_id: i64,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let index = self.mob_index(mob_id)?;
        let minion = self
            .mobs
            .get(index)
            .is_some_and(|m| m.kind == MobKind::Minion);
        if !minion || self.high_stakes_dominance(data)? != Some(Dominance::Sauron) {
            return Ok(());
        }
        self.board.story.heroes += 1;
        self.log(
            ActorRef::Hero(hero_id),
            "story.advance",
            &[("marker_name", json!("heroes")), ("count", json!(1))],
        );
        Ok(())
    }

    /// The dominant side when High stakes stage III applies, `None` otherwise.
    fn high_stakes_dominance(&self, data: &GameData) -> Result<Option<Dominance>, EngineError> {
        if !self.board.options.high_stakes || self.board.story.stage() != HIGH_STAKES_STAGE {
            return Ok(None);
        }
        self.dominance(data).map(Some)
    }
}

/// Code of a marker in choice targets.
fn marker_code(marker: StoryType) -> &'static str {
    match marker {
        StoryType::Ring => "ring",
        StoryType::Conquest => "conquest",
        StoryType::Corruption => "corruption",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, mob, rng};
    use crate::model::board::StoryMarkers;

    /// A hero in Bree about to be defeated in stage III.
    fn stage_iii(story: StoryMarkers) -> Game {
        let mut g = game();
        g.board.options.high_stakes = true;
        g.board.state = BoardState::PlayCombatCardScreenBoardCombats;
        g.board.story = story;
        g.heroes[0].location = "bree".to_owned();
        g.heroes[0].turn = 2;
        g
    }

    /// Heroes 4 spaces from the Finale, Sauron 5: the heroes are dominant.
    fn heroes_dominant() -> StoryMarkers {
        StoryMarkers {
            heroes: 14,
            ring: 13,
            conquest: 0,
            corruption: 0,
        }
    }

    /// Heroes 6 spaces from the Finale, Sauron 5: Sauron is dominant.
    fn sauron_dominant() -> StoryMarkers {
        StoryMarkers {
            heroes: 12,
            ring: 13,
            conquest: 0,
            corruption: 0,
        }
    }

    // § 10.4: heroes dominant in stage III, Sauron picks the marker a defeated hero
    // advances, not necessarily the lowest.
    #[test]
    fn sauron_picks_the_defeat_marker() {
        let mut g = stage_iii(heroes_dominant());
        let outcome = g.start_hero_defeat(1, data(), &mut rng()).unwrap();
        assert_eq!(outcome, None);
        assert_eq!(g.board.state, BoardState::ChoiceScreenBoardChoices);
        let choice = g.board.pending_choice.as_ref().unwrap();
        assert_eq!(choice.chooser, Chooser::Sauron);
        assert!(g.sauron.active);

        let targets = ["marker:ring".to_owned()];
        g.answer_choice(ADVANCE_MARKER, &targets, data(), &mut rng())
            .unwrap();
        assert_eq!(g.board.story.ring, 14);
        assert_eq!(g.board.story.conquest, 0);
        assert_eq!(g.board.turn_effects.defeat_marker, None);
        assert_eq!(g.heroes[0].location, "fornost");
    }

    // § 10.4: without the option, or outside stage III, the lowest marker advances.
    #[test]
    fn rule_applies_only_in_stage_iii_with_the_option() {
        let mut g = stage_iii(heroes_dominant());
        g.board.options.high_stakes = false;
        assert!(!g.sauron_picks_defeat_marker(data()).unwrap());

        let mut g = stage_iii(StoryMarkers {
            heroes: 10,
            ring: 11,
            conquest: 0,
            corruption: 1,
        });
        assert!(!g.sauron_picks_defeat_marker(data()).unwrap());
        g.start_hero_defeat(1, data(), &mut rng()).unwrap();
        assert_eq!(g.board.story.ring, 11);
        assert_eq!(g.board.story.conquest, 1);
    }

    // § 10.4: Sauron dominant in stage III, the heroes' marker advances one space for
    // each minion they defeat, not for monsters.
    #[test]
    fn defeated_minion_advances_the_heroes() {
        let mut g = stage_iii(sauron_dominant());
        g.mobs.push(mob("mouth_of_sauron"));
        g.high_stakes_mob_defeated(1, 1, data()).unwrap();
        assert_eq!(g.board.story.heroes, 13);

        g.mobs = vec![mob("orc")];
        g.high_stakes_mob_defeated(1, 1, data()).unwrap();
        assert_eq!(g.board.story.heroes, 13);

        g.board.story = heroes_dominant();
        g.mobs = vec![mob("mouth_of_sauron")];
        g.high_stakes_mob_defeated(1, 1, data()).unwrap();
        assert_eq!(g.board.story.heroes, 14);
    }
}
