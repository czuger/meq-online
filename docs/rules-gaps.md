# Rules gap analysis

Comparison of the engine (`src/engine/`, branch `rust_conversion`, commit b2a6934) with the official rules in
[`rules.md`](rules.md). Section numbers (§) refer to `rules.md`.

Status legend: **OK** implemented as written · **Partial** present but incomplete or not enforced · **Deviates**
behaves differently from the rules · **Missing** absent · **Bug** wrong even by the engine's own intent.

## 1. Gap matrix

### Setup (§ 3)

| Rule | Status | Notes |
|---|---|---|
| Choose sides, 1 Sauron + 1–3 heroes | Deviates | Every game seats 4 computer heroes (beravor, eleanor, thalin, eometh) and Sauron (`setup.rs` `STARTING_HEROES`). Rules allow at most 3 heroes; Argalad is never playable. Kept quirk. |
| Random Starting Quest + setup instructions | Partial / Bug | Quest is drawn twice independently (`apply_quest_setup` re-draws), so the characters placed may not match the stored quest. Advanced Quest not dealt. |
| Starting Plot on slot 1, its influence | OK | |
| Starting minions | OK | Mouth of Sauron at Dol Guldur, Black Serpent at Near Harad. |
| Shadow, Peril, Advanced Plot decks | Partial | Shadow deck built but disabled; no Peril deck; 5 plots excluded (`EXCLUDED_PLOTS`). |
| Event decks per stage | Missing | Only stage I deck is used (`BASE_EVENT_SET`); stage II/III data exists in `data/events.yaml`. |
| Encounter decks (5 regions + Haven) | Missing | No data, no deck. |
| Skill deck, Item deck, level tokens | Missing | `data/skills.yaml` loaded but unused. Only a `horse` flag exists. |
| Mission cards | Partial | One random index per side (`set_objectives`), shown as a picture, never evaluated. |

### Sauron turn (§ 4, § 5)

| Rule | Status | Notes |
|---|---|---|
| Hero Rally: remove influence at heroes' locations | Partial | Doesn't exempt heroes on a Shadow Stronghold (§ 5.2). |
| Rally: cut chains keep 1 influence | OK | `InfluencePaths::max_for_existing_token`. |
| Story step: heroes +2, plots advance | OK | Run at the end of the heroes' round; skipped on turn 1 by construction. |
| Stage spawns (II: Ringwraiths, Gothmog; III: Witch-king) | OK | Fixed locations; rules let Sauron pick among the printed starting locations. |
| Finale trigger | Deviates | Game ends immediately with a winner (or a draw); no Finale (§ 9). |
| Plot step: play / discard / nothing | OK | Requirements checked for 6 plots. |
| Plot step: full track blocks playing | OK | |
| Sauron discards a plot → discard pile | Deviates | Goes under the Plot deck (`discard_plot`). |
| Hero discards a plot → discard pile | Bug | Card disappears from the game (`remove_plot_at_hero_location`). |
| Event step: 3 cards, lowest/highest priority by dominance | Missing | Always the top card of stage I. |
| Event abilities, green-flag persistence, event marker | Missing | Only character and favor placement. |
| Event: character already on board stays | Deviates | `place_character` moves it. |
| Action step: 2 actions (3 with three heroes) | Missing | Honor system: free checkboxes (`sauron_actions.rs`), only a counter is shown. |
| Action track: leftmost empty space, X value, 4th token clears | Missing | Checkboxes don't enforce order or reset. |
| Place Influence: X tokens, ≤ 2 in Shadow Pool | Missing | Influence set freely on the map; Shadow Pool ± freely. |
| Influence only in extension | OK | `InfluencePaths::max_for_new_token`. |
| No influence in Havens | Missing | Not checked. |
| Stronghold red maximum | Missing | Not modelled. |
| Shadow Pool cap 4 × stage | OK | |
| Draw Shadow and Plot: X of each, keep 1 plot | Deviates | Free count, several plots kept, shadow draw disabled. |
| Command: X commands, 1 per mob, 1 placement per action | Missing | Unlimited moves and placements. |
| Monster placement: influenced, not on heroes | OK | Plus "next to the Black Serpent" (minion ability). |
| Monster moves only onto influence; minions anywhere | OK | |
| Heal minion: up to 4, not in a hero's location | Partial | Location restriction missing. |
| Hero Draw step: up to fortitude | Partial | Count not capped and the draw can be repeated (`heroes::draw_cards`). |
| Shadow cards (§ 5.9) | Missing | Disabled; no timing windows, no one-per-turn limit. |

### Hero turn (§ 6)

| Rule | Status | Notes |
|---|---|---|
| Rest: rest pool → life pool, **shuffled** | Deviates | Not shuffled. Kept quirk. |
| Rest: lowest Sauron marker +1, hero picks among ties | Deviates | All tied markers advance (`advance_lowest(true)`); the choice screen is unreachable from rest. Kept quirk. |
| No rest with a mob present / on a Stronghold | OK | |
| Heal in a Haven | OK | |
| Discard a Corruption card when resting, paying favor | Deviates | Favor not paid, stat malus not removed. Kept quirk. |
| Ambush: **Sauron chooses** the mob | Deviates | Engine picks the strongest automatically (`check_location_encounters`). |
| Ambush: turn continues unless the hero is defeated | Deviates | Turn ends whenever the mob survives (`terminate_combat`). |
| No combat in a Haven without the hero's consent | Missing | |
| Move: 1 matching card or N cards | OK | Mixed payments accepted. |
| Water paths + Boat | Missing | No Boat item. |
| Combat **or** Peril, Sauron chooses | Deviates | Peril then combat are both applied automatically. |
| Peril deck (draw 3, Sauron picks a matching one) | Deviates | Replaced by a 1-in-5 die and a corruption card (`hero/peril.rs`). |
| Blank monster token discarded, hero explores | OK | |
| Explore: retrieve favor | OK | |
| Explore: consult character — favor **or** ability | Deviates | Always 2 favor; abilities (`data/characters.yaml`) unused; per-character favor values missing. |
| Explore: Dark Path | Missing | |
| Explore: complete quests | Missing | |
| Explore: discard a plot for favor | OK | (see the discard pile bug above) |
| Explore: trade with heroes | Missing | |
| Explore: activate the current Event card | Missing | |
| Encounter step (3 cards, lowest matching priority, quests) | Missing | |
| Hero abilities (5 hero sheets) | Partial | Argalad's monster peek only; Eometh starts with a Horse. |

### Other elements (§ 7)

| Rule | Status | Notes |
|---|---|---|
| Level tokens, max 2 per attribute | Missing | |
| Skill cards and training | Missing | |
| Item cards (9), one per title | Partial | Horse only (−1 path difficulty, an approximation). |
| Corruption cards: 16 effects | Partial | Stat modifiers, Cowardly, Dispairing, Isolated, Careless. |
| Corruption from Shadow/Event/Encounter cards | Missing | Only from the peril die. |

### Combat (§ 8)

| Rule | Status | Notes |
|---|---|---|
| Preparation: split agility between draw and strength | Deviates | All-or-nothing choice (`CombatPreparation`). |
| Exhausted when spent strength **exceeds** strength | Deviates | Engine uses `>=` (`reveal.rs` `check_exhaustion`); check `combat/tests/exhaustion.rs` before changing it. |
| Voluntary exhaustion declaration | Missing | Only an empty hand exhausts. |
| Previous / current abilities, hero first | OK | Order kept from Rails (cancel abilities resolve in the current phase). |
| Hero chooses damage from hand or life pool | Deviates | Life pool is drained first; the hand only pays the rest. |
| Monster token discarded after combat | OK | |
| Discarded monster tokens reshuffle when a pile is empty | Missing | Fought monsters are deleted; an empty pile refuses placement. |
| Minion damage persists | OK | |
| Undefeated mob: Sauron places influence = wisdom | Missing | `wisdom` is loaded (`mob_def.rs`) but unused. |
| Ringwraiths return to **Minas Morgul** at Sauron's next Action step | Bug | Moved to `minas_tirith` immediately and **life is not restored** (stays ≤ 0, so it loses every later combat). |
| Defeated hero: lowest marker +1 (choose ties) | Deviates | All tied markers advance. |
| Defeated hero: lose 1 favor **or** an Item | Partial | Favor only. |
| Defeated hero: **closest** Haven, Sauron breaks ties | Deviates | Haven of the hero's region (`move_to_regional_haven`). |
| Defeated hero: rest + damage pools → life pool | OK | (not shuffled) |

### Winning and other rules (§ 9, § 10)

| Rule | Status | Notes |
|---|---|---|
| Dominance | OK | `StoryMarkers::data`. |
| Mission evaluation (10 cards) | Missing | |
| Finale (7 steps, Ringwraiths champion fight) | Missing | |
| Two-player game (hero plays twice) | Partial | `start_hero_second_turn` gives the second turn and a draw, but skips the special Hero Rally step in between. |
| Reshuffling depleted decks | Partial | Event, corruption, shadow decks reshuffle; monster piles don't. |
| Optional rules | Missing | Low priority. |

## 2. Plan

Decisions and phased plan: [`finalization-plan.md`](finalization-plan.md).
