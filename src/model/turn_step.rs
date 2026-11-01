//! Steps of a turn as recorded in the game log: what each step did, or why it couldn't
//! be played.

use serde::{Deserialize, Serialize};

/// Log action opening Sauron's turn.
pub const TURN_SAURON: &str = "turn.sauron";
/// Log action opening the Hero Draw step, shown apart from the turns around it.
pub const TURN_DRAW: &str = "turn.draw";
/// Log action opening a hero's turn.
pub const TURN_HERO: &str = "turn.hero";
/// Log action opening a step.
pub const STEP_BEGIN: &str = "step.begin";
/// Log action of a step that couldn't be played.
pub const STEP_SKIPPED: &str = "step.skipped";
/// Log action of a turn ending before its last step.
pub const TURN_STOPPED: &str = "turn.stopped";

/// A step of a turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnStep {
    /// Heroes remove the influence where they stand.
    Rally,
    /// Story markers advance.
    Story,
    /// Sauron may play a plot card.
    Plot,
    /// An event card is resolved.
    Event,
    /// Sauron takes his actions.
    Action,
    /// Heroes draw their cards.
    HeroDraw,
    /// Monsters and minions on the hero's location attack.
    Ambush,
    /// The hero rests or heals.
    Rest,
    /// The hero travels.
    Movement,
    /// The hero explores his location.
    Exploration,
}

/// Why a step couldn't be played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepSkip {
    /// The game starts at the Event step.
    FirstRound,
    /// No hero stands on influence.
    NoInfluenceUnderHeroes,
    /// The event deck and its discard pile are empty.
    EventDeckEmpty,
    /// Nothing on the hero's location can attack him.
    NoMobHere,
    /// The hero refused combat in a Haven.
    HavenProtects,
    /// A monster or minion stays on the hero's location.
    MobHere,
    /// An active plot forbids resting here.
    PlotForbidsRest,
    /// Nobody rests on a Shadow Stronghold.
    ShadowStronghold,
    /// A hero who fought an ambush doesn't rest.
    AmbushFought,
    /// The hero's life pool is empty.
    EmptyLifePool,
    /// The hero's hand is full.
    HandFull,
}

/// Why a turn ended before its last step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnStop {
    /// The hero didn't defeat the mob he met while travelling.
    CombatNotWon,
    /// The hero was defeated.
    Defeated,
}
