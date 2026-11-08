# Product Requirements Document: Rust PathFinder

## 1. Overview

### Product name
Rust PathFinder

### Product type
Desktop prototype for a grid-based backyard mapping and navigation experience.

### Status
Interactive prototype with working File menu commands (New, Save, Load, recent files, Exit), mouse-based map editing, a configurable grid size and vehicle start, obstacles that block the vehicle, and a color legend.

### Summary
Rust PathFinder is a small desktop application that displays a grid (20 x 20 by default, configurable from 2 to 100 rows and columns) and a simulated vehicle. The user can move the vehicle one cell at a time using arrow keys, obstacles block it, and the user can observe the current row, column, and heading in a status area. The user can also edit the map with the mouse, choose the map size and where the vehicle starts, start a blank map, save or load maps as `.map` files, and reopen recently used maps from the File menu. A legend explains the map colors. The current implementation does not yet provide route planning or autonomous navigation.

## 2. Problem and opportunity

Users exploring early concepts for a backyard mapping or navigation tool need a simple, visual proof of concept that demonstrates a map grid, a vehicle model, and discrete movement controls. A small prototype with clear keyboard interaction and cross-platform native framing provides a foundation for future features such as obstacle handling, map editing, and route planning.

This prototype exists to validate the interaction model and platform structure before investing in more advanced product behavior.

## 3. Product goals

- Display a clear grid with distinct terrain states, 20 x 20 by default and resizable by the user from 2 to 100 rows and columns.
- Show exactly one simulated vehicle at a time.
- Allow the user to move the vehicle in discrete steps using keyboard input.
- Keep vehicle movement bounded within the map and out of obstacle cells.
- Let the user choose where the vehicle starts and which way it faces.
- Present state information in a platform-native status area.
- Let the user start a blank map and save and reload maps as `.map` files in a documented JSON format.
- Let the user reopen recently used maps directly from the File menu.
- Let the user mark cells as obstacles or unknown terrain with the mouse in an explicit edit mode.
- Explain the map colors with an on-screen legend.
- Provide a native desktop shell on both Windows and Linux using the same shared app logic.
- Establish an extensible Rust foundation for future plotting, map editing, and navigation features.

## 4. Non-goals

The current prototype does not include:

- Obstacle avoidance (obstacles block moves, but nothing steers the vehicle around them)
- Editing tools beyond marking cells as obstacle or unknown (for example free terrain, undo, or brushes)
- Persistent storage of the vehicle's current position (map files store the terrain, the map size, and the vehicle's start)
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

On launch, the application opens a desktop window titled with the size of the map, "20 x 20 Grid". The window contains a native menu bar, a central grid visualization, a legend panel, and a status area at the bottom.

The initial view shows a deterministic sample terrain map and a single vehicle positioned in the bottom-center cell facing upward.

### 6.2 Grid and terrain

The map is a grid of 2 to 100 rows and 2 to 100 columns. It starts as a 20 x 20 grid, and the user can change its size (see 6.9). The grid is drawn as large as fits in the window, so larger maps have smaller cells. Each cell is rendered with a border and a fill color representing its terrain:

- Free terrain: green
- Obstacle terrain: brown
- Unknown terrain: gray

The map is generated deterministically on startup as a visual demonstration. The user can edit it with the mouse (see 6.7) or replace it with a blank map or a previously saved map from the File menu (see 6.6). A legend explains the colors (see 6.8).

### 6.3 Vehicle behavior

The vehicle is represented as a red body with a black triangular indicator showing heading direction. The car occupies one grid cell at a time and can be moved with discrete keyboard actions:

- Up arrow: move forward one cell along current heading
- Down arrow: move backward one cell opposite current heading
- Left arrow: rotate 90 degrees counterclockwise in place
- Right arrow: rotate 90 degrees clockwise in place
- Escape: close the application when the grid has keyboard focus

Movement is clamped to the map edges. Attempted movement beyond the edge leaves the vehicle in place.

Obstacles block the vehicle: a forward or backward move into an obstacle cell leaves the vehicle in place, and the status area shows "Blocked by an obstacle" until the next move or turn. Unknown terrain does not block the vehicle, and turning in place is always possible. While the map is in edit mode, the arrow keys do nothing (see 6.7).

### 6.4 Menu experience

The application exposes native File, Map, and About menu groups. File → New, Save, and Load work with map files (see 6.6), the File menu lists recent files (see 6.10), and File → Exit closes the application. About opens a native informational dialog. Map → Edit turns edit mode on and off (see 6.7), and Map → Grid Size... changes the size of the map and the vehicle start (see 6.9).

### 6.5 Status output

The status area reports the current vehicle state as:

- Ready
- Row
- Column
- Heading

The displayed state updates after movement or rotation. After a move blocked by an obstacle, "Blocked by an obstacle" replaces "Ready". In edit mode, "Editing" and a reminder of the mouse actions replace "Ready".

### 6.6 Map files

- **New** immediately replaces the map with a blank one of the same size in which every cell is unknown. The vehicle stays where it is.
- **Save** opens the platform's native save dialog, suggests the name `untitled.map`, asks before overwriting an existing file, and writes the current map as JSON.
- **Load** opens the platform's native open dialog, replaces the current map with the map stored in the chosen file, and puts the vehicle at the map's start.
- Map files use the `.map` extension, and both dialogs list `.map` files by default.
- Map files store the terrain, the size of the map, and the vehicle's start cell and heading as chosen with Map → Grid Size... (see 6.9), not the vehicle's current position. Files written before maps had a start still load, with the vehicle in the bottom-center cell facing up.
- A file that is saved or loaded successfully moves to the top of the recent files (see 6.10).
- If a file cannot be written, read, or validated, the application shows an error dialog and keeps the current map.
- Canceling a dialog leaves everything unchanged.

### 6.7 Map editing

- **Map → Edit** turns edit mode on; selecting it again turns it off. The menu item shows a check mark while edit mode is on.
- While editing, the grid lines turn orange and the status area starts with "Editing", so the mode is always visible.
- In edit mode, a left click on a cell makes it an obstacle and a right click makes it unknown. Holding a button while moving the pointer applies the same change to every cell the pointer crosses.
- Outside edit mode, mouse clicks do not change the map.
- Editing never moves or turns the vehicle, and the vehicle is parked in edit mode: the arrow keys do nothing until edit mode is turned off. The vehicle stays drawn over its cell, so an edit to that cell shows once the vehicle moves away.
- Edits are kept in memory until the user saves the map with File → Save.

### 6.8 Legend

A legend panel between the grid and the status area shows a color sample and a label for free, obstacle, and unknown terrain and for the vehicle, using the same colors as the grid.

### 6.9 Grid size and vehicle start

- **Map → Grid Size...** opens a modal dialog, filled with the current values, with two groups of fields:
  - Grid size: the number of rows and columns, each from 2 to 100.
  - Car start: the start row and column, counted from 1 at the top-left corner like the status area, and the heading (Up, Right, Down, or Left) chosen from a list.
- OK (or Enter) applies the settings; Cancel, Escape, or closing the dialog leaves everything unchanged.
- If a value is missing, not a whole number, or out of range, a message explains what to correct and the dialog stays open with the values entered.
- Applying the settings resizes the map from its top-left corner: cells that still fit keep their terrain, new cells are unknown, and cells beyond the new edges are dropped. The vehicle moves to the start cell and faces the chosen heading. If the start cell is an obstacle, it becomes free so the vehicle never starts inside an obstacle.
- The window title shows the new size, for example "30 x 40 Grid", and every feature (drawing, movement limits, editing, saving, and loading) uses the new size.
- The size and the start are saved with the map (see 6.6).

### 6.10 Recent files

- Following the convention of Microsoft Word and Excel, the File menu lists the most recently saved or loaded map files between Load and Exit, newest first, numbered 1 to 9 so each number is an access key. Long paths are shortened in the middle.
- Choosing a recent file loads it like File → Load.
- Saving or loading a file moves it to the top of the list; a file is never listed twice, and the list keeps at most 9 files.
- If a recent file no longer exists, choosing it shows an error and removes it from the list.
- **Clear Recent Files**, shown below the list when it is not empty, empties the list.
- The list is kept between sessions in the user's configuration folder.

## 7. Functional requirements

### FR-1: Grid display
1. The application shall display a grid of 2 to 100 rows by 2 to 100 columns, 20 x 20 at startup (FR-10).
2. Each cell shall render with a visible border and a terrain-specific fill color.
3. The app shall generate a deterministic sample map on startup.
4. The grid shall be centered within the window and resize proportionally with the window.

### FR-2: Vehicle display
1. The application shall display exactly one simulated vehicle.
2. The vehicle shall occupy one grid cell.
3. The vehicle shall be drawn with a red body and black heading indicator.
4. The initial vehicle shall be in the bottom-center cell and face upward. After Map → Grid Size... or File → Load, it shall be at the map's start (FR-7, FR-10).

### FR-3: Vehicle movement and turning
1. An up-arrow press shall move the vehicle forward one cell in the current heading.
2. A down-arrow press shall move the vehicle backward one cell opposite the current heading.
3. A left-arrow press shall rotate the vehicle 90 degrees counterclockwise in place.
4. A right-arrow press shall rotate the vehicle 90 degrees clockwise in place.
5. Vehicle movement shall not extend beyond the grid boundaries.
6. Movement attempts beyond the grid edge shall leave the vehicle in place.
7. Turning shall not change the vehicle's cell position.
8. A forward or backward move into an obstacle cell shall leave the vehicle in place, and the status area shall report it (FR-6). Unknown cells shall not block the vehicle.
9. In edit mode, the arrow keys shall not move or turn the vehicle (FR-8).

### FR-4: Application lifecycle
1. The application shall launch in a native desktop window.
2. The application shall close when the user presses Escape while the grid has focus.
3. The application shall display startup errors to standard error and exit with a non-zero result if it fails to initialize.
4. Selecting File → Exit shall close the application, including all of its windows.

### FR-5: Menus
1. The application shall show File, Map, and About menu entries.
2. File shall include New, Save, Load, the recent files and Clear Recent Files (FR-11), and Exit.
3. Map shall include Edit and Grid Size... (FR-10).
4. About shall open a native informational dialog directly.
5. Map → Edit shall turn edit mode on and off as described in FR-8, and Map → Grid Size... shall behave as described in FR-10.
6. File → New, Save, and Load shall behave as described in FR-7, and File → Exit as described in FR-4.

### FR-6: Status area
1. The application shall display a status area beneath the grid.
2. The status area shall report the vehicle's row, column, and heading.
3. Status text shall update after movement or turning.
4. In edit mode, the status text shall start with "Editing" and the mouse actions instead of "Ready".
5. After a move blocked by an obstacle, the status text shall start with "Blocked by an obstacle" instead of "Ready" until the next move or turn.

### FR-7: Map files
1. File → New shall set every cell of the map to unknown, keeping the size of the map.
2. File → Save shall open a native save dialog that suggests the name `untitled.map` and write the current map, its size, and the vehicle start to the chosen file as JSON that conforms to `schemas/map.schema.json`.
3. File → Load shall open a native open dialog, replace the current map with the map stored in the chosen file (including its size), and put the vehicle at the stored start, or in the bottom-center cell facing up when the file has no start.
4. New and Save shall not change the vehicle's position or heading. Map files shall store the vehicle's start, not its current position.
5. Loading shall reject files that are not valid JSON, declare an unsupported format version, contain unknown properties or cell values, have rows or columns that do not match their declared size, have fewer than 2 or more than 100 rows or columns, or place the start outside the map.
6. When a file cannot be written, read, or validated, the application shall show an error dialog naming the file and the problem, and keep the current map.
7. Canceling the save or open dialog shall leave the map unchanged.
8. Map files shall use the `.map` extension, and the save and open dialogs shall list `.map` files by default.
9. The application shall write format version 2 and keep loading version 1 files, which have no start.

### FR-8: Map editing
1. Map → Edit shall turn edit mode on when it is off and off when it is on, and the menu item shall show a check mark while edit mode is on.
2. While edit mode is on, the grid lines shall be drawn in a distinct edit color (orange) and the status area shall indicate edit mode (FR-6).
3. In edit mode, a left click on a cell shall make it an obstacle, and a right click shall make it unknown.
4. Holding a mouse button while moving the pointer shall apply the same change to every cell the pointer crosses.
5. Outside edit mode, mouse clicks shall not change the map.
6. Editing shall not move or turn the vehicle, and the vehicle shall remain drawn over its cell.
7. File → New and File → Load shall keep the current edit mode.
8. In edit mode, the arrow keys shall not move or turn the vehicle.

### FR-9: Legend
1. The application shall show a legend panel between the grid and the status area.
2. The legend shall show a color sample and a label for free, obstacle, and unknown terrain and for the vehicle, using the colors drawn on the grid.

### FR-10: Grid size and vehicle start
1. Map → Grid Size... shall open a modal dialog with fields for the number of rows and columns, the start row and column, and the start heading, filled with the current values.
2. Start rows and columns shall be entered counting from 1, and the heading shall be chosen from Up, Right, Down, and Left.
3. OK and Enter shall apply the settings; Cancel, Escape, and closing the dialog shall change nothing.
4. The dialog shall accept only whole numbers of rows and columns from 2 to 100, and a start cell inside the map. Otherwise it shall explain what to correct and stay open.
5. Applying the settings shall resize the map from its top-left corner, keeping the terrain of cells that still fit and making new cells unknown.
6. Applying the settings shall move the vehicle to the start cell with the chosen heading, and shall make the start cell free if it is an obstacle.
7. The window title shall show the map size as "<rows> x <columns> Grid".
8. Drawing, movement limits, editing, and map files shall all use the current map size.

### FR-11: Recent files
1. The File menu shall list up to 9 recently saved or loaded map files, newest first, numbered 1 to 9 with the number as access key.
2. Choosing a recent file shall load it as described in FR-7.
3. Saving or loading a file shall move it to the top of the list without listing it twice.
4. Choosing a recent file that no longer exists shall show an error and remove it from the list.
5. File → Clear Recent Files shall empty the list, and shall be shown only when the list is not empty.
6. The list shall persist between sessions in the user's configuration folder (`%APPDATA%` on Windows, `$XDG_CONFIG_HOME` or `~/.config` on Linux).

## 8. Non-functional requirements

### NFR-1: Cross-platform compatibility
The application shall support both Windows and Linux with OS-native windowing and menu conventions.

### NFR-2: Deterministic rendering
The initial map and vehicle state shall be deterministic across launches for prototype consistency.

### NFR-3: Performance
The app must remain responsive for grids of up to 100 x 100 cells and a single vehicle in a desktop environment.

### NFR-4: Maintainability
The app shall keep map, vehicle, rendering, and platform shell logic separated so future capabilities can be added without rewriting the core behavior.

### NFR-5: Prototype clarity
The implementation shall make it clear that route planning is not yet included.

### NFR-6: Documented map file format
The map file format shall be defined by a standalone JSON Schema (draft 2020-12) in `schemas/map.schema.json`, separate from the application code. Saved files shall reference the schema through `$schema` and carry a `formatVersion` so the format can evolve, and automated tests shall check that the code and the schema agree.

## 9. Acceptance criteria

1. Launching the application opens a window titled "20 x 20 Grid".
2. The initial view shows a 20 x 20 map with a visible car in the bottom-center cell facing up.
3. Pressing Up moves the car one cell forward without changing direction.
4. Pressing Down moves the car one cell backward without changing direction.
5. Pressing Left or Right rotates the car 90 degrees without moving its position.
6. Pressing a movement key at the map edge does not move the car outside the grid.
7. The status area shows the current row, column, and heading and updates after each input.
8. Selecting Map → Edit turns edit mode on (check mark on the menu item, orange grid lines, and "Editing" in the status area), and selecting it again turns edit mode off.
9. Escape closes the application when the grid has keyboard focus.
10. The project compiles successfully in the target environment.
11. Selecting File → New turns every cell unknown without changing the map size or moving or turning the car.
12. Saving a map to a `.map` file and loading it again restores the same terrain, map size, and car start, and the saved file validates against `schemas/map.schema.json`.
13. Loading a file that cannot be read or is not a valid map shows an error dialog and leaves the current map unchanged.
14. Selecting File → Exit closes the application.
15. In edit mode, left-clicking a cell makes it an obstacle and right-clicking makes it unknown, and dragging with a button held changes every cell crossed. Outside edit mode, clicks do not change the map.
16. A legend between the grid and the status area shows the free, obstacle, unknown, and vehicle colors.
17. Pressing Up or Down toward an adjacent obstacle leaves the car in place, and the status area shows "Blocked by an obstacle".
18. In edit mode, the arrow keys neither move nor turn the car.
19. Choosing Map → Grid Size... and entering 30 rows, 40 columns, start row 5, start column 10, and heading Left resizes the map to 30 x 40 (title "30 x 40 Grid") and puts the car at row 5, column 10, facing left. Entering a value out of range shows a message and keeps the dialog open, and Cancel changes nothing.
20. After a map is saved or loaded, it is listed first among the recent files in the File menu; choosing it there loads it, the list survives a restart, and File → Clear Recent Files empties it.

## 10. Product decisions and scope boundaries

This prototype intentionally does not decide:

- Whether maps should be imported from other formats
- Whether route planning should be implemented
- Whether the vehicle's current position should be saved with maps (maps store only its start)
- Whether the app will become a full backyard-planning product or remain a visual simulation tool

Decided: obstacles block the vehicle's moves, unknown terrain does not, and the arrow keys are disabled in edit mode.

## 11. Future candidate enhancements

These items are explicitly out of scope for the current prototype but may be valuable in future milestones:

- Extend the map editor (free terrain, undo, larger brushes)
- Add destination selection and route visualization
- Add unknown-terrain behavior and pathfinding
- Persist the vehicle's current position alongside maps
- Add accessibility improvements
- Expand the app into a broader navigation and planning product

## 12. Technical implementation context

This section is intentionally separated from product requirements so the product intent remains clear.

### Platform architecture
- Linux: GTK 4-based window, menu bar (with a menu section for the recent files), keyboard and mouse handling, native file chooser, modal Grid Size window built from GTK widgets, legend panel built from GTK widgets, and Cairo rendering
- Windows: Win32 window, native menu (rebuilt when the recent files change), keyboard and mouse handling, common Save and Open dialogs, modal Grid Size window built from standard controls, GDI-drawn legend panel, status bar, and GDI rendering

### Core logic
- Shared map generation and grid state live in the Rust application core.
- Vehicle state and movement rules are managed separately from the OS shell.
- Rendering is performed from a shared pixel buffer so the app can paint the same visual state across platforms.
- Edit mode, the mouse editing rules, the mapping from screen points to cells, and the legend entries live in the shared core (`src/app.rs`, `src/render.rs`) so both platforms behave the same.
- The map size, the vehicle start, the validation of the Grid Size dialog, map resizing, and obstacle blocking also live in the shared core (`src/app.rs`, `src/grid.rs`, `src/car.rs`); each platform only builds the dialog.
- The recent files list, including where it is stored, is shared (`src/recent.rs`); each platform only shows it in its File menu.
- Map files are encoded, decoded, and validated by a shared module (`src/map_file.rs`) built on serde and serde_json. The format itself is defined separately in `schemas/map.schema.json`.

### Current technical constraints
- Maps have 2 to 100 rows and columns; larger maps are drawn with smaller cells to fit the window.
- The map is deterministic and generated locally.
- Maps persist only through explicit File → Save and File → Load; nothing is saved automatically.
- The repo currently targets Linux and Windows desktop environments.

## 13. Requirements summary

This product is a minimal visual prototype intended to prove the interaction model: a configurable grid, a single vehicle that obstacles block, native desktop framing, keyboard-driven movement, mouse-based map editing, and saving, loading, and reopening maps. It is not yet a full navigation application. The PRD is intentionally narrow and should remain so until advanced features are explicitly approved.

## 14. Open questions

- Should the application become a map editor, a planner, or a simple simulator?
- Is the target next milestone route planning or richer map editing?
- Should the editor also mark cells as free, and should it offer undo?
- Should Escape leave edit mode instead of closing the application?
- Should File → Exit or closing the window ask before discarding unsaved map changes?
- Should map files also store the vehicle's current position, in addition to its start?
- Should resizing offer other anchors than the top-left corner, for example keeping the map centered?
- Should the recent files list offer pinning, like Microsoft Office does?

