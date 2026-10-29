//! Static definition of hero quests.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Action performed on the board when a quest starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestSetupKind {
    /// Place a character on a location.
    Place,
}

/// One setup step of a quest.
#[derive(Debug, Clone, Deserialize)]
pub struct QuestSetupAction {
    /// What to do.
    pub action: QuestSetupKind,
    /// Character code.
    pub character: String,
    /// Location code.
    pub location: String,
}

/// Quests available to one hero.
#[derive(Debug, Clone, Deserialize)]
pub struct HeroQuest {
    /// Quests a hero may start with.
    pub starting_quests: Vec<u32>,
    /// Later quests.
    #[serde(default)]
    pub advanced_quests: Vec<u32>,
    /// Board setup steps by quest number.
    #[serde(default)]
    pub setup: BTreeMap<u32, Vec<QuestSetupAction>>,
}

/// Quest sets ("regular", expansions) by name, then quests by hero code.
pub type HeroQuests = BTreeMap<String, BTreeMap<String, HeroQuest>>;
