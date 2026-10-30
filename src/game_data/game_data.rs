//! All static game data, loaded once at startup.

use std::collections::BTreeMap;
use std::path::Path;

use crate::game_data::attack_deck::AttackDeck;
use crate::game_data::character::Character;
use crate::game_data::code_map::CodeMap;
use crate::game_data::combat_card_def::{CardNumber, CombatCardDef};
use crate::game_data::corruption_card::{CorruptionCard, CorruptionCards, CorruptionCode};
use crate::game_data::event::Event;
use crate::game_data::game_data_error::GameDataError;
use crate::game_data::hero_def::HeroDef;
use crate::game_data::hero_quests::{HeroQuest, HeroQuests};
use crate::game_data::location_paths::LocationPaths;
use crate::game_data::locations::Locations;
use crate::game_data::mob_def::MobDef;
use crate::game_data::monster_pool_color::MonsterPoolColor;
use crate::game_data::peril::{PerilCard, PerilCode};
use crate::game_data::plot::Plot;
use crate::game_data::yaml_loader::{load, load_map};

/// Name of the event set of a stage deck, as in `data/events.yaml` and the pictures.
pub fn event_set(stage: u8) -> &'static str {
    match stage {
        1 => "I",
        2 => "II",
        _ => "III",
    }
}

/// Quest set used by the base game.
pub const BASE_QUEST_SET: &str = "regular";

/// Every static definition of the game, by code.
#[derive(Debug, Clone)]
pub struct GameData {
    /// Heroes by code, in file order.
    pub heroes: CodeMap<String, HeroDef>,
    /// Mob attack decks by code.
    pub attack_decks: BTreeMap<String, AttackDeck>,
    /// Monsters and minions by code.
    pub mobs: CodeMap<String, MobDef>,
    /// Map locations.
    pub locations: Locations,
    /// Map graph.
    pub paths: LocationPaths,
    /// Plot cards by number.
    pub plots: BTreeMap<u32, Plot>,
    /// Event cards by set, then by number.
    pub events: BTreeMap<String, BTreeMap<u32, Event>>,
    /// Characters by code, in file order.
    pub characters: CodeMap<String, Character>,
    /// Corruption deck and cards.
    pub corruption: CorruptionCards,
    /// Hero quests.
    pub hero_quests: HeroQuests,
    /// Skill cards by number.
    pub skills: BTreeMap<CardNumber, CombatCardDef>,
    /// Starting content of each monster pool.
    pub monster_pools: BTreeMap<MonsterPoolColor, Vec<String>>,
    /// Peril cards by number.
    pub perils: BTreeMap<PerilCode, PerilCard>,
}

impl GameData {
    /// Loads every data file from `dir` (usually `data/`).
    ///
    /// # Errors
    /// Fails when a file is missing or doesn't match its expected shape.
    pub fn load(dir: &Path) -> Result<Self, GameDataError> {
        Ok(Self {
            heroes: load_map(dir, "heroes")?,
            attack_decks: load(dir, "mobs_cards")?,
            mobs: load_map(dir, "mobs")?,
            locations: Locations::new(load_map(dir, "locations")?),
            paths: LocationPaths::new(load_map(dir, "locations_paths")?),
            plots: load(dir, "plots")?,
            events: load(dir, "events")?,
            characters: load_map(dir, "characters")?,
            corruption: load(dir, "corruption_cards")?,
            hero_quests: load(dir, "heroes_quests")?,
            skills: load(dir, "skills")?,
            monster_pools: load(dir, "locations_monsters")?,
            perils: load(dir, "perils")?,
        })
    }

    /// Looks up a hero.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the code doesn't exist.
    pub fn hero(&self, code: &str) -> Result<&HeroDef, GameDataError> {
        self.heroes
            .get(code)
            .ok_or_else(|| GameDataError::unknown("hero", code))
    }

    /// Looks up a card of a hero deck, or a skill when `number` is a skill number.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the hero or the card doesn't exist.
    pub fn hero_card(
        &self,
        hero: &str,
        number: CardNumber,
    ) -> Result<&CombatCardDef, GameDataError> {
        if let Some(skill) = self.skills.get(&number) {
            return Ok(skill);
        }
        self.hero(hero)?
            .cards
            .get(&number)
            .ok_or_else(|| GameDataError::unknown("hero card", format!("{hero}/{number}")))
    }

    /// Looks up a monster or minion.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the code doesn't exist.
    pub fn mob(&self, code: &str) -> Result<&MobDef, GameDataError> {
        self.mobs
            .get(code)
            .ok_or_else(|| GameDataError::unknown("mob", code))
    }

    /// Looks up an attack deck.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the code doesn't exist.
    pub fn attack_deck(&self, code: &str) -> Result<&AttackDeck, GameDataError> {
        self.attack_decks
            .get(code)
            .ok_or_else(|| GameDataError::unknown("attack deck", code))
    }

    /// Looks up a card of an attack deck.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the deck or the card doesn't exist.
    pub fn mob_card(
        &self,
        deck: &str,
        number: CardNumber,
    ) -> Result<&CombatCardDef, GameDataError> {
        self.attack_deck(deck)?
            .cards
            .get(&number)
            .ok_or_else(|| GameDataError::unknown("mob card", format!("{deck}/{number}")))
    }

    /// Looks up a plot card.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the number doesn't exist.
    pub fn plot(&self, number: u32) -> Result<&Plot, GameDataError> {
        self.plots
            .get(&number)
            .ok_or_else(|| GameDataError::unknown("plot", number))
    }

    /// Looks up an event card of a stage deck (1 to 3).
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the card doesn't exist.
    pub fn event(&self, stage: u8, number: u32) -> Result<&Event, GameDataError> {
        self.events
            .get(event_set(stage))
            .and_then(|set| set.get(&number))
            .ok_or_else(|| GameDataError::unknown("event", format!("{stage}/{number}")))
    }

    /// Numbers of the event cards of a stage deck (1 to 3).
    pub fn event_cards(&self, stage: u8) -> Vec<u32> {
        self.events
            .get(event_set(stage))
            .map(|set| set.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Looks up a character.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the code doesn't exist.
    pub fn character(&self, code: &str) -> Result<&Character, GameDataError> {
        self.characters
            .get(code)
            .ok_or_else(|| GameDataError::unknown("character", code))
    }

    /// Looks up a Peril card.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the card doesn't exist.
    pub fn peril(&self, code: PerilCode) -> Result<&PerilCard, GameDataError> {
        self.perils
            .get(&code)
            .ok_or_else(|| GameDataError::unknown("peril card", code))
    }

    /// Looks up a corruption card.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the code doesn't exist.
    pub fn corruption_card(&self, code: CorruptionCode) -> Result<&CorruptionCard, GameDataError> {
        self.corruption
            .cards
            .get(&code)
            .ok_or_else(|| GameDataError::unknown("corruption card", code))
    }

    /// Looks up the base-game quests of a hero.
    ///
    /// # Errors
    /// [`GameDataError::UnknownCode`] when the hero has no quest.
    pub fn hero_quest(&self, hero: &str) -> Result<&HeroQuest, GameDataError> {
        self.hero_quests
            .get(BASE_QUEST_SET)
            .and_then(|set| set.get(hero))
            .ok_or_else(|| GameDataError::unknown("hero quest", hero))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_data::combat_card_def::CardPower;
    use crate::game_data::mob_def::MobKind;
    use crate::game_data::terrain::Terrain;

    fn data() -> GameData {
        GameData::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data")).unwrap()
    }

    #[test]
    fn loads_every_file() {
        let data = data();
        assert_eq!(data.heroes.len(), 5);
        let argalad = data.hero("argalad").unwrap();
        assert_eq!(argalad.start_location, "woodland_realm");
        let quick_shot = data.hero_card("argalad", 0).unwrap();
        assert_eq!(quick_shot.name, "Quick Shot");
        assert_eq!(quick_shot.movement_type, Some(Terrain::Forest));
        assert_eq!(data.mob("gothmog").unwrap().kind, MobKind::Minion);
        assert_eq!(
            data.mob_card("zealot", 10).unwrap().power,
            CardPower::Reckless
        );
        assert_eq!(
            data.mob_card("zealot", 2).unwrap().power,
            CardPower::Inactive
        );
        assert_eq!(data.plots.len(), 18);
        assert_eq!(data.plot(10).unwrap().affect, None);
        assert_eq!(
            data.paths
                .path("barad_dur", "mount_doom")
                .unwrap()
                .difficulty,
            2
        );
        assert_eq!(data.corruption.deck, (0..16).collect::<Vec<_>>());
        assert!(
            data.corruption
                .deck
                .iter()
                .all(|c| data.corruption_card(*c).is_ok())
        );
        assert_eq!(data.perils.len(), 15);
        assert_eq!(data.monster_pools[&MonsterPoolColor::Orange].len(), 10);
        assert!(data.skills.contains_key(&100));
    }

    #[test]
    fn every_card_in_a_deck_has_an_implemented_power() {
        let data = data();
        for (code, hero) in data.heroes.iter() {
            for card in &hero.starting_deck {
                let def = data.hero_card(code, *card).unwrap();
                assert_ne!(def.power, CardPower::Inactive, "{code}/{card}");
            }
        }
        for (code, deck) in &data.attack_decks {
            for card in &deck.deck {
                let def = data.mob_card(code, *card).unwrap();
                assert_ne!(def.power, CardPower::Inactive, "{code}/{card}");
            }
        }
    }

    // Port of game_data_events_test.rb, for the three stage decks.
    #[test]
    fn events_reference_existing_characters_and_locations() {
        let data = data();
        for stage in 1..=3 {
            assert_eq!(data.event_cards(stage).len(), 14);
            for number in data.event_cards(stage) {
                let event = data.event(stage, number).unwrap();
                if let Some(character) = &event.character {
                    assert!(data.character(&character.name).is_ok(), "{number}");
                    assert!(data.locations.exists(&character.location), "{number}");
                }
                let locations = event.favors.iter().chain(event.affect.as_ref());
                for location in locations {
                    assert!(data.locations.exists(location), "{number}: {location}");
                }
            }
        }
    }

    #[test]
    fn every_location_reaches_a_haven() {
        let data = data();
        let havens: Vec<&str> = data
            .locations
            .iter()
            .filter(|(_, l)| l.haven)
            .map(|(code, _)| code.as_str())
            .collect();
        for (code, _) in data.locations.iter() {
            let distances = data.paths.distances_from(code);
            assert!(havens.iter().any(|h| distances.contains_key(h)), "{code}");
        }
    }
}
