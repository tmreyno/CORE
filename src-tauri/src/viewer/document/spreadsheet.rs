// =============================================================================
// CORE-FFX - Forensic File Explorer
// Spreadsheet Viewer - Excel/CSV/ODS parsing for forensic analysis
// =============================================================================

use serde::{Deserialize, Serialize};
use std::path::Path;
use calamine::{Reader, Xlsx, Xls, Ods, open_workbook, Data};

use super::error::{DocumentError, DocumentResult};

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
            Data::String(s) => CellValue::String(s.clone()),
            Data::Int(i) => CellValue::Int(*i),
            Data::Float(f) => CellValue::Float(*f),
            Data::Bool(b) => CellValue::Bool(*b),
            Data::DateTime(dt) => CellValue::DateTime(format!("{}", dt)),
            Data::DateTimeIso(s) => CellValue::DateTime(s.clone()),
            Data::DurationIso(s) => CellValue::String(s.clone()),
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

/// Read spreadsheet metadata
pub fn read_spreadsheet_info(path: impl AsRef<Path>) -> DocumentResult<SpreadsheetInfo> {
    let path = path.as_ref();
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let (sheets, format) = match ext.as_str() {
        "xlsx" | "xlsm" | "xlsb" => {
            let workbook: Xlsx<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open XLSX: {}", e)))?;
            let sheets: Vec<SheetInfo> = workbook.sheet_names()
                .iter()
                .map(|name| SheetInfo { name: name.clone(), row_count: 0, col_count: 0 })
                .collect();
            (sheets, "xlsx".to_string())
        }
        "xls" => {
            let workbook: Xls<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open XLS: {}", e)))?;
            let sheets: Vec<SheetInfo> = workbook.sheet_names()
                .iter()
                .map(|name| SheetInfo { name: name.clone(), row_count: 0, col_count: 0 })
                .collect();
            (sheets, "xls".to_string())
        }
        "ods" => {
            let workbook: Ods<_> = open_workbook(path)
                .map_err(|e| DocumentError::Parse(format!("Failed to open ODS: {}", e)))?;
            let sheets: Vec<SheetInfo> = workbook.sheet_names()
                .iter()
                .map(|name| SheetInfo { name: name.clone(), row_count: 0, col_count: 0 })
                .collect();
            (sheets, "ods".to_string())
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
    
    let range = workbook.worksheet_range(sheet_name)
        .map_err(|e| DocumentError::Parse(format!("Failed to read sheet: {}", e)))?;
    
    let mut rows = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx < start_row { continue; }
        if row_idx >= end_row { break; }
        let cells: Vec<CellValue> = row.iter().map(CellValue::from).collect();
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
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let end_row = start_row + max_rows;
    
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
        "csv" | "tsv" => {
            read_csv_as_cells(path, start_row, max_rows)
        }
        _ => Err(DocumentError::UnsupportedFormat(ext)),
    }
}

/// Helper to read range from any workbook type
fn read_range_from_workbook<R: Reader<std::io::BufReader<std::fs::File>>>(
    workbook: &mut R,
    sheet_name: &str,
    start_row: usize,
    end_row: usize,
) -> DocumentResult<Vec<Vec<CellValue>>> {
    let range = workbook.worksheet_range(sheet_name)
        .map_err(|e| DocumentError::Parse(format!("Failed to read sheet: {:?}", e)))?;
    
    let mut rows = Vec::new();
    for (row_idx, row) in range.rows().enumerate() {
        if row_idx < start_row { continue; }
        if row_idx >= end_row { break; }
        let cells: Vec<CellValue> = row.iter().map(CellValue::from).collect();
        rows.push(cells);
    }
    
    Ok(rows)
}

/// Read CSV as CellValue vectors
fn read_csv_as_cells(
    path: impl AsRef<Path>,
    start_row: usize,
    max_rows: usize,
) -> DocumentResult<Vec<Vec<CellValue>>> {
    let path = path.as_ref();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // Include header row as first row
        .from_path(path)
        .map_err(|e| DocumentError::Parse(format!("Failed to open CSV: {}", e)))?;
    
    let mut rows = Vec::new();
    
    for (idx, result) in reader.records().enumerate() {
        if idx < start_row { continue; }
        if rows.len() >= max_rows { break; }
        
        let record = result
            .map_err(|e| DocumentError::Parse(format!("CSV parse error: {}", e)))?;
        let row: Vec<CellValue> = record.iter()
            .map(|s| {
                // Try to parse as number
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
                    CellValue::String(s.to_string())
                }
            })
            .collect();
        rows.push(row);
    }
    
    Ok(rows)
}

/// CSV reading - simple text-based parsing (legacy)
pub fn read_csv(path: impl AsRef<Path>, max_rows: Option<usize>) -> DocumentResult<(Vec<String>, Vec<Vec<String>>)> {
    let path = path.as_ref();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .map_err(|e| DocumentError::Parse(format!("Failed to open CSV: {}", e)))?;
    
    let headers: Vec<String> = reader.headers()
        .map_err(|e| DocumentError::Parse(format!("Failed to read CSV headers: {}", e)))?
        .iter()
        .map(|s| s.to_string())
        .collect();
    
    let max = max_rows.unwrap_or(1000);
    let mut rows = Vec::new();
    
    for (idx, result) in reader.records().enumerate() {
        if idx >= max { break; }
        let record = result
            .map_err(|e| DocumentError::Parse(format!("CSV parse error: {}", e)))?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
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
}
