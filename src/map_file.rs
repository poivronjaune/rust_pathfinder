//! Map files (`*.map`): JSON documents described by `schemas/map.schema.json`.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize, Serializer};

use crate::config::GRID_SIZE;
use crate::grid::{Cell, Grid};

/// `$id` of `schemas/map.schema.json`, written into saved maps so editors can validate them.
const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/poivronjaune/rust_pathfinder/main/schemas/map.schema.json";
const FORMAT_VERSION: u32 = 1;
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
    #[serde(serialize_with = "serialize_rows")]
    cells: Vec<Vec<Cell>>,
}

/// Reads only the format version, so maps saved in a newer format are reported as such instead
/// of failing on fields this version does not know.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionProbe {
    format_version: u32,
}

pub(crate) fn save(grid: &Grid, path: &Path) -> Result<(), String> {
    let result = match to_json(grid) {
        Ok(json) => fs::write(path, json).map_err(|error| error.to_string()),
        Err(error) => Err(error.to_string()),
    };
    result.map_err(|error| format!("Unable to save the map to {}: {error}", path.display()))
}

pub(crate) fn load(path: &Path) -> Result<Grid, String> {
    let result = match fs::read_to_string(path) {
        Ok(json) => from_json(&json),
        Err(error) => Err(error.to_string()),
    };
    result.map_err(|error| format!("Unable to load the map from {}: {error}", path.display()))
}

fn to_json(grid: &Grid) -> serde_json::Result<String> {
    let document = MapDocument {
        schema: Some(SCHEMA_URL.to_owned()),
        format_version: FORMAT_VERSION,
        rows: GRID_SIZE,
        columns: GRID_SIZE,
        cells: grid.cells.iter().map(|row| row.to_vec()).collect(),
    };
    let mut json = serde_json::to_string_pretty(&document)?;
    json.push('\n');
    Ok(json)
}

fn from_json(json: &str) -> Result<Grid, String> {
    // RFC 8259 lets parsers ignore a byte order mark, which some Windows editors add.
    let json = json.strip_prefix('\u{feff}').unwrap_or(json);
    let probe: VersionProbe = parse(json)?;
    if probe.format_version != FORMAT_VERSION {
        return Err(format!(
            "map format version {} is not supported (expected version {FORMAT_VERSION})",
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
    if (document.rows, document.columns) != (GRID_SIZE, GRID_SIZE) {
        return Err(format!(
            "the map is {} x {} but only {GRID_SIZE} x {GRID_SIZE} maps are supported",
            document.rows, document.columns
        ));
    }

    let mut grid = Grid::unknown();
    for (row, cells) in grid.cells.iter_mut().zip(&document.cells) {
        row.copy_from_slice(cells);
    }
    Ok(grid)
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

    use super::{DEFAULT_FILE_NAME, FILE_EXTENSION, FORMAT_VERSION, SCHEMA_URL, from_json, to_json};
    use crate::config::GRID_SIZE;
    use crate::grid::{Cell, Grid};

    const SCHEMA: &str = include_str!("../schemas/map.schema.json");

    #[test]
    fn saved_json_loads_back_unchanged() {
        let grid = Grid::backyard_with_seed(GRID_SIZE - 1, GRID_SIZE / 2, 7);
        let loaded = from_json(&to_json(&grid).unwrap()).unwrap();
        assert_eq!(loaded.cells, grid.cells);
    }

    #[test]
    fn default_file_name_uses_the_map_extension() {
        let extension = Path::new(DEFAULT_FILE_NAME).extension();
        assert_eq!(extension, Some(OsStr::new(FILE_EXTENSION)));
    }

    #[test]
    fn saved_json_puts_each_row_on_its_own_line() {
        let row = format!("    [{}]", vec!["\"unknown\""; GRID_SIZE].join(","));
        let rows = vec![row; GRID_SIZE].join(",\n");
        let expected = format!(
            r#"{{
  "$schema": "{SCHEMA_URL}",
  "formatVersion": {FORMAT_VERSION},
  "rows": {GRID_SIZE},
  "columns": {GRID_SIZE},
  "cells": [
{rows}
  ]
}}
"#
        );
        assert_eq!(to_json(&Grid::unknown()).unwrap(), expected);
    }

    #[test]
    fn ignores_a_leading_byte_order_mark() {
        let json = format!("\u{feff}{}", to_json(&Grid::unknown()).unwrap());
        assert!(from_json(&json).is_ok());
    }

    #[test]
    fn rejects_unsupported_format_versions() {
        let mut document = saved_document();
        document["formatVersion"] = json!(2);
        assert!(load_error(&document).contains("version 2"));
    }

    #[test]
    fn rejects_maps_whose_cells_do_not_match_their_size() {
        let mut extra_row = saved_document();
        extra_row["rows"] = json!(GRID_SIZE + 1);
        assert!(load_error(&extra_row).contains("rows"));

        let mut short_row = saved_document();
        short_row["cells"][3] = json!(["free"]);
        assert!(load_error(&short_row).contains("cells[3]"));

        let small_map = json!({ "formatVersion": 1, "rows": 1, "columns": 1, "cells": [["free"]] });
        assert!(load_error(&small_map).contains("supported"));
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
            schema["properties"]["formatVersion"]["const"],
            FORMAT_VERSION
        );
        assert_eq!(
            schema["$defs"]["cell"]["enum"],
            serde_json::to_value([Cell::Free, Cell::Obstacle, Cell::Unknown]).unwrap()
        );

        let mut written = keys(&saved_document());
        assert_eq!(written, keys(&schema["properties"]));
        let required = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|key| key.as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        written.remove("$schema");
        assert_eq!(required, written);
    }

    fn saved_document() -> Value {
        serde_json::from_str(&to_json(&Grid::unknown()).unwrap()).unwrap()
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
