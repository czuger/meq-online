//! Static definition of an event card (§ 5.6).

use serde::{Deserialize, Serialize};

use crate::game_data::plot::PlotStory;

/// A character placed by an event card.
#[derive(Debug, Clone, Deserialize)]
pub struct EventCharacter {
    /// Character code.
    pub name: String,
    /// Location code.
    pub location: String,
}

/// Part of the map an ability works on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Area {
    /// Every location of a region (region code).
    Region(String),
    /// A location and the adjacent ones ("within 1 space").
    Near(String),
}

/// What a tempted hero gains when accepting a Corruption card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemptationReward {
    /// The hero receives training.
    Training,
    /// The hero moves to any location.
    MoveAnywhere,
}

/// What Sauron draws when a tempted hero refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemptationRefusal {
    /// One Plot card.
    Plot,
    /// One Shadow card.
    Shadow,
}

/// What a hero exploring the affected location of an event in play may gain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExploreReward {
    /// Remove 1 influence from each adjacent location.
    RemoveAdjacentInfluence,
    /// Spend favor to advance the heroes' story marker 1 space.
    AdvanceHeroes {
        /// Favor spent.
        cost: i32,
    },
    /// Discard one of the hero's Corruption cards.
    DiscardCorruption,
    /// Gain 1 level of fortitude.
    GainFortitude,
    /// Gain favor.
    Favor {
        /// Favor gained.
        count: i32,
    },
    /// Receive training.
    Training,
    /// Gain the "Boat" Item card.
    Boat,
    /// Remove influence from the Shadow Pool.
    ShadowPoolLoss {
        /// Influence removed.
        count: i32,
    },
    /// Move to any location.
    MoveAnywhere,
    /// Shuffle the damage pool into the rest pool.
    DamageToRest,
    /// Place Gandalf on a location with influence and remove its monster tokens.
    GandalfToInfluence,
    /// Gain the "Boat" Item card and may move to a location.
    BoatAndMove {
        /// Where the hero may move.
        location: String,
    },
    /// Gain favor and may remove any 1 monster token.
    FavorAndRemoveMonster {
        /// Favor gained.
        count: i32,
    },
    /// May place Gandalf at least `distance` spaces away, then 1 influence leaves the
    /// Shadow Pool.
    GandalfFarAway {
        /// Smallest distance from the hero.
        distance: u32,
    },
    /// Spend favor to move the corruption (black) marker 1 space back.
    RetreatCorruption {
        /// Favor spent.
        cost: i32,
    },
    /// Gain favor and remove all monster tokens within 1 space.
    FavorAndClearMonsters {
        /// Favor gained.
        count: i32,
    },
}

impl ExploreReward {
    /// Snake-case name, used as the option code of the choice.
    pub fn code(&self) -> &'static str {
        match self {
            Self::RemoveAdjacentInfluence => "remove_adjacent_influence",
            Self::AdvanceHeroes { .. } => "advance_heroes",
            Self::DiscardCorruption => "discard_corruption",
            Self::GainFortitude => "gain_fortitude",
            Self::Favor { .. } => "favor",
            Self::Training => "training",
            Self::Boat => "boat",
            Self::ShadowPoolLoss { .. } => "shadow_pool_loss",
            Self::MoveAnywhere => "move_anywhere",
            Self::DamageToRest => "damage_to_rest",
            Self::GandalfToInfluence => "gandalf_to_influence",
            Self::BoatAndMove { .. } => "boat_and_move",
            Self::FavorAndRemoveMonster { .. } => "favor_and_remove_monster",
            Self::GandalfFarAway { .. } => "gandalf_far_away",
            Self::RetreatCorruption { .. } => "retreat_corruption",
            Self::FavorAndClearMonsters { .. } => "favor_and_clear_monsters",
        }
    }
}

/// Ability of an event card.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventAbility {
    /// Each hero with favor spends 1 favor or receives 1 Corruption card.
    FavorOrCorruption,
    /// The heroes remove monster tokens from, or hurt a minion in, a region.
    ClearMonstersOrHurtMinion {
        /// Region code.
        region: String,
        /// Monster tokens removed at most.
        monsters: u32,
        /// Damage dealt to the minion.
        damage: i32,
    },
    /// Sauron places or moves a character of his choice to a location.
    SauronMovesCharacter {
        /// Location code.
        location: String,
    },
    /// Each hero puts 1 favor on the card, which Sauron gives to a hero; that hero
    /// gets the favor back after exploring a location with a character or a hero.
    TrustInFriendship,
    /// Sauron chooses a hero, who accepts a Corruption card and a reward, or lets
    /// Sauron draw a card.
    Temptation {
        /// Reward of accepting.
        reward: TemptationReward,
        /// Card Sauron draws on a refusal.
        refusal: TemptationRefusal,
    },
    /// The heroes remove influence and monster tokens from an area, or place a favor.
    ClearTokensOrFavor {
        /// Where tokens are removed.
        area: Area,
        /// Tokens removed at most; every token when missing.
        #[serde(default)]
        max: Option<u32>,
        /// `max` counts influence and monster tokens separately.
        #[serde(default)]
        per_kind: bool,
        /// Where the favor goes.
        favor: String,
    },
    /// Sauron may discard 1 character or 1 favor from a location of a region.
    SauronDiscardsCharacterOrFavor {
        /// Region code.
        region: String,
    },
    /// The heroes move a character to an adjacent location that isn't a Shadow
    /// Stronghold and clear that location of influence and monster tokens.
    MoveCharacterAndClear,
    /// Sauron may place monster tokens in a region, without influence.
    SauronPlacesMonsters {
        /// Region code.
        region: String,
        /// Locations at most.
        count: u32,
    },
    /// Characters are placed or moved to a location.
    PlaceCharacters {
        /// Character codes.
        characters: Vec<String>,
        /// Location code.
        location: String,
    },
    /// A hero may spend favor to discard an active Plot card affecting a Haven.
    DiscardHavenPlot {
        /// Favor spent.
        cost: i32,
    },
    /// Heroes with 4 favor or more lose 1; Gandalf moves to Isengard.
    Radagast,
    /// Heroes without favor gain 1; the ring marker advances if it is the lowest.
    BoromirsDream,
    /// Théoden goes to Edoras; 1 influence leaves each location next to Edoras.
    Eowyn,
    /// Sauron may place 1 influence and 1 monster token next to Gothmog.
    WarriorOfTheLidlessEye,
    /// 1 influence within 1 space of The Shire; Sauron may move a minion there.
    HeHasAtLastHeard,
    /// Resolved by a hero exploring the affected location (green flag cards).
    OnExplore {
        /// Rewards to choose from; none just discards the card.
        rewards: Vec<ExploreReward>,
    },
}

/// An event card.
#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    /// Title.
    pub name: String,
    /// Priority number.
    pub priority: u32,
    /// Printed ability.
    #[serde(default)]
    pub text: String,
    /// Ability.
    pub ability: EventAbility,
    /// Character to place, if any.
    #[serde(default)]
    pub character: Option<EventCharacter>,
    /// Locations receiving a favor token.
    #[serde(default)]
    pub favors: Vec<String>,
    /// Affected location; such a card stays in play (green flag).
    #[serde(default)]
    pub affect: Option<String>,
    /// Story marker advanced every Story step while the card is in play.
    #[serde(default)]
    pub story: Option<PlotStory>,
}
