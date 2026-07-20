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
_what to build and why_.

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

```txt
src/
  main.rs              Terminal init + event loop (~40 lines, routes through App::update)
  action.rs            `Action` enum — user intents
  event.rs             `map_event(&app, &event) -> Option<Action>`, mode-aware keymap
  app.rs               `App` state, `Effect`, `update()`, serde, App<->SerializableApp
  editor.rs            `EditorState`/`EditorInput` — editor field state + validation
  ui.rs                 Layout + dispatch; renders read `&mut App` (Step 4 of Phase 0 pending)
  ui/
    table.rs           Initiative table widget
    sidebar.rs          Sidebar placeholder widget
    editor.rs           Editor modal widget (reads `EditorState`)
  storage.rs            XDG paths, versioned EncounterRecord, save/load state
  storage/encounter.rs   `Encounter` domain type (NOTE: lives here, not model/ — see docs/PLAN.md)
  model.rs               Module root re-exporting Creature
  model/
    creature.rs         `Creature` enum (Player/Monster) + CreatureProperties; well-tested
    stats.rs             `Stats` + ability-modifier math
    status.rs            `Status` enum (5e conditions)
docs/PLAN.md            Roadmap + architecture + Phase 0 guide + progress log
```

## Architecture (current vs. intended)

- **Current: Phase 0 is done.** An Elm-style Model → Update(Action) → View loop.
  `event.rs` maps `crossterm` events to `Action`s (mode-aware per `Panel`),
  `main.rs` routes every action through `App::update(Action) -> Effect`, and
  `Effect::Quit`/`Effect::UpdateState` drive the loop. `draw_ui(&App)` is
  read-only — `TableState` is derived fresh each frame from
  `current_encounter.initiative_index` rather than stored on `App`. Persistence
  uses a `state_dirty` flag + a 3s-debounced autosave (`main.rs`'s poll loop),
  with `Effect::UpdateState` forcing an immediate write for actions like
  `SubmitEditor`. `state.json` and encounter files both go through versioned
  records (`AppStateRecord`/`EncounterRecord` in `storage.rs`). State is a single
  `App` holding `current_encounter`, `current_panel`, and `editor_state`.
- See `docs/PLAN.md` §4 and the Phase 0 progress log for the full history,
  including a couple of deliberate deviations from the original guide (dirty-flag
  persistence instead of a bool set only on `Quit`; `Encounter` living in
  `model/encounter.rs`, which the guide's own target layout agrees with).

The `model/` layer is the strongest, best-tested part of the codebase; prefer
extending it over reworking it. Keep `crossterm`/`ratatui` types out of `model/`.

## Conventions

- **Keep `cargo test` green on every commit.** The model layer has thorough tests;
  add tests for new behavior (especially `App::update` logic once it exists — it's
  pure and unit-testable without a terminal).
- Run `cargo fmt` before committing.
- Persisted structs are versioned via a `schema_version` record (`EncounterRecord`
  and `AppStateRecord` in `storage.rs`). When you change a persisted model,
  bump/handle the version.
- Match the surrounding style: doc comments (`///`) on public model methods,
  descriptive commit messages.

## Known rough edges / gotchas

- `App::submit_editor()` works now (validates, spawns single or numbered-copy
  creatures via the `Amount` field), but always builds a `Monster` — the editor
  never lets you choose Player vs Monster. Remaining Phase 1 gap.
- Initiative order is **insertion order**, not sorted by rolled initiative value;
  `roll_initiative()` exists on `Creature` but nothing in the UI triggers it or
  sorts by it. Real initiative tracking is Phase 1.
- Selection position is tracked redundantly (`Encounter::cursor_index`,
  `Encounter::initiative_index`, `App::main_table_state`) and hand-synced via
  `sync_table_state()`, including mid-render in `ui.rs`. Phase 0 collapses this
  to a single source of truth — not done yet.
- `Effect::UpdateState` is returned from `SelectNextRow`/`SelectPreviousRow`/
  `AdvanceTurn`, so cursor movement writes `state.json` synchronously on every
  keypress — the same per-keystroke-persistence smell Phase 0 set out to fix.
  See `docs/PLAN.md`'s Phase 0 progress log before changing this.
- Dead-code warnings exist (`store_encounter`, `load_encounter`,
  `EncounterRecord`, `Action::OpenEditorAtIndex`) — these are intentionally-unused
  scaffolding for upcoming phases (encounter file I/O, edit-existing), not bugs
  to delete.
- `state.json` has no version wrapper yet (unlike `EncounterRecord`) — a model
  change can silently break old saves. Phase 0 task.

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
