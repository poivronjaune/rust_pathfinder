# Product Requirements Document: my_app

## 1. Overview

### Product name
Rust PathFinder

### Product type
Desktop prototype for a grid-based backyard mapping and navigation experience.

### Status
Interactive prototype with placeholder menu actions.

### Summary
Rust PathFinder is a small desktop application that displays a 20 x 20 grid and a simulated vehicle. The user can move the vehicle one cell at a time using arrow keys and can observe the current row, column, and heading in a status area. The current implementation is intentionally limited to visualizing a sample map and vehicle state; it does not yet provide editing, persistence, route planning, or autonomous navigation.

## 2. Problem and opportunity

Users exploring early concepts for a backyard mapping or navigation tool need a simple, visual proof of concept that demonstrates a map grid, a vehicle model, and discrete movement controls. A small prototype with clear keyboard interaction and cross-platform native framing provides a foundation for future features such as obstacle handling, map editing, and route planning.

This prototype exists to validate the interaction model and platform structure before investing in more advanced product behavior.

## 3. Product goals

- Display a clear 20 x 20 grid with distinct terrain states.
- Show exactly one simulated vehicle at a time.
- Allow the user to move the vehicle in discrete steps using keyboard input.
- Keep vehicle movement bounded within the map.
- Present state information in a platform-native status area.
- Provide a native desktop shell on both Windows and Linux using the same shared app logic.
- Establish an extensible Rust foundation for future plotting, map editing, and navigation features.

## 4. Non-goals

The current prototype does not include:

- Real obstacle avoidance or collision handling
- Map editing or import/export
- Persistent storage of map or vehicle state
- Autonomous route planning or destination selection
- Real-world navigation physics or scale
- Multiple vehicles or mission management
- Advanced accessibility controls beyond the current UI shell

## 5. Target users

- Developers prototyping grid-based map or navigation interactions
- Users evaluating a simple visual representation of a vehicle moving within a bounded map
- Early design stakeholders validating the interaction model before feature expansion

## 6. User experience

### 6.1 Startup

On launch, the application opens a desktop window titled "20 x 20 Grid". The window contains a native menu bar, a central grid visualization, and a status area beneath the grid.

The initial view shows a deterministic sample terrain map and a single vehicle positioned in the bottom-center cell facing upward.

### 6.2 Grid and terrain

The map is a fixed 20 x 20 matrix. Each cell is rendered with a border and a fill color representing its terrain:

- Free terrain: green
- Obstacle terrain: brown
- Unknown terrain: gray

The map is generated deterministically on startup and is intended as a visual demonstration rather than a user-authored map.

### 6.3 Vehicle behavior

The vehicle is represented as a red body with a black triangular indicator showing heading direction. The car occupies one grid cell at a time and can be moved with discrete keyboard actions:

- Up arrow: move forward one cell along current heading
- Down arrow: move backward one cell opposite current heading
- Left arrow: rotate 90 degrees counterclockwise in place
- Right arrow: rotate 90 degrees clockwise in place
- Escape: close the application when the grid has keyboard focus

Movement is clamped to the map edges. Attempted movement beyond the edge leaves the vehicle in place.

### 6.4 Menu experience

The application exposes native File, Map, and About menu groups. All menu items currently open placeholder informational dialogs. The menu is intended to represent the future user interaction model for a broader product, not to implement live functionality.

### 6.5 Status output

The status area reports the current vehicle state as:

- Ready
- Row
- Column
- Heading

The displayed state updates after movement or rotation.

## 7. Functional requirements

### FR-1: Grid display
1. The application shall display a 20-row by 20-column grid.
2. Each cell shall render with a visible border and a terrain-specific fill color.
3. The app shall generate a deterministic sample map on startup.
4. The grid shall be centered within the window and resize proportionally with the window.

### FR-2: Vehicle display
1. The application shall display exactly one simulated vehicle.
2. The vehicle shall occupy one grid cell.
3. The vehicle shall be drawn with a red body and black heading indicator.
4. The initial vehicle shall be in the bottom-center cell and face upward.

### FR-3: Vehicle movement and turning
1. An up-arrow press shall move the vehicle forward one cell in the current heading.
2. A down-arrow press shall move the vehicle backward one cell opposite the current heading.
3. A left-arrow press shall rotate the vehicle 90 degrees counterclockwise in place.
4. A right-arrow press shall rotate the vehicle 90 degrees clockwise in place.
5. Vehicle movement shall not extend beyond the grid boundaries.
6. Movement attempts beyond the grid edge shall leave the vehicle in place.
7. Turning shall not change the vehicle's cell position.

### FR-4: Application lifecycle
1. The application shall launch in a native desktop window.
2. The application shall close when the user presses Escape while the grid has focus.
3. The application shall display startup errors to standard error and exit with a non-zero result if it fails to initialize.

### FR-5: Menus and placeholder actions
1. The application shall show File, Map, and About menu entries.
2. File shall include New, Save, Load, and Exit.
3. Map shall include Edit.
4. About shall open a native informational dialog directly.
5. Selecting any menu item shall show a placeholder message indicating the action is not implemented yet.

### FR-6: Status area
1. The application shall display a status area beneath the grid.
2. The status area shall report the vehicle's row, column, and heading.
3. Status text shall update after movement or turning.

## 8. Non-functional requirements

### NFR-1: Cross-platform compatibility
The application shall support both Windows and Linux with OS-native windowing and menu conventions.

### NFR-2: Deterministic rendering
The initial map and vehicle state shall be deterministic across launches for prototype consistency.

### NFR-3: Performance
The app must remain responsive for a fixed-size 20 x 20 grid and a single vehicle in a desktop environment.

### NFR-4: Maintainability
The app shall keep map, vehicle, rendering, and platform shell logic separated so future capabilities can be added without rewriting the core behavior.

### NFR-5: Prototype clarity
The implementation shall make it clear that advanced map-editing, route-planning, and persistence features are not yet included.

## 9. Acceptance criteria

1. Launching the application opens a window titled "20 x 20 Grid".
2. The initial view shows a 20 x 20 map with a visible car in the bottom-center cell facing up.
3. Pressing Up moves the car one cell forward without changing direction.
4. Pressing Down moves the car one cell backward without changing direction.
5. Pressing Left or Right rotates the car 90 degrees without moving its position.
6. Pressing a movement key at the map edge does not move the car outside the grid.
7. The status area shows the current row, column, and heading and updates after each input.
8. Selecting any menu item opens a native informational placeholder dialog rather than performing an action.
9. Escape closes the application when the grid has keyboard focus.
10. The project compiles successfully in the target environment.

## 10. Product decisions and scope boundaries

This prototype intentionally does not decide:

- Whether obstacles should block vehicle movement
- Whether maps should be editable or imported
- Whether route planning should be implemented
- Whether state should be persisted between sessions
- Whether the app will become a full backyard-planning product or remain a visual simulation tool

## 11. Future candidate enhancements

These items are explicitly out of scope for the current prototype but may be valuable in future milestones:

- Add user-editable terrain maps
- Define and implement obstacle collisions
- Add destination selection and route visualization
- Add unknown-terrain behavior and pathfinding
- Persist maps and vehicle state
- Add accessibility improvements and visual legend support
- Expand the app into a broader navigation and planning product

## 12. Technical implementation context

This section is intentionally separated from product requirements so the product intent remains clear.

### Platform architecture
- Linux: GTK 4-based window, menu bar, keyboard handling, and Cairo rendering
- Windows: Win32 window, native menu, status bar, and GDI rendering

### Core logic
- Shared map generation and grid state live in the Rust application core.
- Vehicle state and movement rules are managed separately from the OS shell.
- Rendering is performed from a shared pixel buffer so the app can paint the same visual state across platforms.

### Current technical constraints
- The application is a fixed-size visual prototype.
- The map is deterministic and generated locally.
- There is no persistence layer.
- Menu actions are placeholder UI only.
- The repo currently targets Linux and Windows desktop environments.

## 13. Requirements summary

This product is a minimal visual prototype intended to prove the interaction model: a fixed grid, a single vehicle, native desktop framing, and keyboard-driven movement. It is not yet a full navigation application. The PRD is intentionally narrow and should remain so until advanced features are explicitly approved.

## 14. Open questions

- Should obstacle cells eventually block motion, or remain visual-only indicators?
- Should the application become a map editor, a planner, or a simple simulator?
- Is the target next milestone route planning, map editing, or persistent save/load?
- Should menus remain placeholder controls or be re-scoped before broader feature work begins?

