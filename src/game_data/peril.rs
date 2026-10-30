//! Static definition of Peril cards (§ 6.5).

use serde::Deserialize;

use crate::game_data::location::Location;

/// Peril card number.
pub type PerilCode = u32;

/// Locations a Peril card affects.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PerilArea {
    /// "Any Location".
    Any,
    /// One location.
    Location {
        /// Location code.
        location: String,
    },
    /// Any location of these regions.
    Regions {
        /// Region codes.
        regions: Vec<String>,
    },
}

/// Effect of a Peril card; each card has its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PerilEffect {
    /// 6 damage, reduced by the shields of discarded cards; 0 damage gives favor or training.
    HoundsOfSauron,
    /// 5 damage, or Sauron draws 1 Plot and 1 Shadow card.
    UnexpectedBetrayal,
    /// Corruption and favor loss when wisdom ≤ adjacent influenced locations.
    ShadowInMemories,
    /// 1 Corruption card; the card and the discard go back into the deck.
    OrcScouts,
    /// Lose 2 favor, or Sauron picks one of 2 Corruption cards for the hero.
    IllMetCompany,
    /// Discard a random card per Corruption card, then fight an Orc.
    ArmiesUnderground,
    /// When wisdom ≤ plots in play: 5 damage and the turn ends.
    HillOfBlackMagic,
    /// The minion there fights the hero; 2 favor when defeated.
    DarkTower,
    /// Fight a Balrog; 2 favor and training when defeated.
    KhazadDum,
    /// With influence there: 1 Corruption card and the turn ends.
    EnduringEvil,
    /// 2 influence to the Shadow Pool, the hero discards 2 cards.
    Trapped,
    /// 5 damage, or the corruption marker advances.
    WeaknessOfMen,
    /// 2 influence to the Shadow Pool, Sauron places influence on 3 locations of the region.
    DangerousBusiness,
    /// 2 influence to the Shadow Pool, Sauron places 1 influence and 1 monster in the region.
    UndyingRegret,
    /// 1 Corruption card, then Sauron commands a minion or draws a Plot card.
    NineForMortalMen,
}

/// A Peril card.
#[derive(Debug, Clone, Deserialize)]
pub struct PerilCard {
    /// Display name.
    pub name: String,
    /// Printed text.
    pub text: String,
    /// French name, shown to players.
    pub name_fr: String,
    /// French text, shown to players.
    pub text_fr: String,
    /// Locations the card affects.
    pub affects: PerilArea,
    /// What the card does.
    pub effect: PerilEffect,
}

impl PerilCard {
    /// Tells whether the card affects `location` (coded `code`).
    pub fn affects(&self, code: &str, location: &Location) -> bool {
        match &self.affects {
            PerilArea::Any => true,
            PerilArea::Location { location } => location == code,
            PerilArea::Regions { regions } => regions.contains(&location.region_code),
        }
    }
}
