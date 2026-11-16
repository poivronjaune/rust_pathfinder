# Rust PathFinder — Changelog

All PRD version history belongs here. `prd.md` is a clean, self-contained description of the current target product and contains no change-tracking notes.

## [2.0.0] — 2026-11-15

### Product specification
- Replaced the prototype-only PRD with an integrated specification for the simulator, desktop UI, and autonomous `navigator` module.
- Removed domain-specific wording and defined generic per-cell `action_completed: bool` (`true` = action performed, `false` = not performed), independent of actual travel history.
- Preserved manual vehicle controls, simulator editing, grid configuration, map file operations, recent files, and cross-platform UI requirements.

### Navigation
- Specified a one-cell-wide, three-cell-ahead observation sensor with nearest-first inspection and obstacle/boundary occlusion.
- Added UI-driven terrain classification, expandable Navigator-owned discovered map, local movement decisions, alternating parallel coverage preference, exploration, detours, and backtracking.
- Added one-cell atomic movements, confirmed move history (including repeats), action-enabled/action-completed semantics, completion determination, and blocked-state handling.
- Added continuous and Step execution modes, inspection pause, Reset Session, and Reset Map.
- Kept discovered terrain and session data in memory only (no persistence across app restart).

### Map format and integration
- Defined `Boundary` as distinct from `Obstacle`, in addition to `Free` and `Unknown`.
- Set target `.map` JSON format to version 3 to support `boundary` cells; retained read compatibility requirements for versions 1 and 2. The implementation and `schemas/map.schema.json` must be updated accordingly.
- Clarified that the simulator owns actual environment/collision behavior, while the Navigator cannot inspect undiscovered global terrain or grid dimensions.
- Defined environment-edit invalidation, startup/reset position synchronization, modal inspection behavior, and cross-platform UI expectations.

## [1.0.0] — Existing repository baseline (date not specified)

- Grid-based Rust desktop prototype with native Windows and Linux interfaces.
- Deterministic default terrain map; configurable 2–100 rows and columns, 20 × 20 default.
- Single simulated vehicle, manual cardinal driving, obstacle collision, and status display.
- Map editing, legend, Grid Size/start settings, JSON `.map` saving/loading, and nine recent files.
- Simulator/UI separation and v2 map files with legacy v1 loading.

> The baseline entry describes the specification found in the repository before the integrated Navigator update; its historical release date was not established.
