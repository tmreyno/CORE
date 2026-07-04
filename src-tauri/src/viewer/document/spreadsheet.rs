// =============================================================================
// CORE-FFX - Forensic File Explorer
// Spreadsheet Viewer - Excel/CSV/ODS parsing for forensic analysis
// =============================================================================

use calamine::{open_workbook, open_workbook_auto_from_rs, Data, Ods, Reader, Xls, Xlsx};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Seek};
use std::path::Path;

use super::error::{DocumentError, DocumentResult};

const MAX_SPREADSHEET_ROWS_PER_READ: usize = 10_000;
const MAX_SPREADSHEET_COLUMNS_PER_ROW: usize = 1_024;
const MAX_SPREADSHEET_CELL_CHARS: usize = 4_096;

/// Cell value from spreadsheet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CellValue {
    Empty,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(String),
    Error(String),
}

impl From<&Data> for CellValue {
    fn from(data: &Data) -> Self {
        match data {
            Data::Empty => CellValue::Empty,
            Data::String(s) => CellValue::String(truncate_cell_text(s)),
            Data::Int(i) => CellValue::Int(*i),
            Data::Float(f) => CellValue::Float(*f),
            Data::Bool(b) => CellValue::Bool(*b),
            Data::DateTime(dt) => CellValue::DateTime(format!("{}", dt)),
            Data::DateTimeIso(s) => CellValue::DateTime(truncate_cell_text(s)),
            Data::DurationIso(s) => CellValue::String(truncate_cell_text(s)),
            Data::Error(e) => CellValue::Error(format!("{:?}", e)),
        }
    }
}

/// Sheet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetInfo {
    pub name: String,
    pub row_count: usize,
    pub col_count: usize,
}

/// Spreadsheet information (read-only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetInfo {
    pub path: String,
    pub format: String,
    pub sheets: Vec<SheetInfo>,
    pub total_sheets: usize,
}

fn extension_from_source_id(source_id: &str) -> String {
    Path::new(source_id)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn normalized_row_limit(max_rows: usize) -> usize {
    max_rows.clamp(1, MAX_SPREADSHEET_ROWS_PER_READ)
}

fn end_row_for_read(start_row: usize, max_rows: usize) -> usize {
    start_row.saturating_add(normalized_row_limit(max_rows))
}

fn truncate_cell_text(value: &str) -> String {
    if value.chars().count() <= MAX_SPREADSHEET_CELL_CHARS {
        return value.to_string();
    }

    let mut truncated: String = value.chars().take(MAX_SPREADSHEET_CELL_CHARS).collect();
    truncated.push_str("...");
    truncated
}

/// Read spreadsheet metadata
pub fn read_spreadsheet_info(path: impl AsRef<Path>) -> DocumentResult<SpreadsheetInfo> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let (sheets, format) = match ext.as_str() {
        "xlsx" | "xlsm" | "xlsb" => {
            let mut workbook: Xlsx<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open XLSX: {}", e)))?;
            let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
            let sheets: Vec<SheetInfo> = sheet_names
                .iter()
                .map(|name| {
                    let (row_count, col_count) = workbook
                        .worksheet_range(name)
                        .ok()
                        .map(|range| {
                            let (rows, cols) = range.get_size();
                            (rows, cols)
                        })
                        .unwrap_or((0, 0));
                    SheetInfo {
                        name: name.clone(),
                        row_count,
                        col_count,
                    }
                })
                .collect();
            (sheets, "xlsx".to_string())
        }
        "xls" => {
            let mut workbook: Xls<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open XLS: {}", e)))?;
            let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
            let sheets: Vec<SheetInfo> = sheet_names
                .iter()
                .map(|name| {
                    let (row_count, col_count) = workbook
                        .worksheet_range(name)
                        .ok()
                        .map(|range| {
                            let (rows, cols) = range.get_size();
                            (rows, cols)
                        })
                        .unwrap_or((0, 0));
                    SheetInfo {
                        name: name.clone(),
                        row_count,
                        col_count,
                    }
                })
                .collect();
            (sheets, "xls".to_string())
        }
        "ods" => {
            let mut workbook: Ods<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open ODS: {}", e)))?;
            let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
            let sheets: Vec<SheetInfo> = sheet_names
                .iter()
                .map(|name| {
                    let (row_count, col_count) = workbook
                        .worksheet_range(name)
                        .ok()
                        .map(|range| {
                            let (rows, cols) = range.get_size();
                            (rows, cols)
                        })
                        .unwrap_or((0, 0));
                    SheetInfo {
                        name: name.clone(),
                        row_count,
                        col_count,
                    }
                })
                .collect();
            (sheets, "ods".to_string())
        }
        "csv" | "tsv" => {
            // CSV/TSV files are single-sheet; count rows and columns
            let is_tsv = ext == "tsv";
            let delimiter = if is_tsv { b'\t' } else { b',' };
            let (row_count, col_count) = match csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .from_path(path)
            {
                Ok(mut reader) => {
                    let mut rows = 0usize;
                    let mut max_cols = 0usize;
                    for record in reader.records().flatten() {
                        rows += 1;
                        max_cols = max_cols.max(record.len());
                    }
                    (rows, max_cols)
                }
                Err(_) => (0, 0),
            };
            let sheets = vec![SheetInfo {
                name: "Sheet1".to_string(),
                row_count,
                col_count,
            }];
            (sheets, ext.clone())
        }
        _ => return Err(DocumentError::UnsupportedFormat(ext)),
    };

    let total_sheets = sheets.len();

    Ok(SpreadsheetInfo {
        path: path.to_string_lossy().to_string(),
        format,
        sheets,
        total_sheets,
    })
}

/// Read spreadsheet metadata from bytes read from any evidence source.
pub fn read_spreadsheet_info_bytes(
    source_id: impl Into<String>,
    data: &[u8],
) -> DocumentResult<SpreadsheetInfo> {
    let source_id = source_id.into();
    let ext = extension_from_source_id(&source_id);

    let (sheets, format) = match ext.as_str() {
        "csv" | "tsv" => {
            let is_tsv = ext == "tsv";
            let delimiter = if is_tsv { b'\t' } else { b',' };
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(delimiter)
                .from_reader(Cursor::new(data));
            let mut rows = 0usize;
            let mut max_cols = 0usize;
            for record in reader.records().flatten() {
                rows += 1;
                max_cols = max_cols.max(record.len());
            }
            (
                vec![SheetInfo {
                    name: "Sheet1".to_string(),
                    row_count: rows,
                    col_count: max_cols,
                }],
                ext.clone(),
            )
        }
        _ => {
            let mut workbook = open_workbook_auto_from_rs(Cursor::new(data.to_vec()))
                .map_err(|e| DocumentError::Parse(format!("Failed to open spreadsheet: {}", e)))?;
            let sheets = sheet_infos_from_workbook(&mut workbook);
            (
                sheets,
                if ext.is_empty() {
                    "spreadsheet".to_string()
                } else {
                    ext
                },
            )
        }
    };

    let total_sheets = sheets.len();
    Ok(SpreadsheetInfo {
        path: source_id,
        format,
        sheets,
        total_sheets,
    })
}

fn sheet_infos_from_workbook<RS, R>(workbook: &mut R) -> Vec<SheetInfo>
where
    RS: Read + Seek,
    R: Reader<RS>,
{
    let sheet_names: Vec<String> = workbook.sheet_names().to_vec();
    sheet_names
        .iter()
        .map(|name| {
            let (row_count, col_count) = workbook
                .worksheet_range(name)
                .ok()
                .map(|range| {
                    let (rows, cols) = range.get_size();
                    (rows, cols)
                })
                .unwrap_or((0, 0));
            SheetInfo {
                name: name.clone(),
                row_count,
                col_count,
            }
        })
        .collect()
}

/// Read a range from a sheet in an XLSX file
pub fn read_xlsx_sheet_range(
    path: impl AsRef<Path>,
    sheet_name: &str,
    start_row: usize,
    end_row: usize,
) -> DocumentResult<Vec<Vec<CellValue>>> {
    let path = path.as_ref();
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|e| DocumentError::Parse(format!("Failed to open XLSX: {}", e)))?;

    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| DocumentError::Parse(format!("Failed to read sheet: {}", e)))?;

    let mut rows = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx < start_row {
            continue;
        }
        if row_idx >= end_row {
            break;
        }
        let cells: Vec<CellValue> = row
            .iter()
            .take(MAX_SPREADSHEET_COLUMNS_PER_ROW)
            .map(CellValue::from)
            .collect();
        rows.push(cells);
    }

    Ok(rows)
}

/// Read a sheet from any supported spreadsheet format
pub fn read_sheet(
    path: impl AsRef<Path>,
    sheet_name: &str,
    start_row: usize,
    max_rows: usize,
) -> DocumentResult<Vec<Vec<CellValue>>> {
    let path = path.as_ref();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let max_rows = normalized_row_limit(max_rows);
    let end_row = end_row_for_read(start_row, max_rows);

    match ext.as_str() {
        "xlsx" | "xlsm" | "xlsb" => {
            let mut workbook: Xlsx<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open XLSX: {}", e)))?;
            read_range_from_workbook(&mut workbook, sheet_name, start_row, end_row)
        }
        "xls" => {
            let mut workbook: Xls<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open XLS: {}", e)))?;
            read_range_from_workbook(&mut workbook, sheet_name, start_row, end_row)
        }
        "ods" => {
            let mut workbook: Ods<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open ODS: {}", e)))?;
            read_range_from_workbook(&mut workbook, sheet_name, start_row, end_row)
        }
        "csv" | "tsv" => read_csv_as_cells(path, start_row, max_rows),
        _ => Err(DocumentError::UnsupportedFormat(ext)),
    }
}

/// Read a sheet from spreadsheet bytes read from any evidence source.
pub fn read_sheet_bytes(
    source_id: impl Into<String>,
    data: &[u8],
    sheet_name: &str,
    start_row: usize,
    max_rows: usize,
) -> DocumentResult<Vec<Vec<CellValue>>> {
    let source_id = source_id.into();
    let ext = extension_from_source_id(&source_id);
    let max_rows = normalized_row_limit(max_rows);
    let end_row = end_row_for_read(start_row, max_rows);

    match ext.as_str() {
        "csv" | "tsv" => read_csv_bytes_as_cells(data, ext == "tsv", start_row, max_rows),
        _ => {
            let mut workbook = open_workbook_auto_from_rs(Cursor::new(data.to_vec()))
                .map_err(|e| DocumentError::Parse(format!("Failed to open spreadsheet: {}", e)))?;
            read_range_from_workbook(&mut workbook, sheet_name, start_row, end_row)
        }
    }
}

/// Helper to read range from any workbook type
fn read_range_from_workbook<RS, R>(
    workbook: &mut R,
    sheet_name: &str,
    start_row: usize,
    end_row: usize,
) -> DocumentResult<Vec<Vec<CellValue>>>
where
    RS: Read + Seek,
    R: Reader<RS>,
{
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| DocumentError::Parse(format!("Failed to read sheet: {:?}", e)))?;

    let mut rows = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx < start_row {
            continue;
        }
        if row_idx >= end_row {
            break;
        }
        let cells: Vec<CellValue> = row
            .iter()
            .take(MAX_SPREADSHEET_COLUMNS_PER_ROW)
            .map(CellValue::from)
            .collect();
        rows.push(cells);
    }

    Ok(rows)
}

fn read_csv_bytes_as_cells(
    data: &[u8],
    is_tsv: bool,
    start_row: usize,
    max_rows: usize,
) -> DocumentResult<Vec<Vec<CellValue>>> {
    let delimiter = if is_tsv { b'\t' } else { b',' };
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimiter)
        .from_reader(Cursor::new(data));

    let mut rows = Vec::new();
    for (idx, result) in reader.records().enumerate() {
        if idx < start_row {
            continue;
        }
        if rows.len() >= max_rows {
            break;
        }
        let record = result.map_err(|e| DocumentError::Parse(format!("CSV parse error: {}", e)))?;
        let row: Vec<CellValue> = record
            .iter()
            .take(MAX_SPREADSHEET_COLUMNS_PER_ROW)
            .map(parse_csv_cell)
            .collect();
        rows.push(row);
    }
    Ok(rows)
}

fn parse_csv_cell(s: &str) -> CellValue {
    if let Ok(i) = s.parse::<i64>() {
        CellValue::Int(i)
    } else if let Ok(f) = s.parse::<f64>() {
        CellValue::Float(f)
    } else if s.eq_ignore_ascii_case("true") {
        CellValue::Bool(true)
    } else if s.eq_ignore_ascii_case("false") {
        CellValue::Bool(false)
    } else if s.is_empty() {
        CellValue::Empty
    } else {
        CellValue::String(truncate_cell_text(s))
    }
}

/// Read CSV/TSV as CellValue vectors
fn read_csv_as_cells(
    path: impl AsRef<Path>,
    start_row: usize,
    max_rows: usize,
) -> DocumentResult<Vec<Vec<CellValue>>> {
    let path = path.as_ref();
    let is_tsv = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("tsv"))
        .unwrap_or(false);
    let delimiter = if is_tsv { b'\t' } else { b',' };
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // Include header row as first row
        .delimiter(delimiter)
        .from_path(path)
        .map_err(|e| {
            DocumentError::Parse(format!(
                "Failed to open {}: {}",
                if is_tsv { "TSV" } else { "CSV" },
                e
            ))
        })?;

    let mut rows = Vec::new();

    for (idx, result) in reader.records().enumerate() {
        if idx < start_row {
            continue;
        }
        if rows.len() >= max_rows {
            break;
        }

        let record = result.map_err(|e| DocumentError::Parse(format!("CSV parse error: {}", e)))?;
        let row: Vec<CellValue> = record
            .iter()
            .take(MAX_SPREADSHEET_COLUMNS_PER_ROW)
            .map(parse_csv_cell)
            .collect();
        rows.push(row);
    }

    Ok(rows)
}

/// CSV reading - simple text-based parsing (legacy)
pub fn read_csv(
    path: impl AsRef<Path>,
    max_rows: Option<usize>,
) -> DocumentResult<(Vec<String>, Vec<Vec<String>>)> {
    let path = path.as_ref();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| DocumentError::Parse(format!("Failed to open CSV: {}", e)))?;

    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| DocumentError::Parse(format!("Failed to read CSV headers: {}", e)))?
        .iter()
        .take(MAX_SPREADSHEET_COLUMNS_PER_ROW)
        .map(truncate_cell_text)
        .collect();

    let max = normalized_row_limit(max_rows.unwrap_or(1000));
    let mut rows = Vec::new();

    for (idx, result) in reader.records().enumerate() {
        if idx >= max {
            break;
        }
        let record = result.map_err(|e| DocumentError::Parse(format!("CSV parse error: {}", e)))?;
        let row: Vec<String> = record
            .iter()
            .take(MAX_SPREADSHEET_COLUMNS_PER_ROW)
            .map(truncate_cell_text)
            .collect();
        rows.push(row);
    }

    Ok((headers, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_value_conversion() {
        let empty = CellValue::from(&Data::Empty);
        assert!(matches!(empty, CellValue::Empty));

        let string = CellValue::from(&Data::String("test".to_string()));
        assert!(matches!(string, CellValue::String(s) if s == "test"));
    }

    #[test]
    fn normalized_row_limit_clamps_bounds() {
        assert_eq!(normalized_row_limit(0), 1);
        assert_eq!(normalized_row_limit(500), 500);
        assert_eq!(
            normalized_row_limit(usize::MAX),
            MAX_SPREADSHEET_ROWS_PER_READ
        );
    }

    #[test]
    fn end_row_for_read_saturates_overflow() {
        assert_eq!(end_row_for_read(10, 5), 15);
        assert_eq!(end_row_for_read(usize::MAX - 1, 10), usize::MAX);
    }

    #[test]
    fn truncate_cell_text_preserves_short_multibyte_values() {
        let value = "é".repeat(MAX_SPREADSHEET_CELL_CHARS);

        assert_eq!(truncate_cell_text(&value), value);
    }

    #[test]
    fn truncate_cell_text_caps_long_multibyte_values() {
        let value = "é".repeat(MAX_SPREADSHEET_CELL_CHARS + 1);

        let truncated = truncate_cell_text(&value);

        assert!(truncated.ends_with("..."));
        assert_eq!(
            truncated.trim_end_matches("...").chars().count(),
            MAX_SPREADSHEET_CELL_CHARS
        );
    }

    #[test]
    fn read_sheet_bytes_clamps_zero_max_rows_to_one() {
        let data = b"name,count\nalpha,10\nbeta,3\n";

        let rows =
            read_sheet_bytes("container.ad1:tables/items.csv", data, "Sheet1", 1, 0).unwrap();

        assert_eq!(rows.len(), 1);
        assert!(matches!(&rows[0][0], CellValue::String(value) if value == "alpha"));
    }

    #[test]
    fn read_sheet_bytes_handles_extreme_start_row_without_overflow() {
        let data = b"name,count\nalpha,10\n";

        let rows = read_sheet_bytes(
            "container.ad1:tables/items.csv",
            data,
            "Sheet1",
            usize::MAX,
            usize::MAX,
        )
        .unwrap();

        assert!(rows.is_empty());
    }

    #[test]
    fn read_spreadsheet_info_bytes_reads_csv_source_metadata() {
        let data = b"name,count,active\nalpha,10,true\nbeta,3,false\n";

        let info = read_spreadsheet_info_bytes("container.ad1:tables/items.csv", data).unwrap();

        assert_eq!(info.path, "container.ad1:tables/items.csv");
        assert_eq!(info.format, "csv");
        assert_eq!(info.total_sheets, 1);
        assert_eq!(info.sheets[0].row_count, 3);
        assert_eq!(info.sheets[0].col_count, 3);
    }

    #[test]
    fn read_sheet_bytes_reads_csv_cells_from_source() {
        let data = b"name,count,active\nalpha,10,true\nbeta,3,false\n";

        let rows =
            read_sheet_bytes("container.ad1:tables/items.csv", data, "Sheet1", 1, 2).unwrap();

        assert_eq!(rows.len(), 2);
        assert!(matches!(&rows[0][0], CellValue::String(value) if value == "alpha"));
        assert!(matches!(rows[0][1], CellValue::Int(10)));
        assert!(matches!(rows[0][2], CellValue::Bool(true)));
    }

    #[test]
    fn read_sheet_bytes_caps_wide_csv_rows() {
        let mut header = String::new();
        for index in 0..(MAX_SPREADSHEET_COLUMNS_PER_ROW + 16) {
            if index > 0 {
                header.push(',');
            }
            header.push_str(&format!("c{index}"));
        }
        header.push('\n');

        let rows =
            read_sheet_bytes("container.ad1:wide.csv", header.as_bytes(), "Sheet1", 0, 1).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].len(), MAX_SPREADSHEET_COLUMNS_PER_ROW);
    }

    #[test]
    fn read_sheet_bytes_truncates_long_csv_cells() {
        let cell = "é".repeat(MAX_SPREADSHEET_CELL_CHARS + 1);
        let data = format!("name\n{cell}\n");

        let rows =
            read_sheet_bytes("container.ad1:long.csv", data.as_bytes(), "Sheet1", 1, 1).unwrap();

        match &rows[0][0] {
            CellValue::String(value) => {
                assert!(value.ends_with("..."));
                assert_eq!(
                    value.trim_end_matches("...").chars().count(),
                    MAX_SPREADSHEET_CELL_CHARS
                );
            }
            other => panic!("expected truncated string cell, got {other:?}"),
        }
    }
}
