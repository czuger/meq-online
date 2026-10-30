# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

An online, multiplayer web version of the board game *Middle-earth Quest*: one player is Sauron, up to four play
heroes. Rust: axum + tower-http, askama templates, SQLite through sqlx, tokio. The project is stalled (pre-alpha). It
was ported from a Rails app; git history holds the Rails code, which is the reference for the original behavior.

## Commands

```sh
cargo run                                   # http://127.0.0.1:3000, developer login (any name/e-mail)
cargo test                                  # everything
cargo test --lib engine::combat             # combat rules only
cargo test --test http_flow combat_flow     # one HTTP flow test
cargo clippy --all-targets
cargo fmt
```

Settings come from `MEQ_BIND`, `DATABASE_URL` (default `sqlite://meq.sqlite3`), `MEQ_DATA_DIR`, `MEQ_STATIC_DIR` and
`RUST_LOG`. `schema.sql` is applied at every start with `CREATE TABLE IF NOT EXISTS`; there are no migrations, so a
schema change on an existing database needs a manual `ALTER TABLE` or a fresh database.

## Architecture

- **`game_data/`**: static definitions from `data/*.yaml`, loaded once into `Arc<GameData>`. The YAML was written by
  Ruby (symbol keys `:name:`, symbol values `:ranged`, `!ruby/object` tags); `yaml_loader` normalizes it before serde.
  `CodeMap` keeps file order where the game relied on Ruby hash order.
- **`model/`**: plain game-state structs. `Game` is the aggregate of one board (board, Sauron, heroes, mobs, combat,
  new log entries).
- **`engine/`**: every game rule, as methods on `Game`/`Hero` or free functions. No HTTP or database code. Randomness
  goes through a concrete `StdRng` passed in (seeded in tests).
  - `board_state.rs`: the `BoardState` enum and `BoardEvent` transitions (`transition` returns an error for invalid
    ones). Each screen state is named after its GET route; `route.rs` maps state → URL in one `match`. Rename both
    together.
  - `combat/`: round resolution (`reveal.rs`, card and mob powers) and the combat flow. The `combat/tests/` are ports
    of the Rails `combat_*_test.rb` specs and define the combat rules. `CombatOrigin` (ambush, travel, peril)
    decides how the turn goes on after a combat (`flow.rs` `after_combat`).
  - `threats.rs`: Ambush step and "Combat or Peril" (Sauron's choice, Haven consent); `perils/`: the Peril deck and
    the 15 card effects.
  - `turn.rs`, `rest.rs`, `movement.rs`, `exploration.rs`, `sauron_turn.rs`, `plots.rs`, `story.rs`… one area each.
- **`db/`**: sqlx persistence. Requests load the whole `Game`, let the engine change it and save it in one transaction
  (`game_repo`). Decks, hands, pools and influence are JSON text columns; monsters and minions share `mobs` with a
  `kind`.
- **`web/`**: axum router, handlers (one file per area), askama page structs (private to their handler) and views.
  `access.rs` extractors check that the user is a member of the board in the URL (the only ownership rule).
  `game_tx::change_game` wraps load → change → save → refresh broadcast. Flash messages use a cookie.
- **`refresh.rs`**: a tokio broadcast; `/boards/{id}/events` streams it as server-sent events and the map page reloads
  on each event.
- `templates/` (askama, French text), `static/` (pictures, CSS, vanilla JS, served under `/static`).

Notable behavior kept from the original: new games seat four computer-controlled heroes (beravor, eleanor, thalin,
eometh) and make the creator Sauron; board members play the computer seats. Shadow cards are disabled.

## Coding conventions

**Imports**
- Use `crate::`-rooted paths across modules. `super::` only within tightly nested hierarchies (e.g. a tests module).
  No `super::super::` chains: restructure instead.
- No glob imports, except preludes designed for it and `use super::*;` in tests.

**Types**
- Annotate all `pub` function signatures fully; let inference handle local bindings.
- Prefer concrete types over generics unless genericity is actually used.
- Avoid `Box<dyn Any>` and type erasure unless necessary (justify with a short comment). `Box<dyn Error>` only at
  application boundaries such as `main`.

**Docs and comments**
- Every `pub` item gets a `///` rustdoc with the relevant sections (`# Arguments`, `# Returns`, `# Errors`,
  `# Panics`). Enforced with `#![warn(missing_docs)]`.
- Private functions get docs only when the logic isn't obvious. Keep docs proportionate and don't restate the
  signature.
- Comment only genuinely complex or non-obvious logic, briefly, on intent. Never caption the obvious.

**Error handling**
- `Result` for recoverable errors, `Option` for absence; no panics as control flow.
- Propagate with `?` unless the error needs context at that point.
- No `unwrap()`/`expect()` in production paths (tests and prototypes only); if a failure is provably impossible,
  justify with a short comment.
- Typed error enums (thiserror), never stringly-typed errors.
- Never silently discard errors (`.ok()`, `let _ =`): handle, log or propagate.
- `panic!`, `unreachable!`, `todo!`, `unimplemented!` only for real invariant violations.

**Functions**
- Soft limit of 50 lines; split longer ones into small, single-purpose functions.
- Prefer composition and early returns over nested `match`/`if let` pyramids.

**Files and modules**
- One primary public type per file, named after it in snake_case. Small tightly coupled types (its error enum,
  builder, result type) may share the file; a family of small types expressing one concept (e.g. the board's states)
  may live together until large.
- Traits implemented by several unrelated types get their own file.
- No monolithic files; group growing topics into directory modules.
- `mod.rs`, `lib.rs` and `main.rs` stay thin: declarations, wiring, re-exports only.

**General**
- Readability and small testable units over cleverness; explicit over implicit.
- Follow idiomatic Rust, clippy and rustfmt defaults unless these rules say otherwise.

**Language**
- Code is English: identifiers, comments, docs, commit messages, log lines.
- User-facing text is French (UTF-8, accents kept): templates, labels, error messages shown to players. Don't
  translate existing French text; write new text in French.

## Finalization status (work in progress, 2026-10-30)

The engine is being brought to the official rules following `docs/finalization-plan.md` (rules in `docs/rules.md`,
gap analysis in `docs/rules-gaps.md`, open questions in `docs/open-questions.md`: add new ones there). D1 and D3
are decided: follow the official rules, the engine enforces the action track. Phases 1–5 are committed (1765f01);
Phase 6 is not.

**Done (Phases 1–3, green when finished):** every Phase 1, 2 and 3 item of the plan. Highlights: defeat screens
(`engine/defeat.rs`, `HeroDefeatScreen`, `SauronHavenScreen`); game creation with 1–3 chosen heroes, open or computer
seats (`HeroSeat`, `join_hero_seat`); combat rules (exhaustion "exceeds", agility split, voluntary exhaustion, damage
paid from hand/life pool); action tracks (`model/sauron_action.rs`, `engine/sauron_actions.rs`), influence budget
(`engine/influence.rs`), commands (`engine/commands.rs`), one board state per action. Data fixes: stronghold
`max_influence` in `data/locations.yaml`, `morthond` → `mothond` in `data/locations_paths.yaml`. Many new columns in
`schema.sql`: an existing database must be recreated.

**Card text found (D4):** `work/data_and_rules/MEQ_Mission_Corruption_Peril_Shadow_Cards_Fulltext.xls` holds the text
of the 15 Peril, 21 Shadow, 16 Corruption and the Mission cards (read it with `xlrd` in a scratch virtualenv). Card
pictures exist for events, quests (`hero_cards/quests/standard`), shadows, plots, missions and hero sheets. No
Encounter or Item card text exists anywhere. Character favor values are unreadable (2 favor kept for all).

**Phase 4 (events), done, green:** all 42 Event cards in `data/events.yaml` (typed `ability`, `affect` for
green-flag cards, `story` for marker cards); `game_data/event.rs`; generic pending choice (`model/choice.rs`,
`engine/choices.rs`: `ask`, `answer_choice`, resume to the Action step or to exploration); `engine/events/`
(`event_step.rs`, `abilities.rs`, `explore.rs`, tests in `events/tests/`); training and level tokens
(`engine/hero/training.rs`); Boat item; event marker token. Choice screen: `web/handlers/choices.rs` (GET
`/boards/{id}/choices/choice_screen`, POST `/boards/{id}/choices/choose` with `option` and
`targets=<option>|<code>`; only the chooser may answer: Sauron's user, or for heroes the hero's user or a computer
seat), labels in `web/views/choice_label.rs`, template `choices_choice_screen.html`.

**Phase 5 (hero turn), mostly done, green:** Sauron picks the attacking mob or a Peril (`engine/threats.rs`,
choice sources `Threat`, `HavenCombat`); the 15 Peril cards (`data/perils.yaml` with French `name_fr`/`text_fr`,
`game_data/peril.rs`, `engine/perils/`: `PerilDraw` and `Peril` choices); damage outside combat chosen by the hero
(`engine/hero/damage.rs`, `Damage` choice) and defeat outside combat (`defeat.rs` `start_hero_defeat`); consult a
character for favor or ability (`data/characters.yaml` `ability`), Dark Path; all 16 corruption cards (Wretched
asks Sauron for the Haven, `Wretched` choice); Beravor, Eleanor and Thálin abilities; per-turn record
`Hero::turn_record` (travels, favor gained, Dark Path, Eleanor). New resume points `Movement`, `Rest`, `AfterRest`,
`EndTurn`; `answer_choice` only resumes while the board is still on the choice screen. A travel combat won now
leads to exploration. Schema changed again (`heroes.turn_record`, `combats.origin`).

**Phase 6 (Shadow cards), done, green:** the 24 pictured cards in `data/shadows.yaml` (21 texts, 3 duplicates; the
rules' 26 cards lack 2, Q2), `game_data/shadow.rs` (`ShadowTiming`, `ShadowEffect`), `engine/shadows/`
(`shadow_window.rs`: playable cards, one per player turn, the `ShadowWindow` choice at the start of a hero turn, after
entering a location, at the start of a combat and after a defeat; `effects.rs`, `answers.rs`: the `Shadow` choice;
tests in `shadows/tests.rs`). Action step cards are played from the actions screen (`web/handlers/shadow_cards.rs`).
Effects outliving their card: `model/turn_effects.rs` (`boards.turn_effects`: card played this turn, perilous
location, single move, green marker malus, plot locations) and `CombatModifiers` (`combats.modifiers`). New resume
points `SauronActions`, `Ambush`, `TravelThreats`, `CombatSetup`, `EventStep`; `CombatOrigin::Shadow` leads to the
Rest step. The 5 plots without a printed location are back: Sauron picks it (`engine/plot_locations.rs`,
`PlotLocation` choice); always go through `Game::plot_location`/`plot_at`, not `Plot::affect`. Greedy is enforced.
Schema changed again: an existing database must be recreated. Interpretations to confirm: Q28.

**Next:** Phase 5 leftovers: quests (text readable on `static/images/hero_cards/quests/standard`: Starting and
Advanced quests), trading between heroes, Argalad's peek limited to one adjacent location. Encounter step and Items
blocked on missing text (and water paths missing from `data/locations_paths.yaml`). Phase 7 missions and Finale.
Phase 8 optional rules and end-to-end tests.
