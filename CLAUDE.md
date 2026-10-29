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
    of the Rails `combat_*_test.rb` specs and define the combat rules.
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
