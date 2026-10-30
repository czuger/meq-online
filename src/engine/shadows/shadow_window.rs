//! When Sauron may play a Shadow card, and playing it (§ 5.9).

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::board_state::BoardState;
use crate::engine::choices::{PASS, plain_option, target, target_option};
use crate::engine::deck::SauronDeck;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::corruption_card::Flaw;
use crate::game_data::mob_def::MobKind;
use crate::game_data::shadow::{ShadowCard, ShadowCode, ShadowTiming};
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// Option code of Sauron playing a Shadow card.
pub const PLAY_SHADOW: &str = "play_shadow";

/// A moment of a hero's turn when Sauron may play a Shadow card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowWindow {
    /// The moment.
    pub timing: ShadowTiming,
    /// The hero whose turn it is.
    pub hero_id: i64,
}

impl Game {
    /// Shadow cards of Sauron's hand he may play now: at most one per player's turn,
    /// with enough influence in the Shadow Pool and at the card's timing (§ 5.9).
    ///
    /// # Arguments
    /// * `timing` - the moment of the turn.
    /// * `hero_id` - the hero the card would be played against, outside the Action step.
    ///
    /// # Errors
    /// Lookup errors.
    pub fn playable_shadow_cards(
        &self,
        timing: ShadowTiming,
        hero_id: Option<i64>,
        data: &GameData,
    ) -> Result<Vec<ShadowCode>, EngineError> {
        if self.board.turn_effects.shadow_played {
            return Ok(vec![]);
        }
        let in_haven = match hero_id {
            Some(id) => data.locations.get(&self.hero(id)?.location)?.haven,
            None => false,
        };
        let against_minion = self.combat_against_minion();
        let mut playable = Vec::new();
        for card in &self.sauron.shadow_hand {
            tracing::trace!(?card, "shadow card playable?");
            let def = data.shadow(*card)?;
            let allowed = def.timing == timing
                && self.board.shadow_pool >= def.requirement
                && !(def.outside_haven && in_haven)
                && (!def.against_minion || against_minion);
            if allowed && !playable.contains(card) {
                playable.push(*card);
            }
        }
        Ok(playable)
    }

    /// The combat in progress is against a minion.
    fn combat_against_minion(&self) -> bool {
        self.combat.as_ref().is_some_and(|c| {
            self.mobs
                .iter()
                .any(|m| m.id == Some(c.mob_id) && m.kind == MobKind::Minion)
        })
    }

    /// Lets Sauron play a Shadow card or pass when he holds one he may play now.
    ///
    /// # Arguments
    /// * `resume` - where play goes on once the card is resolved or Sauron passed.
    ///
    /// # Returns
    /// `true` when Sauron's decision is pending.
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(?window, ?resume), err(level = "debug"))]
    pub fn open_shadow_window(
        &mut self,
        window: ShadowWindow,
        resume: Resume,
        data: &GameData,
    ) -> Result<bool, EngineError> {
        let cards = self.playable_shadow_cards(window.timing, Some(window.hero_id), data)?;
        if cards.is_empty() {
            return Ok(false);
        }
        let targets = cards.iter().map(|c| target("shadow", c)).collect();
        self.ask(PendingChoice {
            chooser: Chooser::Sauron,
            source: ChoiceSource::ShadowWindow {
                timing: window.timing,
                hero_id: window.hero_id,
            },
            options: vec![
                target_option(PLAY_SHADOW, targets, 1, 1),
                plain_option(PASS),
            ],
            resume,
        })?;
        Ok(true)
    }

    /// Sauron played the picked Shadow card in a window, or passed.
    ///
    /// # Returns
    /// Where play goes on when the card asks nothing more.
    #[tracing::instrument(level = "debug", skip_all, fields(?window, %option, ?targets, ?resume), err(level = "debug"))]
    pub(crate) fn apply_shadow_window(
        &mut self,
        window: ShadowWindow,
        option: &str,
        targets: &[String],
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        if option == PASS {
            return Ok(resume);
        }
        let card = targets
            .first()
            .and_then(|t| t.strip_prefix("shadow:"))
            .and_then(|c| c.parse().ok())
            .ok_or_else(|| EngineError::Refused("choose a shadow card".to_owned()))?;
        let playable = self.playable_shadow_cards(window.timing, Some(window.hero_id), data)?;
        if !playable.contains(&card) {
            return Err(EngineError::Refused(format!(
                "shadow card {card} can't be played now"
            )));
        }
        self.play_shadow(card, Some(window.hero_id), resume, data, rng)
    }

    /// Sauron plays a Shadow card during his Action step, between two actions.
    ///
    /// # Errors
    /// [`EngineError::Refused`] outside the Action step, during an action, or when the
    /// card isn't playable now; lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(?card), err(level = "debug"))]
    pub fn play_action_shadow(
        &mut self,
        card: ShadowCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if self.board.state != BoardState::EditSauronSauronActions
            || self.board.current_action.is_some()
        {
            return Err(EngineError::Refused(
                "shadow cards are played between two actions".to_owned(),
            ));
        }
        if !self
            .playable_shadow_cards(ShadowTiming::ActionStep, None, data)?
            .contains(&card)
        {
            return Err(EngineError::Refused(format!(
                "shadow card {card} can't be played now"
            )));
        }
        self.play_shadow(card, None, Resume::SauronActions, data, rng)?;
        Ok(())
    }

    /// The card leaves Sauron's hand for the discard pile and resolves. Against a
    /// greedy hero Sauron then draws a Shadow card.
    #[tracing::instrument(level = "debug", skip_all, fields(?card, ?hero_id, ?resume), err(level = "debug"))]
    fn play_shadow(
        &mut self,
        card: ShadowCode,
        hero_id: Option<i64>,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let def: &ShadowCard = data.shadow(card)?;
        let hand = &mut self.sauron.shadow_hand;
        let position =
            hand.iter()
                .position(|c| *c == card)
                .ok_or(EngineError::CardNotAvailable {
                    card,
                    pile: "shadow hand",
                })?;
        hand.remove(position);
        self.board.decks.shadow.discard.push(card);
        self.board.turn_effects.shadow_played = true;
        let params = [("shadow", json!(def.name_fr)), ("shadow_card", json!(card))];
        self.log(ActorRef::Sauron, "shadow.play", &params);
        if let Some(hero_id) = hero_id
            && self.hero(hero_id)?.has_flaw(Flaw::Greedy, data)
        {
            self.draw_to_sauron_hand(SauronDeck::Shadow, 1, rng);
            self.log(ActorRef::Hero(hero_id), "corruption.greedy", &[]);
        }
        self.start_shadow(card, hero_id, resume, data, rng)
    }
}
