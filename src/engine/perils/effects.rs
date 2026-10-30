//! Effects of Peril cards, up to their first decision (§ 6.5).

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::choices::{PASS, available_option, plain_option, target, target_option};
use crate::engine::engine_error::EngineError;
use crate::engine::hero::damage::hand_targets;
use crate::engine::threats::Threat;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::game_data::peril::{PerilCode, PerilEffect};
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceOption, ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::combat::CombatOrigin;
use crate::model::game::Game;

/// Damage of "The Hounds of Sauron".
pub const HOUNDS_DAMAGE: u32 = 6;
/// Damage of the cards offering damage or another loss.
pub const PERIL_DAMAGE: u32 = 5;
/// Favor lost or gained by Peril cards.
pub const PERIL_FAVOR: i32 = 2;
/// Influence Peril cards add to the Shadow Pool.
const SHADOW_POOL_GAIN: i32 = 2;
/// Cards the hero discards in "Trapped".
const TRAPPED_DISCARD: usize = 2;
/// Locations of "It's a Dangerous Business".
const DANGEROUS_BUSINESS_LOCATIONS: usize = 3;
/// Monster fought in "Armies Underground".
pub const ORC: &str = "orc";
/// Monster fought in "Khazad-dûm".
pub const BALROG: &str = "balrog";

/// Option codes of Peril decisions.
pub mod option {
    /// Discard hand cards against the hounds' damage.
    pub const DISCARD_SHIELDS: &str = "discard_shields";
    /// Gain favor.
    pub const FAVOR: &str = "favor";
    /// Receive training.
    pub const TRAINING: &str = "training";
    /// Be dealt the card's damage.
    pub const DAMAGE: &str = "damage";
    /// Let Sauron draw cards.
    pub const SAURON_DRAWS: &str = "sauron_draws";
    /// Advance the corruption (black) marker.
    pub const ADVANCE_CORRUPTION: &str = "advance_corruption";
    /// Lose favor.
    pub const LOSE_FAVOR: &str = "lose_favor";
    /// Receive a Corruption card picked by Sauron.
    pub const CORRUPTION: &str = "corruption";
    /// Sauron gives one of the drawn Corruption cards.
    pub const GIVE_CORRUPTION: &str = "give_corruption";
    /// Start the combat.
    pub const FIGHT: &str = "fight";
    /// Sauron picks the fighting minion.
    pub const MINION: &str = "minion";
    /// Discard hand cards.
    pub const DISCARD_CARDS: &str = "discard_cards";
    /// Sauron places influence.
    pub const PLACE_INFLUENCE: &str = "place_influence";
    /// Sauron places a monster token.
    pub const PLACE_MONSTER: &str = "place_monster";
    /// Sauron commands a minion to move.
    pub const MOVE_MINION: &str = "move_minion";
    /// Sauron draws a Plot card.
    pub const DRAW_PLOT: &str = "draw_plot";
}

impl Game {
    /// Resolves a Peril card for the hero, up to its first decision.
    ///
    /// # Returns
    /// Where play goes on when no decision is pending.
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn start_peril(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let go_on = Resume::Exploration;
        let effect = data.peril(card)?.effect;
        let options = match effect {
            PerilEffect::HoundsOfSauron
            | PerilEffect::UnexpectedBetrayal
            | PerilEffect::WeaknessOfMen
            | PerilEffect::IllMetCompany => self.hero_dilemma(hero_id, effect)?,
            PerilEffect::ArmiesUnderground | PerilEffect::KhazadDum => {
                self.peril_monster(hero_id, effect, data, rng)?
            }
            PerilEffect::Trapped => match self.trapped(hero_id)? {
                Some(options) => options,
                None => return Ok(go_on),
            },
            PerilEffect::DarkTower => return self.dark_tower(hero_id, card, data, rng),
            PerilEffect::DangerousBusiness | PerilEffect::UndyingRegret => {
                self.peril_placement(hero_id, card, effect, data)?;
                return Ok(go_on);
            }
            _ => return self.immediate_peril(hero_id, card, effect, data, rng),
        };
        self.ask_peril(hero_id, card, Chooser::Hero(hero_id), options, vec![])?;
        Ok(go_on)
    }

    /// Options of the Peril cards where the hero picks between two evils.
    fn hero_dilemma(
        &self,
        hero_id: i64,
        effect: PerilEffect,
    ) -> Result<Vec<ChoiceOption>, EngineError> {
        let hero = self.hero(hero_id)?;
        Ok(match effect {
            PerilEffect::HoundsOfSauron => {
                let hand = hand_targets(hero_id, hero);
                let max = hand.len();
                vec![target_option(option::DISCARD_SHIELDS, hand, 0, max)]
            }
            PerilEffect::UnexpectedBetrayal => vec![
                plain_option(option::DAMAGE),
                plain_option(option::SAURON_DRAWS),
            ],
            PerilEffect::WeaknessOfMen => vec![
                plain_option(option::DAMAGE),
                plain_option(option::ADVANCE_CORRUPTION),
            ],
            _ => {
                let lose = (hero.favor >= PERIL_FAVOR).then(|| plain_option(option::LOSE_FAVOR));
                lose.into_iter()
                    .chain([plain_option(option::CORRUPTION)])
                    .collect()
            }
        })
    }

    /// "Armies Underground" and "Khazad-dûm": a monster (not a token of the pools)
    /// appears on the hero's location to fight him.
    fn peril_monster(
        &mut self,
        hero_id: i64,
        effect: PerilEffect,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Vec<ChoiceOption>, EngineError> {
        let monster = if effect == PerilEffect::KhazadDum {
            BALROG
        } else {
            self.armies_underground_discards(hero_id, rng)?;
            ORC
        };
        let location = self.hero(hero_id)?.location.clone();
        self.create_mob(monster, &location, None, data)?;
        Ok(vec![plain_option(option::FIGHT)])
    }

    /// "Trapped": 2 influence to the Shadow Pool, the hero discards 2 hand cards (all
    /// of them, without a choice, when he holds no more).
    fn trapped(&mut self, hero_id: i64) -> Result<Option<Vec<ChoiceOption>>, EngineError> {
        self.add_to_shadow_pool(SHADOW_POOL_GAIN);
        let hero = self.hero_mut(hero_id)?;
        if hero.hand.len() <= TRAPPED_DISCARD {
            let hand = hero.hand.clone();
            hero.hand_to_rest(&hand)?;
            return Ok(None);
        }
        let hand = hand_targets(hero_id, hero);
        Ok(Some(vec![target_option(
            option::DISCARD_CARDS,
            hand,
            TRAPPED_DISCARD,
            TRAPPED_DISCARD,
        )]))
    }

    /// "It's a Dangerous Business" and "Undying Regret": 2 influence to the Shadow
    /// Pool, then Sauron may place influence in the hero's region.
    fn peril_placement(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        effect: PerilEffect,
        data: &GameData,
    ) -> Result<(), EngineError> {
        self.add_to_shadow_pool(SHADOW_POOL_GAIN);
        let max = if effect == PerilEffect::DangerousBusiness {
            DANGEROUS_BUSINESS_LOCATIONS
        } else {
            1
        };
        let locations = self.hero_region_targets(hero_id, false, data)?;
        let Some(place) = available_option(option::PLACE_INFLUENCE, locations, 0, max) else {
            return Ok(());
        };
        self.ask_peril(hero_id, card, Chooser::Sauron, vec![place], vec![])
    }

    /// Peril cards resolving without a decision of their own.
    fn immediate_peril(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        effect: PerilEffect,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let go_on = Resume::Exploration;
        let hero = self.hero(hero_id)?;
        let (wisdom, location) = (hero.wisdom, hero.location.clone());
        match effect {
            PerilEffect::ShadowInMemories => {
                let influenced = data
                    .paths
                    .connected(&location)
                    .into_iter()
                    .filter(|l| self.board.influence.get(*l).is_some_and(|c| *c > 0))
                    .count();
                if usize::try_from(wisdom).unwrap_or(0) > influenced {
                    return Ok(go_on);
                }
                self.hero_mut(hero_id)?.favor_loss(1);
                self.peril_corruption(hero_id, go_on, data, rng)
            }
            PerilEffect::NineForMortalMen => self.nine_for_mortal_men(hero_id, card, data, rng),
            PerilEffect::OrcScouts => {
                let resume = self.peril_corruption(hero_id, go_on, data, rng)?;
                self.shuffle_perils_back(rng);
                Ok(resume)
            }
            PerilEffect::HillOfBlackMagic => {
                let plots = i32::try_from(self.board.plots.len()).unwrap_or(i32::MAX);
                if wisdom > plots {
                    return Ok(go_on);
                }
                self.deal_damage(hero_id, PERIL_DAMAGE, Resume::EndTurn, data, rng)?;
                Ok(Resume::EndTurn)
            }
            PerilEffect::EnduringEvil => {
                if self.board.influence.get(&location).is_none_or(|c| *c < 1) {
                    return Ok(go_on);
                }
                self.peril_corruption(hero_id, Resume::EndTurn, data, rng)
            }
            _ => Ok(go_on),
        }
    }

    /// "Nine for Mortal Men": a Corruption card, then Sauron commands a minion or
    /// draws a Plot card.
    fn nine_for_mortal_men(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let go_on = Resume::Exploration;
        if self.peril_corruption(hero_id, go_on, data, rng)? == Resume::EndTurn {
            return Ok(Resume::EndTurn);
        }
        let moves = self.minion_moves(data)?;
        let options = available_option(option::MOVE_MINION, moves, 1, 1)
            .into_iter()
            .chain([plain_option(option::DRAW_PLOT), plain_option(PASS)])
            .collect();
        self.ask_peril(hero_id, card, Chooser::Sauron, options, vec![])?;
        Ok(go_on)
    }

    /// "The Dark Tower": the minion there fights the hero, Sauron picking among several.
    fn dark_tower(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let here = [self.hero(hero_id)?.location.clone()];
        let minions = self.mob_targets(&here, MobKind::Minion);
        match minions.as_slice() {
            [] => {}
            [only] => {
                let mob_id = target_id(only)?;
                self.attack_hero(hero_id, mob_id, CombatOrigin::Peril { card }, data, rng)?;
            }
            _ => {
                let options = vec![target_option(option::MINION, minions, 1, 1)];
                self.ask_peril(hero_id, card, Chooser::Sauron, options, vec![])?;
            }
        }
        Ok(Resume::Exploration)
    }

    /// Asks a decision of a Peril card.
    pub(crate) fn ask_peril(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        chooser: Chooser,
        options: Vec<ChoiceOption>,
        drawn: Vec<u32>,
    ) -> Result<(), EngineError> {
        self.ask(PendingChoice {
            chooser,
            source: ChoiceSource::Peril {
                hero_id,
                card,
                drawn,
            },
            options,
            resume: Resume::Exploration,
        })
    }

    /// The hero receives a Corruption card from a Peril card.
    ///
    /// # Returns
    /// [`Resume::EndTurn`] when the card was "Wretched", `resume` otherwise.
    pub(crate) fn peril_corruption(
        &mut self,
        hero_id: i64,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let card = self.gain_random_corruption(hero_id, data, rng)?;
        let params = [("corruption", json!(data.corruption_card(card)?.name))];
        self.log(ActorRef::Hero(hero_id), "corruption.gain", &params);
        if self.end_turn_if_wretched(hero_id, card, data)? {
            return Ok(Resume::EndTurn);
        }
        Ok(resume)
    }

    /// "Armies Underground": one random hand card discarded per Corruption card.
    fn armies_underground_discards(
        &mut self,
        hero_id: i64,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        let mut hand = hero.hand.clone();
        hand.shuffle(rng);
        hand.truncate(hero.corruption_cards.len());
        hero.hand_to_rest(&hand)
    }

    /// Locations of the hero's region where Sauron may place influence or monsters:
    /// not Havens, and not the hero's location; monsters also avoid every hero.
    pub(crate) fn hero_region_targets(
        &self,
        hero_id: i64,
        monsters: bool,
        data: &GameData,
    ) -> Result<Vec<String>, EngineError> {
        let here = self.hero(hero_id)?.location.clone();
        let region = data.locations.get(&here)?.region_code.clone();
        Ok(data
            .locations
            .iter()
            .filter(|(code, l)| l.region_code == region && !l.haven && **code != here)
            .filter(|(code, _)| !monsters || !self.heroes.iter().any(|h| h.location == **code))
            .map(|(code, _)| target("location", code))
            .collect())
    }

    /// Every move a minion command allows (`minion_to:<id>:<location>`).
    fn minion_moves(&self, data: &GameData) -> Result<Vec<String>, EngineError> {
        let mut moves = Vec::new();
        for mob in self.mobs.iter().filter(|m| m.kind == MobKind::Minion) {
            let Some(id) = mob.id else { continue };
            for to in self.mob_destinations(id, data)? {
                moves.push(format!("minion_to:{id}:{to}"));
            }
        }
        Ok(moves)
    }

    /// A combat started by a Peril card: what the hero gains for winning it.
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn reward_peril_victory(
        &mut self,
        hero_id: i64,
        card: PerilCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        match data.peril(card)?.effect {
            PerilEffect::DarkTower => {
                self.hero_mut(hero_id)?.gain_favor(PERIL_FAVOR, data);
            }
            PerilEffect::KhazadDum => {
                self.hero_mut(hero_id)?.gain_favor(PERIL_FAVOR, data);
                self.train(hero_id, Resume::Exploration, rng)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Starts the combat a Peril card asked for.
    pub(crate) fn start_peril_combat(
        &mut self,
        hero_id: i64,
        mob_id: i64,
        card: PerilCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Threat, EngineError> {
        self.attack_hero(hero_id, mob_id, CombatOrigin::Peril { card }, data, rng)
    }
}

/// Whether the hero's turn goes on after failing to defeat the mob of a Peril combat.
pub fn peril_combat_continues(effect: PerilEffect) -> bool {
    matches!(
        effect,
        PerilEffect::DarkTower | PerilEffect::NineForMortalMen
    )
}

/// Id in a `kind:<id>[:...]` target.
pub(crate) fn target_id(code: &str) -> Result<i64, EngineError> {
    code.split(':')
        .nth(1)
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| EngineError::Refused(format!("bad target {code}")))
}
