# Product Requirements Document: Rust PathFinder

## 1. Product overview

**Product:** Rust PathFinder  
**Type:** Cross-platform desktop simulator for grid-based autonomous exploration and cell-action coverage  
**Target platforms:** Windows and Linux  
**Implementation:** Rust, with shared simulation and navigation logic and native platform UI

Rust PathFinder displays an editable, bounded grid and one simulated vehicle. Users can drive it manually or run an autonomous Navigator. The Navigator operates with limited forward observation, incrementally builds its own map without knowing the environment's dimensions, prefers systematic alternating straight passes, explores unobserved areas, and records actual travel and the completion state of an action for each reachable free cell.

This document is the **complete product specification** for the target application, not a record of incremental modifications. Requirements apply to the finished version even if some capabilities are not yet implemented. Version history belongs exclusively in `changelog.md`.

## 2. Goals and scope

### 2.1 Goals

- Provide an editable grid-based simulated environment of 2–100 rows and columns, default 20 × 20.
- Display one vehicle, its cell position, and its cardinal heading.
- Preserve manual movement, map editing, JSON `.map` save/load, recent files, and native Windows/Linux interfaces.
- Introduce an autonomous `navigator` that observes only up to three consecutive cells directly ahead and cannot inspect the entire simulator map.
- Let a user classify newly observed unknown cells as `Free`, `Obstacle`, or `Boundary` through a UI dialog.
- Discover both terrain contents and navigable boundaries without exposing grid dimensions to the Navigator.
- Execute at most one cell movement per decision cycle, with validated movement feedback.
- Prefer orderly alternating parallel passes while allowing detours, exploration, and backtracking.
- Track all actual cell entries in order, including repeated visits; independently track a per-cell `action_completed` boolean.
- Offer continuous execution, individual Step execution, pause, session reset, and map reset.
- Finish only when the accessible area has been sufficiently explored and every reachable free cell has `action_completed = true`.

### 2.2 Not in scope

Real cameras, sensor drivers, physical robot control, GPS, odometry, motion physics, batteries and charging, multiple vehicles, persistent Navigator knowledge between launches, complete up-front route optimization, route-to-destination UI, external map imports, and advanced map editing (undo and brush tools).

## 3. Terminology and data ownership

| Term | Definition |
|---|---|
| **Simulator map** | Authoritative finite grid used for rendering, terrain editing, movement collision checks, and `.map` files. The Navigator cannot query it wholesale. |
| **Discovered map** | Navigator-owned, expandable knowledge indexed by signed relative cell coordinates. Initially only the starting cell is known. |
| **Terrain state** | `Unknown`, `Free`, `Obstacle`, or `Boundary`. Only `Free` permits autonomous entry. |
| **Observation** | Up to three forward cells, one cell wide, stopping at an obstacle or boundary. |
| **Travel history** | Chronological list of cells actually occupied following successful translations; repeated entries are retained. |
| **Action state** | `action_completed: bool` per discovered free cell: `true` means the cell's intended action was performed; `false` means it was not. |
| **Action enabled** | Session-level `action_enabled: bool` controlling whether entering a free cell completes its action. Default `true` in the initial version. |
| **Coverage** | Completion of the intended action on all reachable free cells, not simply observing or visiting them. |
| **Exploration frontier** | A reachable known-free cell and heading from which unknown terrain may still be observed. |
| **Session** | A single run's execution state, ordered travel history, and per-cell action completion status. |

### 3.1 Coordinates

- Simulator map coordinates use zero-based `(row, column)` with `(0, 0)` at the upper left; row increases down, column increases right.
- The UI displays rows and columns starting at 1 for the finite simulator grid.
- The Navigator uses signed relative `(row, column)` coordinates with its initial position `(0, 0)`, independent of the simulator's absolute start. `Up = (-1, 0)`, `Right = (0, 1)`, `Down = (1, 0)`, and `Left = (0, -1)`.
- The integration layer translates between relative and absolute coordinates; the Navigator must not receive finite grid dimensions or undiscovered cells.

### 3.2 Terrain and action model

| Terrain | Autonomous traversal | Action state |
|---|---|---|
| Unknown | Forbidden until inspected | Not applicable / not completed |
| Free | Permitted | `action_completed` initially `false` |
| Obstacle | Forbidden | Not applicable |
| Boundary | Forbidden | Not applicable |

`Boundary` is a separately remembered non-traversable observation. The outer simulator edge acts as a boundary even though it lies outside its last valid cell; in-grid boundary cells may also be configured. The Navigator does not infer an entire boundary line from a single observation.

## 4. User experience

### 4.1 Startup and visual layout

- Open a native desktop window titled `<rows> x <columns> Grid` (initially `20 x 20 Grid`).
- Show a centered, proportionally scaled grid, exactly one vehicle, a terrain/action legend, a status area, and Navigator controls.
- Begin with a deterministic sample terrain grid, a vehicle in the bottom-center cell facing Up, manual controls available, and the Navigator in `Idle` state.
- Render the vehicle as a red body with a black triangular heading marker. Display Free green, Obstacle brown, Unknown gray, and Boundary in a distinct fourth color. Visually distinguish cells where `action_completed` is true and overlay the actual traveled path without obscuring the vehicle.
- Make available a view of the Navigator's **discovered map**, distinct from the full simulator/environment view. Discovery visualization must not accidentally reveal hidden simulator cells to the Navigator logic.

### 4.2 Manual vehicle controls

| Input | Result |
|---|---|
| Up | Move forward one cell along heading |
| Down | Move backward one cell |
| Left | Turn 90° counterclockwise in place |
| Right | Turn 90° clockwise in place |
| Escape while grid has focus | Close application, except while a modal dialog owns focus |

- Manual translation is limited to the simulator grid. `Obstacle` and `Boundary` block movement; an out-of-grid move is blocked by the edge.
- Manual movement may enter a simulator `Unknown` cell, preserving the simulator's manual-driving semantics. **Autonomous** movement may not enter unknown cells.
- Turns never change the occupied cell. Failed translations never change vehicle position.
- The UI reports blocked movements and maintains row, column, and heading.
- Manual driving is disabled while automatic navigation is Running or Inspecting. Manual movement outside an autonomous session does not silently modify that session's travel history or action results; starting or resuming a session must use synchronized position/heading (see FR-16).

### 4.3 File, Map, and About menus

- **File:** New, Save, Load, up to nine recent map files, Clear Recent Files, Exit.
- **Map:** Edit, Grid Size… .
- **About:** Native informational dialog.
- A **Navigator** control area provides Start, Pause, Step, Reset Session, Reset Map, execution status, the inspection dialog, and progress feedback. The exact platform-native widget arrangement may differ between Windows and Linux.

### 4.4 Editing the simulator map

- Map → Edit toggles editing; it shows a check mark when active, draws grid lines orange, and reports `Editing` in the status area.
- Left-click paints `Obstacle`, right-click paints `Unknown`; dragging with the corresponding mouse button applies the value across cells. A separate selection or context control must also permit setting `Free` and `Boundary` so all four cell states can be authored without interfering with existing left/right shortcuts.
- Editing does not move or rotate the vehicle; arrow-key movement is disabled in edit mode.
- Editing is allowed only when the Navigator is Idle or Paused. Any edit that changes terrain relevant to a learned map invalidates conflicting Navigator knowledge; require Reset Map before autonomous execution resumes. The UI must clearly prompt this reset rather than silently using stale knowledge.
- The simulator's occupied start/current cell must remain traversable: if a map edit would make it non-free, reject the edit or normalize it to Free with clear feedback.
- Outside edit mode, clicks do not modify terrain. Edits remain in memory until explicitly saved.

### 4.5 Grid size and vehicle start

- Map → Grid Size… presents rows and columns (each 2–100), 1-based start row/column, and Up/Right/Down/Left start heading. Populate with current values.
- OK/Enter validates and applies; Cancel/Escape/window close leaves everything unchanged. Invalid numbers, bounds, or start coordinates keep the dialog open with an explanatory message.
- Resize from the top-left: retain overlapping terrain, set newly created cells to Unknown, drop cells beyond the new edge, move vehicle to configured start/heading, and normalize the start cell to Free if necessary.
- Change the window title and all related drawing, movement, and file behavior to the new dimensions.
- Changing grid dimensions or start settings resets Navigator map and session state, because the simulator/relative-coordinate relationship has changed.

### 4.6 Map file commands

- **New:** Make all cells Unknown, preserving the current dimensions and vehicle's current position/heading; ensure the occupied cell remains Free. Reset the Navigator map/session because the environment has changed.
- **Save:** Native save dialog defaulting to `untitled.map`, confirm overwrite, write terrain, size, and configured vehicle start (not live position) as JSON.
- **Load:** Native open dialog; validate complete file before replacing anything, restore terrain/dimensions/configured start, position the vehicle at start, and reset Navigator map/session.
- Cancellation and failed read/write/validation leave application data unchanged and display an explanatory file-specific error for failures.
- Files use `.map`. A successful load/save updates recent files.

### 4.7 Observation and inspection dialog

- Observation is restricted to the three directly forward cells. The Navigator cannot ask for diagonal, side, rear, or global map scans.
- Process visible cells nearest-first; stop at the first cell discovered or classified as Obstacle or Boundary.
- A previously known Free cell requires no dialog; a previously known Obstacle/Boundary stops the scan.
- For each visible Unknown cell, issue an inspection request identifying its relative cell coordinate. The UI displays `Free`, `Obstacle`, or `Boundary` choices, and the Navigator enters `Inspecting` without moving.
- In the prototype, a selected classification is the observation result and must be applied consistently to the discovered map and the simulated environment when inspecting an unknown in-grid simulator cell. An out-of-grid observation is reported as Boundary automatically and does not create an out-of-grid simulator cell.
- A classification that conflicts with an already defined non-unknown simulator obstruction must not override simulator collision safety. If discovered information proves incorrect, invalidate that cell's free classification and request reconciliation before another attempt.
- The inspection dialog is modal for the observation; canceling or dismissing it keeps the vehicle stationary and pauses execution with the request unresolved. Resume reopens/reissues that request.
- The observation interface must be replaceable by a future sensor implementation without importing UI code into the Navigator.

### 4.8 Autonomous operation and Step mode

- **Start:** Begin/resume continuous navigation from a synchronized state.
- **Pause:** Stop scheduling new navigation actions; preserve discovered map, history, and coverage.
- **Step:** While not Running, advance exactly one atomic Navigator event (one inspection request, one classification processing event, one turn, or one-cell translation), so actions are debuggable; if waiting for inspection, input resolves that event before further movement.
- **Reset Session:** Stop; reposition vehicle to configured start and heading; clear travel history and all `action_completed` values; retain discovered terrain; initialize the starting cell in travel history. Navigator becomes Idle.
- **Reset Map:** Perform Reset Session, then erase learned terrain and reinitialize only the starting cell as known Free. Does not alter underlying simulator terrain.
- Continuous mode uses the identical decision logic as Step mode, automatically scheduling successive events while Running. UI updates and inspection dialogs must remain responsive.
- Display `Idle`, `Running`, `Paused`, `Inspecting`, `Completed`, or `Blocked` states. `Blocked` indicates the Navigator cannot resolve a safe next action, not a successful end of the mission.

### 4.9 Travel history and action completion

- Store travel as an ordered array of relative `(row, column)` positions, beginning with the session's starting position; append once after each **successful translation**, even when revisiting a cell. Turning and failed moves do not append.
- Maintain `action_completed: bool` independently for each known Free cell. It defaults to `false` when a Free cell is discovered, unless the current action is performed there.
- The session has `action_enabled: bool` (default `true`). On successfully entering a Free cell with `action_enabled = true`, set its `action_completed = true`; otherwise leave it unchanged. The starting cell is evaluated at session initialization: when action is enabled, its action is performed and `action_completed` becomes true without a movement-history duplicate.
- Repeat visits do not set `action_completed` back to false. Discovering a cell is not the same as performing its action.
- Expose a read-only snapshot of ordered travel and cell action flags to both desktop UIs. Progress may report completed known-Free cells versus total known-Free cells, but that ratio is provisional while exploration remains.

### 4.10 Execution states and completion

| State | Meaning |
|---|---|
| Idle | Navigator initialized but not executing |
| Running | Automatically processing observation/decision/action events |
| Paused | Session can resume or be stepped |
| Inspecting | Waiting for a cell classification |
| Completed | All reachable Free cells have completed actions and no reachable observation frontier remains |
| Blocked | Unable to continue safely or determine a valid continuation; intervention needed |

Completion requires **both**: (1) all discovered reachable Free cells have `action_completed = true`, and (2) all reachable observation frontiers are exhausted so no additional reachable unknown cells can be revealed by looking from any reachable known-Free cell in a cardinal heading. Non-reachable unknown areas do not prevent completion. The Navigator must not infer completion simply because its preferred straight pass ends or because the last three forward cells are blocked.

## 5. Functional requirements

### FR-1: Grid display
1. Render 2–100 rows by 2–100 columns, default 20 × 20, with borders and scalable cells.
2. Generate a deterministic sample simulator map at startup.
3. Show all four terrain states with legend-matched colors.
4. Overlay vehicle, action-completion state, and travel history; permit separate full-environment and discovered-map views.

### FR-2: Vehicle display
1. Show exactly one vehicle in one cell, red with a heading indicator.
2. Default start to bottom-center facing Up; respect configured or loaded start.
3. Display current row, column, and heading and update after actions.

### FR-3: Simulator driving
1. Support `Forward`, `Reverse`, `TurnLeft`, and `TurnRight` with one-cell translations and 90° turns.
2. Reject translation into Obstacle, Boundary, or beyond grid edges; leave vehicle unchanged and return a specific result.
3. Preserve manual entry into Unknown; forbid autonomous entry into Unknown in the Navigator.
4. Turns shall not change cell position; simulator feedback shall distinguish moved, turned, collision, and outer-edge conditions.

### FR-4: Lifecycle
1. Start in a native desktop window on Windows and Linux.
2. Support File → Exit and Escape when the grid is focused and no modal owns focus.
3. Write initialization errors to standard error and exit nonzero on startup failure.

### FR-5: Menus and controls
1. Provide File, Map, and About menus and Navigator controls described in section 4.
2. Implement native dialogs for About, map file operations, grid settings, and inspection.
3. Prevent conflicting manual/edit/autonomous interactions.

### FR-6: Status and feedback
1. Show current row, column, heading, editing status, and blocked manual moves.
2. Show Navigator execution state, current action/inspection request, and provisional coverage progress.
3. Reflect each confirmed movement, turn, inspection response, and state transition promptly.

### FR-7: Map files and validation
1. Save and load `.map` JSON using `schemas/map.schema.json` (JSON Schema draft 2020-12).
2. Store size, terrain, and configured start, not live vehicle position, travel history, or Navigator knowledge.
3. Write **formatVersion 3** with terrain values `free`, `obstacle`, `unknown`, and `boundary`.
4. Continue accepting legacy versions 1 (without start) and 2 (three terrain states), with documented defaults.
5. Reject invalid JSON, unknown properties/states, unsupported versions, invalid sizes, invalid cell arrays, or out-of-range starts, without altering the active map.
6. Include `$schema` in saved files and verify codec/schema compatibility through automated tests.

### FR-8: Map editing
1. Toggle checked edit mode, orange grid lines, and Editing status.
2. Preserve left-click Obstacle and right-click Unknown, with dragging supported.
3. Provide an additional clear control for Free and Boundary editing.
4. Disable motion during editing and invalidate/reinitialize stale Navigator knowledge before resumption.
5. Keep occupied cell traversable.

### FR-9: Legend and visualization
1. Show all four terrain colors and vehicle symbol.
2. Show distinct styles for `action_completed = true`, the traveled route, and the Navigator's discovered/undiscovered terrain.
3. Keep visualization consistent across Windows and Linux.

### FR-10: Grid configuration
1. Allow rows/columns 2–100 and start row/column/heading through a validated modal dialog.
2. Resize retaining top-left overlap, adding Unknown cells, dropping out-of-bounds cells.
3. Reposition vehicle to valid Free start, update title and file metadata, and reset Navigator knowledge/session.

### FR-11: Recent files
1. Maintain up to nine recent files, newest first, deduplicated and numbered 1–9 in File menu.
2. Handle missing files by showing an error and removing the missing entry.
3. Provide Clear Recent Files when nonempty.
4. Persist list in `%APPDATA%/rust_pathfinder` on Windows or `$XDG_CONFIG_HOME/rust_pathfinder` (default `~/.config/rust_pathfinder`) on Linux.

### FR-12: Restricted forward observation
1. Observe up to three consecutive cells directly ahead, one-cell width, with no sideways/rear sensing.
2. Stop observation at the first Obstacle or Boundary.
3. Process unknown cells nearest-first and request classification before traversal.
4. Never expose whole simulator map or its dimensions to Navigator decision-making.

### FR-13: Discovered map
1. Store learned terrain in a dynamically expandable, sparse map of signed relative coordinates.
2. Initially know only the starting cell as Free and starting heading.
3. Track Free, Obstacle, Boundary, and Unknown; allow repeated observations to reuse known classifications.
4. Preserve learned map through Reset Session and clear on Reset Map.
5. Keep learned knowledge in memory only for this version.

### FR-14: Observation/classification integration
1. Represent unknown observation as a request/response interaction, independent of UI implementation.
2. Display Free/Obstacle/Boundary choices while vehicle remains stationary.
3. Apply observation results to learned map and consistent simulated terrain when applicable.
4. Report the simulator's outer edge as Boundary without revealing overall dimensions.
5. Never bypass simulator collision constraints on contradictory responses.

### FR-15: Incremental navigation strategy
1. Prefer straight movement in the current pass direction and adjacent alternating parallel passes.
2. Make local decisions using discovered terrain, vehicle heading, previous travel, and action status; do not precompute the entire environment route.
3. On a blocked preferred pass, select a safe turn/detour or switch to exploration.
4. Permit revisiting known Free cells to reach unfinished areas.
5. Avoid indefinite nonproductive loops through progress tracking or a detectable Blocked state.

### FR-16: Safe atomic execution
1. Execute at most one successful cell translation per movement event.
2. Observe/inspect necessary unknown cells before entering them.
3. Send actions through the simulator's standard command interface; consume the returned outcome.
4. Append travel/complete cell action only after a successful translation.
5. On failed movement, do not record travel; invalidate inconsistent learned terrain and reevaluate.
6. Keep Simulator and Navigator positions/headings synchronized; initiating autonomous navigation after manual repositioning shall establish a fresh session anchored at the current pose or require Reset Session, never silently reuse an inconsistent pose.

### FR-17: Exploration/backtracking
1. Actively search for reachable frontiers and reachable Free cells with `action_completed = false`.
2. Travel through previously visited Free cells to reach those targets.
3. Resume a systematic coverage pass whenever practical.
4. Distinguish unreachable areas and exhaustion of observation frontiers from temporary dead ends.
5. A discovered graph of traversable cells may be used for local navigation to a selected target; a precomputed global coverage tour is not required.

### FR-18: Travel and action tracking
1. Maintain a chronological vector of visited relative cell coordinates, including duplicates.
2. Initialize the vector with the starting cell and append only on successful translations.
3. Maintain independent per-Free-cell `action_completed: bool`, initially false.
4. When `action_enabled` is true, mark the entered Free cell complete; do not clear completion on revisits.
5. Evaluate the starting cell's action on session initialization when action is enabled.
6. Provide read-only history and action-state snapshots for rendering and tests.

### FR-19: Execution modes and reset
1. Implement Idle, Running, Paused, Inspecting, Completed, and Blocked states.
2. Provide Start/Pause and one-event Step; both shall use the same decision logic.
3. Pause for unresolved inspections without moving the vehicle.
4. Reset Session clears travel/action status and returns to configured start while keeping learned terrain.
5. Reset Map additionally clears discovered knowledge without modifying simulator terrain.
6. Respond safely to map replacement, edit, resize, and loading by resetting or requiring reset before navigation resumes.

### FR-20: Completion determination
1. Complete only when every discovered reachable Free cell has `action_completed = true` and no reachable observation frontiers remain.
2. Stop issuing actions upon completion and preserve the result until reset.
3. Report Blocked rather than Completed when further safe exploration/coverage cannot be established.
4. Account for boundaries, obstacles, disconnected regions, and repeated travel.

## 6. Navigation decision model

The Navigator runs an event-driven observe–classify–decide–execute–record loop. An event is either an observation/inspection request, a processed classification, a 90° turn, or one-cell translation. Its internal logical modes are **Coverage** (prefer straight, alternating passes) and **Exploration** (seek unfinished Free cells and observation frontiers). These logical modes are separate from UI execution states.

1. Read the current known position/heading and local discovered map.
2. Inspect forward cells nearest-first, up to distance three; if an unknown requires user input, suspend in `Inspecting`.
3. Prefer safe continuation of a systematic pass when there is useful action work or exploration in that direction.
4. If blocked or the pass is exhausted, attempt a safe adjacent parallel pass.
5. Otherwise select an outstanding unfinished reachable Free cell or observation frontier, navigate through known Free cells toward it (revisiting allowed), and resume coverage.
6. Execute no more than one translation, or a single turn, and verify feedback before updating state.
7. Continue until the completion criterion holds or safe progress cannot be made.

An implementation may use graph search over **known Free cells** to reach a local target. It must not plan a full visit-all-cells tour using hidden global terrain. Deterministic tie-breaking must make tests reproducible. A visited cell with `action_completed = false` remains a coverage target; a Free cell whose action is complete may still be a transit cell.

### 6.1 Event/result contract

The Navigator interface should express, without depending on GTK/Win32, results equivalent to:
- `RequestInspection(relative_cell)`;
- `Drive(Forward | Reverse | TurnLeft | TurnRight)`;
- `WaitForInspection`;
- `StateChanged(state)`;
- `MissionCompleted` or `UnableToProgress`.

The integration layer provides classified observations and simulator drive outcomes. Exact Rust types and filenames are implementation choices; these behaviors and invariants are mandatory.

### 6.2 Observation limitations and boundary cases

- A three-cell range is a **maximum**, not a command to translate three cells.
- A cell obscured by a known obstacle/boundary cannot be inspected from that heading.
- An outer-edge detection returns Boundary for the first inaccessible coordinate; farther out-of-grid positions are not exposed.
- Unknown coordinates are not presumed safe. A known Free corridor may be used for a detour, but unknown terrain must first be classified.
- A successful movement into a classified Free cell is still subject to Simulator collision verification.

## 7. Non-functional requirements

**NFR-1 Cross-platform:** Identical core behavior and controls on Windows (Win32/GDI) and Linux (GTK 4/Cairo); platform-specific UI code remains isolated.

**NFR-2 Determinism:** Sample map generation and Navigator action choices (for a fixed observation/classification sequence) must be reproducible.

**NFR-3 Responsiveness:** Remain interactive with 100 × 100 simulator maps; continuous navigation must not block repainting, dialogs, or Pause.

**NFR-4 Separation:** Simulator owns environment and final movement outcome; Navigator owns learned map, exploration, history, action flags; UI owns presentation and inspection input. No circular module dependencies.

**NFR-5 Data integrity:** Reject invalid map files without losing current state; never log a failed move as successful; reset learned state when backing environment changes.

**NFR-6 Testability:** Core navigation/observation/coverage logic must be testable headlessly with scripted classifications and movement outcomes.

**NFR-7 Schema compatibility:** Maintain a versioned standalone JSON Schema; write v3 and load valid v1/v2 files through explicit compatibility rules.

**NFR-8 Prototype scope:** Learned map, travel history, and action-completion flags remain in memory; no implicit persistence on application exit.

## 8. Acceptance tests

### 8.1 Existing desktop and simulator behavior

1. Startup shows a deterministic 20 × 20 map, a single vehicle at bottom-center facing Up, a legend, and a native window titled `20 x 20 Grid`.
2. Arrow keys move one cell or turn 90°; obstacle, boundary, and edge collisions do not move the vehicle; status updates correctly.
3. Manual Unknown traversal remains possible; editing disables manual controls.
4. Editing obstacle/unknown by mouse drag works; Free/Boundary can also be configured explicitly.
5. Grid Size validates inputs, resizes from top-left, updates start, and resets Navigator learned state.
6. New/Save/Load and failure/cancel behavior work as specified.
7. Saved v3 files validate against the updated JSON schema; legacy v1/v2 files load correctly.
8. Nine-entry recent files list persists across launches and supports clearing and missing-file handling.
9. Windows and Linux builds expose equivalent behavior; `cargo build` and `cargo test` succeed.

### 8.2 Observation and discovery

10. Starting a new Navigator map reveals only `(0,0)` and no map dimensions or hidden terrain.
11. From a fixed heading, observation reaches at most three cells in one straight column and stops at first obstacle/boundary.
12. Classifying an Unknown pauses motion, presents three choices, and updates the discovered map when answered.
13. Canceling inspection leaves it unresolved and the vehicle stationary.
14. Encountering an actual simulator edge records a Boundary observation without granting the Navigator grid dimensions.
15. Discovered terrain is retained on Reset Session, erased on Reset Map, and never written to `.map` files.

### 8.3 Movement, history and actions

16. Each accepted translation advances exactly one cell; each turn stays in place.
17. The ordered travel history begins with the start cell, contains repeated entries for revisits, and excludes turns/failed translations.
18. On successful entry to a Free cell with `action_enabled = true`, `action_completed` becomes true. With action disabled, it stays false.
19. A known Free cell can be visited with action still false; observations alone never mark action true.
20. All autonomous attempted entries into Unknown, Obstacle, and Boundary are prohibited or blocked, and collision failures never produce false history.

### 8.4 Navigation and session controls

21. On an unobstructed reachable test area, the Navigator prefers alternating straight passes and ultimately completes actions on all Free cells.
22. On a map with obstacles and a dead end, it detours or backtracks via known Free cells to explore and complete reachable regions.
23. Both continuous Start/Pause and manual Step yield equivalent decisions given the same inputs; Step advances only one atomic event.
24. A missing observation response pauses execution rather than guessing.
25. Reset Session clears history and action flags but keeps terrain knowledge and returns the vehicle to configured start; Reset Map additionally erases knowledge.
26. The Navigator reports Completed only when all reachable known Free cells have completed actions **and** all reachable observation frontiers are exhausted.
27. An unreachable region does not prevent completion, but a temporarily blocked unexplored frontier must not be mistaken for successful completion.
28. Detectable nonproductive cycles or inconsistent movement state produce a Blocked/error condition rather than unlimited movement.
29. The UI shows execution state, progress, actual path, discovered knowledge, and boolean action completion without leaking hidden simulator information to Navigator logic.

## 9. Technical architecture

### 9.1 Project structure

| Path | Responsibility |
|---|---|
| `src/simulator/` | Grid and terrain (`grid.rs`), vehicle (`car.rs`), simulation/drive results (`simulation.rs`), `.map` codec (`map_file.rs`) |
| `src/navigator/` | Observation policy, expandable discovered map, navigation controller, local coverage/exploration strategy, travel history, action state |
| `src/ui/` | Shared app state/presentation (`app.rs`), pixel-buffer rendering (`render.rs`), recent file list (`recent.rs`), GTK/Win32 windows and dialogs |
| `src/main.rs` | Application startup and platform UI entry |
| `schemas/map.schema.json` | Source of truth for `.map` JSON schema including v3 `boundary` terrain support |

The simulator remains UI-free. The Navigator is UI-free and uses an abstract observation/movement contract. The UI/application coordination layer invokes Navigator decisions, shows inspection requests, submits classifications, drives the simulator, and returns feedback. Core state changes must not depend on platform widgets.

### 9.2 Existing technical implementation context

- Linux UI: GTK 4 menus, keyboard/mouse input, modal dialogs, native file chooser, legend, and Cairo rendering.
- Windows UI: Win32 menus, keyboard/mouse input, modal dialogs, common file dialogs, legend/status controls, and GDI rendering.
- Shared UI: pixel-buffer rendering, edit mode, screen-to-grid mapping, Grid Size input validation, and recent files.
- Simulator: standard `Simulation::drive` commands and detailed outcomes. Extend terrain serialization/collision results for Boundary as necessary.
- Map storage: `serde` / `serde_json`, `schemas/map.schema.json`; maintain backward-readable map files.
- Repository uses Rust edition 2024; build and test with `cargo build` and `cargo test`.

### 9.3 Map file contract (format version 3)

Map file properties:

- `$schema`: link to `schemas/map.schema.json`.
- `formatVersion`: `3` for newly saved maps.
- `rows`, `columns`: integers in `[2, 100]`.
- `start`: `{ "row": <zero-based>, "column": <zero-based>, "heading": "up" | "right" | "down" | "left" }`.
- `cells`: rectangular 2D array of `"free"`, `"obstacle"`, `"unknown"`, or `"boundary"`.

Example:

```json
{
  "$schema": "https://raw.githubusercontent.com/poivronjaune/rust_pathfinder/main/schemas/map.schema.json",
  "formatVersion": 3,
  "rows": 2,
  "columns": 3,
  "start": { "row": 1, "column": 2, "heading": "right" },
  "cells": [
    ["boundary", "obstacle", "unknown"],
    ["free", "free", "free"]
  ]
}
```

Version 1 files lacking `start` default to bottom-center facing Up; version 2 files support the original three states. Version 3 makes Boundary a first-class simulator terrain value; outer grid edges are implicit boundaries and do not require extra array cells. The simulator and schema must reject any file that positions a vehicle on non-Free terrain, or normalize valid legacy cases according to an explicit, tested compatibility rule.

### 9.4 State transitions and conflict rules

- **Environment mutation** (New, Load, resize, edit) resets or invalidates Navigator knowledge. No autonomous execution may continue on stale terrain.
- **Reset Session** affects only Navigator session data and vehicle pose, not the learned map or simulator terrain.
- **Reset Map** affects Navigator knowledge and session, not simulator terrain or saved `.map` file.
- **Classification** changes learned state and, for an in-grid Unknown environment cell, its simulated terrain classification; it does not rewrite the `.map` file unless the user later saves.
- **Manual control** cannot interleave with Running/Inspecting steps; an explicitly initiated session from a new pose must synchronize origin and heading.
- **Completion** is a derived navigation result, not a persisted cell type.

## 10. Implementation sequence

1. **Map and contracts:** Extend terrain schema/codec to Boundary; define Navigator relative coordinates, learned terrain, observation requests/results, and independent action flags.
2. **Controlled execution:** Wire three-cell forward observation, inspection dialog, Step execution, one-cell movement feedback, and state rendering.
3. **Autonomy:** Implement deterministic alternating-pass preference, exploration frontier selection, backtracking over known Free cells, completion and blocked detection.
4. **Product integration:** Continuous execution, Pause, two resets, path/action visualization, edge-case handling, and cross-platform tests.

## 11. Verification and release definition

A release conforming to this PRD must satisfy all acceptance tests, preserve core manual/editor/file functionality, expose Navigator as an independent Rust module, and prohibit hidden simulator-map access by Navigator decision logic. The complete PRD is authoritative; `changelog.md` documents revisions but is not an implementation input.
