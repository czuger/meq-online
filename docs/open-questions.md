# Open questions

Questions that need a decision, missing source material or a check against the physical game before the rules work
can be called finished. Section numbers (§) refer to [`rules.md`](rules.md); the phased plan is
[`finalization-plan.md`](finalization-plan.md).

Each entry gives the context, what the engine does today and a recommendation. When a question is settled, write the
decision and the date under it, apply it, then move it to **Settled** at the end of this file.

Status: **Open** (needs an answer) · **Blocked** (needs material nobody has yet) · **To confirm** (the engine follows
an interpretation that should be checked against the real cards or rulebook).

## 1. Data errors

### Q1. Mirkwood and Misty Mountains region codes look swapped — Open

`data/locations.yaml` gives the `mirkwood` region code to Rivendell, the High Pass, the Redhorn Gate, Lothlórien,
Fangorn, the Gladden Fields and Rhosgobel, and the `misty_mountains` code to the Woodland Realm, the Forest Road, the
Forest Trail, the Bight, the South Eaves and Lake Esgaroth. The colors look right (the Peril sheet of the xls gives
"Misty Mountains or Mirkwood: green, light green", which matches `dark_green` for the Rivendell group), so only the
names and codes seem swapped.

- **Impact**: "Beornings, the Skin Changers" (stage I event 1, `region: mirkwood`) acts on the Misty Mountains
  locations. Anything else matching regions by code, and the region names shown to players, are affected the same
  way. The Peril cards are not, because "Undying Regret" names both regions.
- **Recommendation**: swap `region`/`region_code` between the two groups in `data/locations.yaml` after checking one
  location on the board, then check the other region codes the same way (`rhoan`, `rhuadur` and `endwith` are
  spelled oddly but look consistent).

### Q2. How many Shadow cards are there? — Open

The plan says 26 Shadow cards, the xls has the text of 21, `static/images/sauron/shadows` has 24 pictures, and
`setup.rs` builds the deck from cards 0–23 minus `EXCLUDED_SHADOW_CARDS` (1, 21).

- **Needed before Phase 6**: the real card count, and which pictures go with the 21 texts. Cards with no text can't
  be implemented.

### Q3. Character favor values — Blocked

The favor printed on the 8 character tokens can't be read on the pictures. Every character gives 2 favor
(`data/characters.yaml`).

- **Needed**: the real values (a sharper scan or the physical tokens).

## 2. Missing card text

### Q4. Encounter cards — Blocked

No text exists for the 90 Encounter cards (5 region decks and the Haven deck). The Encounter step (§ 6.6), the quests
found in encounters, the Indifferent corruption card and every quest step saying "when you would draw Encounter
cards" (for example Argalad's "The Forest Must Provide No Refuge") wait on it.

- **Options**: find or transcribe the cards; or leave the step out and show the hero that his turn ends after
  travelling, as today.

### Q5. Item cards and water paths — Blocked

No text exists for the 9 Item cards. Only the Horse (a −1 path difficulty, an approximation) and the Boat exist.
`data/locations_paths.yaml` has no water paths, so the Boat does nothing while moving.

- **Needed**: the Item texts; the list of water paths (from the board picture `static/images/map.jpg`).
- **Also open**: the Horse rule itself, once its card text is known.

## 3. Interpretations made in Phase 5 — To confirm

The cards or the rulebook leave these open; the engine follows the choice given.

| # | Topic | What the engine does | Source |
|---|---|---|---|
| Q6 | Reckless ("Sauron **may** draw 1 additional card") | Sauron always draws 4 Peril cards; drawing more never hurts him. | `perils/peril_deck.rs` |
| Q7 | Wretched drawn outside the hero's turn (from an Event) | The card only goes back into the deck; nothing ends and the hero doesn't move. | `hero/corruption.rs` |
| Q8 | "Ill-met Company": lose 2 favor | Offered only to a hero holding at least 2 favor. | `perils/effects.rs` |
| Q9 | "Armies Underground" and "Khazad-dûm": the hero fails to defeat the Orc or the Balrog | The turn ends, as for any monster met while travelling (the cards say nothing). The monster is not taken from a monster pool and leaves the board after the combat. | `combat/flow.rs` `after_combat` |
| Q10 | "The Dark Tower" with no minion on Barad-dûr | No effect. With several minions there, Sauron picks one. | `perils/effects.rs` |
| Q11 | "Nine for Mortal Men": "command 1 minion" | Only a move to an adjacent location; healing isn't offered. | `perils/effects.rs` |
| Q12 | "It's a Dangerous Business" and "Undying Regret" placements | Influence may go on any location of the hero's region except Havens and the hero's location, ignoring the extension rule and the stronghold maximum. Monster tokens also avoid every hero's location. | `perils/effects.rs` `hero_region_targets` |
| Q13 | "The Hounds of Sauron": shield icons | Shields are the printed defense of the discarded cards. The discarded cards go to the rest pool. | `perils/answers.rs` |
| Q14 | Mistrusted: "1 favor during each future turn" | The count restarts when the hero's turn starts and when it ends; favor gained during Sauron's turn counts against nothing. | `hero/training.rs` `gain_favor` |
| Q15 | Denethor: "advance story marker 1 space" | The heroes' (green) marker advances. | `exploration.rs` |
| Q16 | Saruman: "Sauron discards 2 random Shadow cards" | Taken from Sauron's hand, put on the Shadow discard pile. | `exploration.rs` |
| Q17 | Beravor training when resting | Training is resolved before the Sauron story marker advances. | `rest.rs` |
| Q18 | Eleanor's Noble Bloodline | One bonus per turn, shared between consulting a character and completing a quest. | `exploration.rs` |
| Q19 | Haven with a monster at the Ambush step | The hero is asked first. If he refuses, he can't rest (a monster is on his location) and goes straight to movement. | `threats.rs`, `rest.rs` |
| Q20 | Ambush: Sauron picks a blank token while other mobs stand there | The token is discarded and nothing else attacks this step. | `threats.rs` |
| Q21 | After every move | The hero lands on the exploration screen even when there is nothing to explore (the Dark Path is nearly always available there). | `movement.rs` |

## 4. Deviations still in the engine

### Q22. Stage II and III minions start on fixed locations — Open

Gothmog enters on Mount Gundabad, the Ringwraiths and the Witch-king on Minas Morgul (`turn.rs` `STAGE_SPAWNS`).
The rules let Sauron pick among the starting locations printed on each minion space (§ 5.3).

- **Needed**: the printed starting locations of each minion; then a Sauron choice like the others.

### Q23. Argalad's Scout ability — Open

The sheet says "look at the faces of all monster tokens in **an** adjacent location"; the engine shows every adjacent
location at once (`movement.rs` `surrounding_monsters`).

- **Recommendation**: let Argalad pick one adjacent location first.

### Q24. Excluded plots — Open

Five plot cards (`setup.rs` `EXCLUDED_PLOTS`: 10, 11, 14, 15, 17) are left out of the deck, together with the Shadow
deck. Planned for Phase 6; it needs their requirements and effects checked against the pictures.

### Q25. Greedy and Indifferent — Blocked

Both corruption cards sit in the deck without effect until Shadow cards (Phase 6) and Encounter cards (Q4) exist.

## 5. Project questions

### Q26. `rules-gaps.md` is out of date — Open

It describes commit b2a6934, before Phases 1–5, and still lists as missing most of what is done.

- **Options**: refresh it against the current engine; or mark it as a historical snapshot and keep
  `finalization-plan.md` as the only status tracker.
- **Recommendation**: mark it as a snapshot (one line at the top) and stop maintaining it.

### Q27. Quests beyond the Starting and Advanced quests — Open

Only the 15 Starting and Advanced quest cards are readable (`static/images/hero_cards/quests/standard`). Quests found
in encounters wait on Q4, and some Starting quests refer to the Encounter step.

- **Decision needed**: implement the Starting and Advanced quests now with their Encounter-related setup steps left
  out (logged as such), or wait for the Encounter cards.

## Settled

- **D1** (2026-10-29): follow the official rules rather than the behavior kept from the Rails app.
- **D2** (2026-10-30): the peril die is replaced by the 15 Peril cards.
- **D3** (2026-10-29): the engine enforces Sauron's action track.
