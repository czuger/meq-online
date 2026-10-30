//! Abilities of event cards resolved during the Event step (§ 5.6).

use rand::rngs::StdRng;
use serde_json::json;

use crate::engine::choices::{
    PASS, available_option, parse_target, plain_option, target, target_option,
};
use crate::engine::deck::SauronDeck;
use crate::engine::engine_error::EngineError;
use crate::engine::mobs::RINGWRAITHS;
use crate::game_data::GameData;
use crate::game_data::event::{Area, EventAbility, TemptationRefusal, TemptationReward};
use crate::game_data::mob_def::MobKind;
use crate::model::actor_ref::ActorRef;
use crate::model::board::{EventRef, HeldEvent};
use crate::model::choice::{ChoiceOption, ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;

/// Minion near whom "Warrior of the Lidless Eye" places tokens.
const GOTHMOG: &str = "gothmog";

/// Locations of an area.
pub fn area_locations(area: &Area, data: &GameData) -> Vec<String> {
    match area {
        Area::Region(region) => data
            .locations
            .iter()
            .filter(|(_, l)| l.region_code == *region)
            .map(|(code, _)| code.clone())
            .collect(),
        Area::Near(location) => std::iter::once(location.as_str())
            .chain(data.paths.connected(location))
            .map(str::to_owned)
            .collect(),
    }
}

impl Game {
    /// Heroes in playing order.
    pub fn heroes_in_order(&self) -> Vec<i64> {
        let mut heroes: Vec<_> = self.heroes.iter().collect();
        heroes.sort_by_key(|h| h.playing_order);
        heroes.iter().filter_map(|h| h.id).collect()
    }

    /// One target per influence token on `locations` (`influence:<location>:<n>`).
    pub fn influence_targets(&self, locations: &[String]) -> Vec<String> {
        locations
            .iter()
            .flat_map(|l| {
                let count = self.board.influence.get(l).copied().unwrap_or(0);
                (1..=count).map(move |n| format!("influence:{l}:{n}"))
            })
            .collect()
    }

    /// Monster tokens (or minions) on `locations`.
    pub fn mob_targets(&self, locations: &[String], kind: MobKind) -> Vec<String> {
        let prefix = match kind {
            MobKind::Monster => "monster",
            MobKind::Minion => "minion",
        };
        self.mobs
            .iter()
            .filter(|m| m.kind == kind && locations.contains(&m.location))
            .filter_map(|m| m.id.map(|id| target(prefix, id)))
            .collect()
    }

    /// Characters on `locations`, or every character of the game when `None`.
    fn character_targets(&self, locations: Option<&[String]>, data: &GameData) -> Vec<String> {
        match locations {
            Some(locations) => self
                .board
                .characters
                .iter()
                .filter(|(_, l)| locations.contains(l))
                .map(|(c, _)| target("character", c))
                .collect(),
            None => data
                .characters
                .iter()
                .map(|(c, _)| target("character", c))
                .collect(),
        }
    }

    /// Removes the influence, monster token and favor targets picked; influence cut from
    /// its strongholds is then reduced (§ 5.8).
    pub fn remove_token_targets(
        &mut self,
        targets: &[String],
        data: &GameData,
    ) -> Result<(), EngineError> {
        for code in targets {
            match parse_target(code) {
                ("influence", values) => {
                    if let Some(location) = values.first() {
                        self.remove_influence(location, 1);
                    }
                }
                ("monster", values) => {
                    if let Some(id) = values.first().and_then(|v| v.parse().ok()) {
                        self.discard_monster(id)?;
                    }
                }
                ("favor", values) => {
                    if let Some(location) = values.first()
                        && let Some(i) = self.board.favors.iter().position(|f| f == location)
                    {
                        self.board.favors.remove(i);
                    }
                }
                _ => {}
            }
        }
        self.trim_unsupported_influence(data);
        Ok(())
    }

    /// Deals damage to a minion outside of combat; a defeated minion leaves the map.
    pub fn hurt_minion(&mut self, mob_id: i64, damage: i32) -> Result<(), EngineError> {
        let index = self.mob_index(mob_id)?;
        let Some(mob) = self.mobs.get_mut(index) else {
            return Ok(());
        };
        mob.life -= damage;
        if mob.life <= 0 {
            if mob.code == RINGWRAITHS {
                self.board.ringwraiths_return = true;
            }
            self.mobs.remove(index);
        }
        Ok(())
    }

    fn ask_event(
        &mut self,
        event: EventRef,
        chooser: Chooser,
        heroes: Vec<i64>,
        options: Vec<ChoiceOption>,
    ) -> Result<(), EngineError> {
        self.ask(PendingChoice {
            chooser,
            source: ChoiceSource::Event {
                stage: event.stage,
                card: event.card,
                heroes,
            },
            options,
            resume: Resume::ActionStep,
        })
    }

    /// Starts the ability of an event: applies it, or asks for the needed choice.
    ///
    /// # Errors
    /// Lookup and choice errors.
    pub fn start_event_ability(
        &mut self,
        event: EventRef,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let ability = data.event(event.stage, event.card)?.ability.clone();
        match ability {
            EventAbility::FavorOrCorruption => {
                let heroes = self
                    .heroes_in_order()
                    .into_iter()
                    .filter(|id| self.hero(*id).is_ok_and(|h| h.favor > 0))
                    .collect();
                self.ask_favor_or_corruption(event, heroes)
            }
            EventAbility::TrustInFriendship => self.start_trust_in_friendship(event),
            EventAbility::Temptation { .. } => {
                let heroes = self.hero_targets();
                let options = vec![target_option("choose_hero", heroes, 1, 1)];
                self.ask_event(event, Chooser::Sauron, vec![], options)
            }
            EventAbility::PlaceCharacters {
                characters,
                location,
            } => {
                for character in characters {
                    self.place_character(ActorRef::System, &character, &location, data)?;
                }
                Ok(())
            }
            EventAbility::Radagast => self.radagast(data),
            EventAbility::BoromirsDream => {
                self.boromirs_dream(data);
                Ok(())
            }
            EventAbility::Eowyn => {
                self.place_character(ActorRef::System, "theoden", "edoras", data)?;
                for location in data.paths.connected("edoras") {
                    self.remove_influence(location, 1);
                }
                self.trim_unsupported_influence(data);
                Ok(())
            }
            EventAbility::HeHasAtLastHeard => self.he_has_at_last_heard(event, data),
            EventAbility::OnExplore { .. } => Ok(()),
            ability => self.start_choice_ability(event, &ability, data),
        }
    }

    /// Abilities that only offer a choice to Sauron or to the heroes.
    fn start_choice_ability(
        &mut self,
        event: EventRef,
        ability: &EventAbility,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let (chooser, options): (Chooser, Vec<Option<ChoiceOption>>) = match ability {
            EventAbility::ClearMonstersOrHurtMinion {
                region, monsters, ..
            } => {
                let locations = area_locations(&Area::Region(region.clone()), data);
                let monster_targets = self.mob_targets(&locations, MobKind::Monster);
                let minion_targets = self.mob_targets(&locations, MobKind::Minion);
                let max = usize::try_from(*monsters).unwrap_or(0);
                let options = vec![
                    available_option("remove_monsters", monster_targets, 1, max),
                    available_option("hurt_minion", minion_targets, 1, 1),
                ];
                (Chooser::Heroes, options)
            }
            EventAbility::SauronMovesCharacter { .. } => {
                let characters = self.character_targets(None, data);
                // Sauron must choose: no pass.
                let options = vec![target_option("place", characters, 1, 1)];
                return self.ask_event(event, Chooser::Sauron, vec![], options);
            }
            EventAbility::ClearTokensOrFavor {
                area,
                max,
                per_kind,
                ..
            } => {
                let locations = area_locations(area, data);
                let remove = match max {
                    None => Some(plain_option("remove_all")),
                    Some(max) => {
                        let mut targets = self.influence_targets(&locations);
                        targets.extend(self.mob_targets(&locations, MobKind::Monster));
                        let per = usize::try_from(*max).unwrap_or(0);
                        let total = if *per_kind { per * 2 } else { per };
                        available_option("remove_tokens", targets, 1, total)
                    }
                };
                (
                    Chooser::Heroes,
                    vec![remove, Some(plain_option("place_favor"))],
                )
            }
            EventAbility::SauronDiscardsCharacterOrFavor { region } => {
                let locations = area_locations(&Area::Region(region.clone()), data);
                let favors = self
                    .board
                    .favors
                    .iter()
                    .filter(|f| locations.contains(f))
                    .enumerate()
                    .map(|(n, f)| format!("favor:{f}:{}", n + 1))
                    .collect();
                let characters = self.character_targets(Some(&locations), data);
                let options = vec![
                    available_option("discard_character", characters, 1, 1),
                    available_option("discard_favor", favors, 1, 1),
                ];
                (Chooser::Sauron, options)
            }
            EventAbility::MoveCharacterAndClear => {
                let targets = self.character_moves(data);
                (
                    Chooser::Heroes,
                    vec![available_option("move_character", targets, 1, 1)],
                )
            }
            EventAbility::SauronPlacesMonsters { region, count } => {
                let targets = area_locations(&Area::Region(region.clone()), data)
                    .into_iter()
                    .filter(|l| !self.heroes.iter().any(|h| h.location == *l))
                    .map(|l| target("location", l))
                    .collect();
                let count = usize::try_from(*count).unwrap_or(0);
                let option = available_option("place_monsters", targets, 1, count);
                (Chooser::Sauron, vec![option])
            }
            EventAbility::DiscardHavenPlot { cost } => {
                let targets = self.haven_plot_targets(*cost, data);
                (
                    Chooser::Heroes,
                    vec![available_option("discard_plot", targets, 1, 1)],
                )
            }
            EventAbility::WarriorOfTheLidlessEye => {
                if !self.mobs.iter().any(|m| m.code == GOTHMOG) {
                    return Ok(());
                }
                (Chooser::Sauron, vec![Some(plain_option("place"))])
            }
            _ => return Ok(()),
        };
        let mut options: Vec<ChoiceOption> = options.into_iter().flatten().collect();
        if options.is_empty() {
            return Ok(());
        }
        options.push(plain_option(PASS));
        self.ask_event(event, chooser, vec![], options)
    }

    /// Every hero, in playing order, as choice targets.
    pub(crate) fn hero_targets(&self) -> Vec<String> {
        self.heroes_in_order()
            .into_iter()
            .map(|id| target("hero", id))
            .collect()
    }

    /// Characters on the map and the adjacent locations, not Shadow Strongholds, they
    /// could move to (`character_to:<character>:<location>`).
    fn character_moves(&self, data: &GameData) -> Vec<String> {
        self.board
            .characters
            .iter()
            .flat_map(|(character, location)| {
                data.paths
                    .connected(location)
                    .into_iter()
                    .filter(|l| data.locations.get(l).is_ok_and(|d| !d.shadow_fortress))
                    .map(move |l| format!("character_to:{character}:{l}"))
            })
            .collect()
    }

    /// Plots affecting a Haven, with each hero able to pay (`plot:<slot>:<hero>`).
    fn haven_plot_targets(&self, cost: i32, data: &GameData) -> Vec<String> {
        let plots = self.board.plots.iter().filter(|(slot, _)| {
            self.plot_location(**slot, data)
                .and_then(|l| data.locations.get(&l).ok())
                .is_some_and(|l| l.haven)
        });
        plots
            .flat_map(|(slot, _)| {
                self.heroes
                    .iter()
                    .filter(|h| h.favor >= cost)
                    .filter_map(move |h| h.id.map(|id| format!("plot:{slot}:{id}")))
            })
            .collect()
    }

    fn ask_favor_or_corruption(
        &mut self,
        event: EventRef,
        heroes: Vec<i64>,
    ) -> Result<(), EngineError> {
        let Some(first) = heroes.first().copied() else {
            return Ok(());
        };
        let options = vec![plain_option("spend_favor"), plain_option("corruption")];
        self.ask_event(event, Chooser::Hero(first), heroes, options)
    }

    fn start_trust_in_friendship(&mut self, event: EventRef) -> Result<(), EngineError> {
        let mut favor = 0;
        for hero in &mut self.heroes {
            if hero.favor > 0 {
                hero.favor -= 1;
                favor += 1;
            }
        }
        self.board.held_event = Some(HeldEvent {
            event,
            hero_id: 0,
            favor,
        });
        let heroes = self.hero_targets();
        let options = vec![target_option("give", heroes, 1, 1)];
        self.ask_event(event, Chooser::Sauron, vec![], options)
    }

    fn radagast(&mut self, data: &GameData) -> Result<(), EngineError> {
        for hero in &mut self.heroes {
            if hero.favor >= 4 {
                hero.favor -= 1;
            }
        }
        if self.board.characters.contains_key("gandalf") {
            self.place_character(ActorRef::System, "gandalf", "isengard", data)?;
        }
        Ok(())
    }

    fn boromirs_dream(&mut self, data: &GameData) {
        for hero in &mut self.heroes {
            if hero.favor == 0 {
                hero.gain_favor(1, data);
            }
        }
        let story = self.board.story;
        if story.ring <= story.heroes.min(story.conquest).min(story.corruption) {
            story_advance(self, crate::game_data::plot::StoryType::Ring);
        }
    }

    fn he_has_at_last_heard(
        &mut self,
        event: EventRef,
        data: &GameData,
    ) -> Result<(), EngineError> {
        for location in area_locations(&Area::Near("the_shire".to_owned()), data) {
            self.add_influence(&location, 1, data);
        }
        self.trim_unsupported_influence(data);
        let minions: Vec<String> = self
            .mobs
            .iter()
            .filter(|m| m.kind == MobKind::Minion)
            .filter_map(|m| m.id.map(|id| target("minion", id)))
            .collect();
        if minions.is_empty() {
            return Ok(());
        }
        let options = vec![
            target_option("move_minion", minions, 1, 1),
            plain_option(PASS),
        ];
        self.ask_event(event, Chooser::Sauron, vec![], options)
    }

    /// Applies an answer to an event choice.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply_event_choice(
        &mut self,
        event: EventRef,
        heroes: &[i64],
        option: &str,
        targets: &[String],
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        if option == PASS {
            return Ok(());
        }
        let ability = data.event(event.stage, event.card)?.ability.clone();
        match ability {
            EventAbility::FavorOrCorruption => {
                let (hero_id, rest) = heroes
                    .split_first()
                    .ok_or_else(|| EngineError::Refused("no hero to ask".to_owned()))?;
                if option == "spend_favor" {
                    self.hero_mut(*hero_id)?.favor_loss(1);
                } else {
                    self.gain_random_corruption(*hero_id, data, rng)?;
                }
                self.ask_favor_or_corruption(event, rest.to_vec())
            }
            EventAbility::Temptation { reward, refusal } => self.apply_temptation(
                event,
                heroes,
                option,
                targets,
                (reward, refusal),
                resume,
                data,
                rng,
            ),
            ability => self.apply_target_choice(event, &ability, option, targets, data, rng),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_temptation(
        &mut self,
        event: EventRef,
        heroes: &[i64],
        option: &str,
        targets: &[String],
        (reward, refusal): (TemptationReward, TemptationRefusal),
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let chosen = || first_value(targets, "hero").and_then(|v| v.parse::<i64>().ok());
        match option {
            "choose_hero" => {
                let hero_id = chosen().ok_or_else(|| EngineError::Refused("no hero".to_owned()))?;
                let options = vec![plain_option("accept"), plain_option("refuse")];
                self.ask_event(event, Chooser::Hero(hero_id), vec![hero_id], options)
            }
            "accept" => {
                let hero_id = heroes.first().copied().unwrap_or_default();
                self.gain_random_corruption(hero_id, data, rng)?;
                match reward {
                    TemptationReward::Training => self.train(hero_id, resume, rng),
                    TemptationReward::MoveAnywhere => {
                        let locations = data.locations.iter().map(|(c, _)| target("location", c));
                        let options = vec![target_option("move", locations.collect(), 1, 1)];
                        self.ask_event(event, Chooser::Hero(hero_id), vec![hero_id], options)
                    }
                }
            }
            "move" => {
                let hero_id = heroes.first().copied().unwrap_or_default();
                if let Some(location) = first_value(targets, "location") {
                    self.hero_mut(hero_id)?.location = location.to_owned();
                }
                Ok(())
            }
            _ => {
                let deck = match refusal {
                    TemptationRefusal::Plot => SauronDeck::Plot,
                    TemptationRefusal::Shadow => SauronDeck::Shadow,
                };
                self.draw_to_sauron_hand(deck, 1, rng);
                Ok(())
            }
        }
    }

    fn apply_target_choice(
        &mut self,
        event: EventRef,
        ability: &EventAbility,
        option: &str,
        targets: &[String],
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        match (ability, option) {
            (EventAbility::ClearMonstersOrHurtMinion { damage, .. }, "hurt_minion") => {
                if let Some(id) = first_value(targets, "minion").and_then(|v| v.parse().ok()) {
                    self.hurt_minion(id, *damage)?;
                }
            }
            (EventAbility::SauronMovesCharacter { location }, _) => {
                if let Some(character) = first_value(targets, "character") {
                    self.place_character(ActorRef::Sauron, character, location, data)?;
                }
            }
            (EventAbility::TrustInFriendship, _) => {
                let hero_id = first_value(targets, "hero").and_then(|v| v.parse().ok());
                if let (Some(held), Some(hero_id)) = (self.board.held_event.as_mut(), hero_id) {
                    held.hero_id = hero_id;
                }
            }
            (EventAbility::ClearTokensOrFavor { favor, .. }, "place_favor") => {
                self.board.favors.push(favor.clone());
            }
            (EventAbility::ClearTokensOrFavor { area, per_kind, .. }, _) => {
                let targets = self.clear_targets(area, option, targets, *per_kind, data)?;
                self.remove_token_targets(&targets, data)?;
            }
            (EventAbility::SauronDiscardsCharacterOrFavor { .. }, "discard_character") => {
                if let Some(character) = first_value(targets, "character") {
                    self.remove_character(ActorRef::Sauron, character, data)?;
                }
            }
            (EventAbility::MoveCharacterAndClear, _) => {
                self.move_character_and_clear(targets, data)?
            }
            (EventAbility::SauronPlacesMonsters { .. }, _) => {
                for code in targets {
                    if let ("location", values) = parse_target(code) {
                        let location = values.first().copied().unwrap_or_default();
                        self.place_new_monster(location, data, rng)?;
                    }
                }
            }
            (EventAbility::DiscardHavenPlot { cost }, _) => {
                self.discard_haven_plot(targets, *cost)?
            }
            (EventAbility::WarriorOfTheLidlessEye, _) => {
                self.warrior_of_the_lidless_eye(data, rng)?
            }
            (EventAbility::HeHasAtLastHeard, _) => {
                if let Some(id) = first_value(targets, "minion").and_then(|v| v.parse().ok()) {
                    self.mob_mut(id)?.location = "the_shire".to_owned();
                }
            }
            // Removing monster tokens or favor tokens.
            _ => self.remove_token_targets(targets, data)?,
        }
        let params = [("event", json!(data.event(event.stage, event.card)?.name))];
        self.log(ActorRef::System, "event.resolved", &params);
        Ok(())
    }

    /// Tokens removed by "remove ... or place 1 favor" cards: every token of the area,
    /// or the picked ones (at most one of each kind when counted per kind).
    fn clear_targets(
        &self,
        area: &Area,
        option: &str,
        targets: &[String],
        per_kind: bool,
        data: &GameData,
    ) -> Result<Vec<String>, EngineError> {
        if option == "remove_all" {
            let locations = area_locations(area, data);
            let mut all = self.influence_targets(&locations);
            all.extend(self.mob_targets(&locations, MobKind::Monster));
            return Ok(all);
        }
        let count = |kind: &str| targets.iter().filter(|t| parse_target(t).0 == kind).count();
        if per_kind && (count("influence") > 1 || count("monster") > 1) {
            return Err(EngineError::Refused(
                "at most one influence and one monster token".to_owned(),
            ));
        }
        Ok(targets.to_vec())
    }

    fn move_character_and_clear(
        &mut self,
        targets: &[String],
        data: &GameData,
    ) -> Result<(), EngineError> {
        let Some(code) = targets.first() else {
            return Ok(());
        };
        let (_, values) = parse_target(code);
        let (Some(character), Some(location)) = (values.first(), values.get(1)) else {
            return Ok(());
        };
        self.place_character(ActorRef::System, character, location, data)?;
        let here = [(*location).to_owned()];
        let mut tokens = self.influence_targets(&here);
        tokens.extend(self.mob_targets(&here, MobKind::Monster));
        self.remove_token_targets(&tokens, data)
    }

    fn discard_haven_plot(&mut self, targets: &[String], cost: i32) -> Result<(), EngineError> {
        let Some(code) = targets.first() else {
            return Ok(());
        };
        let (_, values) = parse_target(code);
        let slot = values.first().and_then(|v| v.parse::<u8>().ok());
        let hero_id = values.get(1).and_then(|v| v.parse::<i64>().ok());
        let (Some(slot), Some(hero_id)) = (slot, hero_id) else {
            return Ok(());
        };
        self.hero_mut(hero_id)?.favor -= cost;
        if let Some(card) = self.board.plots.remove(&slot) {
            self.forget_plot_location(slot);
            self.board.decks.plot.discard.push(card);
        }
        Ok(())
    }

    fn warrior_of_the_lidless_eye(
        &mut self,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let Some(gothmog) = self.mobs.iter().find(|m| m.code == GOTHMOG) else {
            return Ok(());
        };
        let around: Vec<String> = data
            .paths
            .connected(&gothmog.location)
            .into_iter()
            .map(str::to_owned)
            .collect();
        for location in around {
            self.add_influence(&location, 1, data);
            if !self.heroes.iter().any(|h| h.location == location) {
                self.place_new_monster(&location, data, rng)?;
            }
        }
        self.trim_unsupported_influence(data);
        Ok(())
    }
}

/// Advances one of Sauron's markers by 1 and logs it.
fn story_advance(game: &mut Game, track: crate::game_data::plot::StoryType) {
    game.board.story.advance(track, 1);
    game.log(
        ActorRef::System,
        "story.advance",
        &[("marker_name", json!(track)), ("count", json!(1))],
    );
}

/// The first value of the first target of a kind.
fn first_value<'a>(targets: &'a [String], kind: &str) -> Option<&'a str> {
    targets.iter().find_map(|t| {
        let (k, values) = parse_target(t);
        (k == kind).then(|| values.first().copied()).flatten()
    })
}
