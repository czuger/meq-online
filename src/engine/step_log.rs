//! Records the steps of a turn in the game log, so players can see what each step did
//! or why it couldn't be played.

use serde_json::json;

use crate::engine::board_state::BoardState;
use crate::engine::engine_error::EngineError;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::turn_step::{
    STEP_BEGIN, STEP_SKIPPED, StepSkip, TURN_DRAW, TURN_HERO, TURN_SAURON, TURN_STOPPED, TurnStep,
    TurnStop,
};

/// The step a screen state belongs to; combats and choices stay in the step they
/// interrupt.
fn screen_step(state: BoardState) -> Option<TurnStep> {
    match state {
        BoardState::PlayScreenSauronPlotCards => Some(TurnStep::Plot),
        BoardState::EditSauronSauronActions => Some(TurnStep::Action),
        BoardState::HeroRestScreen => Some(TurnStep::Rest),
        BoardState::HeroMovementScreen => Some(TurnStep::Movement),
        BoardState::HeroExploration => Some(TurnStep::Exploration),
        _ => None,
    }
}

impl Game {
    /// Records the step a state change enters, if any.
    pub(crate) fn log_state_step(&mut self, from: BoardState, to: BoardState) {
        if let Some(step) = screen_step(to).filter(|_| screen_step(from) != screen_step(to)) {
            self.log_step(step);
        }
    }

    /// Records the start of Sauron's turn.
    pub(crate) fn log_sauron_turn(&mut self) {
        self.log(ActorRef::System, TURN_SAURON, &[]);
    }

    /// Records the start of a Hero Draw step.
    pub(crate) fn log_draw_turn(&mut self) {
        self.log(ActorRef::System, TURN_DRAW, &[]);
    }

    /// Records the start of a hero's turn.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub(crate) fn log_hero_turn(&mut self, hero_id: i64) -> Result<(), EngineError> {
        let code = self.hero(hero_id)?.code.clone();
        self.log(ActorRef::Hero(hero_id), TURN_HERO, &[("hero", json!(code))]);
        Ok(())
    }

    /// Records the start of a step.
    pub(crate) fn log_step(&mut self, step: TurnStep) {
        self.log(ActorRef::System, STEP_BEGIN, &[("step", json!(step))]);
    }

    /// Records a step that couldn't be played, and why.
    pub(crate) fn log_step_skipped(&mut self, step: TurnStep, reason: StepSkip) {
        let params = [("step", json!(step)), ("reason", json!(reason))];
        self.log(ActorRef::System, STEP_SKIPPED, &params);
    }

    /// Records a turn ending before its last step, and why.
    pub(crate) fn log_turn_stopped(&mut self, hero_id: i64, reason: TurnStop) {
        let params = [("reason", json!(reason))];
        self.log(ActorRef::Hero(hero_id), TURN_STOPPED, &params);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::engine::board_state::{BoardEvent, BoardState};
    use crate::engine::test_support::{data, game, rng};

    fn skipped(g: &Game, step: TurnStep) -> Option<serde_json::Value> {
        g.new_logs
            .iter()
            .find(|l| l.action == STEP_SKIPPED && l.params.get("step") == Some(&json!(step)))
            .and_then(|l| l.params.get("reason").cloned())
    }

    fn logged(g: &Game, action: &str) -> bool {
        g.new_logs.iter().any(|l| l.action == action)
    }

    #[test]
    fn entering_a_screen_records_its_step_once() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        g.new_logs.clear();
        g.fire(BoardEvent::NextToHeroMovementScreen).unwrap();
        let begun: Vec<_> = g
            .new_logs
            .iter()
            .filter(|l| l.action == STEP_BEGIN)
            .collect();
        assert_eq!(begun.len(), 1);
        assert_eq!(
            begun[0].params.get("step"),
            Some(&json!(TurnStep::Movement))
        );
    }

    #[test]
    fn a_monster_on_the_location_prevents_the_rest_and_says_why() {
        let mut g = game();
        g.board.state = BoardState::HeroRestScreen;
        let location = g.heroes[0].location.clone();
        g.create_mob("snaga", &location, None, data()).unwrap();
        g.board.current_hero_id = g.heroes[0].id;
        g.enter_rest_step(data()).unwrap();
        assert_eq!(g.board.state, BoardState::HeroMovementScreen);
        assert_eq!(skipped(&g, TurnStep::Rest), Some(json!(StepSkip::MobHere)));
    }

    #[test]
    fn no_mob_means_no_ambush_and_says_why() {
        let mut g = game();
        g.board.state = BoardState::HeroDrawCardsScreen;
        g.mobs.clear();
        g.board.current_hero_id = g.heroes[0].id;
        g.heroes[0].location = "the_shire".to_owned();
        g.ambush_step(data(), &mut rng()).unwrap();
        assert_eq!(
            skipped(&g, TurnStep::Ambush),
            Some(json!(StepSkip::NoMobHere))
        );
    }

    #[test]
    fn travelling_shows_the_cards_paid() {
        let mut g = game();
        g.board.state = BoardState::HeroMovementScreen;
        g.heroes[0].location = "barad_dur".to_owned();
        g.heroes[0].hand = vec![4];
        g.board.current_hero_id = g.heroes[0].id;
        g.move_hero(1, "mount_doom", &[4], data(), &mut rng())
            .unwrap();
        let travel = g
            .new_logs
            .iter()
            .find(|l| l.action == "movement.travel")
            .expect("travel logged");
        assert_eq!(travel.params.get("hero_cards"), Some(&json!([4])));
    }

    #[test]
    fn a_hero_draw_only_tells_how_many_cards() {
        let mut g = game();
        g.heroes[0].life_pool = vec![1, 2, 3];
        g.heroes[0].hand.clear();
        g.draw_step(1, 2, data()).unwrap();
        let draw = g
            .new_logs
            .iter()
            .find(|l| l.action == "draw_cards")
            .expect("draw logged");
        assert_eq!(draw.params.get("cards_drawn"), Some(&json!(2)));
        assert!(!draw.params.contains_key("hero_cards"));
    }

    #[test]
    fn nothing_drawn_tells_why_when_the_life_pool_is_empty() {
        let mut g = game();
        g.heroes[0].life_pool.clear();
        g.heroes[0].hand.clear();
        g.heroes[0].cards_drawn_this_round = 0;
        g.board.state = BoardState::HeroDrawCardsScreen;
        g.heroes[0].active = true;
        g.draw_cards_finished(1, data(), &mut rng()).unwrap();
        assert!(logged(&g, "draw_cards.blocked"));
    }
}
