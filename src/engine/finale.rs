//! The Finale (§ 9): missions revealed, preparation, and the fight of the heroes'
//! champion against the Ringwraiths that decides the game.

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::choices::{target, target_option};
use crate::engine::combat::start::start_combat;
use crate::engine::engine_error::EngineError;
use crate::engine::mobs::{RINGWRAITHS, RINGWRAITHS_HOME};
use crate::engine::perils::effects::target_id;
use crate::engine::story::{Dominance, FINAL_SPACE};
use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::board::Winner;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::combat::CombatOrigin;
use crate::model::game::Game;
use crate::model::mob::Mob;

/// Option code of picking the champion.
pub const CHAMPION: &str = "champion";

/// The Ringwraiths keep at least this health, whatever the heroes' lead.
const MIN_RINGWRAITHS_HEALTH: i32 = 1;

impl Game {
    /// Whether the Finale begins at the end of the Story step: a marker reached the
    /// Finale space, or Sauron's three markers reached The Shadow Falls (§ 5.3, § 9).
    pub fn finale_triggered(&self) -> bool {
        let story = self.board.story.data();
        story.heroes_to_final <= 0 || story.sauron_to_final <= 0 || story.sauron_to_shadowfall <= 0
    }

    /// Plays the Finale up to the champion choice (§ 9): a side may win at once by its
    /// Mission; otherwise fulfilled Missions advance their markers, the Ringwraiths and
    /// the heroes get ready, the Ringwraiths are adjusted by dominance and the heroes
    /// pick their champion.
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn start_finale(&mut self, data: &GameData, rng: &mut StdRng) -> Result<(), EngineError> {
        self.log(ActorRef::System, "finale.start", &[]);
        let sauron = self.sauron_mission_fulfilled();
        let heroes = self.heroes_mission_fulfilled();
        let dominance = self.dominance(data)?;
        if let Some(winner) = self.reveal_missions(dominance, sauron, heroes) {
            return self.end_game(winner);
        }
        self.advance_for_missions(sauron, heroes);
        self.prepare_finale(data, rng)?;
        self.adjust_ringwraiths(data)?;
        self.ask_champion()
    }

    /// Step 1: the dominant side reveals its Mission and wins if it is fulfilled; with
    /// no dominant side both reveal, and only a single fulfilled Mission wins.
    fn reveal_missions(
        &mut self,
        dominance: Dominance,
        sauron: bool,
        heroes: bool,
    ) -> Option<Winner> {
        if dominance != Dominance::Heroes {
            let params = [
                ("mission", json!(self.sauron_mission())),
                ("fulfilled", json!(sauron)),
            ];
            self.log(ActorRef::Sauron, "finale.sauron_mission", &params);
        }
        if dominance != Dominance::Sauron {
            let params = [
                ("mission", json!(self.heroes_mission())),
                ("fulfilled", json!(heroes)),
            ];
            self.log(ActorRef::System, "finale.heroes_mission", &params);
        }
        match (dominance, sauron, heroes) {
            (Dominance::Sauron, true, _) | (Dominance::Nobody, true, false) => Some(Winner::Sauron),
            (Dominance::Heroes, _, true) | (Dominance::Nobody, false, true) => Some(Winner::Heroes),
            _ => None,
        }
    }

    /// Step 2: a fulfilled Mission advances its side's markers one space, except a
    /// marker already on the Finale space.
    fn advance_for_missions(&mut self, sauron: bool, heroes: bool) {
        let step = |position: &mut i32| {
            if *position < FINAL_SPACE {
                *position += 1;
            }
        };
        let story = &mut self.board.story;
        if sauron {
            step(&mut story.ring);
            step(&mut story.conquest);
            step(&mut story.corruption);
        }
        if heroes {
            step(&mut story.heroes);
        }
        for (side, advanced) in [("sauron", sauron), ("heroes", heroes)] {
            if advanced {
                self.log(ActorRef::System, "finale.advance", &[("side", json!(side))]);
            }
        }
    }

    /// Step 3: the Ringwraiths lose their damage (they come back first if defeated);
    /// each hero shuffles hand, rest and damage pools into his life pool and draws his
    /// fortitude.
    fn prepare_finale(&mut self, data: &GameData, rng: &mut StdRng) -> Result<(), EngineError> {
        match self.mobs.iter_mut().find(|m| m.code == RINGWRAITHS) {
            Some(ringwraiths) => ringwraiths.life = ringwraiths.max_life,
            None => {
                self.board.ringwraiths_return = false;
                self.create_mob(RINGWRAITHS, RINGWRAITHS_HOME, None, data)?;
            }
        }
        for hero in &mut self.heroes {
            hero.life_pool.append(&mut hero.hand);
            hero.heal(rng);
            let fortitude = usize::try_from(hero.fortitude).unwrap_or(0);
            hero.draw_cards(fortitude.min(hero.room_in_hand(data)));
        }
        Ok(())
    }

    /// Step 4: the Ringwraiths gain health and fortitude equal to the heroes' distance
    /// to the Finale when Sauron is dominant; they lose Sauron's distance (the lower of
    /// his closest marker to the Finale and his markers to The Shadow Falls) when the
    /// heroes are.
    fn adjust_ringwraiths(&mut self, data: &GameData) -> Result<(), EngineError> {
        let story = self.board.story.data();
        let change = match self.dominance(data)? {
            Dominance::Sauron => story.heroes_to_final.max(0),
            Dominance::Heroes => -story.sauron_to_final.min(story.sauron_to_shadowfall).max(0),
            Dominance::Nobody => 0,
        };
        let ringwraiths = self.ringwraiths_mut()?;
        ringwraiths.max_life = (ringwraiths.max_life + change).max(MIN_RINGWRAITHS_HEALTH);
        ringwraiths.life = ringwraiths.max_life;
        ringwraiths.fortitude = (ringwraiths.fortitude + change).max(0);
        let params = [
            ("life", json!(ringwraiths.life)),
            ("fortitude", json!(ringwraiths.fortitude)),
        ];
        self.log(ActorRef::Sauron, "finale.ringwraiths", &params);
        Ok(())
    }

    /// Step 5: the heroes, who already drew, pick their champion.
    fn ask_champion(&mut self) -> Result<(), EngineError> {
        let heroes = self
            .heroes
            .iter()
            .filter_map(|h| h.id)
            .map(|id| target("hero", id))
            .collect();
        self.ask(PendingChoice {
            chooser: Chooser::Heroes,
            source: ChoiceSource::Champion,
            options: vec![target_option(CHAMPION, heroes, 1, 1)],
            resume: Resume::CombatSetup,
        })
    }

    /// Step 6: the champion picked becomes the current hero and fights the Ringwraiths
    /// with the standard combat rules; no Shadow card is played (no player turn).
    pub(crate) fn apply_champion(
        &mut self,
        targets: &[String],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let picked = targets
            .first()
            .ok_or_else(|| EngineError::Refused("pick a champion".to_owned()))?;
        let hero_id = target_id(picked)?;
        let hero = self.hero(hero_id)?.clone();
        let mut combat = start_combat(&hero, self.ringwraiths_mut()?, data, rng)?;
        combat.origin = CombatOrigin::Finale;
        self.combat = Some(combat);
        self.board.current_hero_id = Some(hero_id);
        let name = data.hero(&hero.code)?.name.clone();
        self.log(
            ActorRef::Hero(hero_id),
            "finale.champion",
            &[("hero", json!(name))],
        );
        Ok(())
    }

    /// Step 7: the heroes win when their champion defeated the Ringwraiths, Sauron
    /// otherwise.
    ///
    /// # Errors
    /// Transition errors.
    pub(crate) fn finish_finale(&mut self, ringwraiths_defeated: bool) -> Result<(), EngineError> {
        self.end_game(if ringwraiths_defeated {
            Winner::Heroes
        } else {
            Winner::Sauron
        })
    }

    fn end_game(&mut self, winner: Winner) -> Result<(), EngineError> {
        self.board.winner = Some(winner);
        self.set_heroes_active(false);
        self.sauron.active = false;
        self.log(
            ActorRef::System,
            "finale.winner",
            &[("winner", json!(winner))],
        );
        self.fire(BoardEvent::FinishGame)
    }

    fn ringwraiths_mut(&mut self) -> Result<&mut Mob, EngineError> {
        self.mobs
            .iter_mut()
            .find(|m| m.code == RINGWRAITHS)
            .ok_or_else(|| EngineError::Refused("the Ringwraiths are not in play".to_owned()))
    }
}

#[cfg(test)]
mod tests;
