//! Static definition of Shadow cards (§ 5.9).

use serde::{Deserialize, Serialize};

/// Shadow card number, as in the pictures.
pub type ShadowCode = u32;

/// When Sauron may play a Shadow card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowTiming {
    /// During his Action step.
    ActionStep,
    /// At the start of a hero's turn.
    HeroTurnStart,
    /// At the start of a combat.
    CombatStart,
    /// After a hero enters a location.
    EnteredLocation,
    /// After a hero is defeated.
    HeroDefeated,
}

/// Effect of a Shadow card; cards existing twice share theirs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShadowEffect {
    /// Fight a Crebain with no agility; no Ambush step.
    FlocksOfCrebain,
    /// The location is perilous until the end of the hero's turn.
    ArmGrownLong,
    /// Peek at the top plot card; the black marker advances against corrupted heroes.
    SarumansLies,
    /// Move a hero; he discards a random card per Corruption card.
    EvilFog,
    /// Redistribute two heroes' Corruption cards, then give one.
    LidlessEye,
    /// 1 Corruption card, then -1 strength and agility per Corruption card.
    BlackBreath,
    /// Heal a minion (or bring a defeated one back) onto an influenced location.
    TheyAreTerrible,
    /// Fight a Giant Spider, or a Cave Troll with 2 Corruption cards; no Ambush step.
    CreaturesOfTheWild,
    /// Fetch the Denethor or Saruman plot card.
    IthilStone,
    /// 1 Corruption card or 5 damage, the hero's choice.
    DarkPromises,
    /// Heroes discard down to 5 cards, then a random card per Corruption card.
    StormsOfMordor,
    /// 2 influence on each location of Mordor, or up to 2 monster tokens.
    NewPowerRising,
    /// Take back a discarded Plot or Shadow card.
    LordOfTheRings,
    /// Influence in the Shadow Pool and the Shire; the yellow marker may advance.
    ShireBaggins,
    /// 1 influence on a hero's location; he loses 1 favor per Corruption card.
    DoNotTemptMe,
    /// 5 damage per Corruption card, or 3 damage and 1 Corruption card.
    MorgulBlade,
    /// Influence per Corruption card; the green marker advances 1 less.
    GrievousLoss,
    /// 4 damage plus 2 per Corruption card after the defeat.
    EvilWound,
    /// The hero moves at most once.
    WinterStorm,
    /// Fight an Agent; Sauron draws a plot against a corrupted hero; no Ambush step.
    SeemFairer,
    /// 2 random cards discarded, -1 strength per Corruption card.
    Betrayed,
}

/// A Shadow card.
#[derive(Debug, Clone, Deserialize)]
pub struct ShadowCard {
    /// Display name.
    pub name: String,
    /// Printed text.
    pub text: String,
    /// French name, shown to players.
    pub name_fr: String,
    /// French text, shown to players.
    pub text_fr: String,
    /// Influence needed in the Shadow Pool.
    pub requirement: i32,
    /// When the card may be played.
    pub timing: ShadowTiming,
    /// Only against a hero outside a Haven.
    #[serde(default)]
    pub outside_haven: bool,
    /// Only in a combat against a minion.
    #[serde(default)]
    pub against_minion: bool,
    /// What the card does.
    pub effect: ShadowEffect,
}
