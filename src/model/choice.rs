//! A decision a player must take before the game goes on.
//!
//! Card effects that need a player's choice store a [`PendingChoice`]; the board waits
//! on the choice screen until the chooser answers, then play resumes where it stopped.

use serde::{Deserialize, Serialize};

/// Who answers a choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Chooser {
    /// The Sauron player.
    Sauron,
    /// One hero, by id.
    Hero(i64),
    /// The heroes as a side (any hero player answers).
    Heroes,
}

/// One answer to a choice, with what it acts on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOption {
    /// Option code, unique within the choice.
    pub code: String,
    /// Targets to pick from: a location, a token, a card... coded `kind:value[:value]`,
    /// e.g. `location:bree`, `monster:12`, `influence:bree:2` (the second influence
    /// token of Bree).
    #[serde(default)]
    pub targets: Vec<String>,
    /// Targets to pick at least.
    #[serde(default)]
    pub min: usize,
    /// Targets to pick at most.
    #[serde(default)]
    pub max: usize,
}

/// The card or rule a choice belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChoiceSource {
    /// An event card being resolved. `heroes` are the heroes concerned by the next
    /// steps, in order.
    Event {
        /// Stage deck (1 to 3).
        stage: u8,
        /// Card number.
        card: u32,
        /// Heroes still concerned.
        #[serde(default)]
        heroes: Vec<i64>,
    },
    /// The event in play, activated by a hero exploring its location.
    ExploreEvent {
        /// Stage deck (1 to 3).
        stage: u8,
        /// Card number.
        card: u32,
        /// The exploring hero.
        hero_id: i64,
    },
    /// A hero receives training: keep one of the drawn Skill cards (§ 7).
    Training {
        /// The hero.
        hero_id: i64,
        /// The two Skill cards drawn.
        cards: Vec<u32>,
    },
}

/// Where play goes on once the choices are answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resume {
    /// Sauron's Action step (after the Event step).
    ActionStep,
    /// The current hero's exploration.
    Exploration,
}

/// A decision waiting for its answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingChoice {
    /// Who answers.
    pub chooser: Chooser,
    /// What the choice belongs to.
    pub source: ChoiceSource,
    /// The possible answers.
    pub options: Vec<ChoiceOption>,
    /// Where play goes on afterwards.
    pub resume: Resume,
}
