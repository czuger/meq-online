//! Steps of a combat around the round resolution: setup, card choice, losses, end.

use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::combat::reveal::reveal_secretly_played_cards;
use crate::engine::engine_error::EngineError;
use crate::engine::turn::TurnEnd;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::corruption_card::Flaw;
use crate::game_data::mob_def::MobKind;
use crate::model::actor_ref::ActorRef;
use crate::model::combat::Combat;
use crate::model::game::Game;

/// Minion that survives defeat by fleeing to Minas Tirith.
const RINGWRAITHS: &str = "ringwraiths";

/// The hero's choice at the start of a combat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatPreparation {
    /// Draw as many cards as agility.
    DrawCards,
    /// Add agility to strength for this combat.
    GainStrength,
}

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
    /// The mob was defeated: the hero moves on.
    Movement,
    /// The hero's turn ended.
    TurnEnded(TurnEnd),
}

impl Game {
    fn combat_ref(&self) -> Result<&Combat, EngineError> {
        self.combat.as_ref().ok_or(EngineError::NoCombat)
    }

    fn combat_ids(&self) -> Result<(i64, i64), EngineError> {
        let combat = self.combat_ref()?;
        Ok((combat.hero_id, combat.mob_id))
    }

    /// The hero prepares: a cowardly hero first discards a random card.
    ///
    /// # Errors
    /// [`EngineError::NoCombat`]; lookup and transition errors.
    pub fn combat_setup(
        &mut self,
        choice: CombatPreparation,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let (hero_id, _) = self.combat_ids()?;
        let hero = self.hero_mut(hero_id)?;
        if hero.has_flaw(Flaw::Cowardly, data) {
            if let Some(card) = hero.hand.choose(rng).copied() {
                hero.hand_to_rest(&[card])?;
            }
            self.log(ActorRef::Hero(hero_id), "corruption.cowardly", &[]);
        }

        let hero = self.hero_mut(hero_id)?;
        let mut strength = hero.strength;
        let (code, base_strength) = (hero.code.clone(), hero.strength);
        match choice {
            CombatPreparation::DrawCards => {
                let drawn = hero.draw_cards(usize::try_from(hero.agility).unwrap_or(0));
                let left = hero.life_pool.len();
                let params = [
                    ("name", json!(code)),
                    ("cards_drawn", json!(drawn)),
                    ("lp_cards", json!(left)),
                ];
                self.log(ActorRef::Hero(hero_id), "combat.draw_cards", &params);
            }
            CombatPreparation::GainStrength => {
                strength += hero.agility;
                let params = [
                    ("name", json!(code)),
                    ("str", json!(base_strength)),
                    ("nstr", json!(strength)),
                ];
                self.log(ActorRef::Hero(hero_id), "combat.inc_strength", &params);
            }
        }

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

        let hero = self.hero_mut(hero_id)?;
        let paid = usize::try_from(hero.temporary_damages)
            .unwrap_or(0)
            .min(hero.life_pool.len());
        let lost: Vec<CardNumber> = hero.life_pool.drain(..paid).collect();
        hero.temporary_damages -= i32::try_from(paid).unwrap_or(0);
        hero.damage_pool.extend(lost);
        if hero.temporary_damages <= 0 {
            return Ok(CardPlayed::Resolved);
        }
        if usize::try_from(hero.temporary_damages).unwrap_or(0)
            < hero.life_pool.len() + hero.hand.len()
        {
            self.fire(BoardEvent::NextToCardsLossScreenBoardCombats)?;
            return Ok(CardPlayed::CardsLoss);
        }
        // Not enough cards left: the hero is defeated.
        let mut everything = std::mem::take(&mut hero.life_pool);
        everything.append(&mut hero.hand);
        hero.damage_pool.extend(everything);
        Ok(CardPlayed::Resolved)
    }

    /// The hero pays the remaining damage with cards from its hand.
    ///
    /// # Returns
    /// `true` when all damage is paid and the combat goes on.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when a card isn't in hand; transition errors.
    pub fn combat_cards_loss(&mut self, selected: &[CardNumber]) -> Result<bool, EngineError> {
        let (hero_id, _) = self.combat_ids()?;
        let hero = self.hero_mut(hero_id)?;
        let required = usize::try_from(hero.temporary_damages)
            .unwrap_or(0)
            .min(selected.len());
        let paid = selected.get(..required).unwrap_or_default();
        hero.hand_to_damages(paid)?;
        hero.temporary_damages -= i32::try_from(paid.len()).unwrap_or(0);
        if hero.temporary_damages > 0 {
            return Ok(false);
        }
        self.fire(BoardEvent::NextToPlayCombatCardScreenBoardCombats)?;
        Ok(true)
    }

    /// Current combat result. A side with an empty hand becomes exhausted and, like a
    /// side out of strength, stops being expected to play.
    ///
    /// # Errors
    /// [`EngineError::NoCombat`]; lookup errors.
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

    /// Ends the combat: played cards go to the rest pool, defeated mobs leave the map
    /// and a defeated hero heals in its regional haven.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the combat isn't over; lookup and transition errors.
    pub fn terminate_combat(&mut self, data: &GameData) -> Result<CombatEnd, EngineError> {
        let result = self.refresh_combat_result()?;
        let both_exhausted = result.hero_exhausted && result.mob_exhausted;
        if !result.hero_defeated && !result.mob_defeated && !both_exhausted {
            return Err(EngineError::Refused("the combat is not over".to_owned()));
        }
        let combat = self.combat.take().ok_or(EngineError::NoCombat)?;
        let played: Vec<CardNumber> = combat.hero_cards.iter().filter_map(|c| c.card).collect();
        self.hero_mut(combat.hero_id)?.rest_pool.extend(played);
        self.remove_fought_mob(combat.mob_id)?;

        if result.hero_defeated {
            self.board.story.advance_lowest(true);
            let hero = self.hero_mut(combat.hero_id)?;
            hero.favor_loss(1);
            hero.move_to_regional_haven(data)?;
            hero.heal();
        } else if result.mob_defeated {
            self.fire(BoardEvent::NextToHeroMovementScreen)?;
            self.activate_current_hero()?;
            return Ok(CombatEnd::Movement);
        }
        self.fire(BoardEvent::NextToCardsLossScreenBoardCombats)?;
        self.fire(BoardEvent::NextToAfterDefeatAdvanceStoryMarker)?;
        Ok(CombatEnd::TurnEnded(
            self.finish_heroes_turn(combat.hero_id, data)?,
        ))
    }

    /// Monsters always leave the map after a combat; minions only when defeated.
    fn remove_fought_mob(&mut self, mob_id: i64) -> Result<(), EngineError> {
        let index = self.mob_index(mob_id)?;
        let Some(mob) = self.mobs.get_mut(index) else {
            return Ok(());
        };
        let remove = match mob.kind {
            MobKind::Monster => true,
            MobKind::Minion if mob.life <= 0 && mob.code == RINGWRAITHS => {
                mob.location = "minas_tirith".to_owned();
                false
            }
            MobKind::Minion => mob.life <= 0,
        };
        if remove {
            self.mobs.remove(index);
        }
        Ok(())
    }
}
