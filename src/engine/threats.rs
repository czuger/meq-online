//! Monsters, minions and perils threatening a hero on its location: the Ambush step
//! and the "Combat or Peril" part of the Travel step (§ 6.4, § 6.5, § 8).

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::choices::{PASS, parse_target, plain_option, target_option};
use crate::engine::combat::start::start_combat;
use crate::engine::engine_error::EngineError;
use crate::engine::shadows::shadow_window::ShadowWindow;
use crate::game_data::GameData;
use crate::game_data::mob_def::MobKind;
use crate::game_data::shadow::ShadowTiming;
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::combat::CombatOrigin;
use crate::model::game::Game;
use crate::model::mob::NOTHING_CODE;
use crate::model::turn_step::{StepSkip, TurnStep};

/// Option code of a monster or minion attack.
pub const ATTACK: &str = "attack";
/// Option code of a Peril.
pub const PERIL: &str = "peril";
/// Option code of a hero allowing combat in a Haven.
pub const ALLOW_COMBAT: &str = "allow_combat";

/// What a threat check led to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Threat {
    /// Nothing happened (or a blank token was discarded): play goes on.
    None,
    /// A combat started.
    Combat,
    /// A player must decide on the choice screen.
    Choice,
}

impl Game {
    /// Tells whether the hero's location is perilous for him: printed so, holding
    /// more influence than his wisdom (§ 6.2, § 6.5), or made so by "His Arm Has Grown
    /// Long".
    ///
    /// # Errors
    /// Lookup errors.
    pub fn is_perilous(&self, hero_id: i64, data: &GameData) -> Result<bool, EngineError> {
        let hero = self.hero(hero_id)?;
        let influence = self
            .board
            .influence
            .get(&hero.location)
            .copied()
            .unwrap_or(0);
        let shadowed = self.board.turn_effects.perilous_location.as_ref() == Some(&hero.location);
        Ok(data.locations.get(&hero.location)?.perilous || influence > hero.wisdom || shadowed)
    }

    /// Monster tokens and minions on the hero's location, as choice targets.
    fn threatening_mobs(&self, hero_id: i64) -> Result<Vec<String>, EngineError> {
        let here = [self.hero(hero_id)?.location.clone()];
        let mut mobs = self.mob_targets(&here, MobKind::Monster);
        mobs.extend(self.mob_targets(&here, MobKind::Minion));
        Ok(mobs)
    }

    /// Tells whether a monster or minion stands on the hero's location.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub(crate) fn has_threatening_mobs(&self, hero_id: i64) -> Result<bool, EngineError> {
        Ok(!self.threatening_mobs(hero_id)?.is_empty())
    }

    /// Checks what threatens the hero on its location. At the Ambush step only
    /// monsters and minions count; while travelling a perilous location may also bring
    /// a Peril, Sauron choosing between them. In a Haven the hero first allows combat
    /// or not.
    ///
    /// # Errors
    /// Lookup and transition errors.
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(hero_id, ambush),
        err(level = "debug")
    )]
    pub fn face_location_threats(
        &mut self,
        hero_id: i64,
        ambush: bool,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Threat, EngineError> {
        let mobs = self.threatening_mobs(hero_id)?;
        let perilous = !ambush && self.is_perilous(hero_id, data)?;
        if mobs.is_empty() && !perilous {
            if !ambush {
                let location = self.hero(hero_id)?.location.clone();
                let params = [("location", json!(data.locations.name(&location)))];
                self.log(ActorRef::Hero(hero_id), "movement.safe", &params);
            }
            return Ok(Threat::None);
        }
        let haven = data.locations.get(&self.hero(hero_id)?.location)?.haven;
        if haven && !mobs.is_empty() {
            self.ask(PendingChoice {
                chooser: Chooser::Hero(hero_id),
                source: ChoiceSource::HavenCombat { hero_id, ambush },
                options: vec![plain_option(ALLOW_COMBAT), plain_option(PASS)],
                resume: threat_resume(ambush),
            })?;
            return Ok(Threat::Choice);
        }
        self.sauron_threatens(hero_id, ambush, mobs, perilous, data, rng)
    }

    /// Sauron resolves an attack or a Peril; he only decides when there is a choice.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ambush, ?mobs, perilous), err(level = "debug"))]
    fn sauron_threatens(
        &mut self,
        hero_id: i64,
        ambush: bool,
        mobs: Vec<String>,
        perilous: bool,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Threat, EngineError> {
        let origin = if ambush {
            CombatOrigin::Ambush
        } else {
            CombatOrigin::Travel
        };
        match (mobs.as_slice(), perilous) {
            ([], true) => self.draw_perils(hero_id, data, rng),
            ([only], false) => {
                let mob_id = mob_id(only)?;
                self.attack_hero(hero_id, mob_id, origin, data, rng)
            }
            _ => {
                let mut options = vec![target_option(ATTACK, mobs, 1, 1)];
                if perilous {
                    options.push(plain_option(PERIL));
                }
                self.ask(PendingChoice {
                    chooser: Chooser::Sauron,
                    source: ChoiceSource::Threat { hero_id, ambush },
                    options,
                    resume: threat_resume(ambush),
                })?;
                Ok(Threat::Choice)
            }
        }
    }

    /// A monster or minion attacks the hero. A blank monster token is discarded
    /// instead and the hero goes on (§ 6.5). Sauron may play a Shadow card before the
    /// combat (§ 5.9).
    ///
    /// # Errors
    /// [`EngineError::CombatInProgress`]; lookup and transition errors.
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(hero_id, mob_id),
        err(level = "debug")
    )]
    pub fn attack_hero(
        &mut self,
        hero_id: i64,
        mob_id: i64,
        origin: CombatOrigin,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Threat, EngineError> {
        let mob_index = self.mob_index(mob_id)?;
        let code = self
            .mobs
            .get(mob_index)
            .map(|m| m.code.clone())
            .unwrap_or_default();
        if code == NOTHING_CODE {
            let location = self.hero(hero_id)?.location.clone();
            self.discard_monster(mob_id)?;
            let hero_name = data.hero(&self.hero(hero_id)?.code)?.name.clone();
            let params = [("location", json!(location)), ("hero", json!(hero_name))];
            self.log(ActorRef::Hero(hero_id), "combat.encounter_nothing", &params);
            return Ok(Threat::None);
        }
        if self.combat.is_some() {
            return Err(EngineError::CombatInProgress);
        }
        let hero_index = self.hero_index(hero_id)?;
        let (Some(hero), Some(mob)) = (self.heroes.get(hero_index), self.mobs.get_mut(mob_index))
        else {
            return Err(EngineError::NotFound {
                kind: "mob",
                id: mob_id,
            });
        };
        let mut combat = start_combat(hero, mob, data, rng)?;
        combat.origin = origin;
        let params = [
            ("hero", json!(hero.code)),
            ("mob", json!(mob.name)),
            ("location", json!(data.locations.name(&mob.location))),
        ];
        self.combat = Some(combat);
        self.log(ActorRef::Hero(hero_id), "combat.start", &params);
        let window = ShadowWindow {
            timing: ShadowTiming::CombatStart,
            hero_id,
        };
        if self.open_shadow_window(window, Resume::CombatSetup, data)? {
            return Ok(Threat::Choice);
        }
        self.fire(BoardEvent::NextToCombatSetupScreenBoardCombats)?;
        self.activate_current_hero()?;
        Ok(Threat::Combat)
    }

    /// The hero in a Haven allowed combat (Sauron then threatens as usual) or not.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ambush, %option), err(level = "debug"))]
    pub(crate) fn apply_haven_choice(
        &mut self,
        hero_id: i64,
        ambush: bool,
        option: &str,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if option != ALLOW_COMBAT {
            if ambush {
                self.log_step_skipped(TurnStep::Ambush, StepSkip::HavenProtects);
            }
            return Ok(());
        }
        let mobs = self.threatening_mobs(hero_id)?;
        self.sauron_threatens(hero_id, ambush, mobs, false, data, rng)?;
        Ok(())
    }

    /// Sauron's pick: the attacking monster or minion, or a Peril.
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ambush, %option, ?targets), err(level = "debug"))]
    pub(crate) fn apply_threat_choice(
        &mut self,
        hero_id: i64,
        ambush: bool,
        option: &str,
        targets: &[String],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if option == PERIL {
            self.draw_perils(hero_id, data, rng)?;
            return Ok(());
        }
        let picked = targets
            .first()
            .ok_or_else(|| EngineError::Refused("choose the attacker".to_owned()))?;
        let origin = if ambush {
            CombatOrigin::Ambush
        } else {
            CombatOrigin::Travel
        };
        self.attack_hero(hero_id, mob_id(picked)?, origin, data, rng)?;
        Ok(())
    }
}

/// Where play goes on when a threat ends without combat: the Rest step after the
/// Ambush step, exploration after a move.
fn threat_resume(ambush: bool) -> Resume {
    if ambush {
        Resume::Rest
    } else {
        Resume::Exploration
    }
}

/// Mob id of a `monster:<id>` or `minion:<id>` target.
fn mob_id(target: &str) -> Result<i64, EngineError> {
    let (_, values) = parse_target(target);
    values
        .first()
        .and_then(|id| id.parse().ok())
        .ok_or_else(|| EngineError::Refused(format!("{target} is not a monster or minion")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board_state::BoardState;
    use crate::engine::choices::target;
    use crate::engine::test_support::{data, game, mob, rng};

    fn travelling_to(location: &str) -> Game {
        let mut g = game();
        g.board.state = BoardState::HeroMovementScreen;
        g.heroes[0].location = location.to_owned();
        g
    }

    fn mob_at(g: &mut Game, code: &str, id: i64, location: &str) {
        let mut m = mob(code);
        m.id = Some(id);
        m.location = location.to_owned();
        g.mobs.push(m);
    }

    // § 6.4, § 6.5: Sauron chooses which monster or minion attacks.
    #[test]
    fn sauron_chooses_the_attacker() {
        let mut g = travelling_to("bree");
        mob_at(&mut g, "orc", 1, "bree");
        mob_at(&mut g, "cave_troll", 2, "bree");
        let threat = g.face_location_threats(1, false, data(), &mut rng());
        assert_eq!(threat.unwrap(), Threat::Choice);
        let choice = g.board.pending_choice.clone().unwrap();
        assert_eq!(choice.chooser, Chooser::Sauron);
        assert_eq!(choice.options.len(), 1);
        g.answer_choice(ATTACK, &[target("monster", 1)], data(), &mut rng())
            .unwrap();
        assert_eq!(g.combat.as_ref().unwrap().mob_id, 1);
        assert_eq!(g.board.state, BoardState::CombatSetupScreenBoardCombats);
    }

    // § 6.5: on a perilous location with a monster, Sauron chooses combat or peril.
    #[test]
    fn sauron_chooses_combat_or_peril() {
        let mut g = travelling_to("the_ruins_of_angmar");
        mob_at(&mut g, "orc", 1, "the_ruins_of_angmar");
        g.face_location_threats(1, false, data(), &mut rng())
            .unwrap();
        let options = &g.board.pending_choice.as_ref().unwrap().options;
        assert_eq!(options.len(), 2);
        g.answer_choice(PERIL, &[], data(), &mut rng()).unwrap();
        assert!(g.combat.is_none());
        assert!(matches!(
            g.board.pending_choice.as_ref().unwrap().source,
            ChoiceSource::PerilDraw { .. }
        ));
    }

    // § 6.4: a lone monster ambushes without a choice; no Peril at the Ambush step.
    #[test]
    fn lone_monster_ambushes_at_once() {
        let mut g = travelling_to("the_ruins_of_angmar");
        g.board.state = BoardState::HeroDrawCardsScreen;
        mob_at(&mut g, "orc", 1, "the_ruins_of_angmar");
        let threat = g.face_location_threats(1, true, data(), &mut rng());
        assert_eq!(threat.unwrap(), Threat::Combat);
        assert_eq!(g.combat.as_ref().unwrap().origin, CombatOrigin::Ambush);
    }

    // § 8: no combat in a Haven unless the hero allows it.
    #[test]
    fn hero_refuses_combat_in_a_haven() {
        let haven = data()
            .locations
            .iter()
            .find(|(_, l)| l.haven)
            .map(|(c, _)| c.clone())
            .unwrap();
        let mut g = travelling_to(&haven);
        mob_at(&mut g, "orc", 1, &haven);
        g.face_location_threats(1, false, data(), &mut rng())
            .unwrap();
        assert_eq!(
            g.board.pending_choice.as_ref().unwrap().chooser,
            Chooser::Hero(1)
        );
        g.answer_choice(PASS, &[], data(), &mut rng()).unwrap();
        assert!(g.combat.is_none());
        assert_eq!(g.board.state, BoardState::HeroExploration);

        let mut g = travelling_to(&haven);
        mob_at(&mut g, "orc", 1, &haven);
        g.face_location_threats(1, false, data(), &mut rng())
            .unwrap();
        g.answer_choice(ALLOW_COMBAT, &[], data(), &mut rng())
            .unwrap();
        assert!(g.combat.is_some());
    }

    // § 6.5: a perilous location is printed so or holds more influence than wisdom.
    #[test]
    fn perilous_locations() {
        let mut g = travelling_to("bree");
        assert!(!g.is_perilous(1, data()).unwrap());
        g.board.influence.insert("bree".to_owned(), 3);
        assert!(g.is_perilous(1, data()).unwrap());
        g.heroes[0].location = "the_ruins_of_angmar".to_owned();
        assert!(g.is_perilous(1, data()).unwrap());
    }
}
