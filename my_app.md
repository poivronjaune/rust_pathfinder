# Product Requirements Document: my_app

## 1. Product summary

**Product name:** my_app  
**Product type:** Desktop grid-visualization and simulated-vehicle prototype  
**Status:** Interactive prototype with placeholder menus

my_app is a small Rust desktop application that displays a color-coded, 20 × 20 backyard-style grid and a simulated car. The user can drive the car one cell at a time using the arrow keys: up moves forward, down reverses, and left/right turn the car without moving it. The car is drawn above the map with a red body and a black triangular indicator showing its heading. Platform-native menus provide File, Map, and About; their current actions open informational placeholder dialogs. A platform-styled status area reports the car's position and heading.

The current product is a visual and input-handling prototype. It does not yet provide map editing, persistent data, route planning, or obstacle-aware navigation.

## 2. Problem and opportunity

A grid map provides a simple visual foundation for experimenting with a backyard mapping and navigation application. A visible car, discrete movement, and clear keyboard controls make it possible to explore how vehicle orientation and position relate to a mapped environment before adding more advanced mapping or path-planning capabilities.

## 3. Goals

- Display a legible, color-coded grid representing free, obstacle, and unknown terrain.
- Represent a simulated car in a single grid cell, including its current heading.
- Provide intuitive, discrete vehicle controls using the arrow keys.
- Provide native File, Map, and About menus with placeholder dialogs for their menu items.
- Show the vehicle position and heading in a status area.
- Use a native window shell on Windows and Linux while sharing the map and vehicle logic.
- Keep the vehicle within the grid boundaries.
- Establish a compact, maintainable Rust foundation for future mapping and navigation features.

## 4. Non-goals

The current version does not:

- Detect, avoid, or prevent movement into obstacle or unknown cells.
- Include collision response, pathfinding, route visualization, or autonomous driving.
- Allow users to edit, import, export, or save maps.
- Persist vehicle state or map state between application launches.
- Provide mouse-based map editing, touch controls, game-controller support, or configurable vehicle keys.
- Model real-world scale, vehicle physics, speed, acceleration, or turning radius.
- Include multiple vehicles, a mission system, or telemetry.

## 5. Target users

- Developers prototyping a grid-based backyard mapping or navigation application.
- Users who want to inspect a simple map and manually move a simulated vehicle.

## 6. Product experience

### 6.1 Startup

On launch, the application opens a resizable desktop window titled **“20 x 20 Grid”**. It renders a 20 × 20 grid in the window's client area, below the native menu and above the status area. Each cell has a one-pixel blue border and an interior fill representing the cell type. A red car appears in the center column of the bottom row, initially facing upward.

The grid is generated deterministically on each launch using row and column patterns. It is a sample map rather than user-created or loaded backyard data.

### 6.2 Map and visual encoding

| Map element | Appearance | Meaning |
|---|---|---|
| Free cell | Green interior | Traversable/free terrain in the map representation |
| Obstacle cell | Brown interior | Obstacle terrain in the map representation |
| Unknown cell | Gray interior | Unexplored or unknown terrain |
| Cell border | Blue | Grid-cell boundary |
| Car body | Red | Simulated vehicle location |
| Car indicator | Black triangle | The direction in which the car is facing |

The car is drawn after the map, so its body and indicator cover the underlying cell. The triangle rotates to match the car's heading.

### 6.3 Vehicle controls

Controls are handled as discrete key presses; holding a key does not continuously move or rotate the car.

| Key | Action |
|---|---|
| Up arrow | Move forward one cell in the current heading |
| Down arrow | Move backward one cell, opposite the current heading; heading is unchanged |
| Left arrow | Turn 90° counterclockwise in place |
| Right arrow | Turn 90° clockwise in place |
| Escape | Close the application when the grid has keyboard focus |

Forward and reverse movement are clamped at the grid edges. A blocked move leaves the vehicle at the edge; it does not wrap to the opposite side. Turning is allowed at any cell, including the grid edges.

### 6.4 Window behavior

The window is resizable. The grid image scales to fit while preserving its aspect ratio; unused space is centered around it. The operating system's menu occupies the top of the window and the status area sits below the grid. Rendering is event-driven and refreshes when the vehicle changes or the window needs repainting.

### 6.5 Menus

The application provides a native menu bar, with platform-specific implementations:

| Menu | Items |
|---|---|
| File | New, Save, Load, Exit |
| Map | Edit |
| About | No submenu |

Selecting File or Map opens the platform's dropdown. Selecting About immediately opens a native informational dialog. Selecting any submenu item opens a native informational dialog titled with the selected item and indicating that the feature is not implemented. Menu commands, including Exit, do not yet perform their eventual operations. Dialogs and menus are dismissed using the platform's standard interactions. Escape closes the application when focus is on the grid.

### 6.6 Status area

The status area uses the platform's standard control styling and displays **Ready**, the car's one-based row and column, and its heading. The displayed vehicle state updates after every arrow-key press.

## 7. Functional requirements

### FR-1: Grid display

1. The application shall display a 20-row by 20-column grid.
2. The grid shall occupy a 400 × 400 pixel region in the window client area, with each logical cell rendered as a 20 × 20 pixel region.
3. Each cell shall have a one-pixel blue border and a fill color corresponding to its terrain type.
4. The application shall create a deterministic demonstration map containing free, obstacle, and unknown cells.

### FR-2: Vehicle display

1. The application shall display exactly one simulated car.
2. The car shall occupy one grid cell and have a red body.
3. A black triangular indicator shall identify the car's forward direction.
4. On startup, the car shall be in the bottom row, center column, facing up.
5. The car shall render over the map cell at its current position.

### FR-3: Vehicle movement and turning

1. An up-arrow press shall advance the car one cell along its current heading.
2. A down-arrow press shall move the car one cell opposite its current heading without changing its heading.
3. A left-arrow press shall rotate the car 90° counterclockwise in place.
4. A right-arrow press shall rotate the car 90° clockwise in place.
5. Movement shall not take the car outside the grid.
6. Attempted movement beyond an edge shall leave the car in its current cell.
7. Turning shall not change the car's grid position.
8. The rendered car position and indicator shall reflect updated state on the next repaint.

### FR-4: Application lifecycle

1. The application shall remain responsive while its window is open and redraw when its state or size changes.
2. The application shall report startup errors to standard error and exit with a non-zero status.
3. Escape shall close the application when the grid has keyboard focus; menus and dialogs shall use their native dismissal behavior.

### FR-5: Menus and placeholder actions

1. The application shall display File, Map, and About in a platform-native menu bar above the map.
2. File shall contain New, Save, Load, and Exit submenu items.
3. Map shall contain an Edit submenu item.
4. About shall open its informational dialog directly and shall not display a submenu.
5. Selecting any menu item shall show a native informational dialog; menu items shall not perform file, map-editing, or exit operations in this prototype.

### FR-6: Status area

1. The application shall display a platform-styled status area below the grid.
2. The status area shall report the vehicle's row, column, and heading.
3. Vehicle state shown in the status area shall update after each arrow-key press.

## 8. User stories

- **As a user**, I want to see a grid with distinct terrain colors so that I can distinguish free, obstacle, and unexplored areas.
- **As a user**, I want to see a car's heading so that its orientation is clear while navigating the grid.
- **As a user**, I want to move forward or backward one cell at a time so that I can control the car precisely.
- **As a user**, I want to turn left or right without moving so that I can orient the car before driving.
- **As a user**, I want the car to remain within the map so that movement at boundaries is predictable.
- **As a user**, I want Escape to close the application so that I can exit using the keyboard.
- **As a user**, I want File and Map menus with the requested items so that the planned actions are discoverable.
- **As a user**, I want native-looking menus and dialogs so that the application follows my operating system's conventions.
- **As a user**, I want to see the car's position and heading in the status area so that I can track its state while driving.

## 9. Acceptance criteria

1. Launching the application opens a window showing a 20 × 20 colored grid.
2. The initial car is visible in the bottom-center cell and points up.
3. Pressing Up moves the car one cell toward the top of the grid without changing its heading.
4. Pressing Down moves it one cell opposite its heading without changing its heading.
5. Pressing Left or Right rotates the indicator by 90° in the corresponding direction and leaves the car in the same cell.
6. Repeated movement at any grid edge never moves the car outside the grid.
7. The terrain remains visible and the car is rendered above the cell it occupies.
8. Pressing Escape exits the application.
9. File displays New, Save, Load, and Exit; Map displays Edit; About opens directly without a submenu.
10. Selecting any menu item displays a native placeholder dialog.
11. The status area shows the current row, column, and heading and updates after movement or turning.
12. Menu items use the native menu implementation for the current platform, and selecting one opens a platform-styled placeholder dialog.
13. The Rust project passes `cargo check` on the target platform.

## 10. Usability and accessibility considerations

- Terrain types are currently distinguished by color; future iterations should consider non-color cues, such as symbols or patterns, to improve accessibility.
- The black heading indicator should remain visually distinct against the red body.
- Keyboard controls should respond once per press rather than continuously while a key is held.
- The application currently provides no on-screen legend or vehicle-control instructions; users may need this PRD or other documentation to discover the controls.

## 11. Technical context and constraints

- **Language:** Rust, edition 2024.
- **Windows shell:** Win32 window, native menus, common-controls status bar, message dialogs, and GDI display of the shared grid buffer.
- **Linux shell:** GTK 4 application window, popover menu bar, message dialogs, status bar, and Cairo display of the shared grid buffer. Linux builds require GTK 4 development libraries.
- **Grid representation:** A fixed-size 20 × 20 array with `Free`, `Obstacle`, and `Unknown` cell states.
- **Vehicle state:** Grid row, grid column, and one of four cardinal headings.
- **Rendering:** A shared CPU-side 400 × 400 pixel buffer containing only the map and vehicle. Vehicle input redraws the old and new car cells in that buffer. Windows refreshes the scaled off-screen GDI bitmap from the buffer, then invalidates and blits only the affected cell rectangles, keeping display updates buffered and cell-scoped.
- **Menus and status:** Platform-native menu, dialog, and status-bar implementations.
- **Code organization:** Rust modules separate application state and input actions, vehicle state, map data, shared rendering, and Windows/Linux window backends.
- **Map generation:** Deterministic row/column rules in the current implementation.
- **Persistence:** None.
- **Platform:** Windows and Linux are the supported targets. Each target uses its native windowing toolkit while sharing the grid, vehicle model, and pixel renderer.

## 12. Assumptions and open product decisions

- Terrain colors and the fixed 20 × 20 grid are suitable for the initial prototype.
- The demonstration map is intended for visualization, not as authoritative backyard data.
- Movement is intentionally discrete and one cell per key press.
- Obstacle and unknown cells are currently visual categories only; the car can enter them.
- Future product work should decide whether maps are editable or imported, whether obstacles block motion, and whether the app should support route planning or autonomous navigation.

## 13. Potential future enhancements

These are not part of the current requirements:

- Add a visible legend and in-app control hints.
- Allow users to edit terrain cells and create or load a real map.
- Define and implement collision and unknown-terrain behavior.
- Add path planning, destination selection, and route visualization.
- Persist maps and vehicle state.
- Add configurable controls, accessibility options, and multiple window/display settings.
- Add unit tests for heading changes, forward/reverse movement, and boundary behavior.
