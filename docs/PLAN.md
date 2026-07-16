# Intuitive — Implementation Plan & Architecture Notes

A D&D 5e initiative tracker TUI in Rust + ratatui.

This document captures (1) where the codebase is today, (2) a phased plan to
reach the full feature set, (3) the architectural refactors that unblock that
work, and (4) an honest assessment of a possible port to Python + Textual.

---

## 1. Where the code is today

### What exists and is solid

- **Model layer** (`src/model/`) is the strongest part of the project and is
  well unit-tested (14 passing tests).
  - `Creature` is an enum with `Player { props, level }` and
    `Monster { props, cr }` variants sharing a `CreatureProperties` struct
    (name, hp, max_hp, ac, is_dead, statuses, initiative, stats). Accessors and
    `props_mut()` keep the shared/variant split clean.
  - `damage`/`heal` encode the 5e down-vs-dead rule (`DamageOutcome`), `Stats`
    computes ability modifiers, `Status` covers the 5e conditions (with data on
    `Exhaustion(u8)` and `Grappled(CreatureId)`).
  - `roll_initiative()` already uses the `dice-parser` crate to roll
    `1d20 + dex_mod`.
- **Storage** (`src/storage.rs`) writes to XDG dirs, with an `EncounterRecord`
  wrapper carrying a `schema_version` and version-checked loading.
- **UI** (`src/ui.rs`) renders a titled header, an initiative `Table` with
  cursor/turn highlighting and a keybind hint bar, a sidebar placeholder, and a
  partially-built modal creature editor.

### What is stubbed, missing, or inconsistent

These are the concrete blockers, roughly in priority order:

1. **`App::submit_editor()` is `todo!()`** (`src/app.rs:145`). The editor renders
   and captures keystrokes but cannot actually create a creature. This is the
   single biggest "the app doesn't do its core job yet" gap.
2. **Initiative is order-of-insertion, not sorted by rolled value.**
   `initiative_index` just walks the `Vec` in push order (`select_next_initiative`
   wraps `+1 % len`). There is no sort-by-initiative-descending, no tie-break, and
   no UI to trigger `roll_initiative()`. "Initiative tracking" is the headline
   feature and is not yet real.
3. **Editor `Amount` field is enumerated but has no `Input`** (`EditorState` has
   no `amount_input`), and the Monster/Player distinction is never chosen in the
   editor — everything would be one type. The "Lvl / CR" field is a raw string.
4. **Sidebar is an empty placeholder** — intended home for statblock/detail view.
5. **No delete / edit-existing / reorder** of creatures.
6. **No damage/heal/status UI** despite full model support.
7. **Encounter file load/save has no UI or commands** (`store_encounter` is
   dead code today; `load_encounter` is never called).
8. **No party concept** distinct from encounters.
9. **No 5e.tools statblock database** at all.
10. **Dice rolling is internal-only** — no UI to roll arbitrary expressions.

### Architectural smells worth fixing before piling on features

- **Input handling lives in `main.rs`** as nested `match` blocks with
  `todo`/empty arms (`handle_main_view_key_event` is empty, dead). Every new
  panel/mode will bloat this. There is no separation between "an input event"
  and "a thing that happens to the model."
- **Persistence on every keystroke.** `store_state` is called after each
  up/down/space in the panel handler. It re-serializes and rewrites the whole
  state file synchronously inside the render loop. Fine at current scale, but it
  couples navigation to disk I/O and will not scale to a statblock DB.
- **Two inconsistent persistence schemes.** Encounters get a versioned
  `EncounterRecord`; the live `App`/`state.json` is serialized raw with no
  version wrapper, so a future model change silently breaks old state files.
- **`App` owns ratatui `TableState`** and must call `sync_table_state()`
  everywhere. View state (scroll/selection) is entangled with domain state.
- **Redundant selection state:** `Encounter.cursor_index` +
  `App.main_table_state` + `initiative_index` all track "position" and must be
  manually kept in sync.

---

## 2. Recommended architecture (target shape)

Adopt a lightweight **Model → Update(Action) → View** loop (Elm/TEA-style),
which ratatui projects converge on naturally and which makes each feature a
localized change instead of another `match` arm in `main.rs`.

```
src/
  main.rs            // terminal init + event loop only
  event.rs           // crossterm Event -> Action (per-mode keymaps)
  action.rs          // enum Action { Move(Dir), NextTurn, RollAllInitiative,
                     //   Damage(u32), OpenEditor(Mode), SubmitEditor, Quit, ... }
  app.rs             // App state + `fn update(&mut self, Action) -> Effect`
  ui/
    mod.rs           // draw dispatch
    table.rs         // initiative table
    sidebar.rs       // statblock/detail view
    editor.rs        // creature editor modal
    roller.rs        // dice roll modal + log
  model/             // (unchanged, already good)
  storage/
    mod.rs           // versioned records, save/load
    encounter.rs     // encounter files
    party.rs         // party persistence
    statblocks.rs    // 5e.tools bestiary index + lookup
  data/              // bundled/imported 5e.tools JSON (see Phase 4)
```

Key moves:

- **Introduce an `Action` enum** and a single `App::update(Action)`. `event.rs`
  translates keys → `Action` based on the current mode/panel; `update` mutates
  the model and returns an `Effect` (e.g. `Effect::Persist`, `Effect::None`).
  This kills the scattered `match` blocks and the empty/`todo` handlers.
- **Debounce persistence.** Persist on meaningful mutations and on quit, not on
  every cursor move — return `Effect::Persist` only from mutating actions, and
  coalesce (dirty flag + save on quit / on a timer).
- **Give `App`/`state.json` the same versioned-record treatment** as encounters
  (an `AppStateRecord { schema_version, .. }`), so state migrations are possible.
- **Collapse selection state.** Treat `Encounter` as the source of truth for
  `cursor_index`/`initiative_index`; derive ratatui `TableState` at render time
  from it rather than storing and syncing a second copy.
- Keep the model layer as-is — it is the part that's right.

None of this is a rewrite; it's a reshaping that can land incrementally
(Phase 0) before feature work.

---

## 3. Phased implementation plan

### Phase 0 — Foundation refactor (1 focused pass)
- Add `action.rs` + `event.rs`; route `main.rs` through `App::update`.
- Version-wrap `state.json`; add a `save-on-quit + dirty-flag` persistence path.
- Derive `TableState` from `Encounter` instead of storing it.
- **Exit criteria:** identical behavior, `main.rs` under ~40 lines, tests green.

### Phase 1 — Make the core loop actually work (highest user value)
- Implement `submit_editor()`: parse fields, choose Player vs Monster (add a
  type toggle + the missing `amount_input` to spawn N copies like "Goblin 1..3"),
  validate numeric input, push creature(s).
- Add edit-existing and delete (`e` / `d`), and a confirm for delete.
- **Real initiative:** an action to roll initiative for all (or per-creature),
  sort the encounter descending by initiative with a dex-mod / manual tie-break,
  and make "next turn" advance through the *sorted* order with round counting.
- Wire damage/heal/status keys to the existing model methods (small modal or
  inline prompt).
- **Exit criteria:** you can run a full combat from an empty screen without
  touching a JSON file.

### Phase 2 — Encounter files & party persistence
- Command/menu to **save** current encounter (reuse `store_encounter`) and
  **load** one via a file picker over `$XDG_DATA_HOME/intuitive/encounters`.
- **Party** = a persisted roster of `Player` creatures (`party.rs`, own record +
  version). Actions: "add party to encounter", "save current players as party".
- Auto-restore last session (already the default-load behavior) but keep the
  live-session state and the named-encounter files distinct.
- **Exit criteria:** quit mid-fight, relaunch, resume; load a prepped encounter
  and drop the standing party into it in a couple of keystrokes.

### Phase 3 — Dice roller surface
- A roller modal (`r`) that takes a `dice-parser` expression string, shows the
  total and the per-die breakdown, and keeps a small scrollback log.
- Surface initiative rolls and (optionally) attack/damage rolls through the same
  log so there's one roll history.
- **Exit criteria:** arbitrary `2d6+3`, advantage/disadvantage, kept-highest
  work from the UI with visible breakdowns.

### Phase 4 — 5e.tools statblock database (largest subsystem)
This is the one that needs its own design decisions — flag them explicitly:

- **Data sourcing & licensing.** 5e.tools bestiary data is community-maintained
  JSON. Decide up front: bundle a snapshot vs. import-on-first-run from a
  user-provided path. Bundling redistributes their data (check the 5e.tools /
  underlying SRD-vs-non-SRD licensing before committing to redistribution). A
  safe default: ship SRD-only monsters, and support importing a local 5e.tools
  data dir for everything else.
- **Ingestion.** Write a `statblocks.rs` importer that parses the bestiary JSON
  into an internal `Statblock` type (superset of `CreatureProperties`: actions,
  traits, speeds, saves, resistances, source book, page). Build a lightweight
  on-disk index (name → file/offset) so startup isn't "load 5e.tools into RAM."
  Consider a small embedded store (`sqlite`/`redb`) if fuzzy search over
  thousands of entries gets slow; start with an in-memory index of names.
- **Search & attach.** A search modal to find a monster by name/CR/type and
  instantiate it as a `Monster` creature (mapping HP/AC/stats/CR across).
- **Sidebar detail view.** Render the selected creature's full statblock in the
  now-empty sidebar.
- **"Open in browser"** action: build the 5e.tools URL for the statblock and
  launch it (`open`/`xdg-open`/`start` via a tiny cross-platform opener; the
  `open` crate is the standard choice).
- **Exit criteria:** search "Goblin", preview its statblock, add it to the
  encounter, and open its 5e.tools page in the browser.

### Cross-cutting, do continuously
- Keep the model's unit-test discipline; add tests for initiative sorting,
  editor parsing, and statblock import (golden JSON fixtures).
- A `--help`/keybind overlay as the keymap grows.
- Config file (theme, data paths, default dice) once there's more than one knob.

---

## 4. Should this be ported to Python + Textual?

Short version: **the Rust foundation here is good, the risky/uncertain work is
still ahead, and Textual would buy real velocity on exactly that work — but the
one already-solved asset (the `dice-parser` crate) and the well-modeled domain
are Rust. Recommendation: stay in Rust; only reconsider if UI iteration speed
becomes the actual bottleneck.**

### Where Textual would genuinely help (dev comforts / velocity)

- **UI iteration is much faster.** Textual has CSS-like styling (TCSS with hot
  reload), a real widget/layout system, built-in focus/scroll/mouse, `DataTable`,
  `Input`, `ModalScreen`, and reactive attributes. Much of what `ui.rs` builds by
  hand (centered-rect math, manual focus tracking, per-field `Input` wiring,
  manual highlight styling) is out-of-the-box. The editor + sidebar + roller
  modals would be dramatically less code.
- **Async is first-class.** Phase 4's browser-open, statblock import, and any
  network/data fetching are trivial with `async` workers; in Rust you'd reach for
  threads/channels and keep the render loop non-blocking by hand.
- **The event/action pattern is native.** Textual's message system *is* the
  Elm-style loop recommended in §2 — you'd get it for free instead of building
  `action.rs`/`event.rs`.
- **Data wrangling for 5e.tools is Python's home turf.** JSON ingestion, fuzzy
  search (`rapidfuzz`), and an embedded `sqlite3` are batteries-included.
- **Faster edit-run-see cycle** overall (no compile step; `textual run --dev`).

### Where the port costs you

- **You throw away the strongest, already-working code.** The whole `model/`
  layer + tests, the storage/versioning, and the mostly-built table/editor would
  be rewritten. That's re-doing solved work to redo the *unsolved* UI work faster.
- **`dice-parser` is a Rust crate you own and already integrated.** In Python
  you'd either reimplement it, call it via `pyo3`/subprocess (friction), or adopt
  a different Python dice library and re-validate behavior. This is the biggest
  single argument against the port.
- **Type safety / correctness.** The domain leans on Rust's enums and exhaustive
  matches (`Creature`, `Status`, `DamageOutcome`). Python + `mypy`/`pydantic`
  gets close but won't match the compiler-enforced guarantees the model relies on.
- **Distribution.** Rust ships a single static binary; a Textual app needs a
  Python runtime + deps (mitigable with `uv`/`pipx`/PyInstaller, but it's real
  friction for a "download and run" TUI).
- **Performance headroom** for a large statblock DB and instant search favors
  Rust, though at TUI scale this is unlikely to matter.

### Net assessment

| Dimension | Rust + ratatui | Python + Textual |
|---|---|---|
| UI iteration speed | Slower (manual layout/focus, compile) | **Fast** (TCSS, widgets, hot reload) |
| Reuse of existing code | **Keeps model, storage, dice-parser** | Rewrite everything |
| Domain correctness | **Compiler-enforced enums** | mypy/pydantic (weaker) |
| 5e.tools data/search | Manual index or embedded DB | **sqlite + rapidfuzz built-in** |
| Async (browser/import) | Threads + channels by hand | **Native async workers** |
| Distribution | **Single binary** | Runtime + deps |
| Dice engine | **Already done (your crate)** | Reimplement / FFI |

Because the parts Textual accelerates (modals, styling, async, data search) are
still *unbuilt*, a switch now is more defensible than it would be later — but the
`dice-parser` investment and the solid model layer tip the balance toward
**staying in Rust**. A reasonable middle path if UI friction bites: keep the Rust
core and lean harder on higher-level ratatui helper crates (e.g. `tui-textarea`
for editing, `tui-popup`/component patterns) to close the ergonomics gap without
abandoning the binary or the dice crate.

**Trigger to revisit:** if, during Phase 4, hand-rolling the sidebar statblock
view + search UX in ratatui is what's actually slowing you down, prototype *that
one screen* in Textual before committing to a full port.

---

## 5. Suggested immediate next step

Do **Phase 0 + the `submit_editor()` half of Phase 1** together: it's the
smallest change that turns the app from "renders" into "usable," and doing the
`Action`/`update` refactor first means every feature after it is additive rather
than another `match` arm in `main.rs`.
