//! A decision a player must take before the game goes on.
//!
//! Card effects that need a player's choice store a [`PendingChoice`]; the board waits
//! on the choice screen until the chooser answers, then play resumes where it stopped.

use serde::{Deserialize, Serialize};

use crate::game_data::shadow::{ShadowCode, ShadowTiming};
use crate::model::board::PlotSlot;

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
        /// Trainings still to receive afterwards.
        #[serde(default)]
        more: u8,
    },
    /// Sauron picks the monster or minion attacking the hero, or a Peril (§ 6.4, § 6.5).
    Threat {
        /// The threatened hero.
        hero_id: i64,
        /// Ambush step (otherwise Travel step).
        ambush: bool,
    },
    /// A hero in a Haven allows combat or not (§ 6.4, § 8).
    HavenCombat {
        /// The hero.
        hero_id: i64,
        /// Ambush step (otherwise Travel step).
        ambush: bool,
    },
    /// Sauron picks the Peril card the hero resolves among those drawn (§ 6.5).
    PerilDraw {
        /// The hero.
        hero_id: i64,
        /// The Peril cards drawn.
        cards: Vec<u32>,
    },
    /// A decision within a Peril card's effect.
    Peril {
        /// The hero resolving the card.
        hero_id: i64,
        /// The Peril card.
        card: u32,
        /// Cards drawn for the effect (Corruption cards of "Ill-met Company").
        #[serde(default)]
        drawn: Vec<u32>,
    },
    /// A hero pays damage dealt outside combat from hand and life pool (§ 6.3).
    Damage {
        /// The hero.
        hero_id: i64,
        /// Damage to pay.
        amount: u32,
    },
    /// Sauron may send a hero who drew "Wretched" to the nearest Haven.
    Wretched {
        /// The hero.
        hero_id: i64,
    },
    /// Sauron may play a Shadow card now, or pass (§ 5.9).
    ShadowWindow {
        /// The moment of the hero's turn.
        timing: ShadowTiming,
        /// The hero whose turn it is.
        hero_id: i64,
    },
    /// A decision within a Shadow card's effect.
    Shadow {
        /// The Shadow card.
        card: ShadowCode,
        /// The hero the card is played against, if any.
        #[serde(default)]
        hero_id: Option<i64>,
        /// Heroes still concerned by the next steps, in order.
        #[serde(default)]
        heroes: Vec<i64>,
        /// Targets picked or cards seen in the previous steps.
        #[serde(default)]
        picked: Vec<String>,
    },
    /// Sauron picks the affected location of the Plot card he plays (§ 5.4).
    PlotLocation {
        /// The Plot card.
        card: u32,
        /// The Active Plot Track slot.
        slot: PlotSlot,
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
    /// The current hero's Travel step.
    Movement,
    /// The current hero's Rest step, after the Ambush step (movement when a mob stays).
    Rest,
    /// The story marker advance ending a rest.
    AfterRest,
    /// The current hero's turn ends.
    EndTurn,
    /// Sauron's Action step goes on, between two actions.
    SauronActions,
    /// The current hero's Ambush step, then his Rest step.
    Ambush,
    /// The monster, minion or Peril threatening the hero who just moved.
    TravelThreats,
    /// The preparation of the combat in progress.
    CombatSetup,
    /// Sauron's Event step, after his Plot step.
    EventStep,
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
