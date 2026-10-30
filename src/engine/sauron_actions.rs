//! Sauron's action tracks (§ 5.7).

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::deck::SauronDeck;
use crate::engine::engine_error::EngineError;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::sauron_action::{ActionInProgress, ActionToken, SauronAction};

/// Tokens on the tracks when Sauron places one too many: the others are removed.
const TOKENS_BEFORE_RESET: usize = 4;

/// Influence tokens a Place Influence action may put in the Shadow Pool.
const SHADOW_POOL_PER_ACTION: i32 = 2;

/// X printed on the spaces of a track, left to right.
pub fn space_values(action: SauronAction) -> [i32; 3] {
    match action {
        SauronAction::PlaceInfluence => [6, 5, 4],
        SauronAction::DrawCards => [2, 2, 1],
        SauronAction::Command => [3, 2, 1],
    }
}

/// X of a space (1 is the leftmost).
pub fn space_value(action: SauronAction, space: u8) -> i32 {
    let values = space_values(action);
    values
        .get(usize::from(space.saturating_sub(1)))
        .copied()
        .unwrap_or(0)
}

impl Game {
    /// Actions Sauron takes each turn: two, three in a four-player game.
    pub fn sauron_actions_per_turn(&self) -> i32 {
        if self.heroes.len() >= 3 { 3 } else { 2 }
    }

    /// Actions Sauron still has to take this turn, the one in progress included.
    pub fn sauron_actions_left(&self) -> i32 {
        let in_progress = i32::from(self.board.current_action.is_some());
        (self.sauron_actions_per_turn() - self.board.sauron_actions_count).max(in_progress)
    }

    /// The leftmost empty space of a track, if any.
    pub fn next_action_space(&self, action: SauronAction) -> Option<u8> {
        (1..=3).find(|space| {
            !self
                .board
                .sauron_actions
                .iter()
                .any(|t| t.action == action && t.space == *space)
        })
    }

    /// Sauron places an action token on the leftmost empty space of a track and starts
    /// resolving the action with X, the number of that space. His fourth token on the
    /// tracks removes the three others.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when an action is in progress, no action is left this
    /// turn or the track is full; transition errors.
    pub fn start_sauron_action(
        &mut self,
        action: SauronAction,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if self.board.current_action.is_some() {
            return Err(EngineError::Refused("an action is in progress".to_owned()));
        }
        if self.sauron_actions_left() == 0 {
            return Err(EngineError::Refused("no action left this turn".to_owned()));
        }
        let space = self
            .next_action_space(action)
            .ok_or_else(|| EngineError::Refused("this action track is full".to_owned()))?;
        let token = ActionToken { action, space };
        let tokens = &mut self.board.sauron_actions;
        tokens.push(token);
        if tokens.len() >= TOKENS_BEFORE_RESET {
            tokens.retain(|t| *t == token);
        }
        let value = space_value(action, space);
        self.log(
            ActorRef::Sauron,
            &format!("sauron_actions.place.{token}"),
            &[],
        );
        self.board.current_action = Some(ActionInProgress {
            action,
            value,
            commands_left: if action == SauronAction::Command {
                value
            } else {
                0
            },
            monster_placed: false,
        });
        match action {
            SauronAction::PlaceInfluence => {
                self.grant_influence(value, SHADOW_POOL_PER_ACTION);
                self.fire(BoardEvent::NextToPlaceInfluenceScreenSauronActions)
            }
            SauronAction::DrawCards => {
                self.draw_shadow_and_plot_cards(value, rng);
                self.fire(BoardEvent::NextToKeepScreenSauronPlotCards)
            }
            SauronAction::Command => self.fire(BoardEvent::NextToCommandScreenSauronMobs),
        }
    }

    /// Draw Shadow and Plot Cards: X Shadow cards go to the hand, X Plot cards wait for
    /// Sauron to keep one.
    fn draw_shadow_and_plot_cards(&mut self, value: i32, rng: &mut StdRng) {
        let count = usize::try_from(value).unwrap_or(0);
        self.draw_sauron_cards(SauronDeck::Shadow, count, rng);
        let drawn = std::mem::take(&mut self.sauron.drawn_shadow);
        self.log(
            ActorRef::Sauron,
            "shadow_cards.keep",
            &[("count", json!(drawn.len()))],
        );
        self.sauron.shadow_hand.extend(drawn);
        self.draw_sauron_cards(SauronDeck::Plot, count, rng);
    }

    /// Sauron ends the action in progress. Influence left to place is lost.
    ///
    /// # Errors
    /// [`EngineError::Refused`] without an action, or while drawn plot cards wait to be
    /// kept; transition errors.
    pub fn finish_sauron_action(&mut self) -> Result<(), EngineError> {
        let action = self
            .board
            .current_action
            .ok_or_else(|| EngineError::Refused("no action in progress".to_owned()))?;
        if action.action == SauronAction::DrawCards && !self.sauron.drawn_plot.is_empty() {
            return Err(EngineError::Refused(
                "keep one of the drawn plot cards first".to_owned(),
            ));
        }
        if action.action == SauronAction::PlaceInfluence {
            self.close_influence_budget();
        }
        for mob in &mut self.mobs {
            mob.commanded = false;
        }
        self.board.current_action = None;
        self.board.sauron_actions_count += 1;
        self.fire(BoardEvent::NextToEditSauronSauronActions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::test_support::{data, game, rng};

    fn action_step() -> Game {
        let mut g = game();
        g.board.state = BoardState::EditSauronSauronActions;
        g
    }

    fn take(g: &mut Game, action: SauronAction) {
        g.start_sauron_action(action, &mut rng()).unwrap();
        if action == SauronAction::DrawCards {
            let card = g.sauron.drawn_plot[0];
            g.keep_sauron_cards(SauronDeck::Plot, &[card], data())
                .unwrap();
        }
        g.finish_sauron_action().unwrap();
    }

    // § 5.7: two actions per turn (three in a four-player game).
    #[test]
    fn two_actions_with_fewer_than_three_heroes() {
        let mut g = action_step();
        assert_eq!(g.sauron_actions_per_turn(), 2);
        take(&mut g, SauronAction::Command);
        take(&mut g, SauronAction::Command);
        assert_eq!(g.sauron_actions_left(), 0);
        assert!(
            g.start_sauron_action(SauronAction::Command, &mut rng())
                .is_err()
        );
        g.terminate_sauron_turn().unwrap();
    }

    // § 5.7 example: tokens on "6" and "5" of Place Influence; the next goes on "4", and
    // the fourth token removes the three others.
    #[test]
    fn leftmost_space_and_fourth_token() {
        let mut g = action_step();
        g.sauron.plot_hand.clear();
        g.board.sauron_actions = vec![
            ActionToken {
                action: SauronAction::PlaceInfluence,
                space: 1,
            },
            ActionToken {
                action: SauronAction::PlaceInfluence,
                space: 2,
            },
        ];
        g.start_sauron_action(SauronAction::PlaceInfluence, &mut rng())
            .unwrap();
        assert_eq!(g.board.influence_budget.tokens, 4);
        assert_eq!(g.board.influence_budget.shadow_pool_max, 2);
        assert_eq!(g.next_action_space(SauronAction::PlaceInfluence), None);
        g.finish_sauron_action().unwrap();
        assert_eq!(
            g.board.influence_budget.tokens, 0,
            "unplaced tokens are lost"
        );

        g.start_sauron_action(SauronAction::Command, &mut rng())
            .unwrap();
        assert_eq!(
            g.board.sauron_actions,
            vec![ActionToken {
                action: SauronAction::Command,
                space: 1
            }]
        );
        assert_eq!(g.board.current_action.unwrap().commands_left, 3);
        assert_eq!(g.next_action_space(SauronAction::PlaceInfluence), Some(1));
    }

    // § 5.7 Draw Shadow and Plot Cards: X of each, only one plot card kept.
    #[test]
    fn draw_keeps_exactly_one_plot() {
        let mut g = action_step();
        g.sauron.plot_hand.clear();
        g.sauron.shadow_hand.clear();
        g.sauron.drawn_plot.clear();
        g.sauron.drawn_shadow.clear();
        g.start_sauron_action(SauronAction::DrawCards, &mut rng())
            .unwrap();
        assert_eq!(g.board.state, BoardState::KeepScreenSauronPlotCards);
        assert_eq!(g.sauron.shadow_hand.len(), 2);
        assert_eq!(g.sauron.drawn_plot.len(), 2);
        assert!(g.finish_sauron_action().is_err());
        let drawn = g.sauron.drawn_plot.clone();
        assert!(
            g.keep_sauron_cards(SauronDeck::Plot, &drawn, data())
                .is_err()
        );
        g.keep_sauron_cards(SauronDeck::Plot, &drawn[..1], data())
            .unwrap();
        assert_eq!(g.sauron.plot_hand, vec![drawn[0]]);
        assert_eq!(g.board.decks.plot.draw.last(), Some(&drawn[1]));
    }
}
