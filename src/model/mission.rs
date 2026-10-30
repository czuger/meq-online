//! Mission cards of both sides (§ 9).
//!
//! Each side draws one secretly at setup. A card is stored by its number, which is also
//! the number of its picture (`objectives/original/<side>/<side>_<number>.png`).

use serde::{Deserialize, Serialize};

/// Sauron's Mission cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SauronMission {
    /// Three active Plot cards in play.
    HisDarkThrone,
    /// The black (corruption) marker in stage III.
    WhereTheShadowsLie,
    /// The yellow (ring) marker in stage III.
    InDarknessBindThem,
    /// The red (conquest) marker in stage III.
    ToRuleThemAll,
    /// The Ringwraiths, or 6 influence, on the Shire.
    ToFindThemAll,
}

/// The heroes' Mission cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeroesMission {
    /// Each hero completed both his Starting and Advanced Quest.
    SpearOfTheWest,
    /// The heroes hold 1 Corruption card or fewer in all.
    NobleBlood,
    /// 2 minions or fewer in play.
    MinasMorgulKeptAtBay,
    /// 2 monster tokens or fewer in play.
    AgainstTheShadow,
    /// The heroes hold at least 5 favor in all.
    IsildursSecret,
}

/// Sauron's cards, by card number.
pub const SAURON_MISSIONS: [SauronMission; 5] = [
    SauronMission::HisDarkThrone,
    SauronMission::WhereTheShadowsLie,
    SauronMission::InDarknessBindThem,
    SauronMission::ToRuleThemAll,
    SauronMission::ToFindThemAll,
];

/// The heroes' cards, by card number.
pub const HEROES_MISSIONS: [HeroesMission; 5] = [
    HeroesMission::SpearOfTheWest,
    HeroesMission::NobleBlood,
    HeroesMission::MinasMorgulKeptAtBay,
    HeroesMission::AgainstTheShadow,
    HeroesMission::IsildursSecret,
];

impl SauronMission {
    /// The card numbered `card`, if it exists.
    pub fn from_card(card: u32) -> Option<Self> {
        SAURON_MISSIONS.get(usize::try_from(card).ok()?).copied()
    }
}

impl HeroesMission {
    /// The card numbered `card`, if it exists.
    pub fn from_card(card: u32) -> Option<Self> {
        HEROES_MISSIONS.get(usize::try_from(card).ok()?).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cards_match_their_pictures() {
        assert_eq!(
            SauronMission::from_card(1),
            Some(SauronMission::WhereTheShadowsLie)
        );
        assert_eq!(
            HeroesMission::from_card(0),
            Some(HeroesMission::SpearOfTheWest)
        );
        assert_eq!(SauronMission::from_card(5), None);
    }
}
