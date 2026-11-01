//! What happened during the latest turns, step by step, built from the game log.

use serde_json::Value;

use crate::game_data::GameData;
use crate::model::actor_ref::ActorRef;
use crate::model::game::Game;
use crate::model::log_entry::LogEntry;
use crate::model::turn_step::{
    STEP_BEGIN, STEP_SKIPPED, StepSkip, TURN_DRAW, TURN_HERO, TURN_SAURON, TURN_STOPPED, TurnStep,
};
use crate::web::views::log_text::{log_card_pics, log_text};
use crate::web::views::step_label::{skip_text, step_label};

/// Turns kept: a whole round of four heroes, each with his draw, and Sauron's turn.
const SECTIONS_SHOWN: usize = 9;
/// Steps repeated at the top of the screen.
const LAST_RESULTS: usize = 2;

/// Whose turn a section tells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionSide {
    /// Sauron's turn.
    Sauron,
    /// A Hero Draw step.
    Draw,
    /// A hero's turn.
    Hero(i64),
    /// Entries older than the first recorded turn.
    Earlier,
}

/// A line of the log with the public cards it shows.
#[derive(Debug, Clone)]
pub struct LogLine {
    /// French sentence.
    pub text: String,
    /// Pictures of the public cards.
    pub cards: Vec<String>,
    /// The line tells something that couldn't be done.
    pub blocked: bool,
}

/// What a step did.
#[derive(Debug, Clone)]
pub struct StepBlock {
    /// The step, unless the entries came before any step.
    pub step: Option<TurnStep>,
    /// French name of the step.
    pub label: &'static str,
    /// Why the step couldn't be played, if so.
    pub skipped: Option<&'static str>,
    /// What happened, in order.
    pub lines: Vec<LogLine>,
}

/// One turn of the game.
#[derive(Debug, Clone)]
pub struct TurnSection {
    /// Whose turn it is.
    pub side: SectionSide,
    /// French title.
    pub title: String,
    /// The steps played, in order.
    pub blocks: Vec<StepBlock>,
}

/// Result of a recent step, repeated at the top of the screen.
#[derive(Debug, Clone)]
pub struct LastResult {
    /// Turn and step in French.
    pub heading: String,
    /// What happened.
    pub lines: Vec<LogLine>,
}

/// The latest turns of a board.
#[derive(Debug, Clone, Default)]
pub struct TurnLog {
    /// Turns, newest first.
    pub sections: Vec<TurnSection>,
    /// Results of the latest steps, in order.
    pub last_results: Vec<LastResult>,
}

impl TurnLog {
    /// Builds the turns from log entries.
    ///
    /// # Arguments
    /// * `entries` - latest entries, oldest first.
    /// * `game` - the board, to name the heroes.
    /// * `data` - static data.
    pub fn new(entries: &[&LogEntry], game: &Game, data: &GameData) -> Self {
        let mut sections: Vec<TurnSection> = Vec::new();
        for entry in entries {
            tracing::trace!(action = %entry.action, "turn log entry");
            add_entry(&mut sections, entry, game, data);
        }
        sections.retain(|s| !s.blocks.is_empty());
        sections.reverse();
        sections.truncate(SECTIONS_SHOWN);
        let last_results = last_results(&sections);
        Self {
            sections,
            last_results,
        }
    }

    /// Steps of the latest turn of a side that couldn't be played, with the reason.
    pub fn skipped_steps(&self, sauron: bool) -> Vec<(TurnStep, &'static str)> {
        let section = self.sections.iter().find(|s| match s.side {
            SectionSide::Sauron => sauron,
            SectionSide::Hero(_) => !sauron,
            SectionSide::Draw | SectionSide::Earlier => false,
        });
        section
            .into_iter()
            .flat_map(|s| &s.blocks)
            .filter_map(|b| Some((b.step?, b.skipped?)))
            .collect()
    }
}

/// Adds an entry to the latest section, or opens the section it starts.
fn add_entry(sections: &mut Vec<TurnSection>, entry: &LogEntry, game: &Game, data: &GameData) {
    if let Some(side) = section_side(entry) {
        sections.push(TurnSection {
            side,
            title: log_text(entry, data),
            blocks: Vec::new(),
        });
        return;
    }
    if sections.is_empty() {
        sections.push(TurnSection {
            side: SectionSide::Earlier,
            title: "Événements précédents".to_owned(),
            blocks: Vec::new(),
        });
    }
    let Some(section) = sections.last_mut() else {
        return;
    };
    let step = entry_step(entry);
    if entry.action == STEP_BEGIN {
        // A step without lines is only shown once something happens in it.
        open_block(section, step);
        return;
    }
    let (text, blocked, skipped) = match entry.action.as_str() {
        STEP_SKIPPED => {
            let reason = entry_skip(entry).map(skip_text);
            open_block(section, step);
            (reason.unwrap_or_default().to_owned(), true, reason)
        }
        TURN_STOPPED | "draw_cards.blocked" => (log_text(entry, data), true, None),
        _ => (actor_prefixed(entry, section.side, game, data), false, None),
    };
    if section.blocks.is_empty() {
        open_block(section, None);
    }
    if let Some(block) = section.blocks.last_mut() {
        block.skipped = block.skipped.or(skipped);
        block.lines.push(LogLine {
            text,
            cards: log_card_pics(entry, data),
            blocked,
        });
    }
}

/// The last block when it already tells `step`, otherwise a new one: a step entered
/// again after a combat or a choice goes on in the same block.
fn open_block(section: &mut TurnSection, step: Option<TurnStep>) {
    if section.blocks.last().is_none_or(|b| b.step != step) {
        section.blocks.push(StepBlock {
            step,
            label: step.map_or("", step_label),
            skipped: None,
            lines: Vec::new(),
        });
    }
}

fn section_side(entry: &LogEntry) -> Option<SectionSide> {
    match (entry.action.as_str(), entry.actor) {
        (TURN_SAURON, _) => Some(SectionSide::Sauron),
        (TURN_DRAW, _) => Some(SectionSide::Draw),
        (TURN_HERO, ActorRef::Hero(id)) => Some(SectionSide::Hero(id)),
        _ => None,
    }
}

fn entry_step(entry: &LogEntry) -> Option<TurnStep> {
    serde_json::from_value(entry.params.get("step")?.clone()).ok()
}

fn entry_skip(entry: &LogEntry) -> Option<StepSkip> {
    serde_json::from_value(entry.params.get("reason").cloned().unwrap_or(Value::Null)).ok()
}

/// The sentence of an entry, naming the hero when it isn't clear from the section.
fn actor_prefixed(entry: &LogEntry, side: SectionSide, game: &Game, data: &GameData) -> String {
    let text = log_text(entry, data);
    let ActorRef::Hero(id) = entry.actor else {
        return text;
    };
    if side == SectionSide::Hero(id) {
        return text;
    }
    let name = game
        .heroes
        .iter()
        .find(|h| h.id == Some(id))
        .and_then(|h| data.heroes.get(&h.code))
        .map(|d| d.name.as_str());
    match name {
        Some(name) if !text.contains(name) => format!("{name} : {text}"),
        _ => text,
    }
}

/// Results of the latest steps with something to tell, across turns.
fn last_results(sections: &[TurnSection]) -> Vec<LastResult> {
    let mut results: Vec<LastResult> = sections
        .iter()
        .flat_map(|section| {
            section
                .blocks
                .iter()
                .rev()
                .filter(|b| !b.lines.is_empty())
                .map(move |block| LastResult {
                    heading: if block.label.is_empty() {
                        section.title.clone()
                    } else {
                        format!("{} · {}", section.title, block.label)
                    },
                    lines: block.lines.clone(),
                })
        })
        .take(LAST_RESULTS)
        .collect();
    results.reverse();
    results
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::engine::test_support::{data, game};

    fn entry(action: &str, params: &[(&str, Value)]) -> LogEntry {
        LogEntry::new(ActorRef::System, action, params)
    }

    fn built(entries: &[LogEntry]) -> TurnLog {
        let refs: Vec<&LogEntry> = entries.iter().collect();
        TurnLog::new(&refs, &game(), data())
    }

    #[test]
    fn groups_lines_by_turn_and_step_newest_turn_first() {
        let log = built(&[
            entry(TURN_SAURON, &[]),
            entry(STEP_BEGIN, &[("step", json!("story"))]),
            entry(
                "story.advance",
                &[("marker_name", json!("heroes")), ("count", json!(2))],
            ),
            entry(TURN_DRAW, &[]),
            entry(
                "draw_cards",
                &[
                    ("name", json!("argalad")),
                    ("cards_drawn", json!(3)),
                    ("lp_cards", json!(9)),
                ],
            ),
        ]);
        assert_eq!(log.sections.len(), 2);
        assert_eq!(log.sections[0].side, SectionSide::Draw);
        assert_eq!(log.sections[1].side, SectionSide::Sauron);
        assert_eq!(log.sections[1].blocks[0].step, Some(TurnStep::Story));
        assert!(log.sections[1].blocks[0].lines[0].text.contains("héros"));
    }

    #[test]
    fn a_skipped_step_tells_why_and_is_reported_for_the_stepper() {
        let log = built(&[
            LogEntry::new(ActorRef::Hero(1), TURN_HERO, &[("hero", json!("argalad"))]),
            entry(
                STEP_SKIPPED,
                &[("step", json!("rest")), ("reason", json!("mob_here"))],
            ),
        ]);
        let block = &log.sections[0].blocks[0];
        assert!(block.lines[0].blocked);
        assert!(block.lines[0].text.contains("Repos impossible"));
        assert_eq!(
            log.skipped_steps(false),
            [(TurnStep::Rest, skip_text(StepSkip::MobHere))]
        );
        assert!(log.skipped_steps(true).is_empty());
    }

    #[test]
    fn a_step_entered_again_stays_one_block_and_empty_steps_are_not_results() {
        let log = built(&[
            LogEntry::new(ActorRef::Hero(1), TURN_HERO, &[("hero", json!("argalad"))]),
            entry(STEP_BEGIN, &[("step", json!("movement"))]),
            LogEntry::new(ActorRef::Hero(1), "movement.none", &[]),
            entry(STEP_BEGIN, &[("step", json!("movement"))]),
            entry(STEP_BEGIN, &[("step", json!("exploration"))]),
        ]);
        assert_eq!(log.sections[0].blocks.len(), 2);
        assert_eq!(log.last_results.len(), 1);
        assert!(log.last_results[0].heading.ends_with("Déplacement"));
    }

    #[test]
    fn keeps_only_the_latest_turns() {
        let mut entries = Vec::new();
        for _ in 0..SECTIONS_SHOWN + 3 {
            entries.push(entry(TURN_SAURON, &[]));
            entries.push(entry("plot_cards.pass", &[]));
        }
        assert_eq!(built(&entries).sections.len(), SECTIONS_SHOWN);
    }
}
