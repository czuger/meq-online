//! Sauron's turn: setup, the start of his Action step and the end of his turn.

use rand::rngs::StdRng;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::game::Game;
use crate::model::turn_step::{StepSkip, TurnStep};

impl Game {
    /// Sauron accepts the starting setup (§ 3): objectives, first event, then his
    /// actions. The starting plot's influence must be placed and its plot cards kept.
    ///
    /// # Errors
    /// [`EngineError::Refused`] while influence or drawn cards are left; transition and
    /// event errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn sauron_setup_finished(
        &mut self,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if self.board.influence_budget.tokens > 0 {
            return Err(EngineError::Refused(
                "the starting influence isn't placed yet".to_owned(),
            ));
        }
        if !self.sauron.drawn_plot.is_empty() {
            return Err(EngineError::Refused(
                "the starting plot cards aren't kept yet".to_owned(),
            ));
        }
        self.close_influence_budget();
        self.set_objectives(rng);
        self.sauron.active = true;
        self.log_sauron_turn();
        self.log_step_skipped(TurnStep::Story, StepSkip::FirstRound);
        self.log_step_skipped(TurnStep::Plot, StepSkip::FirstRound);
        self.event_step(data, rng)
    }

    /// Starts Sauron's Action step; defeated Ringwraiths come back first (§ 8.8).
    ///
    /// # Errors
    /// Transition errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn start_action_step(&mut self, data: &GameData) -> Result<(), EngineError> {
        self.fire(BoardEvent::NextToEditSauronSauronActions)?;
        self.board.sauron_actions_count = 0;
        self.return_ringwraiths(data)
    }

    /// Sauron ends his turn once his actions are taken; the Hero Draw step starts for
    /// every hero.
    ///
    /// # Errors
    /// [`EngineError::Refused`] while actions are left; transition errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn terminate_sauron_turn(&mut self) -> Result<(), EngineError> {
        if self.sauron_actions_left() > 0 {
            return Err(EngineError::Refused(
                "Sauron still has actions to take".to_owned(),
            ));
        }
        self.log_draw_turn();
        self.set_heroes_active(true);
        for hero in &mut self.heroes {
            tracing::trace!(hero = %hero.code, "resetting cards drawn this round");
            hero.cards_drawn_this_round = 0;
        }
        self.fire(BoardEvent::NextToFinishSauronTurn)?;
        self.fire(BoardEvent::NextToHeroDrawCardsScreen)?;
        self.board.sauron_actions_count = 0;
        Ok(())
    }
}
