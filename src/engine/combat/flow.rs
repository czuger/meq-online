//! Steps of a combat around the round resolution: setup, card choice, losses, end.

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::combat::reveal::reveal_secretly_played_cards;
use crate::engine::deck::SauronDeck;
use crate::engine::defeat::DefeatOutcome;
use crate::engine::engine_error::EngineError;
use crate::engine::mobs::RINGWRAITHS;
use crate::engine::perils::effects::peril_combat_continues;
use crate::engine::turn::TurnEnd;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::corruption_card::Flaw;
use crate::game_data::mob_def::MobKind;
use crate::game_data::shadow::ShadowEffect;
use crate::model::actor_ref::ActorRef;
use crate::model::combat::{Combat, CombatOrigin};
use crate::model::game::Game;

/// Hero whose turn doesn't end after failing to defeat an opponent (Iron Will).
const IRON_WILL_HERO: &str = "thalin";

/// Side of a combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatSide {
    /// The hero.
    Hero,
    /// The mob, played by Sauron.
    Mob,
}

/// What follows a card choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardPlayed {
    /// The other side hasn't chosen yet.
    Waiting,
    /// The round was resolved.
    Resolved,
    /// The round was resolved and the hero must choose cards to lose.
    CardsLoss,
}

/// State of a combat as shown to players.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatResult {
    /// The mob has no life left.
    pub mob_defeated: bool,
    /// The hero has no card left in hand or life pool.
    pub hero_defeated: bool,
    /// The mob can't play anymore.
    pub mob_exhausted: bool,
    /// The hero can't play anymore.
    pub hero_exhausted: bool,
}

/// Where the hero goes after a combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEnd {
    /// The hero survived an ambush and travels.
    Movement,
    /// The hero survived a Shadow card's combat and goes to his Rest step.
    Rest,
    /// The hero survived a combat met while travelling and explores.
    Exploration,
    /// The hero's reward asks a choice (training).
    Choice,
    /// The hero was defeated and has choices to make.
    Defeat,
    /// The hero was defeated; Sauron picks its Haven.
    ChooseHaven,
    /// The hero's turn ended.
    TurnEnded(TurnEnd),
    /// The Finale combat decided the game.
    GameOver,
}

impl Game {
    fn combat_ref(&self) -> Result<&Combat, EngineError> {
        self.combat.as_ref().ok_or(EngineError::NoCombat)
    }

    fn combat_ids(&self) -> Result<(i64, i64), EngineError> {
        let combat = self.combat_ref()?;
        Ok((combat.hero_id, combat.mob_id))
    }

    /// Preparation (§ 8.3): the hero spends `agility_to_draw` of its agility to draw
    /// as many cards; each unspent agility adds 1 strength for this combat. A cowardly
    /// hero first discards a random card.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when `agility_to_draw` exceeds the hero's agility;
    /// [`EngineError::NoCombat`]; lookup and transition errors.
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(agility_to_draw),
        err(level = "debug")
    )]
    pub fn combat_setup(
        &mut self,
        agility_to_draw: i32,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let (hero_id, _) = self.combat_ids()?;
        let modifiers = self.combat_ref()?.modifiers;
        let agility = modifiers.agility(self.hero(hero_id)?.agility);
        if !(0..=agility).contains(&agility_to_draw) {
            return Err(EngineError::Refused(format!(
                "agility to spend must be between 0 and {agility}"
            )));
        }
        let hero = self.hero_mut(hero_id)?;
        if hero.has_flaw(Flaw::Cowardly, data) {
            if let Some(card) = hero.hand.choose(rng).copied() {
                tracing::debug!(hero_id, card, "Cowardly: random card lost");
                hero.hand_to_rest(&[card])?;
            }
            self.log(ActorRef::Hero(hero_id), "corruption.cowardly", &[]);
        }

        let hero = self.hero_mut(hero_id)?;
        let code = hero.code.clone();
        let base_strength = hero.strength + modifiers.strength;
        let strength = base_strength + agility - agility_to_draw;
        let wanted = usize::try_from(agility_to_draw).unwrap_or(0);
        let drawn = hero.draw_cards(wanted.min(hero.room_in_hand(data)));
        let left = hero.life_pool.len();
        let params = [
            ("name", json!(code)),
            ("cards_drawn", json!(drawn)),
            ("lp_cards", json!(left)),
            ("str", json!(base_strength)),
            ("nstr", json!(strength)),
        ];
        self.log(ActorRef::Hero(hero_id), "combat.prepare", &params);

        if let Some(combat) = self.combat.as_mut() {
            combat.hero_strength = Some(strength);
        }
        self.fire(BoardEvent::NextToPlayCombatCardScreenBoardCombats)?;
        self.hero_mut(hero_id)?.active = true;
        self.sauron.active = true;
        Ok(())
    }

    /// One side secretly chooses a card; once both have chosen, the round is resolved
    /// and the hero immediately pays damage from its life pool.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when the card isn't in the player's hand.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn play_combat_card(
        &mut self,
        side: CombatSide,
        card: CardNumber,
        data: &GameData,
    ) -> Result<CardPlayed, EngineError> {
        let (hero_id, mob_id) = self.combat_ids()?;
        match side {
            CombatSide::Hero => {
                let hero = self.hero_mut(hero_id)?;
                hero.take_from_hand(&[card])?;
                hero.active = false;
                if let Some(combat) = self.combat.as_mut() {
                    combat.hero_secret_card = Some(card);
                }
            }
            CombatSide::Mob => {
                let mob = self.mob_mut(mob_id)?;
                let position = mob.hand.iter().position(|c| *c == card).ok_or(
                    EngineError::CardNotAvailable {
                        card,
                        pile: "mob hand",
                    },
                )?;
                mob.hand.remove(position);
                self.sauron.active = false;
                if let Some(combat) = self.combat.as_mut() {
                    combat.mob_secret_card = Some(card);
                }
            }
        }
        if self.hero(hero_id)?.active || self.sauron.active {
            return Ok(CardPlayed::Waiting);
        }
        self.resolve_round(hero_id, mob_id, data)
    }

    /// A side declares itself exhausted before choosing its card (§ 8.7). Sauron may
    /// only do so once the hero has chosen its card or declared, the hero deciding first.
    ///
    /// # Returns
    /// What follows, as for a card choice; both sides exhausted end the combat.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the side already chose its card, or Sauron declares
    /// before the hero decided; lookup errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn declare_exhaustion(
        &mut self,
        side: CombatSide,
        data: &GameData,
    ) -> Result<CardPlayed, EngineError> {
        let (hero_id, mob_id) = self.combat_ids()?;
        let hero_deciding = self.hero(hero_id)?.active;
        let refused = |why: &str| Err(EngineError::Refused(why.to_owned()));
        match side {
            CombatSide::Hero if !hero_deciding => return refused("the hero already decided"),
            CombatSide::Mob if hero_deciding => return refused("the hero declares first"),
            CombatSide::Mob if !self.sauron.active => return refused("Sauron already decided"),
            CombatSide::Hero => self.hero_mut(hero_id)?.active = false,
            CombatSide::Mob => self.sauron.active = false,
        }
        let combat = self.combat.as_mut().ok_or(EngineError::NoCombat)?;
        match side {
            CombatSide::Hero => combat.hero_exhausted = true,
            CombatSide::Mob => combat.mob_exhausted = true,
        }
        if combat.hero_exhausted && combat.mob_exhausted {
            return Ok(CardPlayed::Resolved);
        }
        if self.hero(hero_id)?.active || self.sauron.active {
            return Ok(CardPlayed::Waiting);
        }
        self.resolve_round(hero_id, mob_id, data)
    }

    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(hero_id, mob_id),
        err(level = "debug")
    )]
    fn resolve_round(
        &mut self,
        hero_id: i64,
        mob_id: i64,
        data: &GameData,
    ) -> Result<CardPlayed, EngineError> {
        let hero_index = self.hero_index(hero_id)?;
        let mob_index = self.mob_index(mob_id)?;
        let (Some(combat), Some(hero), Some(mob)) = (
            self.combat.as_mut(),
            self.heroes.get_mut(hero_index),
            self.mobs.get_mut(mob_index),
        ) else {
            return Err(EngineError::NoCombat);
        };
        reveal_secretly_played_cards(combat, hero, mob, data)?;
        // The original engine lost this reactivation when Sauron got his own table.
        if !combat.mob_exhausted {
            self.sauron.active = true;
        }

        // § 6.3, § 8.8: the hero pays each damage with a card from its hand or the top
        // of its life pool, in any combination; with no hand there is nothing to choose.
        let hero = self.hero_mut(hero_id)?;
        let damage = usize::try_from(hero.temporary_damages).unwrap_or(0);
        if damage == 0 {
            return Ok(CardPlayed::Resolved);
        }
        if damage >= hero.life_pool.len() + hero.hand.len() {
            // Not enough cards left: the hero is defeated.
            let mut everything = std::mem::take(&mut hero.life_pool);
            everything.append(&mut hero.hand);
            hero.damage_pool.extend(everything);
            hero.temporary_damages = 0;
            return Ok(CardPlayed::Resolved);
        }
        if hero.hand.is_empty() {
            hero.pay_damage(&[])?;
            return Ok(CardPlayed::Resolved);
        }
        self.fire(BoardEvent::NextToCardsLossScreenBoardCombats)?;
        Ok(CardPlayed::CardsLoss)
    }

    /// The hero pays the round's damage: the selected hand cards, the rest from the
    /// top of its life pool (§ 6.3).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when more cards are selected than the damage, or the
    /// life pool can't pay the rest; [`EngineError::CardNotAvailable`] when a card isn't
    /// in hand; transition errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn combat_cards_loss(&mut self, selected: &[CardNumber]) -> Result<(), EngineError> {
        let (hero_id, _) = self.combat_ids()?;
        self.hero_mut(hero_id)?.pay_damage(selected)?;
        self.fire(BoardEvent::NextToPlayCombatCardScreenBoardCombats)
    }

    /// Current combat result. A side with an empty hand becomes exhausted and, like a
    /// side out of strength, stops being expected to play.
    ///
    /// # Errors
    /// [`EngineError::NoCombat`]; lookup errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn refresh_combat_result(&mut self) -> Result<CombatResult, EngineError> {
        let (hero_id, mob_id) = self.combat_ids()?;
        let hero = self.hero(hero_id)?;
        let (hero_defeated, hero_empty) = (
            hero.life_pool.len() + hero.hand.len() == 0,
            hero.hand.is_empty(),
        );
        let mob = self.mob_mut(mob_id)?;
        let (mob_defeated, mob_empty) = (mob.life <= 0, mob.hand.is_empty());
        if hero_empty {
            self.hero_mut(hero_id)?.active = false;
        }
        if mob_empty {
            self.sauron.active = false;
        }
        let combat = self.combat.as_mut().ok_or(EngineError::NoCombat)?;
        combat.hero_exhausted |= hero_empty;
        combat.mob_exhausted |= mob_empty;
        Ok(CombatResult {
            mob_defeated,
            hero_defeated,
            mob_exhausted: combat.mob_exhausted,
            hero_exhausted: combat.hero_exhausted,
        })
    }

    /// Ends the combat (§ 8.3 Resolution): played cards go to the rest pool, the
    /// monster token is discarded, an undefeated mob lets Sauron place influence equal
    /// to its wisdom. A defeated hero resolves its defeat (§ 8.9); otherwise see
    /// [`Game::after_combat`]. The Finale combat only decides the winner (§ 9).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the combat isn't over; lookup and transition errors.
    #[tracing::instrument(level = "debug", skip_all, err(level = "debug"))]
    pub fn terminate_combat(
        &mut self,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<CombatEnd, EngineError> {
        let result = self.refresh_combat_result()?;
        let both_exhausted = result.hero_exhausted && result.mob_exhausted;
        if !result.hero_defeated && !result.mob_defeated && !both_exhausted {
            return Err(EngineError::Refused("the combat is not over".to_owned()));
        }
        let combat = self.combat.take().ok_or(EngineError::NoCombat)?;
        if combat.origin == CombatOrigin::Finale {
            self.finish_finale(result.mob_defeated)?;
            return Ok(CombatEnd::GameOver);
        }
        let played: Vec<CardNumber> = combat.hero_cards.iter().filter_map(|c| c.card).collect();
        self.hero_mut(combat.hero_id)?.rest_pool.extend(played);
        if result.mob_defeated {
            self.high_stakes_mob_defeated(combat.hero_id, combat.mob_id, data)?;
        } else {
            self.grant_undefeated_mob_influence(combat.mob_id, data)?;
        }
        self.remove_fought_mob(combat.mob_id)?;
        self.after_shadow_combat(combat.origin, combat.hero_id, data, rng)?;

        if result.hero_defeated {
            return Ok(match self.start_hero_defeat(combat.hero_id, data, rng)? {
                None => CombatEnd::Defeat,
                Some(DefeatOutcome::ChooseHaven) => CombatEnd::ChooseHaven,
                Some(DefeatOutcome::TurnEnded(end)) => CombatEnd::TurnEnded(end),
            });
        }
        self.after_combat(
            combat.hero_id,
            combat.origin,
            result.mob_defeated,
            data,
            rng,
        )
    }

    /// The hero survived: a Peril combat won gives its reward. The turn ends when a mob
    /// met while travelling survived (§ 6.5), except after an ambush (§ 6.4) or a
    /// Shadow card's combat, for Thálin (Iron Will) and for the Peril cards saying so.
    /// Otherwise the hero travels after an ambush, goes to his Rest step after a Shadow
    /// card's combat (it replaced the Ambush step) and explores after a combat met while
    /// travelling.
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(hero_id, mob_defeated),
        err(level = "debug")
    )]
    fn after_combat(
        &mut self,
        hero_id: i64,
        origin: CombatOrigin,
        mob_defeated: bool,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<CombatEnd, EngineError> {
        let peril = match origin {
            CombatOrigin::Peril { card } => Some(data.peril(card)?.effect),
            CombatOrigin::Ambush
            | CombatOrigin::Travel
            | CombatOrigin::Shadow { .. }
            | CombatOrigin::Finale => None,
        };
        let turn_start = matches!(origin, CombatOrigin::Ambush | CombatOrigin::Shadow { .. });
        let goes_on = mob_defeated
            || turn_start
            || self.hero(hero_id)?.code == IRON_WILL_HERO
            || peril.is_some_and(peril_combat_continues);
        if !goes_on {
            return Ok(CombatEnd::TurnEnded(
                self.finish_heroes_turn(hero_id, data, rng)?,
            ));
        }
        if let (true, CombatOrigin::Peril { card }) = (mob_defeated, origin) {
            self.reward_peril_victory(hero_id, card, data, rng)?;
            if self.board.pending_choice.is_some() {
                return Ok(CombatEnd::Choice);
            }
        }
        if let CombatOrigin::Shadow { .. } = origin {
            self.activate_current_hero()?;
            self.enter_rest_step(data)?;
            return Ok(CombatEnd::Rest);
        }
        if origin == CombatOrigin::Ambush {
            self.fire(BoardEvent::NextToHeroMovementScreen)?;
            self.activate_current_hero()?;
            return Ok(CombatEnd::Movement);
        }
        self.fire(BoardEvent::NextToHeroExploration)?;
        self.activate_current_hero()?;
        Ok(CombatEnd::Exploration)
    }

    /// "Seem Fairer, Feel Fouler": after the combat, Sauron draws a Plot card when the
    /// hero holds a Corruption card.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id), err(level = "debug"))]
    fn after_shadow_combat(
        &mut self,
        origin: CombatOrigin,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let CombatOrigin::Shadow { card } = origin else {
            return Ok(());
        };
        let corrupted = !self.hero(hero_id)?.corruption_cards.is_empty();
        if corrupted && data.shadow(card)?.effect == ShadowEffect::SeemFairer {
            self.draw_to_sauron_hand(SauronDeck::Plot, 1, rng);
        }
        Ok(())
    }

    /// Sauron may place influence equal to the wisdom of an undefeated mob (§ 8.3).
    #[tracing::instrument(level = "debug", skip_all, fields(mob_id), err(level = "debug"))]
    fn grant_undefeated_mob_influence(
        &mut self,
        mob_id: i64,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let code = self
            .mobs
            .get(self.mob_index(mob_id)?)
            .map(|m| m.code.clone())
            .unwrap_or_default();
        let Ok(def) = data.mob(&code) else {
            // The blank token has no definition, and no wisdom.
            return Ok(());
        };
        if def.wisdom > 0 {
            self.grant_influence(def.wisdom, 0);
            let params = [("name", json!(def.name)), ("count", json!(def.wisdom))];
            self.log(ActorRef::Sauron, "combat.undefeated_influence", &params);
        }
        Ok(())
    }

    /// A fought monster token is discarded, defeated or not; a minion leaves the map
    /// only when defeated, the Ringwraiths coming back later (§ 8.3, § 8.8).
    #[tracing::instrument(level = "debug", skip_all, fields(mob_id), err(level = "debug"))]
    fn remove_fought_mob(&mut self, mob_id: i64) -> Result<(), EngineError> {
        let index = self.mob_index(mob_id)?;
        let Some(mob) = self.mobs.get(index) else {
            return Ok(());
        };
        match mob.kind {
            MobKind::Monster => self.discard_monster(mob_id)?,
            MobKind::Minion if mob.life <= 0 => {
                if mob.code == RINGWRAITHS {
                    self.board.ringwraiths_return = true;
                }
                self.mobs.remove(index);
            }
            MobKind::Minion => {}
        }
        Ok(())
    }
}
