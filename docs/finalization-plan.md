# Finalization plan

Plan for bringing the engine to the official *Middle-earth Quest* rules. Written on 2026-10-29 against commit b2a6934
(branch `rust_conversion`).

- Official rules: [`rules.md`](rules.md) (section numbers § below refer to it).
- Rule-by-rule status of the engine: [`rules-gaps.md`](rules-gaps.md).
- Open questions, missing card text and interpretations to confirm: [`open-questions.md`](open-questions.md).

Each phase ends green (`cargo test`, `cargo clippy --all-targets`, `cargo fmt`). Each new rule gets engine unit tests
whose name or comment cites the `rules.md` section it covers. Tick the boxes as items land.

## Decisions to take first

- [x] **D1. Behavior kept from the Rails app.** The port deliberately kept behavior that contradicts the rules:
  4 computer heroes seated in every game, tied Sauron markers all advancing, no shuffle on rest/heal, free corruption
  discard, combat exhaustion at `>=` instead of "exceeds". Recommendation: follow the official rules (phase 2) and
  rewrite the affected tests. *Decided 2026-10-29: follow the official rules.*
- [x] **D2. Homebrew peril die** (`engine/hero/peril.rs`). Replace it with the 15 Peril cards, or keep it as a
  stopgap until their text is entered. *Done 2026-10-30: the die is gone, the 15 cards are in `data/perils.yaml`.*
- [x] **D3. Enforcement level.** Sauron's turn is an honor system (free action checkboxes, free influence and Shadow
  Pool edits on the map). Recommendation: have the engine enforce the action track (phase 3); every Sauron rule
  depends on it. *Decided 2026-10-29: the engine enforces the action track.*
- [ ] **D4. Card text source.** Needed: Encounter (90), Peril (15), Shadow (26), Quest (15), Item (9), Mission (10),
  Event abilities (42), character favor values, hero abilities. `work/data_and_rules/*.xls` should hold the full text
  of the mission, corruption, peril and shadow cards (going by the file names; not opened yet). Nothing in the
  repository has the encounter or quest text.

## Phase 1 — Bugs and cheap rule deviations (no new data, no decision needed)

Bugs:

- [x] Ringwraiths: when defeated they are moved to `minas_tirith` with life ≤ 0 and lose every later combat
  (`combat/flow.rs` `remove_fought_mob`). Take them off the board; at the start of Sauron's next Action step, put them
  back on Minas Morgul at full life (§ 8.8).
- [x] A plot discarded by a hero vanishes (`exploration.rs` `remove_plot_at_hero_location`). Send it to the plot
  discard pile (§ 5.4).
- [x] The starting quest is drawn twice independently (`setup.rs` `apply_quest_setup`), so the characters placed may
  not match the hero's quest. Draw once (§ 3).

Rule deviations:

- [x] Sauron's plot discard goes to the discard pile, not under the deck (`plots.rs` `discard_plot`).
- [x] Monster discard pile per color; reshuffle into the pile when it is empty (§ 10.2). Fought monsters are
  currently deleted and an empty pile refuses placement.
- [x] No influence in Havens (§ 5.8) in `set_influence`.
- [x] Stronghold maximum influence: add the red number to the strongholds in `data/locations.yaml` and enforce it.
- [x] Hero Rally skips heroes standing on a Shadow Stronghold (`turn.rs` `rally_step`, § 5.2).
- [x] Heal minion refused when a hero shares its location (`sauron_turn.rs` `heal_minion`, § 5.7).
- [x] Hero Draw step: at most fortitude cards, once per round (`heroes::draw_cards`, § 5.12).
- [x] Undefeated monster or minion: Sauron places influence equal to its wisdom in extension (§ 8.3); `wisdom` is
  already loaded in `mob_def.rs`.
- [x] Turn end after combat: Ambush ends the turn only if the hero is defeated; Travel ends it if the mob survives
  (§ 6.4, § 6.5). Today any surviving mob ends the turn.
- [x] Defeated hero goes to the closest Haven by number of locations, Sauron breaking ties (§ 8.9); today it is the
  Haven of the hero's region.
- [x] Defeated hero loses one favor **or** one Item, his choice (§ 8.9).
- [x] Advanced Quest dealt at setup (§ 3).
- [x] Event placing a character already on the board leaves it where it is (`events.rs` `place_character`, § 5.6).
- [x] Two-player game: special Hero Rally step between the hero's two turns (`turn.rs` `start_hero_second_turn`,
  § 10.1).

## Phase 2 — Drop the behavior kept from Rails (after D1)

- [x] Game creation: 1–3 real heroes chosen by the players (Argalad included); Sauron takes 2 actions, 3 with three
  heroes.
- [x] Rest, heal and defeat recovery shuffle the life pool (`hero/pools.rs`).
- [x] Lowest Sauron marker +1 on rest and defeat; on a tie the hero chooses (`StoryMarkers::advance_lowest`, the
  `AfterRestAdvanceStoryMarker` screen already exists).
- [x] Discarding a corruption card costs its favor and undoes its attribute malus (`hero/corruption.rs`).
- [x] Exhaustion when spent strength **exceeds** strength (`combat/reveal.rs` `check_exhaustion`); review
  `combat/tests/exhaustion.rs` first.
- [x] Combat preparation: the hero splits agility between cards drawn and strength (§ 8.3).
- [x] Voluntary exhaustion before a round, hero declaring first (§ 8.7).
- [x] The hero chooses how to pay damage, from hand and/or life pool (§ 6.3); today the life pool is drained first.
- [x] Update the ported Rails tests that encoded the old behavior.

## Phase 3 — Engine-driven Sauron turn (after D3)

- [x] Action track as engine state: 3 tracks × 3 spaces (Influence 6/5/4, Draw 2/2/1, Command 3/2/1 as in
  `sauron_actions.rs`), token on the leftmost empty space, the 4th token clears the other three (§ 5.7).
- [x] `SauronAction` enum and a per-action budget X; 2 or 3 actions per turn.
- [x] Place Influence: X tokens, at most 2 into the Shadow Pool, the rest in extension. Remove free influence and
  Shadow Pool editing outside this action.
- [x] Draw Shadow and Plot: X of each; keep exactly 1 plot, the others under the deck in the order Sauron chooses.
- [x] Command Monsters and Minions: X commands, at most one per monster or minion, at most one monster placement.
- [x] One board state per action, with its route (`engine/route.rs`) and template.

## Status (2026-10-29, end of session 1)

- Phases 1–3 are done. They add database columns (see `schema.sql`), so an existing database needs a fresh start.
- D2: the Peril card text is in `work/data_and_rules/MEQ_Mission_Corruption_Peril_Shadow_Cards_Fulltext.xls`, so
  the die can be replaced. That file also has the text of 21 Shadow cards, the 16 Corruption cards and the Missions.
- D4: all 42 Event cards are transcribed into `data/events.yaml` (name, priority, text, typed `ability`, `affect` for
  flag cards, `story` for the cards that advance a marker). The engine doesn't read the new fields yet. Quest cards,
  Shadow cards and hero sheets exist as pictures. **No Encounter or Item card text exists in the repository.**
- Phase 4 is done (session 2): event step, abilities, card in play, pending choice and its screen, training and
  level tokens, Boat item.

- Phase 5 (session 3, 2026-10-30): threats, Peril cards, consult, Dark Path, corruption effects, hero abilities.
  Schema changes (`heroes.turn_record`, `combats.origin`): an existing database must be recreated. Left: quests and
  trading; Encounters and Items stay blocked.

- Phase 6 (session 4, 2026-10-30): Shadow cards and the plots choosing their location. Schema changes
  (`boards.turn_effects`, `combats.modifiers`): an existing database must be recreated.

- Phase 7 (session 5, 2026-10-30): Mission cards and the Finale. No schema change; `Winner::Draw` is gone, so a
  finished board stored as a draw no longer loads. "The Spear of the West" waits on quests (Q27); interpretations in
  Q29.

## Phase 4 — Events

- [x] Use the stage II and III decks from `data/events.yaml`; the current stage picks the deck (§ 5.3).
- [x] Draw 3 and resolve the lowest priority (heroes dominant) or highest (Sauron dominant); 1 card when nobody is
  dominant (§ 5.6).
- [x] Event data gains `priority`, `flag` (stays in play), `affected_location` and `ability` (`affect` marks the flag
  cards, `story` the cards advancing a marker).
- [x] Current Event card space and event marker on the map; heroes activate it while exploring its location.
- [x] `EventAbility` enum interpreted in `engine/events/`; decisions go through the generic pending choice
  (`model/choice.rs`, `engine/choices.rs`) and the choice screen (`/boards/{id}/choices/choice_screen`).

## Phase 5 — Hero turn content

- [x] Sauron chooses which monster or minion ambushes or attacks (§ 6.4, § 6.5) (`engine/threats.rs`).
- [x] Combat **or** Peril, Sauron's choice (§ 6.5).
- [x] No combat in a Haven unless the hero accepts (§ 8).
- [x] Peril deck: draw 3 (4 against Reckless), Sauron picks one matching the location or region; `PerilEffect`
  enum (`game_data/peril.rs`, `engine/perils/`). Damage outside combat with the hero's choice of hand cards
  (`engine/hero/damage.rs`), defeat outside combat.
- [ ] Encounter step: 5 region decks + Haven deck, draw 3, resolve the lowest matching priority, quests go to the
  play area; none on perilous locations (§ 6.6). **Blocked: no Encounter card text.**
- [x] Explore: consult a character for its favor **or** its ability (8 characters, `data/characters.yaml`; favor
  values unreadable, 2 for all).
- [x] Explore: Dark Path (≤ 3 corruption cards: +1 favor, +1 corruption card; once per turn).
- [ ] Explore: complete quests; trade favor, items and quests with heroes on the same location. Quest text is
  readable on `static/images/hero_cards/quests/standard` (Starting and Advanced quests only).
- [ ] Items (9, one per title; Boat makes water paths cost any one card). **Blocked: no Item card text, and the path
  data has no water paths.** Skills, training and level tokens are done (Phase 4).
- [x] Hero abilities: Beravor (heal or train when resting outside a Haven), Eleanor (+1 favor once per turn when
  consulting; quests later), Thálin (turn goes on after a lost combat), Eometh (Horse). Argalad still sees every
  adjacent location instead of one.
- [x] Remaining corruption card effects: all 16 cards in the deck; Despondent, Hopeless, Mistrusted, Reckless,
  Distraught, Wretched enforced. Indifferent and Greedy wait for Encounter and Shadow cards. Wretched only ends the
  turn when drawn during the hero's turn (Peril, Dark Path).

## Phase 6 — Shadow cards

- [x] The 24 pictured cards (21 different texts; Storms of Mordor, His Arm Has Grown Long and Dark Promises exist
  twice) in `data/shadows.yaml`: Shadow Pool requirement, timing, `ShadowEffect`; at most one per player turn
  (§ 5.9, `TurnEffects::shadow_played`). The rules count 26 cards: 2 have neither picture nor text (Q2).
- [x] Interrupt windows (`engine/shadows/shadow_window.rs`): start of a hero turn, after entering a location, start
  of a combat, after a defeat. Sauron gets a "play a Shadow card or pass" choice (`ShadowWindow`), skipped when he
  holds no playable card. Action step cards are played from the actions screen between two actions.
- [x] Re-enable the shadow deck and the 5 excluded plots: "A Broken Line of Kings" (×2), "Loss of an Heir", "A Promise
  of Rings" and "Urgent Summons" let Sauron pick their location (`engine/plot_locations.rs`, `PlotLocation`
  choice). Greedy is enforced.

## Phase 7 — Finale and missions

- [x] `SauronMission` and `HeroesMission` (`model/mission.rs`, numbered as their pictures) evaluated in
  `engine/missions.rs`. "The Spear of the West" is never fulfilled until quests can be completed (Q27).
- [x] The Finale replaces the immediate winner (`engine/finale.rs`): triggered at the end of the Story step, after
  the stage minions enter; immediate victory check, marker advance for fulfilled missions, preparation (Ringwraiths
  healed or brought back, heroes reshuffle and draw), Ringwraiths adjustment by dominance, champion choice
  (`Champion` choice, any hero player answers), final combat (`CombatOrigin::Finale`), winner (§ 9).
- [x] Each side sees its own Mission on the story screen, both once the game is over; French log lines for the
  Finale.

## Phase 8 — Optional rules and polish

- [ ] Per-game options: Advanced dominance, High stakes stage III (§ 10.4).
- [ ] End-to-end HTTP flow tests (`tests/http_flow.rs`) covering a full round for each player count.

## Card effect modelling (phases 4–7)

Card effects are typed enums in `game_data/`, deserialized from YAML, and interpreted by free functions in `engine/`
with one `match` per enum. An effect that needs a player choice returns a pending decision that becomes a board state,
the way combat card loss and the Gollum choice already work. No scripting language.
