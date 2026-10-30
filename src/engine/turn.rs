//! Hero order, end of turns and the switch to Sauron.

use serde_json::json;

use crate::engine::board_state::BoardEvent;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::hero::TurnRecord;

/// Where the hero who ended its turn goes next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnEnd {
    /// Back to the board list (another actor plays).
    Boards,
    /// Single-hero game: the same hero starts its second turn.
    HeroDrawCards,
}

/// Minions entering the map when the story reaches a stage: `(stage, mob, location)`.
const STAGE_SPAWNS: [(i32, &str, &str); 3] = [
    (2, "gothmog", "mount_gundabad"),
    (2, "ringwraiths", "minas_morgul"),
    (3, "witch_king", "minas_morgul"),
];

impl Game {
    /// Makes the hero with the lowest playing order the current hero and starts a round.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the board has no hero.
    pub fn set_first_hero_to_play(&mut self) -> Result<(), EngineError> {
        let first = self
            .heroes
            .iter()
            .min_by_key(|h| h.playing_order)
            .and_then(|h| h.id)
            .ok_or_else(|| EngineError::Refused("no hero on the board".to_owned()))?;
        self.board.current_hero_id = Some(first);
        self.activate_current_hero()?;
        for hero in &mut self.heroes {
            hero.turn_finished = false;
        }
        Ok(())
    }

    /// Ends the turn of `hero_id`: next hero, second turn, or Sauron's turn.
    ///
    /// # Errors
    /// Fails when the hero is unknown or a transition isn't allowed now.
    pub fn finish_heroes_turn(
        &mut self,
        hero_id: i64,
        data: &GameData,
    ) -> Result<TurnEnd, EngineError> {
        self.hero_end_turn_operations(hero_id)?;
        if self.heroes.len() > 1 {
            if !self.switch_to_next_hero()? {
                self.actions_before_switch_to_sauron(data)?;
            }
            return Ok(TurnEnd::Boards);
        }
        if self.start_hero_second_turn(hero_id, data)? {
            Ok(TurnEnd::HeroDrawCards)
        } else {
            Ok(TurnEnd::Boards)
        }
    }

    /// Resets the per-turn record of a hero.
    ///
    /// # Errors
    /// [`EngineError::NotFound`] when the hero is unknown.
    pub fn hero_end_turn_operations(&mut self, hero_id: i64) -> Result<(), EngineError> {
        self.hero_mut(hero_id)?.turn_record = TurnRecord::default();
        Ok(())
    }

    /// In a single-hero game the hero plays two turns per round, with a special Hero
    /// Rally step and Hero Draw step in between (§ 10.1).
    ///
    /// # Returns
    /// `true` when the second turn starts, `false` when the round ended.
    fn start_hero_second_turn(
        &mut self,
        hero_id: i64,
        data: &GameData,
    ) -> Result<bool, EngineError> {
        let hero = self.hero_mut(hero_id)?;
        if hero.turn == 2 {
            hero.turn = 1;
            self.actions_before_switch_to_sauron(data)?;
            return Ok(false);
        }
        hero.turn = 2;
        hero.cards_drawn_this_round = 0;
        self.rally_step(data);
        self.activate_current_hero()?;
        self.fire(BoardEvent::NextToFinishHeroTurn)?;
        self.fire(BoardEvent::NextToHeroDrawCardsScreen)?;
        Ok(true)
    }

    /// Passes the turn to the next hero that hasn't played this round.
    ///
    /// # Returns
    /// `false` when every hero has played.
    fn switch_to_next_hero(&mut self) -> Result<bool, EngineError> {
        let current = self.current_hero_id()?;
        self.hero_mut(current)?.turn_finished = true;

        let next = self
            .heroes
            .iter()
            .filter(|h| !h.turn_finished)
            .min_by_key(|h| h.playing_order)
            .and_then(|h| h.id);
        let Some(next) = next else { return Ok(false) };

        self.board.current_hero_id = Some(next);
        self.activate_current_hero()?;
        self.fire(BoardEvent::NextToFinishHeroTurn)?;
        self.fire(BoardEvent::NextToHeroDrawCardsScreen)?;
        Ok(true)
    }

    /// End of the heroes' round: rally, story advance, stage spawns, then Sauron plays.
    fn actions_before_switch_to_sauron(&mut self, data: &GameData) -> Result<(), EngineError> {
        self.rally_step(data);
        self.fire(BoardEvent::NextToFinishHeroTurn)?;

        let old_stage = self.board.story.stage();
        self.advance_stories_markers(data)?;
        let new_stage = self.board.story.stage();
        if new_stage > old_stage {
            for (stage, mob, location) in STAGE_SPAWNS {
                if stage == new_stage {
                    self.create_mob(mob, location, None, data)?;
                }
            }
        }

        if self.board.winner.is_none() {
            self.fire(BoardEvent::NextToPlayScreenSauronPlotCards)?;
            self.switch_to_sauron()?;
            self.board.turn += 1;
        }
        Ok(())
    }

    /// Heroes remove the influence where they stand, except on a Shadow Stronghold;
    /// influence cut from the strongholds is reduced to what they can still support
    /// (§ 5.2, § 5.8).
    fn rally_step(&mut self, data: &GameData) {
        let on_stronghold = |location: &str| {
            data.locations
                .get(location)
                .is_ok_and(|l| l.shadow_fortress)
        };
        let hero_spots: Vec<(String, String)> = self
            .heroes
            .iter()
            .filter(|h| !on_stronghold(&h.location))
            .map(|h| {
                (
                    data.heroes
                        .get(&h.code)
                        .map_or_else(|| h.code.clone(), |d| d.name.clone()),
                    h.location.clone(),
                )
            })
            .collect();
        for (hero, location) in hero_spots {
            let count = self.board.influence.get(&location).copied().unwrap_or(0);
            if count > 0 {
                self.board.influence.remove(&location);
                let params = [
                    ("hero", json!(hero)),
                    ("location", json!(data.locations.name(&location))),
                    ("count", json!(count)),
                ];
                self.log(ActorRef::System, "rally.hero_remove", &params);
            }
        }
        self.trim_unsupported_influence(data);
    }
}
