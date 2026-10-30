//! Effects of Shadow cards, up to their first decision (§ 5.9).

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::choices::{available_option, plain_option, target, target_option};
use crate::engine::engine_error::EngineError;
use crate::engine::hero::damage::{Dealt, hand_targets};
use crate::game_data::GameData;
use crate::game_data::corruption_card::Immediate;
use crate::game_data::mob_def::MobKind;
use crate::game_data::plot::StoryType;
use crate::game_data::shadow::{ShadowCode, ShadowEffect};
use crate::model::actor_ref::ActorRef;
use crate::model::choice::{ChoiceOption, ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::combat::CombatModifiers;
use crate::model::game::Game;

/// Hand size "Storms of Mordor" brings heroes down to.
pub const STORMS_HAND: usize = 5;
/// Damage of "Dark Promises".
pub const DARK_PROMISES_DAMAGE: u32 = 5;
/// Damage per Corruption card of the first "Morgul-blade" option.
pub const MORGUL_DAMAGE_PER_CORRUPTION: u32 = 5;
/// Damage of the second "Morgul-blade" option.
pub const MORGUL_DAMAGE: u32 = 3;
/// Cards "Betrayed" discards.
const BETRAYED_DISCARD: usize = 2;
/// Base damage of "Evil Wound", and the damage added per Corruption card.
const EVIL_WOUND_DAMAGE: (u32, u32) = (4, 2);
/// Influence "A New Power Is Rising" puts on each location of Mordor.
pub const MORDOR_INFLUENCE: i32 = 2;
/// Monster tokens "A New Power Is Rising" may place.
const NEW_POWER_MONSTERS: usize = 2;
/// Region of "A New Power Is Rising".
pub const MORDOR: &str = "mordor";
/// Location of "Shire, Baggins".
pub const THE_SHIRE: &str = "the_shire";
/// Plot card fetched by "The Ithil-stone": "Denethor Falls into Madness".
const DENETHOR_PLOT: u32 = 8;
/// Plot card fetched by "The Ithil-stone": "Saruman Falls to Corruption".
const SARUMAN_PLOT: u32 = 6;
/// Monster of "Flocks of Crebain".
const CREBAIN: &str = "crebain";
/// Monster of "Seem Fairer, Feel Fouler".
const AGENT: &str = "agent";
/// Monsters of "Creatures of the Wild", and the Corruption cards making it the second.
const CREATURES: (&str, &str, usize) = ("giant_spider", "cave_troll", 2);

/// Option codes of Shadow card decisions.
pub mod option {
    /// Fight the monster the card brought.
    pub const FIGHT: &str = "fight";
    /// Receive a Corruption card.
    pub const CORRUPTION: &str = "corruption";
    /// Be dealt the card's damage.
    pub const DAMAGE: &str = "damage";
    /// "Morgul-blade": 5 damage per Corruption card.
    pub const DAMAGE_PER_CORRUPTION: &str = "damage_per_corruption";
    /// "Morgul-blade": 3 damage and a Corruption card.
    pub const DAMAGE_AND_CORRUPTION: &str = "damage_and_corruption";
    /// Move the chosen hero to an adjacent location.
    pub const MOVE_HERO: &str = "move_hero";
    /// Choose a hero without moving him or placing influence.
    pub const CHOOSE_HERO: &str = "choose_hero";
    /// Choose a hero and place 1 influence on his location.
    pub const INFLUENCE_ON_HERO: &str = "influence_on_hero";
    /// Place 2 influence on each location of Mordor.
    pub const MORDOR_INFLUENCE: &str = "mordor_influence";
    /// Place monster tokens.
    pub const PLACE_MONSTERS: &str = "place_monsters";
    /// Discard hand cards.
    pub const DISCARD_CARDS: &str = "discard_cards";
    /// Leave the plot card on top of the deck.
    pub const PLOT_TOP: &str = "plot_top";
    /// Put the plot card under the deck.
    pub const PLOT_BOTTOM: &str = "plot_bottom";
    /// Take a discarded card into Sauron's hand.
    pub const TAKE_CARD: &str = "take_card";
    /// Take the plot card into Sauron's hand.
    pub const TAKE_PLOT: &str = "take_plot";
    /// Play the plot card at once.
    pub const PLAY_PLOT: &str = "play_plot";
    /// Pick the minion.
    pub const PICK_MINION: &str = "pick_minion";
    /// Place the minion.
    pub const PLACE_MINION: &str = "place_minion";
    /// Choose the two heroes sharing their Corruption cards.
    pub const CHOOSE_HEROES: &str = "choose_heroes";
    /// Corruption cards going to the first hero.
    pub const REDISTRIBUTE: &str = "redistribute";
    /// The hero receiving a Corruption card.
    pub const CORRUPT_HERO: &str = "corrupt_hero";
}

/// A decision of a Shadow card to ask.
#[derive(Debug)]
pub(crate) struct ShadowAsk {
    /// The Shadow card.
    pub card: ShadowCode,
    /// The hero the card is played against.
    pub hero_id: Option<i64>,
    /// Who answers.
    pub chooser: Chooser,
    /// The possible answers.
    pub options: Vec<ChoiceOption>,
    /// Heroes still concerned.
    pub heroes: Vec<i64>,
    /// Targets picked before.
    pub picked: Vec<String>,
}

impl Game {
    /// Resolves a Shadow card, up to its first decision.
    ///
    /// # Arguments
    /// * `hero_id` - the hero the card is played against (none in the Action step).
    /// * `resume` - where play goes on once the card is resolved.
    ///
    /// # Returns
    /// Where play goes on when no decision is pending.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when a card needing a hero has none; lookup and
    /// transition errors.
    #[tracing::instrument(level = "debug", skip_all, fields(?card, ?hero_id, ?resume), err(level = "debug"))]
    pub fn start_shadow(
        &mut self,
        card: ShadowCode,
        hero_id: Option<i64>,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let effect = data.shadow(card)?.effect;
        let Some(hero_id) = hero_id else {
            return self.start_action_shadow(card, effect, resume, data, rng);
        };
        match effect {
            ShadowEffect::FlocksOfCrebain
            | ShadowEffect::SeemFairer
            | ShadowEffect::CreaturesOfTheWild => {
                self.shadow_monster(card, hero_id, effect, data)?
            }
            ShadowEffect::WinterStorm => self.board.turn_effects.single_move = true,
            ShadowEffect::ArmGrownLong => {
                let location = self.hero(hero_id)?.location.clone();
                self.board.turn_effects.perilous_location = Some(location);
            }
            ShadowEffect::GrievousLoss => self.grievous_loss(hero_id)?,
            ShadowEffect::EvilWound => {
                let (base, per_card) = EVIL_WOUND_DAMAGE;
                let damage = base + per_card * self.corruption_count(hero_id)?;
                return self.shadow_damage(hero_id, damage, resume, data, rng);
            }
            _ => return self.start_combat_shadow(card, hero_id, effect, resume, data, rng),
        }
        Ok(resume)
    }

    /// Cards played at the start of a combat.
    #[tracing::instrument(level = "debug", skip_all, fields(?card, hero_id, ?effect, ?resume), err(level = "debug"))]
    fn start_combat_shadow(
        &mut self,
        card: ShadowCode,
        hero_id: i64,
        effect: ShadowEffect,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let (chooser, options) = match effect {
            ShadowEffect::Betrayed | ShadowEffect::BlackBreath => {
                return self.weaken_for_combat(hero_id, effect, resume, data, rng);
            }
            ShadowEffect::DarkPromises => (
                Chooser::Hero(hero_id),
                vec![
                    plain_option(option::CORRUPTION),
                    plain_option(option::DAMAGE),
                ],
            ),
            ShadowEffect::MorgulBlade => (
                Chooser::Sauron,
                vec![
                    plain_option(option::DAMAGE_PER_CORRUPTION),
                    plain_option(option::DAMAGE_AND_CORRUPTION),
                ],
            ),
            _ => return Ok(resume),
        };
        self.ask_shadow(
            ShadowAsk {
                card,
                hero_id: Some(hero_id),
                chooser,
                options,
                heroes: vec![],
                picked: vec![],
            },
            resume,
        )?;
        Ok(resume)
    }

    /// "Betrayed" (2 random cards discarded) and "The Black Breath" (1 Corruption card):
    /// then -1 strength (and agility for the Black Breath) per Corruption card.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ?effect, ?resume), err(level = "debug"))]
    fn weaken_for_combat(
        &mut self,
        hero_id: i64,
        effect: ShadowEffect,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let breath = effect == ShadowEffect::BlackBreath;
        let resume = if breath {
            self.shadow_corruption(hero_id, resume, data, rng)?
        } else {
            self.discard_random(hero_id, BETRAYED_DISCARD, rng)?;
            resume
        };
        let malus = self.corruption_malus(hero_id)?;
        self.combat_modifiers(|m| {
            m.strength -= malus;
            if breath {
                m.agility -= malus;
            }
        });
        Ok(resume)
    }

    /// Cards bringing a monster the hero fights at the start of his turn.
    #[tracing::instrument(level = "debug", skip_all, fields(?card, hero_id, ?effect), err(level = "debug"))]
    fn shadow_monster(
        &mut self,
        card: ShadowCode,
        hero_id: i64,
        effect: ShadowEffect,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let (spider, troll, corrupted) = CREATURES;
        let monster = match effect {
            ShadowEffect::FlocksOfCrebain => CREBAIN,
            ShadowEffect::SeemFairer => AGENT,
            _ if self.hero(hero_id)?.corruption_cards.len() >= corrupted => troll,
            _ => spider,
        };
        let location = self.hero(hero_id)?.location.clone();
        self.create_mob(monster, &location, None, data)?;
        // The hero answers so that the new monster is saved (and gets its id) first.
        self.ask_shadow(
            ShadowAsk {
                card,
                hero_id: Some(hero_id),
                chooser: Chooser::Hero(hero_id),
                options: vec![plain_option(option::FIGHT)],
                heroes: vec![],
                picked: vec![target("monster_code", monster)],
            },
            Resume::Rest,
        )
    }

    /// "Grievous Loss": influence per Corruption card, the green marker held back.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id), err(level = "debug"))]
    fn grievous_loss(&mut self, hero_id: i64) -> Result<(), EngineError> {
        let count = i32::try_from(self.corruption_count(hero_id)?).unwrap_or(i32::MAX);
        self.grant_influence(count, count);
        self.board.turn_effects.heroes_story_malus += 1;
        self.log(
            ActorRef::Sauron,
            "shadow.grievous_loss",
            &[("count", json!(count))],
        );
        Ok(())
    }

    /// Cards played during Sauron's Action step.
    #[tracing::instrument(level = "debug", skip_all, fields(?card, ?effect, ?resume), err(level = "debug"))]
    fn start_action_shadow(
        &mut self,
        card: ShadowCode,
        effect: ShadowEffect,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let mut ask = ShadowAsk {
            card,
            hero_id: None,
            chooser: Chooser::Sauron,
            options: vec![],
            heroes: vec![],
            picked: vec![],
        };
        match effect {
            ShadowEffect::EvilFog => ask.options = self.evil_fog_options(data)?,
            ShadowEffect::DoNotTemptMe => ask.options = self.tempt_options(data)?,
            ShadowEffect::NewPowerRising => ask.options = self.new_power_options(data),
            ShadowEffect::StormsOfMordor => {
                let heroes = self.heroes.iter().filter_map(|h| h.id).collect();
                return self.storms_of_mordor(card, heroes, resume, rng);
            }
            ShadowEffect::ShireBaggins => {
                self.shire_baggins(data);
                return Ok(resume);
            }
            _ => return self.start_search_shadow(card, effect, resume, data, rng),
        }
        self.ask_shadow(ask, resume)?;
        Ok(resume)
    }

    /// Action step cards looking at decks, piles and the board.
    #[tracing::instrument(level = "debug", skip_all, fields(?card, ?effect, ?resume), err(level = "debug"))]
    fn start_search_shadow(
        &mut self,
        card: ShadowCode,
        effect: ShadowEffect,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let (options, picked) = match effect {
            ShadowEffect::SarumansLies => match self.top_plot_card(rng) {
                Some(plot) => (
                    vec![
                        plain_option(option::PLOT_TOP),
                        plain_option(option::PLOT_BOTTOM),
                    ],
                    vec![target("plot_card", plot)],
                ),
                None => (vec![], vec![]),
            },
            ShadowEffect::LordOfTheRings => (self.lord_of_the_rings_options(card), vec![]),
            ShadowEffect::IthilStone => (self.ithil_stone_options(data)?, vec![]),
            ShadowEffect::TheyAreTerrible => (self.terrible_minion_options(), vec![]),
            ShadowEffect::LidlessEye => (self.lidless_eye_options(), vec![]),
            _ => (vec![], vec![]),
        };
        if options.is_empty() {
            return self.finish_search_shadow(effect, resume);
        }
        self.ask_shadow(
            ShadowAsk {
                card,
                hero_id: None,
                chooser: Chooser::Sauron,
                options,
                heroes: vec![],
                picked,
            },
            resume,
        )?;
        Ok(resume)
    }

    /// The last part of a search card when there was nothing to pick.
    #[tracing::instrument(level = "debug", skip_all, fields(?effect, ?resume), err(level = "debug"))]
    fn finish_search_shadow(
        &mut self,
        effect: ShadowEffect,
        resume: Resume,
    ) -> Result<Resume, EngineError> {
        match effect {
            ShadowEffect::SarumansLies => self.sarumans_lies_marker(),
            ShadowEffect::LordOfTheRings => self.dark_messenger_marker(),
            _ => {}
        }
        Ok(resume)
    }

    /// "An Evil Fog": a hero and an adjacent location, or a hero who stays.
    fn evil_fog_options(&self, data: &GameData) -> Result<Vec<ChoiceOption>, EngineError> {
        let mut moves = Vec::new();
        for hero in &self.heroes {
            tracing::trace!(hero = %hero.code, "hero move options");
            let Some(id) = hero.id else { continue };
            for to in data.paths.connected(&hero.location) {
                tracing::trace!(%to, "hero destination");
                moves.push(format!("hero_to:{id}:{to}"));
            }
        }
        let heroes = self.hero_targets();
        Ok(available_option(option::MOVE_HERO, moves, 1, 1)
            .into_iter()
            .chain(available_option(option::CHOOSE_HERO, heroes, 1, 1))
            .collect())
    }

    /// "Do Not Tempt Me!": a hero, with influence on his location outside Havens.
    fn tempt_options(&self, data: &GameData) -> Result<Vec<ChoiceOption>, EngineError> {
        let mut outside_havens = Vec::new();
        for hero in &self.heroes {
            tracing::trace!(hero = %hero.code, location = %hero.location, "haven check");
            if let (Some(id), false) = (hero.id, data.locations.get(&hero.location)?.haven) {
                outside_havens.push(target("hero", id));
            }
        }
        Ok(
            available_option(option::INFLUENCE_ON_HERO, outside_havens, 1, 1)
                .into_iter()
                .chain(available_option(
                    option::CHOOSE_HERO,
                    self.hero_targets(),
                    1,
                    1,
                ))
                .collect(),
        )
    }

    /// "A New Power Is Rising": influence in Mordor, or up to 2 monster tokens.
    fn new_power_options(&self, data: &GameData) -> Vec<ChoiceOption> {
        let spots = self
            .monster_placement_locations(data)
            .into_iter()
            .map(|l| target("location", l))
            .collect();
        [plain_option(option::MORDOR_INFLUENCE)]
            .into_iter()
            .chain(available_option(
                option::PLACE_MONSTERS,
                spots,
                1,
                NEW_POWER_MONSTERS,
            ))
            .collect()
    }

    /// "Shire, Baggins": influence in the Shadow Pool and the Shire, then the yellow
    /// marker advances 2 spaces with 3 influence or a minion there.
    #[tracing::instrument(level = "debug", skip_all)]
    fn shire_baggins(&mut self, data: &GameData) {
        self.add_to_shadow_pool(1);
        self.add_influence(THE_SHIRE, 1, data);
        let influence = self.board.influence.get(THE_SHIRE).copied().unwrap_or(0);
        let minion = self
            .mobs
            .iter()
            .any(|m| m.kind == MobKind::Minion && m.location == THE_SHIRE);
        if influence >= 3 || minion {
            self.advance_marker(StoryType::Ring, 2);
        }
    }

    /// "Storms of Mordor" for the heroes left, in order: each discards down to 5 cards
    /// (choosing which), then a random card per Corruption card.
    #[tracing::instrument(level = "debug", skip_all, fields(?card, ?heroes, ?resume), err(level = "debug"))]
    pub(crate) fn storms_of_mordor(
        &mut self,
        card: ShadowCode,
        mut heroes: Vec<i64>,
        resume: Resume,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        while !heroes.is_empty() {
            tracing::trace!(left = heroes.len(), "storm: next hero");
            let hero_id = heroes.remove(0);
            let hero = self.hero(hero_id)?;
            let excess = hero.hand.len().saturating_sub(STORMS_HAND);
            if excess > 0 {
                let hand = hand_targets(hero_id, hero);
                let options = vec![target_option(option::DISCARD_CARDS, hand, excess, excess)];
                self.ask_shadow(
                    ShadowAsk {
                        card,
                        hero_id: Some(hero_id),
                        chooser: Chooser::Hero(hero_id),
                        options,
                        heroes,
                        picked: vec![],
                    },
                    resume,
                )?;
                return Ok(resume);
            }
            let count = self.hero(hero_id)?.corruption_cards.len();
            self.discard_random(hero_id, count, rng)?;
        }
        Ok(resume)
    }

    /// The top card of the Plot deck, rebuilt from its discard pile when empty.
    #[tracing::instrument(level = "debug", skip_all)]
    fn top_plot_card(&mut self, rng: &mut StdRng) -> Option<u32> {
        let pile = &mut self.board.decks.plot;
        if pile.draw.is_empty() {
            pile.draw.append(&mut pile.discard);
            pile.draw.shuffle(rng);
            tracing::trace!(pile = "pile.draw", order = ?pile.draw, "shuffled");
        }
        pile.draw.first().copied()
    }

    /// "Lord of the Rings": the discarded Plot and Shadow cards, this one excepted.
    fn lord_of_the_rings_options(&self, card: ShadowCode) -> Vec<ChoiceOption> {
        let decks = &self.board.decks;
        let plots = decks.plot.discard.iter().map(|c| target("plot_card", c));
        let shadows = decks
            .shadow
            .discard
            .iter()
            .filter(|c| **c != card)
            .map(|c| target("shadow", c));
        let mut cards: Vec<String> = plots.chain(shadows).collect();
        cards.dedup();
        available_option(option::TAKE_CARD, cards, 1, 1)
            .into_iter()
            .collect()
    }

    /// "The Ithil-stone": the Denethor and Saruman plots found in the deck or discard
    /// pile, to take or, when Sauron could play them now, to play.
    fn ithil_stone_options(&self, data: &GameData) -> Result<Vec<ChoiceOption>, EngineError> {
        let plot = &self.board.decks.plot;
        let found: Vec<u32> = [DENETHOR_PLOT, SARUMAN_PLOT]
            .into_iter()
            .filter(|c| plot.draw.contains(c) || plot.discard.contains(c))
            .collect();
        let take = found.iter().map(|c| target("plot_card", c)).collect();
        let mut play = Vec::new();
        for card in &found {
            tracing::trace!(?card, "plot card found");
            if !self.free_plot_slots().is_empty() && self.plot_playable(*card, data)? {
                play.push(target("plot_card", card));
            }
        }
        Ok(available_option(option::TAKE_PLOT, take, 1, 1)
            .into_iter()
            .chain(available_option(option::PLAY_PLOT, play, 1, 1))
            .collect())
    }

    /// "They Are Terrible": minions on the board and defeated ones.
    fn terrible_minion_options(&self) -> Vec<ChoiceOption> {
        let on_board = self
            .mobs
            .iter()
            .filter(|m| m.kind == MobKind::Minion)
            .filter_map(|m| m.id)
            .map(|id| target("minion", id));
        let defeated = self
            .defeated_minions()
            .into_iter()
            .map(|code| target("defeated_minion", code));
        let minions = on_board.chain(defeated).collect();
        available_option(option::PICK_MINION, minions, 1, 1)
            .into_iter()
            .collect()
    }

    /// "The Lidless Eye": two heroes to share their Corruption cards, or (with one
    /// hero) the hero receiving a Corruption card.
    pub(crate) fn lidless_eye_options(&self) -> Vec<ChoiceOption> {
        let heroes = self.hero_targets();
        if heroes.len() >= 2 {
            vec![target_option(option::CHOOSE_HEROES, heroes, 2, 2)]
        } else {
            available_option(option::CORRUPT_HERO, heroes, 1, 1)
                .into_iter()
                .collect()
        }
    }

    /// "Saruman's Lies": the black marker advances when a hero holds as many
    /// Corruption cards as his wisdom.
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn sarumans_lies_marker(&mut self) {
        let corrupted = self
            .heroes
            .iter()
            .any(|h| i32::try_from(h.corruption_cards.len()).unwrap_or(i32::MAX) >= h.wisdom);
        if corrupted {
            self.advance_marker(StoryType::Corruption, 1);
        }
    }

    /// "Lord of the Rings": the yellow marker advances while "A Dark Messenger" is
    /// active.
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn dark_messenger_marker(&mut self) {
        const DARK_MESSENGER: u32 = 3;
        if self.board.plots.values().any(|c| *c == DARK_MESSENGER) {
            self.advance_marker(StoryType::Ring, 1);
        }
    }

    /// Advances one of Sauron's markers because of a Shadow card.
    #[tracing::instrument(level = "debug", skip_all, fields(?marker, spaces))]
    pub(crate) fn advance_marker(&mut self, marker: StoryType, spaces: i32) {
        self.board.story.advance(marker, spaces);
        let params = [("marker_name", json!(marker)), ("count", json!(spaces))];
        self.log(ActorRef::Sauron, "story.advance", &params);
    }

    /// Asks a decision of a Shadow card.
    #[tracing::instrument(level = "debug", skip_all, fields(?ask, ?resume), err(level = "debug"))]
    pub(crate) fn ask_shadow(&mut self, ask: ShadowAsk, resume: Resume) -> Result<(), EngineError> {
        self.ask(PendingChoice {
            chooser: ask.chooser,
            source: ChoiceSource::Shadow {
                card: ask.card,
                hero_id: ask.hero_id,
                heroes: ask.heroes,
                picked: ask.picked,
            },
            options: ask.options,
            resume,
        })
    }

    /// Number of Corruption cards of a hero.
    pub(crate) fn corruption_count(&self, hero_id: i64) -> Result<u32, EngineError> {
        let count = self.hero(hero_id)?.corruption_cards.len();
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    /// Attribute malus of one per Corruption card.
    fn corruption_malus(&self, hero_id: i64) -> Result<i32, EngineError> {
        Ok(i32::try_from(self.corruption_count(hero_id)?).unwrap_or(i32::MAX))
    }

    /// Changes the hero's attributes for the combat in progress.
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn combat_modifiers(&mut self, change: impl FnOnce(&mut CombatModifiers)) {
        if let Some(combat) = self.combat.as_mut() {
            change(&mut combat.modifiers);
        }
    }

    /// The hero discards `count` random hand cards to his rest pool.
    #[tracing::instrument(
        level = "debug",
        skip_all,
        fields(hero_id, count),
        err(level = "debug")
    )]
    pub(crate) fn discard_random(
        &mut self,
        hero_id: i64,
        count: usize,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let hero = self.hero_mut(hero_id)?;
        let mut hand = hero.hand.clone();
        hand.shuffle(rng);
        tracing::trace!(pile = "hand", order = ?hand, "shuffled");
        hand.truncate(count);
        hero.hand_to_rest(&hand)
    }

    /// The hero receives a random Corruption card from a Shadow card played during his
    /// turn: "Wretched" ends the turn, and the combat about to start with it.
    ///
    /// # Returns
    /// [`Resume::EndTurn`] for "Wretched", `resume` otherwise.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, ?resume), err(level = "debug"))]
    pub(crate) fn shadow_corruption(
        &mut self,
        hero_id: i64,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let card = self.gain_random_corruption(hero_id, data, rng)?;
        let params = [("corruption", json!(data.corruption_card(card)?.name))];
        self.log(ActorRef::Hero(hero_id), "corruption.gain", &params);
        if resume == Resume::SauronActions {
            return Ok(resume);
        }
        if data.corruption_card(card)?.immediate == Some(Immediate::Wretched) {
            self.abandon_combat();
        }
        if self.end_turn_if_wretched(hero_id, card, data)? {
            return Ok(Resume::EndTurn);
        }
        Ok(resume)
    }

    /// Deals damage from a Shadow card; a combat about to start is dropped when the
    /// damage defeats the hero.
    #[tracing::instrument(level = "debug", skip_all, fields(hero_id, amount, ?resume), err(level = "debug"))]
    pub(crate) fn shadow_damage(
        &mut self,
        hero_id: i64,
        amount: u32,
        resume: Resume,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<Resume, EngineError> {
        let hero = self.hero(hero_id)?;
        let cards = hero.hand.len() + hero.life_pool.len();
        if usize::try_from(amount).unwrap_or(usize::MAX) >= cards {
            self.abandon_combat();
        }
        match self.deal_damage(hero_id, amount, resume, data, rng)? {
            Dealt::Paid | Dealt::Choice => Ok(resume),
            Dealt::Defeat => Ok(Resume::EndTurn),
        }
    }

    /// Drops a combat that never started; a monster brought by a card leaves the board.
    #[tracing::instrument(level = "debug", skip_all)]
    fn abandon_combat(&mut self) {
        let Some(combat) = self.combat.take() else {
            return;
        };
        self.mobs.retain(|m| {
            m.id != Some(combat.mob_id) || m.kind == MobKind::Minion || m.pool.is_some()
        });
    }
}
