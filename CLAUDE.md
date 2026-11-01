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
`RUST_LOG` (console, default `info`). `schema.sql` is applied at every start with `CREATE TABLE IF NOT EXISTS`; there
are no migrations, so a schema change on an existing database needs a manual `ALTER TABLE` or a fresh database.

Debug log (`logging.rs`): dev builds (`cargo run`) write a detailed daily file `logs/meq-debug.YYYY-MM-DD.log`
(`MEQ_DEBUG_LOG=0` turns it off, `=1` turns it on in release builds; `MEQ_LOG_DIR`, `MEQ_LOG_FILTER`, default
`config::DEFAULT_LOG_FILTER`). It holds every request, every engine step (`#[tracing::instrument]` spans with entry,
exit and timing on state-changing functions, handlers and repos), board state transitions (`Game::fire`), game log
entries (`Game::log`), shuffles and random draws, a `trace!` in each loop, and SQL statements. New state-changing
functions and loops get the same; never log session tokens or e-mails.

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
  - `step_log.rs`: what each step did goes in the game log. Turn markers (`turn.sauron`, `turn.draw`, `turn.hero`),
    `step.begin` (recorded by `Game::fire` for screen states, by hand for automatic steps), `step.skipped` with a
    `StepSkip` reason, `turn.stopped` (`model/turn_step.rs`). A new step result is a `Game::log` plus its text in
    `views/log_text.rs`; a card goes in the params only once it is public (`log_card_pics` draws it). Never log
    hands, facedown monster tokens, drawn Shadow/Plot cards or kept Skill cards. A reason for something unavailable
    has an engine enum and its French text in a view (`TravelBlock`, `PlotBlock`).
  - `ai/`: the computer heroes. `waiting_hero.rs` finds the computer hero the game waits on, `candidates.rs`
    lists a few `HeroMove`s per screen (random order wherever a strategy would get complex), and
    `Game::play_computer_heroes` (`hero_ai.rs`) applies each candidate to a copy and keeps the first one the rules
    accept that changed something besides the log. It never reimplements legality: the engine refuses illegal
    moves. When nothing is accepted it stops and the board members play the seat by hand.
- **`db/`**: sqlx persistence. Requests load the whole `Game`, let the engine change it and save it in one transaction
  (`game_repo`). Decks, hands, pools and influence are JSON text columns; monsters and minions share `mobs` with a
  `kind`.
- **`web/`**: axum router, handlers (one file per area), askama page structs (private to their handler) and views.
  `access.rs` extractors check that the user is a member of the board in the URL (the only ownership rule).
  `game_tx::change_game` wraps load → change → computer heroes play (unless
  `GameOptions::computer_heroes_by_hand`, the new game form's "seats=computer"; "seats=ai" lets the AI play) → save
  → board version raised (only when the game changed, so a
  screen that saves on display doesn't reload itself). Flash messages use a cookie. State-changing actions are POST
  forms.
  - **Game table**: every screen of a seat is framed by `views/table_view.rs` (`TableView`: seats, turn steps, map,
    recent log lines, the seat's cards in a tray), attached to the `Layout` by `game_layout.rs` (`hero_layout`,
    `sauron_layout`, `viewer_layout` for screens shared by both sides). `layout.html` wraps the page's `content`
    block in the table when `layout.table` is set; partials live in `templates/partials/`. A screen makes
    locations clickable on the map with `Layout::with_targets` (`MapTarget`: a submit button of the step's form
    drawn over the location), hides the tray hand with `without_hand` when it shows the hand itself.
  - `views/turn_steps.rs` (the stepper, an exhaustive `match BoardState`; a step that couldn't be played is
    `skipped`, with its reason as tooltip), `views/seat_view.rs` (`controls`: own seat or computer seat).
  - `views/turn_log.rs` (`TurnLog`): the log grouped by turn and step, newest turn first. `partials/step_results.html`
    repeats the latest results above the screen, `partials/turn_log.html` shows the whole round below it. A screen
    shows what can't be done now disabled, with the reason, instead of hiding it.
  - **Play page**: `GET /boards/{id}/play` (`handlers/play.rs`) redirects to the screen of the user's seat the game
    waits on (`game_layout::my_screen`, using `route::screen_side`) or shows the waiting page; turn-ending actions
    and transient states (`route_for`) go there. `GET /boards/{id}/status?version=…` returns
    `{ "version": …, "url": … }` for `static/js/game.js` (the game is only loaded when the version differs).
- **`board_versions.rs`**: an in-memory version per board (`AppState::versions`, opaque text, new on every
  server start). The table renders it as `data-board-version` and `game.js` polls `/status` every 2 s (not while
  the tab is hidden); when it changed, it goes to the user's screen, otherwise reloads, or shows an "Actualiser"
  banner when the player has started filling the screen. No server-sent events: one long-lived connection per tab
  exhausted the browser's 6 HTTP/1.1 connections per host and froze the interface with several tabs open.
- `templates/` (askama, French text), `static/` (pictures, `css/app.css` (own stylesheet, no Bootstrap), vanilla
  JS: `app.js` card selection and the French text of a hovered card (`static/data/cards_fr.json`, keyed by picture
  path, combat cards by code), `game.js` map zoom levels/drag, influence clicks and polling the board; served under
  `/static`; scripts, styles and data are sent with `Cache-Control: no-cache`, see `router.rs`). The map picture (`map.jpg`) is drawn at its natural size (4890 × 3362) and scaled with a CSS transform
  to one of three zoom levels (natural size, half, whole map), remembered per board in the browser.
  Story markers are CSS discs placed from `STORY_SPACES` in `map_view.rs` (no picture exists for them).

Notable behavior kept from the original: new games seat computer-controlled heroes and make the creator Sauron;
the AI plays the computer seats, or the board members do when the game was created that way. Shadow cards are disabled.

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
are decided: follow the official rules, the engine enforces the action track. Phases 1–7 are committed (b965361);
Phase 8 is not.

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

**Phase 7 (missions and Finale), done, green:** `model/mission.rs` (`SauronMission`, `HeroesMission`, card number
= picture number, stored as `boards.sauron_objective`/`heroes_objective`), evaluated in `engine/missions.rs` ("The
Spear of the West" waits on quests). `engine/finale.rs` replaces `check_game_end`: `finale_triggered` is checked at
the end of the Story step after the stage minions (`turn.rs`), then immediate victory, marker advance, preparation,
Ringwraiths adjustment, the `Champion` choice (`Chooser::Heroes`) and the final combat (`CombatOrigin::Finale`,
`CombatEnd::GameOver`; no Shadow window, no defeat steps). `finish_heroes_turn` now takes the rng. `Winner::Draw` is
gone. The story screen shows each side its Mission (`web/views/mission_label.rs`). Interpretations to confirm: Q29.

**Phase 8 (optional rules, end-to-end tests), done, green:** `model/game_options.rs` (`boards.options`, JSON;
`ALTER TABLE boards ADD COLUMN options TEXT NOT NULL DEFAULT '{}'` upgrades a database), chosen on the new game
form. Always read dominance through `Game::dominance(data)` (`engine/dominance.rs`, Advanced dominance by simulated
Story steps), not `StoryMarkers::data().dominance`. High stakes stage III in `engine/high_stakes.rs`: Sauron picks
the defeated hero's marker (`DefeatMarker` choice, kept in `TurnEffects::defeat_marker`), a minion defeated in
combat advances the heroes' marker. `tests/common/driver.rs` plays whole rounds over HTTP (one per player count in
`http_flow.rs`). Interpretations to confirm: Q30.

**Computer heroes (AI), done, green:** `engine/ai/`, hooked in `change_game` and the play page; tests in
`engine/ai/tests.rs` (100 seeded games of three rounds against a passive Sauron) and `computer_plays_*` in
`http_flow.rs` (the driver only plays Sauron, `Driver::heroes_by_ai`). Not played by the AI: training, corruption
discard, Dark Path, Argalad's power, returning to movement after exploring.

**Next:** Phase 5 leftovers: quests (text readable on `static/images/hero_cards/quests/standard`: Starting and
Advanced quests), trading between heroes, Argalad's peek limited to one adjacent location. Encounter step and Items
blocked on missing text (and water paths missing from `data/locations_paths.yaml`).
