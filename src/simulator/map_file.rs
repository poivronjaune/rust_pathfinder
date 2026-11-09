//! Map files (`*.map`): JSON documents described by `schemas/map.schema.json`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize, Serializer};

use crate::simulator::{Car, Cell, Direction, Grid, MAX_GRID_SIZE, MIN_GRID_SIZE};

/// `$id` of `schemas/map.schema.json`, written into saved maps so editors can validate them.
const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/poivronjaune/rust_pathfinder/main/schemas/map.schema.json";
/// Version written by File → Save. Version 2 added the optional car start.
const FORMAT_VERSION: u32 = 2;
/// Versions that File → Load can read.
const SUPPORTED_VERSIONS: [u32; 2] = [1, FORMAT_VERSION];
/// Extension of map files, without the leading dot.
pub(crate) const FILE_EXTENSION: &str = "map";
/// Name suggested when saving a map.
pub(crate) const DEFAULT_FILE_NAME: &str = "untitled.map";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MapDocument {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    schema: Option<String>,
    format_version: u32,
    rows: usize,
    columns: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    start: Option<StartDocument>,
    #[serde(serialize_with = "serialize_rows")]
    cells: Vec<Vec<Cell>>,
}

/// Where the car starts on the map, with `row` and `column` counted from 0.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StartDocument {
    row: usize,
    column: usize,
    heading: Direction,
}

/// Reads only the format version, so maps saved in a newer format are reported as such instead
/// of failing on fields this version does not know.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionProbe {
    format_version: u32,
}

/// A map read from a file, with the car start if the file has one.
pub(crate) struct MapData {
    pub(crate) grid: Grid,
    pub(crate) start: Option<Car>,
}

pub(crate) fn save(grid: &Grid, start: Car, path: &Path) -> Result<(), String> {
    let result = match to_json(grid, start) {
        Ok(json) => fs::write(path, json).map_err(|error| error.to_string()),
        Err(error) => Err(error.to_string()),
    };
    result.map_err(|error| format!("Unable to save the map to {}: {error}", path.display()))
}

pub(crate) fn load(path: &Path) -> Result<MapData, String> {
    let result = match fs::read_to_string(path) {
        Ok(json) => from_json(&json),
        Err(error) => Err(error.to_string()),
    };
    result.map_err(|error| format!("Unable to load the map from {}: {error}", path.display()))
}

fn to_json(grid: &Grid, start: Car) -> serde_json::Result<String> {
    let document = MapDocument {
        schema: Some(SCHEMA_URL.to_owned()),
        format_version: FORMAT_VERSION,
        rows: grid.rows,
        columns: grid.cols,
        start: Some(StartDocument {
            row: start.row,
            column: start.col,
            heading: start.direction,
        }),
        cells: grid.cells.clone(),
    };
    let mut json = serde_json::to_string_pretty(&document)?;
    json.push('\n');
    Ok(json)
}

fn from_json(json: &str) -> Result<MapData, String> {
    // RFC 8259 lets parsers ignore a byte order mark, which some Windows editors add.
    let json = json.strip_prefix('\u{feff}').unwrap_or(json);
    let probe: VersionProbe = parse(json)?;
    if !SUPPORTED_VERSIONS.contains(&probe.format_version) {
        return Err(format!(
            "map format version {} is not supported (expected version {FORMAT_VERSION} or older)",
            probe.format_version
        ));
    }

    let document: MapDocument = parse(json)?;
    if document.cells.len() != document.rows {
        return Err(format!(
            "the map declares {} rows but cells contains {}",
            document.rows,
            document.cells.len()
        ));
    }
    if let Some((index, row)) = document
        .cells
        .iter()
        .enumerate()
        .find(|(_, row)| row.len() != document.columns)
    {
        return Err(format!(
            "cells[{index}] contains {} cells but the map declares {} columns",
            row.len(),
            document.columns
        ));
    }
    let sizes = MIN_GRID_SIZE..=MAX_GRID_SIZE;
    if !sizes.contains(&document.rows) || !sizes.contains(&document.columns) {
        return Err(format!(
            "the map is {} x {} but maps must have between {MIN_GRID_SIZE} and {MAX_GRID_SIZE} \
             rows and columns",
            document.rows, document.columns
        ));
    }
    let start = match document.start {
        Some(start) if start.row >= document.rows || start.column >= document.columns => {
            return Err(format!(
                "the car start (row {}, column {}) is outside the {} x {} map",
                start.row, start.column, document.rows, document.columns
            ));
        }
        Some(start) => Some(Car {
            row: start.row,
            col: start.column,
            direction: start.heading,
        }),
        None => None,
    };

    let grid = Grid {
        rows: document.rows,
        cols: document.columns,
        cells: document.cells,
    };
    Ok(MapData { grid, start })
}

fn parse<'a, T: Deserialize<'a>>(json: &'a str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|error| error.to_string())
}

/// Writes each grid row on a single line so a saved map reads like the grid it describes.
fn serialize_rows<S: Serializer>(cells: &[Vec<Cell>], serializer: S) -> Result<S::Ok, S::Error> {
    let rows = cells
        .iter()
        .map(serde_json::value::to_raw_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(<S::Error as serde::ser::Error>::custom)?;
    rows.serialize(serializer)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::ffi::OsStr;
    use std::path::Path;

    use serde_json::{Value, json};

    use super::{
        DEFAULT_FILE_NAME, FILE_EXTENSION, FORMAT_VERSION, SCHEMA_URL, SUPPORTED_VERSIONS,
        from_json, to_json,
    };
    use crate::simulator::{
        Car, Cell, DEFAULT_GRID_SIZE, Direction, Grid, MAX_GRID_SIZE, MIN_GRID_SIZE,
    };

    const SCHEMA: &str = include_str!("../../schemas/map.schema.json");
    const SIZE: usize = DEFAULT_GRID_SIZE;

    fn start() -> Car {
        Car::start_for(SIZE, SIZE)
    }

    #[test]
    fn saved_json_loads_back_unchanged() {
        let grid = Grid::backyard_with_seed(SIZE, SIZE, SIZE - 1, SIZE / 2, 7);
        let loaded = from_json(&to_json(&grid, start()).unwrap()).unwrap();
        assert_eq!(loaded.grid, grid);
        assert_eq!(loaded.start, Some(start()));
    }

    #[test]
    fn saves_and_loads_rectangular_maps_and_the_car_start() {
        let mut grid = Grid::unknown(7, 31);
        grid.cells[6][30] = Cell::Obstacle;
        let start = Car {
            row: 2,
            col: 30,
            direction: Direction::Left,
        };

        let loaded = from_json(&to_json(&grid, start).unwrap()).unwrap();
        assert_eq!(loaded.grid, grid);
        assert_eq!(loaded.start, Some(start));
    }

    #[test]
    fn loads_version_1_maps_without_a_car_start() {
        let mut document = saved_document();
        document["formatVersion"] = json!(1);
        document.as_object_mut().unwrap().remove("start");

        let loaded = from_json(&document.to_string()).unwrap();
        assert_eq!(loaded.grid, Grid::unknown(SIZE, SIZE));
        assert_eq!(loaded.start, None);
    }

    #[test]
    fn default_file_name_uses_the_map_extension() {
        let extension = Path::new(DEFAULT_FILE_NAME).extension();
        assert_eq!(extension, Some(OsStr::new(FILE_EXTENSION)));
    }

    #[test]
    fn saved_json_puts_each_row_on_its_own_line() {
        let row = format!("    [{}]", vec!["\"unknown\""; 3].join(","));
        let rows = vec![row; 2].join(",\n");
        let start = Car {
            row: 1,
            col: 2,
            direction: Direction::Right,
        };
        let expected = format!(
            r#"{{
  "$schema": "{SCHEMA_URL}",
  "formatVersion": {FORMAT_VERSION},
  "rows": 2,
  "columns": 3,
  "start": {{
    "row": 1,
    "column": 2,
    "heading": "right"
  }},
  "cells": [
{rows}
  ]
}}
"#
        );
        assert_eq!(to_json(&Grid::unknown(2, 3), start).unwrap(), expected);
    }

    #[test]
    fn ignores_a_leading_byte_order_mark() {
        let json = format!("\u{feff}{}", to_json(&Grid::unknown(SIZE, SIZE), start()).unwrap());
        assert!(from_json(&json).is_ok());
    }

    #[test]
    fn rejects_unsupported_format_versions() {
        let mut document = saved_document();
        document["formatVersion"] = json!(FORMAT_VERSION + 1);
        let expected = format!("version {}", FORMAT_VERSION + 1);
        assert!(load_error(&document).contains(&expected));
    }

    #[test]
    fn rejects_maps_whose_cells_do_not_match_their_size() {
        let mut extra_row = saved_document();
        extra_row["rows"] = json!(SIZE + 1);
        assert!(load_error(&extra_row).contains("rows"));

        let mut short_row = saved_document();
        short_row["cells"][3] = json!(["free"]);
        assert!(load_error(&short_row).contains("cells[3]"));
    }

    #[test]
    fn rejects_maps_that_are_too_small_or_too_large() {
        let tiny = json!({ "formatVersion": 1, "rows": 1, "columns": 1, "cells": [["free"]] });
        assert!(load_error(&tiny).contains("between"));

        let wide = MAX_GRID_SIZE + 1;
        let cells = vec![vec!["free"; wide]; MIN_GRID_SIZE];
        let too_wide =
            json!({ "formatVersion": 1, "rows": MIN_GRID_SIZE, "columns": wide, "cells": cells });
        assert!(load_error(&too_wide).contains("between"));
    }

    #[test]
    fn rejects_a_car_start_outside_the_map() {
        let mut document = saved_document();
        document["start"]["column"] = json!(SIZE);
        assert!(load_error(&document).contains("outside"));

        let mut heading = saved_document();
        heading["start"]["heading"] = json!("north");
        assert!(load_error(&heading).contains("north"));
    }

    #[test]
    fn rejects_unknown_cells_and_properties() {
        let mut unknown_cell = saved_document();
        unknown_cell["cells"][0][0] = json!("tree");
        assert!(load_error(&unknown_cell).contains("tree"));

        let mut unknown_property = saved_document();
        unknown_property["name"] = json!("Backyard");
        assert!(load_error(&unknown_property).contains("name"));
    }

    #[test]
    fn schema_describes_the_saved_format() {
        let schema: Value = serde_json::from_str(SCHEMA).unwrap();
        assert_eq!(schema["$id"], SCHEMA_URL);
        assert_eq!(
            schema["properties"]["formatVersion"]["enum"],
            json!(SUPPORTED_VERSIONS)
        );
        assert_eq!(
            schema["$defs"]["cell"]["enum"],
            serde_json::to_value([Cell::Free, Cell::Obstacle, Cell::Unknown]).unwrap()
        );
        let headings = [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ];
        assert_eq!(
            schema["$defs"]["start"]["properties"]["heading"]["enum"],
            serde_json::to_value(headings).unwrap()
        );
        for size in ["rows", "columns"] {
            assert_eq!(schema["properties"][size]["minimum"], MIN_GRID_SIZE);
            assert_eq!(schema["properties"][size]["maximum"], MAX_GRID_SIZE);
        }

        let document = saved_document();
        let mut written = keys(&document);
        assert_eq!(written, keys(&schema["properties"]));
        assert_eq!(
            keys(&document["start"]),
            keys(&schema["$defs"]["start"]["properties"])
        );
        let required = |object: &Value| {
            object["required"]
                .as_array()
                .unwrap()
                .iter()
                .map(|key| key.as_str().unwrap().to_owned())
                .collect::<BTreeSet<_>>()
        };
        // `$schema` and `start` are optional, so version 1 files stay valid.
        written.remove("$schema");
        written.remove("start");
        assert_eq!(required(&schema), written);
        assert_eq!(required(&schema["$defs"]["start"]), keys(&document["start"]));
    }

    fn saved_document() -> Value {
        serde_json::from_str(&to_json(&Grid::unknown(SIZE, SIZE), start()).unwrap()).unwrap()
    }

    fn keys(object: &Value) -> BTreeSet<String> {
        object.as_object().unwrap().keys().cloned().collect()
    }

    fn load_error(document: &Value) -> String {
        match from_json(&document.to_string()) {
            Ok(_) => panic!("the map should have been rejected"),
            Err(error) => error,
        }
    }
}
