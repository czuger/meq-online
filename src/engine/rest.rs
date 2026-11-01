//! Start of a hero's turn: drawing, resting and healing.

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::engine::shadows::shadow_window::ShadowWindow;
use crate::engine::threats::Threat;
use crate::game_data::GameData;
use crate::game_data::corruption_card::CorruptionCode;
use crate::game_data::plot::StoryType;
use crate::game_data::shadow::ShadowTiming;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::Resume;
use crate::model::game::Game;
use crate::model::hero::TurnRecord;
use crate::model::turn_step::{StepSkip, TurnStep};

/// Hero whose ability lets him heal or train when resting outside a Haven.
const BERAVOR: &str = "beravor";

/// How a hero rests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rest {
    /// Heal as well (in a Haven, or anywhere for Beravor).
    pub heal: bool,
    /// Receive training (Beravor outside a Haven, instead of healing).
    pub train: bool,
    /// Corruption card to discard.
    pub discard: Option<CorruptionCode>,
}

/// Where a hero goes after resting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestOutcome {
    /// Sauron's lowest marker advanced; the hero moves.
    Movement,
    /// Several markers are tied: the hero must choose which one advances.
    ChooseStoryMarker,
    /// The hero keeps a Skill card on the choice screen first.
    Choice,
}

impl Game {
    /// The hero ends its draw; when no hero is drawing anymore, the current hero's
    /// turn starts with a check of its location.
    ///
    /// At the start of a round the first hero in playing order plays. Later in the
    /// round (after the next hero drew) that hero keeps playing.
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id), err(level = "debug"))]
    pub fn draw_cards_finished(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        self.log_draw_result(hero_id, data)?;
        self.hero_mut(hero_id)?.active = false;
        if self.heroes_active() {
            return Ok(());
        }
        if self.heroes.iter().any(|h| h.turn_finished) {
            self.activate_current_hero()?;
        } else {
            self.set_first_hero_to_play()?;
        }
        self.check_rest(data, rng)
    }

    /// Hero Draw step: the hero draws up to `count` cards. Only how many is public.
    ///
    /// # Returns
    /// Number of cards actually drawn.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(hero_id, count),
        err(level = "debug")
    )]
    pub fn draw_step(
        &mut self,
        hero_id: i64,
        count: usize,
        data: &GameData,
    ) -> Result<usize, EngineError> {
        let hero = self.hero_mut(hero_id)?;
        let drawn = hero.draw_step_cards(count, data);
        if drawn > 0 {
            let params = [
                ("name", json!(hero.code)),
                ("cards_drawn", json!(drawn)),
                ("lp_cards", json!(hero.life_pool.len())),
            ];
            self.log(ActorRef::Hero(hero_id), "draw_cards", &params);
        }
        Ok(drawn)
    }

    /// A hero ending his draw without a single card says so, and why when he couldn't.
    fn log_draw_result(&mut self, hero_id: i64, data: &GameData) -> Result<(), EngineError> {
        let hero = self.hero(hero_id)?;
        if hero.cards_drawn_this_round > 0 {
            return Ok(());
        }
        let name = json!(hero.code);
        let blocked = if hero.cards_left_to_draw(data) > 0 {
            None
        } else if hero.life_pool.is_empty() {
            Some(StepSkip::EmptyLifePool)
        } else {
            Some(StepSkip::HandFull)
        };
        let action = blocked.map_or("draw_cards.none", |_| "draw_cards.blocked");
        let reason = blocked.map_or(json!(null), |reason| json!(reason));
        let params = [("name", name), ("reason", reason)];
        self.log(ActorRef::Hero(hero_id), action, &params);
        Ok(())
    }

    /// The current hero's turn starts: Sauron may play a Shadow card (§ 5.9), then
    /// monsters and minions on its location ambush it (§ 6.4), then it may rest.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    fn check_rest(&mut self, data: &GameData, rng: &mut StdRng) -> Result<(), EngineError> {
        let hero_id = self.current_hero_id()?;
        self.log_hero_turn(hero_id)?;
        self.log_step(TurnStep::Ambush);
        self.hero_mut(hero_id)?.turn_record = TurnRecord::default();
        self.board.turn_effects.start_turn();
        let window = ShadowWindow {
            timing: ShadowTiming::HeroTurnStart,
            hero_id,
        };
        if self.open_shadow_window(window, Resume::Ambush, data)? {
            return Ok(());
        }
        self.ambush_step(data, rng)
    }

    /// The current hero's Ambush step (§ 6.4), then his Rest step when nothing
    /// attacked him.
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn ambush_step(&mut self, data: &GameData, rng: &mut StdRng) -> Result<(), EngineError> {
        let hero_id = self.current_hero_id()?;
        if !self.has_threatening_mobs(hero_id)? {
            self.log_step_skipped(TurnStep::Ambush, StepSkip::NoMobHere);
        }
        if self.face_location_threats(hero_id, true, data, rng)? == Threat::None {
            self.enter_rest_step(data)?;
        }
        Ok(())
    }

    /// Opens the Rest step, or goes straight to movement when the current hero can't
    /// rest: a monster or minion stays on its location, an active plot forbids resting
    /// there, or it stands on a Shadow Stronghold (§ 6.1).
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn enter_rest_step(&mut self, data: &GameData) -> Result<(), EngineError> {
        let hero_id = self.current_hero_id()?;
        let location = self.hero(hero_id)?.location.clone();
        let mob_here = self.mobs.iter().any(|m| m.location == location);
        if mob_here || self.rest_forbidden_by_plot(&location, data) {
            let reason = if mob_here {
                StepSkip::MobHere
            } else {
                StepSkip::PlotForbidsRest
            };
            self.log_step_skipped(TurnStep::Rest, reason);
            return self.fire(BoardEvent::NextToHeroMovementScreen);
        }
        self.fire(BoardEvent::NextToHeroRestScreen)?;
        if data.locations.get(&location)?.shadow_fortress {
            self.log_step_skipped(TurnStep::Rest, StepSkip::ShadowStronghold);
            self.fire(BoardEvent::NextToHeroMovementScreen)?;
        }
        Ok(())
    }

    /// The hero rests, or heals when `heal` is set and it stands in a haven, and may
    /// discard one corruption card; then the lowest Sauron marker advances (§ 6.1).
    /// Beravor resting outside a Haven may heal or receive training (hero sheet).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when healing or training isn't allowed or the
    /// corruption card can't be paid; transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ?rest), err(level = "debug"))]
    pub fn rest_hero(
        &mut self,
        hero_id: i64,
        rest: Rest,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<RestOutcome, EngineError> {
        let haven = data.locations.get(&self.hero(hero_id)?.location)?.haven;
        let survivalist = self.survivalist_rest(hero_id, data)?;
        if (rest.heal && !haven && !survivalist) || (rest.train && !survivalist) {
            return Err(EngineError::Refused(
                "healing is only possible in a haven".to_owned(),
            ));
        }
        if rest.heal && rest.train {
            return Err(EngineError::Refused("heal or train, not both".to_owned()));
        }
        if let Some(card) = rest.discard {
            self.discard_corruption_card(hero_id, card, data)?;
        }
        let hero = self.hero_mut(hero_id)?;
        let life_before = hero.life_pool.len();
        if rest.heal {
            hero.heal(rng);
        } else {
            hero.rest(rng);
        }
        let params = [
            ("count", json!(hero.life_pool.len() - life_before)),
            ("lp_cards", json!(hero.life_pool.len())),
        ];
        let action = if rest.heal { "rest.heal" } else { "rest.rest" };
        self.log(ActorRef::Hero(hero_id), action, &params);
        if rest.train {
            self.train(hero_id, Resume::AfterRest, rng)?;
            if self.board.pending_choice.is_some() {
                return Ok(RestOutcome::Choice);
            }
        }
        self.advance_story_after_rest(hero_id, data)
    }

    /// Tells whether the hero rests as Beravor outside a Haven (Survivalist), free to
    /// heal or receive training.
    ///
    /// # Errors
    /// Lookup errors.
    pub fn survivalist_rest(&self, hero_id: i64, data: &GameData) -> Result<bool, EngineError> {
        let hero = self.hero(hero_id)?;
        Ok(hero.code == BERAVOR && !data.locations.get(&hero.location)?.haven)
    }

    /// The lowest Sauron marker advances after a rest, 2 spaces for a despondent hero;
    /// on a tie the hero chooses (§ 6.1).
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id), err(level = "debug"))]
    pub fn advance_story_after_rest(
        &mut self,
        hero_id: i64,
        data: &GameData,
    ) -> Result<RestOutcome, EngineError> {
        let spaces = self.story_penalty(hero_id, data)?;
        let lowest = self.board.story.lowest_sauron_markers();
        if self.board.story.advance_lowest(spaces) {
            for marker in lowest {
                tracing::trace!(?marker, spaces, "marker advanced by a rest");
                self.log_rest_marker(marker, spaces);
            }
            self.fire(BoardEvent::NextToHeroMovementScreen)?;
            Ok(RestOutcome::Movement)
        } else {
            self.fire(BoardEvent::NextToAfterRestAdvanceStoryMarker)?;
            Ok(RestOutcome::ChooseStoryMarker)
        }
    }

    /// The hero skips resting.
    ///
    /// # Errors
    /// Transition errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn skip_rest(&mut self) -> Result<(), EngineError> {
        let hero_id = self.current_hero_id()?;
        self.log(ActorRef::Hero(hero_id), "rest.skip", &[]);
        self.fire(BoardEvent::NextToHeroMovementScreen)
    }

    /// The hero picks the tied story marker to advance after resting (§ 6.1).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when `marker` isn't one of the lowest; transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(?marker), err(level = "debug"))]
    pub fn advance_chosen_story_marker(
        &mut self,
        marker: StoryType,
        data: &GameData,
    ) -> Result<(), EngineError> {
        if !self.board.story.lowest_sauron_markers().contains(&marker) {
            return Err(EngineError::Refused(
                "only one of the lowest markers may advance".to_owned(),
            ));
        }
        let spaces = self.story_penalty(self.current_hero_id()?, data)?;
        self.board.story.advance(marker, spaces);
        self.log_rest_marker(marker, spaces);
        self.fire(BoardEvent::NextToHeroMovementScreen)
    }

    fn log_rest_marker(&mut self, marker: StoryType, spaces: i32) {
        let params = [("marker_name", json!(marker)), ("count", json!(spaces))];
        self.log(ActorRef::System, "story.advance", &params);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};

    // § 6.1: the lowest Sauron marker advances; on a tie the hero chooses.
    #[test]
    fn rest_advances_the_lowest_marker_or_asks() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        g.board.story.ring = 1;
        g.board.story.conquest = 1;
        assert_eq!(
            g.rest_hero(1, Rest::default(), data(), &mut rng()).unwrap(),
            RestOutcome::Movement
        );
        assert_eq!(g.board.story.corruption, 1);

        g.board.state = BoardState::HeroRestScreen;
        assert_eq!(
            g.rest_hero(1, Rest::default(), data(), &mut rng()).unwrap(),
            RestOutcome::ChooseStoryMarker
        );
        assert!(
            g.advance_chosen_story_marker(StoryType::Ring, data())
                .is_ok()
        );
        assert_eq!(g.board.story.ring, 2);
    }

    #[test]
    fn only_a_lowest_marker_may_be_chosen() {
        let mut g = game();
        g.board.state = BoardState::AfterRestAdvanceStoryMarker;
        g.board.story.ring = 1;
        assert!(
            g.advance_chosen_story_marker(StoryType::Ring, data())
                .is_err()
        );
    }

    // Despondent: resting advances the lowest Sauron marker 2 spaces.
    #[test]
    fn despondent_rest_advances_two_spaces() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        g.board.story.ring = 1;
        g.board.story.conquest = 1;
        g.heroes[0].corruption_cards = vec![0];
        g.rest_hero(1, Rest::default(), data(), &mut rng()).unwrap();
        assert_eq!(g.board.story.corruption, 2);
    }

    // Beravor (Survivalist): resting outside a Haven, heal or receive training.
    #[test]
    fn beravor_heals_or_trains_outside_havens() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        g.board.story.ring = 1;
        g.board.story.conquest = 1;
        g.heroes[0].location = "bree".to_owned();
        let heal = Rest {
            heal: true,
            ..Rest::default()
        };
        assert!(g.rest_hero(1, heal, data(), &mut rng()).is_err());

        g.heroes[0] = crate::engine::test_support::hero("beravor");
        g.heroes[0].location = "bree".to_owned();
        g.heroes[0].damage_pool = vec![1];
        g.rest_hero(1, heal, data(), &mut rng()).unwrap();
        assert!(g.heroes[0].damage_pool.is_empty());

        g.board.state = BoardState::HeroRestScreen;
        g.board.story.ring = 5;
        g.board.story.conquest = 5;
        let train = Rest {
            train: true,
            ..Rest::default()
        };
        let outcome = g.rest_hero(1, train, data(), &mut rng()).unwrap();
        assert_eq!(outcome, RestOutcome::Choice);
        g.answer_choice("keep_skill", &["skill:100".to_owned()], data(), &mut rng())
            .unwrap();
        assert_eq!(g.board.state, BoardState::HeroMovementScreen);
        assert_eq!(g.board.story.corruption, 2);
    }

    // § 6.1: when resting, a corruption card is discarded by spending its favor; its
    // malus ends.
    #[test]
    fn resting_hero_pays_to_discard_a_corruption_card() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        g.board.decks.corruption.draw = vec![14];
        g.gain_random_corruption(1, data(), &mut rng()).unwrap();
        let strength = g.heroes[0].strength;
        let rest = Rest {
            discard: Some(14),
            ..Rest::default()
        };
        g.heroes[0].favor = 1;
        assert!(g.rest_hero(1, rest, data(), &mut rng()).is_err());
        g.heroes[0].favor = 3;
        g.rest_hero(1, rest, data(), &mut rng()).unwrap();
        let hero = &g.heroes[0];
        assert_eq!(hero.favor, 1);
        assert_eq!(hero.strength, strength + 1);
        assert!(hero.corruption_cards.is_empty());
        assert_eq!(g.board.decks.corruption.discard, vec![14]);
    }
}
