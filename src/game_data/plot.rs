//! Static definition of a plot card.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Story track advanced by a plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoryType {
    /// Ring track.
    Ring,
    /// Conquest track.
    Conquest,
    /// Corruption track.
    Corruption,
}

/// Starting influence of a starting plot.
#[derive(Debug, Clone, Deserialize)]
pub struct PlotInfluence {
    /// Influence placed on locations at setup.
    pub init: BTreeMap<String, i32>,
    /// Influence Sauron may add.
    pub add: i32,
    /// Starting shadow pool.
    pub shadow_pool: i32,
}

/// Number of cards to draw and to keep.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct DrawKeep {
    /// Cards drawn.
    pub draw: u32,
    /// Cards kept.
    pub keep: u32,
}

/// Story effect of a plot.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PlotStory {
    /// Advanced track.
    #[serde(rename = "type")]
    pub story_type: StoryType,
    /// Spaces advanced each turn.
    pub advance: i32,
}

/// Extra condition for playing a plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotRequirement {
    /// A minion stands in Erebor.
    MinionInErebor,
    /// A monster stands in Osgiliath.
    MonsterInOsgiliath,
    /// 6 influence around Edoras, or plot 6 is in play.
    InfluenceNearEdoras,
    /// 3 influence in Isengard.
    InfluenceInIsengard,
    /// 8 influence around Mount Gundabad.
    InfluenceNearMountGundabad,
    /// A mob stands near the Woodland Realm.
    RequireMobNearWoddenLand,
}

/// A plot card.
#[derive(Debug, Clone, Deserialize)]
pub struct Plot {
    /// Setup influence, for starting plots only.
    #[serde(default)]
    pub influence: Option<PlotInfluence>,
    /// Shadow cards drawn/kept, for starting plots only.
    #[serde(default)]
    pub shadow_cards: Option<DrawKeep>,
    /// Plot cards drawn/kept, for starting plots only.
    #[serde(default)]
    pub plot_cards: Option<DrawKeep>,
    /// Favor a hero must spend to remove the plot.
    pub favor_to_discard: i32,
    /// Story effect.
    pub story: PlotStory,
    /// Location where the plot token is placed.
    #[serde(default)]
    pub affect: Option<String>,
    /// Shadow pool required to play it.
    pub required_influence: i32,
    /// Extra condition.
    #[serde(default)]
    pub require: Option<PlotRequirement>,
}
