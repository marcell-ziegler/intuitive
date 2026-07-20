# Intuitive — Implementation Plan & Architecture Notes

A D&D 5e initiative tracker TUI in Rust + ratatui.

This document captures (1) where the codebase is today, (2) the target
architecture, (3) a phased plan to reach the full feature set, and (4) an
in-depth, beginner-oriented guide to the Phase 0 foundation refactor.

> **Stack decision:** the project stays in **Rust + ratatui**. A Python +
> Textual port was evaluated and rejected — it would discard the tested model
> layer and orphan the `dice-parser` crate (owned and already integrated), in
> exchange for faster UI iteration we can approximate with helper crates. Revisit
> only if hand-rolling the Phase 4 statblock/search UI becomes the real
> bottleneck; if so, prototype that one screen in Textual before committing.

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

1. ~~**`App::submit_editor()` is `todo!()`**~~ **Done.** (`src/app.rs:71`) validates
   input, spawns single or numbered-copy creatures (`Amount` field), and closes the
   editor on success — see the Phase 0 progress log below. Remaining gap: it always
   builds a `Monster`, never a `Player` (no type toggle in the editor yet).
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

```txt
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

#### Phase 0 — progress log (updated 2026-07-20 — third pass, branch `event-loop-refactor`)

**Status: done.** The `AppStateRecord` version wrapper — the one item left after
the second pass — is landed. `state.json` now round-trips through a
`schema_version`-checked record exactly like `EncounterRecord`, with tests for
accept/reject-version, missing-file, and store→load round-trip
(`storage::state_tests`). 45 tests green (up from 19 at the start of today).
Every bullet in §3's Phase 0 spec and every step in the §4 guide is closed. Full
history from today's three passes, mapped against the Step 1–7 guide:

- ✅ **`action.rs` + `event.rs` + `App::update(Action) -> Effect`** — the TEA loop
  is fully in place. `map_event(&app, &event)` is mode-aware (Step 2), `update` is
  the single mutation entry point returning `Effect` (Step 3), and `main.rs` is a
  thin loop matching on the effect. `Action` now derives `Debug, PartialEq`
  (`f86f5f7`) so keymap results are assertable. Minor naming drift from the guide
  persists (`SelectNextRow`, `OpenEditorWithNewCreature`) — cosmetic, no action.
- ✅ **Persistence resolved cleanly (Step 5), the earlier smell fixed.** The
  per-keystroke-write regression flagged last pass is gone. `App` now has a
  `state_dirty: bool` (`43d76d5`); cursor/turn actions (`SelectNextRow`,
  `SelectPreviousRow`, `AdvanceTurn`) set it and return `Effect::None` — **no disk
  write per keypress**. `Effect::UpdateState` is reserved for mutations that want
  an immediate flush (`SubmitEditor`). `main.rs` polls with a 3-second timeout
  (`AUTOSAVE_INTERVAL`) and saves when either `state_dirty` has aged past the
  interval or `Effect::UpdateState` fires, plus an unconditional save-on-quit after
  the loop. This is exactly the two-tier "simplest, and fine now" path from Step 5,
  with the optional poll-based tick added.
- ✅ **`draw_ui` takes `&App`; `TableState` is derived at render time** (`a6ecfdd`).
  `main_table_state` is deleted from `App` and `sync_table_state()` is gone;
  `ui.rs` builds a fresh `TableState` each frame from
  `current_encounter.initiative_index`. The compiler now enforces "views don't
  mutate." Steps 4 + 6 done.
- ✅ **Redundant selection state collapsed** (`a6ecfdd`/`2c8c895`). `Encounter` is
  the single source of truth; the render-time `TableState` is a pure derivative.
  Load-time safety added via `Encounter::clamp_indices()`, called from
  `From<SerializableApp>`, so a stale/hand-edited `state.json` can't select past
  the end of the list (covered by tests).
- ✅ **`App::update` + `event::map_event` unit tests exist** (`f86f5f7`, Step 7).
  One test per `Action` variant asserting the returned `Effect` and resulting
  state; a `map_event` suite locking the mode-aware keymap (incl. per-panel
  divergence and non-key events). 41 tests total.
- ✅ **`AppStateRecord` version wrapper** (final pass, `storage.rs`). Mirrors
  `EncounterRecord`: a `schema_version`-checked record wraps `App`, with
  `store_state_to`/`load_state_from` split out from the public XDG-path
  functions so the version-check logic is unit-testable against temp files
  instead of the real `$XDG_STATE_HOME` (same pattern `load_encounter` already
  used). A malformed/future-schema `state.json` now fails loudly on load instead
  of silently deserializing into whatever `App`'s `Deserialize` impl tolerates.

**Resolved this round-trip (previously flagged, now closed):**

- **`submit_editor()` works** — validates, spawns single or numbered-copy
  monsters. Still Monster-only (no Player/Monster toggle); that's a Phase 1 item,
  not Phase 0.
- **`Encounter` moved to `model/encounter.rs`** (`75836a5`) — the storage-vs-model
  location question is settled in favor of the domain layer, matching CLAUDE.md's
  original map and §2's target. `crossterm`/`ratatui` types stay out of it.
- **`draw_ui(frame, &mut app)` → `draw_ui(frame, &app)` in `main.rs`** — the last
  place still passing a mutable reference into the read-only render path (it
  compiled via auto-reborrow, but the intent was stale). Clippy's
  `unnecessary_mut_passed` caught it.

**Exit-criteria check (§3):** identical behavior ✅; tests green ✅ (45); `main.rs`
is ~56 lines, over the "~40" proxy — but the excess is the autosave-timer
calibration from Step 5, not loop bloat; the event-dispatch core is thin. Call the
line target met in spirit. **Phase 0 is closed.**

**Deliberate deviations from the §4 guide, ratified rather than reverted:**

- Persistence is `state_dirty: bool` + a 3s-debounced autosave via
  `crossterm::event::poll(timeout)`, not the guide's simpler "set `dirty`, save
  once per loop iteration + on quit." This was a considered upgrade (Step 5
  explicitly names the poll-based tick as the next tier up), not scope creep.
- `Encounter` lives in `model/encounter.rs`, matching the guide's own target
  layout in §2 — the earlier `storage/encounter.rs` placement was the deviation,
  now corrected.

**Deferred, not blocking Phase 0 (carry into Phase 1):**

- `Action::OpenEditorAtIndex` is constructed only in tests — wire it to a key
  (`e`, for edit-existing) when Phase 1's edit-existing lands, or drop it.
- The editor never chooses Player vs. Monster (`to_creature()` always builds a
  `Monster`) — tracked under Phase 1 below.

### Phase 1 — Make the core loop actually work (highest user value)

- Implement `submit_editor()`: parse fields, choose Player vs Monster (add a
  type toggle + the missing `amount_input` to spawn N copies like "Goblin 1..3"),
  validate numeric input, push creature(s).
- Add edit-existing and delete (`e` / `d`), and a confirm for delete.
- **Real initiative:** an action to roll initiative for all (or per-creature),
  sort the encounter descending by initiative with a dex-mod / manual tie-break,
  and make "next turn" advance through the _sorted_ order with round counting.
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

## 4. Phase 0 in depth — a foundation-refactor guide

Context: this project's author comes from library work (`dice-parser`) and is new
to application/TUI architecture. This section expands Phase 0 into a concrete,
teachable migration. It is deliberately detailed because getting this shape right
is what makes Phases 1–4 additive instead of another `match` arm in `main.rs`.

### The core mental shift: library vs. app

A library exposes an **API surface** and hands control to a caller. In an app,
_you are the caller_, and the hard part is managing **mutable state over time** as
events stream in. Phase 0 imposes one disciplined shape on that: **The Elm
Architecture (TEA)** — a one-directional loop.

```txt
          ┌───────────────────────────────────────┐
          │                                       │
    event.read() ──▶ map to Action ──▶ App::update(Action) ──▶ mutate state
          ▲                                                          │
          │                                                          ▼
          └──────────────── draw_ui(&App)  ◀───────────────── (next frame)
```

Three rules make it work:

1. **State lives in exactly one place** (`App`).
2. **Nothing mutates state except `update`.** Rendering only reads.
3. **Events are translated into _intent_ (an `Action`) before touching state.**
   Keys are an input detail; `AdvanceTurn` is the intent.

### Step 1 — Model intent with an `Action` enum

Actions are **semantic** — what the user wants, independent of which key fired.
This is what collapses the duplicated `j`/`Down` arms (`main.rs:49-56`) into one
path and lets keys be rebound later without touching `update`.

```rust
// action.rs
pub enum Action {
    // navigation / turn tracking
    SelectNext,
    SelectPrevious,
    AdvanceTurn,
    SwitchPanel,

    // editor lifecycle
    OpenEditor,
    CloseEditor,
    EditorNextField,
    EditorPrevField,
    SubmitEditor,
    EditorInput(crossterm::event::Event), // raw event — see note

    Quit,
}
```

**Text input note:** don't abstract character-by-character editing into semantic
actions — `tui-input` already handles cursor/backspace. Carry the raw `Event` in
one `EditorInput` variant and delegate to `handle_event` inside `update`, exactly
like `handle_editor_input_event_delegation` does today (`main.rs:93`). Pragmatism
over purity here is correct.

### Step 2 — Translate events → actions (the keymap layer)

This pure, _mode-aware_ function replaces every scattered `match key_event.code`
block. The same key means different things per panel — that mode-dependence is
why it stays separate from `update`.

```rust
// event.rs
pub fn map_event(app: &App, event: &Event) -> Option<Action> {
    let key = event.as_key_event()?;
    match app.current_panel {
        Panel::Editor => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(Action::CloseEditor),
            KeyCode::Tab | KeyCode::Down       => Some(Action::EditorNextField),
            KeyCode::BackTab | KeyCode::Up      => Some(Action::EditorPrevField),
            KeyCode::Enter                      => Some(Action::SubmitEditor),
            _ => Some(Action::EditorInput(event.clone())),
        },
        Panel::InitiativeTable | Panel::Sidebar => match key.code {
            KeyCode::Char('q')                  => Some(Action::Quit),
            KeyCode::Char('j') | KeyCode::Down  => Some(Action::SelectNext),
            KeyCode::Char('k') | KeyCode::Up    => Some(Action::SelectPrevious),
            KeyCode::Char(' ')                  => Some(Action::AdvanceTurn),
            KeyCode::Tab                        => Some(Action::SwitchPanel),
            KeyCode::Char('n')                  => Some(Action::OpenEditor),
            _ => None, // key means nothing in this mode → ignore
        },
    }
}
```

`Option<Action>` cleanly encodes "this key does nothing here" — no more
`_ => return Ok(false)` noise.

### Step 3 — The `update` function and the `Effect` pattern

`update` is the _only_ place mutation is allowed. It takes an `Action`, changes
state, and returns an **`Effect`** — a description of a side effect the loop must
perform. This keeps I/O out of state logic: `update` doesn't quit or write files,
it _asks_ the loop to. Start minimal; resist a big effect system on day one.

```rust
pub enum Effect { None, Quit }

impl App {
    pub fn update(&mut self, action: Action) -> Effect {
        match action {
            Action::Quit => return Effect::Quit,

            Action::SelectNext     => { self.select_next_row();           self.dirty = true; }
            Action::SelectPrevious => { self.select_previous_row();       self.dirty = true; }
            Action::AdvanceTurn    => { self.increment_initiative_order(); self.dirty = true; }
            Action::SwitchPanel    => self.toggle_panel(),
            Action::OpenEditor     => self.current_panel = Panel::Editor,
            Action::CloseEditor    => { self.reset_editor(); self.current_panel = Panel::InitiativeTable; }
            Action::EditorNextField => self.editor_state.next_field(),
            Action::EditorPrevField => self.editor_state.previous_field(),
            Action::SubmitEditor    => { self.submit_editor(); self.dirty = true; }
            Action::EditorInput(e)  => self.editor_state.handle_event(&e),
        }
        Effect::None
    }
}
```

The loop then shrinks to:

```rust
// main.rs — the whole loop
loop {
    term.draw(|f| draw_ui(f, &app))?;   // note: &app, not &mut
    let event = event::read()?;
    if let Some(action) = map_event(&app, &event) {
        if let Effect::Quit = app.update(action) {
            break;
        }
    }
}
app.save_if_dirty()?;
```

Why an `Effect` enum rather than a `bool`? Today a bool would do. You keep the
enum because Phase 1+ adds `Effect::OpenBrowser(url)` (the 5e.tools link) and
similar — things `update` shouldn't _do_ but must _request_. Then it's an added
variant, not a signature change across the codebase. That is the whole
"make later features additive" goal.

### Step 4 — Rendering reads, never mutates

Concrete cleanup: `render_initiative_table` currently takes `&mut App` and calls
`app.sync_table_state()` mid-render (`ui.rs:142,147`), and `draw_ui` takes
`&mut App`. Mutation hidden inside the view is a classic "why did state change
when I only redrew?" bug source. Pair with Step 6 and **derive** the `TableState`
at render time so `draw_ui` can take `&App`:

```rust
fn render_initiative_table(frame: &mut Frame, app: &App, area: Rect) {
    let mut table_state = TableState::default();
    if !app.current_encounter.creatures.is_empty() {
        table_state.select(Some(app.current_encounter.cursor_index));
    }
    // ... build rows ...
    frame.render_stateful_widget(tab, area, &mut table_state);
}
```

Once `draw_ui(&App)`, accidental mutation in a view won't compile — the compiler
enforces the discipline, the same guarantee `&self` methods gave in the library.

### Step 5 — Persistence: dirty flag, calibrated

The plan's "per-keystroke save is a smell" is right in principle, but calibrate to
scale — knowing _when not to optimize_ is an app skill. Add `dirty: bool` to `App`
(`#[serde(skip)]`), set it in `update` on mutating actions, and:

```rust
impl App {
    fn save_if_dirty(&mut self) -> color_eyre::Result<()> {
        if self.dirty {
            storage::store_state(self)?;
            self.dirty = false;
        }
        Ok(())
    }
}
```

Two tiers for _when_ to call it:

- **Simplest, and fine now:** call `save_if_dirty()` once per loop iteration
  (after `update`) _and_ on quit. The state file is a few KB; a write is
  sub-millisecond. The real smell was coupling saves to navigation handlers —
  writing "when something changed" from one place fixes that. Don't build more.
- **Batching (not needed until the statblock DB):** switch to
  `event::poll(timeout)? + event::read()` so the loop can wake on a timer with no
  input, then save at most every N seconds. This introduces a **tick** (also
  useful later for timers/animation) but is scope creep for Phase 0 — note and
  move on.

### Step 6 — Collapse the redundant selection state

"Position" is currently tracked in three hand-synced places:
`Encounter::cursor_index`, `Encounter::initiative_index`, and
`App::main_table_state` — which is why `sync_table_state()` (`app.rs:107`) is
sprinkled across 4+ call sites. Make `Encounter` the **single source of truth**,
delete `main_table_state` from `App`, and derive `TableState` in the renderer
(Step 4). `sync_table_state` and the `From<SerializableApp>` juggling around it
(`app.rs:88-102`) then largely disappear. Store facts once, compute the rest.

While here, give `state.json` the same versioned wrapper `EncounterRecord` already
has (`storage.rs:12`): an `AppStateRecord { schema_version, state }` costs ~10
lines now and prevents a "why won't my old save load" headache after the first
model-field change.

### Step 7 — Test `update` (this is where library instincts pay off)

Because `update` is `(&mut App, Action) -> Effect` with **no terminal and no
I/O**, the entire interaction layer is unit-testable the same way `DiceExpr` was:

```rust
#[test]
fn advancing_turn_moves_and_marks_dirty() {
    let mut app = App::default();
    app.add_creature(Creature::new_player("Alice", 10, 10, None, None, None));
    app.add_creature(Creature::new_player("Bob", 10, 10, None, None, None));

    assert_eq!(app.update(Action::AdvanceTurn), Effect::None);
    assert_eq!(app.current_encounter.initiative_index, 1);
    assert!(app.dirty);
}
```

Key handling tangled in `main.rs` can't be tested (it needs real `event::read`);
after Phase 0 every behavior is a pure call. Aim for a test per `Action` — that
suite is what lets you refactor fearlessly through Phases 1–4.

### Migration strategy: green at every commit

Do **not** big-bang this. Sequence so each commit compiles and all tests pass:

1. Add `action.rs` + `event.rs` + `update`, but have `update` call the _existing_
   methods. Route `main.rs` through it. Behavior is byte-for-byte identical — only
   the plumbing changed. Commit.
2. Introduce the `dirty` flag; remove inline `store_state` calls from old
   handlers. Commit.
3. Make `draw_ui` take `&App`; derive `TableState`; delete `main_table_state`.
   Commit.
4. Add the `AppStateRecord` version wrapper. Commit.

### Pitfalls to watch

- **`event::read()` blocks** — the loop only wakes on input (good: no CPU spin)
  until you want timers; that's what motivates the `poll` tick.
- **Don't leak `crossterm`/`ratatui` types into the model.** `EditorInput(Event)`
  is the one deliberate exception, confined to the editor.
- **Resist a premature effect system.** `Effect { None, Quit }` is enough; add
  variants when a feature needs one.
- **Don't mutate in `draw`** — the `&App` signature enforces it.
- **`#[serde(skip)]` transient fields** (`dirty`, later any pure-UI scratch state)
  so they never pollute the save file or schema version.

---

## 5. Suggested immediate next step

Do **Phase 0 + the `submit_editor()` half of Phase 1** together: it's the
smallest change that turns the app from "renders" into "usable," and doing the
`Action`/`update` refactor first means every feature after it is additive rather
than another `match` arm in `main.rs`.
