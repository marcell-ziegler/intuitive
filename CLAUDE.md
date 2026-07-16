# CLAUDE.md

Guidance for AI agents (and humans) working in this repository.

## What this is

**Intuitive** is a D&D 5e initiative-tracker TUI written in **Rust + ratatui**.
Planned feature set: initiative tracking for arbitrary players/creatures,
loadable encounter files, party persistence between sessions, a searchable
5e.tools statblock database (with "open in browser"), and integrated dice rolling.

**Read `docs/PLAN.md` first.** It holds the current-state assessment, the target
architecture, the phased roadmap, and a detailed Phase 0 refactor guide. This
file is the quick operational reference; `docs/PLAN.md` is the source of truth for
*what to build and why*.

## Commands

```bash
cargo build          # compile
cargo test           # run all unit tests (currently 14, must stay green)
cargo run            # launch the TUI (needs a real terminal; not a headless CI)
cargo clippy         # lints — the tree has some dead-code warnings, see below
cargo fmt            # format before committing
```

Notes:
- Rust **edition 2024**.
- `main()` calls `debug_assert!(dotenvy::dotenv().is_ok())`, so in debug builds a
  missing `.env` will panic on startup. Provide a `.env` (may be empty) or run a
  release build if this bites.
- The app persists to XDG dirs: live state in `$XDG_STATE_HOME/intuitive/state.json`,
  encounter files under `$XDG_DATA_HOME/intuitive/encounters/`. Delete `state.json`
  to reset to a clean launch.

## Code layout

```
src/
  main.rs              Event loop + key handling (to be refactored in Phase 0)
  app.rs               `App` state, `EditorState`, serde, App<->SerializableApp
  ui.rs                All ratatui rendering (header, table, sidebar, editor modal)
  storage.rs           XDG paths, versioned EncounterRecord, save/load state
  model.rs             Module root re-exporting Creature, Encounter
  model/
    creature.rs        `Creature` enum (Player/Monster) + CreatureProperties; well-tested
    encounter.rs       `Encounter` (creatures Vec, cursor_index, initiative_index)
    stats.rs           `Stats` + ability-modifier math
    status.rs          `Status` enum (5e conditions)
docs/PLAN.md           Roadmap + architecture + Phase 0 guide
```

## Architecture (current vs. intended)

- **Current:** event handling lives in `main.rs` as nested `match` blocks; `ui.rs`
  renders. State is a single `App` holding the `current_encounter`, `current_panel`
  (`InitiativeTable` / `Sidebar` / `Editor`), and `editor_state`.
- **Intended (Phase 0):** an Elm-style **Model → Update(Action) → View** loop.
  Add `action.rs` (an `Action` enum of user *intents*) and `event.rs` (pure
  key→`Action` mapping), route everything through `App::update(Action) -> Effect`,
  and make rendering read-only (`draw_ui(&App)`). See `docs/PLAN.md` §4 for the
  step-by-step migration — follow it in small, test-green commits.

The `model/` layer is the strongest, best-tested part of the codebase; prefer
extending it over reworking it. Keep `crossterm`/`ratatui` types out of `model/`.

## Conventions

- **Keep `cargo test` green on every commit.** The model layer has thorough tests;
  add tests for new behavior (especially `App::update` logic once it exists — it's
  pure and unit-testable without a terminal).
- Run `cargo fmt` before committing.
- Persisted structs are versioned via a `schema_version` record (see
  `EncounterRecord` in `storage.rs`). When you change a persisted model, bump/handle
  the version. `state.json` does **not** yet have this wrapper — adding
  `AppStateRecord` is a Phase 0 task.
- Match the surrounding style: doc comments (`///`) on public model methods,
  descriptive commit messages.

## Known rough edges / gotchas

- `App::submit_editor()` is `todo!()` (`app.rs`) — the creature editor renders and
  captures input but cannot yet create a creature. This is the top Phase 1 task.
- Initiative order is **insertion order**, not sorted by rolled initiative value;
  `roll_initiative()` exists on `Creature` but nothing in the UI triggers it or
  sorts by it. Real initiative tracking is Phase 1.
- Selection position is tracked redundantly (`Encounter::cursor_index`,
  `Encounter::initiative_index`, `App::main_table_state`) and hand-synced via
  `sync_table_state()`. Phase 0 collapses this to a single source of truth.
- Dead-code warnings exist (`store_encounter`, `centered_rect`) — these are
  intentionally-unused scaffolding for upcoming phases, not bugs to delete.
- The editor's `Amount` field is enumerated but has no backing `Input`, and the
  editor never chooses Player vs Monster — both are Phase 1 gaps.

## Dependencies of note

- `dice-parser` (crates.io, authored by this project's owner) — the dice engine.
  Used today only for `roll_initiative`; Phase 3 exposes it to the UI. Prefer it
  over any other dice logic.
- `ratatui` 0.30, `crossterm` 0.29, `tui-input` 0.15 (text fields), `serde` +
  `serde_json` (persistence), `uuid` (creature IDs), `color-eyre` (error reports).

## Working agreement

- Develop on the designated feature branch; do not push to `master` without
  explicit permission.
- Do not open a pull request unless explicitly asked.
- When in doubt about scope or an ambiguous design choice, check `docs/PLAN.md`
  before improvising — the phasing is deliberate.
