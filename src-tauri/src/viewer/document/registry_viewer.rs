// =============================================================================
// CORE-FFX - Forensic File Explorer
// Registry Viewer - Windows Registry hive parsing for forensic analysis
// =============================================================================

use notatin::cell_key_value::CellKeyValueDataTypes;
use notatin::cell_value::CellValue;
use notatin::parser::ParserIterator;
use notatin::parser_builder::ParserBuilder;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::error::{DocumentError, DocumentResult};

// =============================================================================
// Types
// =============================================================================

/// Registry hive overview information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryHiveInfo {
    pub path: String,
    pub root_key_name: String,
    pub root_key_path: String,
    pub root_timestamp: Option<String>,
    pub total_keys: usize,
    pub total_values: usize,
    pub root_subkey_count: u32,
    pub root_value_count: u32,
    /// True when key/value counts were capped due to hive size limits
    pub capped: bool,
}

/// A registry key (node) with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryKey {
    pub name: String,
    pub path: String,
    pub timestamp: Option<String>,
    pub subkey_count: u32,
    pub value_count: u32,
    pub has_subkeys: bool,
}

/// A registry value with its data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryValue {
    pub name: String,
    pub data_type: String,
    pub data: String,
    pub size: usize,
}

/// Subkeys response for a given key path
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrySubkeysResponse {
    pub parent_path: String,
    pub subkeys: Vec<RegistryKey>,
}

/// Values response for a given key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryValuesResponse {
    pub key_path: String,
    pub values: Vec<RegistryValue>,
}

/// Detailed key info (used for inspector panel)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryKeyInfo {
    pub name: String,
    pub path: String,
    pub pretty_path: String,
    pub timestamp: Option<String>,
    pub subkey_count: u32,
    pub value_count: u32,
    pub values: Vec<RegistryValue>,
    pub subkeys: Vec<RegistryKey>,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Format a CellKeyValueDataTypes variant to a human-readable string
fn data_type_name(dt: &CellKeyValueDataTypes) -> &'static str {
    match dt {
        CellKeyValueDataTypes::REG_NONE => "REG_NONE",
        CellKeyValueDataTypes::REG_SZ => "REG_SZ",
        CellKeyValueDataTypes::REG_EXPAND_SZ => "REG_EXPAND_SZ",
        CellKeyValueDataTypes::REG_BIN => "REG_BINARY",
        CellKeyValueDataTypes::REG_DWORD => "REG_DWORD",
        CellKeyValueDataTypes::REG_DWORD_BIG_ENDIAN => "REG_DWORD_BIG_ENDIAN",
        CellKeyValueDataTypes::REG_LINK => "REG_LINK",
        CellKeyValueDataTypes::REG_MULTI_SZ => "REG_MULTI_SZ",
        CellKeyValueDataTypes::REG_RESOURCE_LIST => "REG_RESOURCE_LIST",
        CellKeyValueDataTypes::REG_FULL_RESOURCE_DESCRIPTOR => "REG_FULL_RESOURCE_DESCRIPTOR",
        CellKeyValueDataTypes::REG_RESOURCE_REQUIREMENTS_LIST => "REG_RESOURCE_REQUIREMENTS_LIST",
        CellKeyValueDataTypes::REG_QWORD => "REG_QWORD",
        CellKeyValueDataTypes::REG_FILETIME => "REG_FILETIME",
        _ => "REG_UNKNOWN",
    }
}

/// Format a CellValue to a display string
fn format_cell_value(value: &CellValue) -> String {
    match value {
        CellValue::String(s) => s.clone(),
        CellValue::U32(d) => format!("0x{:08x} ({})", d, d),
        CellValue::I32(d) => format!("{}", d),
        CellValue::U64(q) => format!("0x{:016x} ({})", q, q),
        CellValue::I64(q) => format!("{}", q),
        CellValue::MultiString(ms) => ms.join("; "),
        CellValue::Binary(b) => {
            let hex: String = b
                .iter()
                .take(64)
                .map(|byte| format!("{:02x}", byte))
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|pair| pair.join(""))
                .collect::<Vec<_>>()
                .join(" ");
            if b.len() > 64 {
                format!("{} ... ({} bytes)", hex, b.len())
            } else {
                format!("{} ({} bytes)", hex, b.len())
            }
        }
        CellValue::Error => "(error parsing value)".to_string(),
        CellValue::None => "(empty)".to_string(),
    }
}

/// Estimate byte size of a CellValue
fn cell_value_size(value: &CellValue) -> usize {
    match value {
        CellValue::String(s) => s.len() * 2, // UTF-16 stored
        CellValue::U32(_) | CellValue::I32(_) => 4,
        CellValue::U64(_) | CellValue::I64(_) => 8,
        CellValue::MultiString(ms) => ms.iter().map(|s| s.len() * 2 + 2).sum(),
        CellValue::Binary(b) => b.len(),
        CellValue::Error | CellValue::None => 0,
    }
}

/// Format a Windows FILETIME (100-ns intervals since 1601-01-01) as ISO timestamp
/// Used for test validation only
#[cfg(test)]
fn format_timestamp(filetime: u64) -> Option<String> {
    if filetime == 0 {
        return None;
    }
    // Windows FILETIME: 100-nanosecond intervals since 1601-01-01
    // Unix epoch offset: 11644473600 seconds
    let secs_from_1601 = filetime / 10_000_000;
    let unix_secs = secs_from_1601.checked_sub(11_644_473_600)?;
    let dt = chrono::DateTime::from_timestamp(unix_secs as i64, 0)?;
    Some(dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
}

// =============================================================================
// Public API
// =============================================================================

/// Get overview information about a registry hive file
pub fn get_hive_info(path: &str) -> DocumentResult<RegistryHiveInfo> {
    let hive_path = PathBuf::from(path);
    let mut parser = ParserBuilder::from_path(hive_path.clone())
        .build()
        .map_err(|e| DocumentError::Parse(format!("Failed to open registry hive: {}", e)))?;

    let root_key = parser
        .get_root_key()
        .map_err(|e| DocumentError::Parse(format!("Failed to read root key: {}", e)))?
        .ok_or_else(|| DocumentError::Parse("Registry hive has no root key".to_string()))?;

    let root_key_name = root_key.key_name.clone();
    let root_key_path = root_key.path.clone();
    let root_timestamp = {
        let ts = root_key.last_key_written_date_and_time();
        Some(ts.format("%Y-%m-%d %H:%M:%S UTC").to_string())
    };
    let root_subkey_count = root_key.detail.number_of_sub_keys();
    let root_value_count = root_key.detail.number_of_key_values();

    // Count total keys and values by iterating
    let parser_for_iter = ParserBuilder::from_path(hive_path)
        .build()
        .map_err(|e| DocumentError::Parse(format!("Failed to reopen hive for counting: {}", e)))?;

    let mut total_keys: usize = 0;
    let mut total_values: usize = 0;

    let mut capped = false;

    for key in ParserIterator::new(&parser_for_iter).iter() {
        total_keys += 1;
        total_values += key.value_iter().count();
        // Safety: cap at 100K keys to avoid hang on very large hives
        if total_keys > 100_000 {
            capped = true;
            break;
        }
    }

    Ok(RegistryHiveInfo {
        path: path.to_string(),
        root_key_name,
        root_key_path,
        root_timestamp,
        total_keys,
        total_values,
        root_subkey_count,
        root_value_count,
        capped,
    })
}

/// Get the immediate subkeys of a key at the given path
pub fn get_subkeys(hive_path: &str, key_path: &str) -> DocumentResult<RegistrySubkeysResponse> {
    let mut parser = ParserBuilder::from_path(PathBuf::from(hive_path))
        .build()
        .map_err(|e| DocumentError::Parse(format!("Failed to open registry hive: {}", e)))?;

    let key = if key_path.is_empty() || key_path == "\\" || key_path == "/" {
        // Root key
        parser
            .get_root_key()
            .map_err(|e| DocumentError::Parse(format!("Failed to read root key: {}", e)))?
            .ok_or_else(|| DocumentError::Parse("Registry hive has no root key".to_string()))?
    } else {
        // Navigate to the key path
        let clean_path = key_path.trim_start_matches('\\').trim_start_matches('/');
        parser
            .get_key(clean_path, false)
            .map_err(|e| {
                DocumentError::Parse(format!("Failed to navigate to key '{}': {}", key_path, e))
            })?
            .ok_or_else(|| DocumentError::Parse(format!("Key not found: {}", key_path)))?
    };

    let mut subkey = key.clone();
    let sub_keys = subkey.read_sub_keys(&mut parser);

    let subkeys: Vec<RegistryKey> = sub_keys
        .iter()
        .map(|sk| RegistryKey {
            name: sk.key_name.clone(),
            path: sk.path.clone(),
            timestamp: Some(
                sk.last_key_written_date_and_time()
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            ),
            subkey_count: sk.detail.number_of_sub_keys(),
            value_count: sk.detail.number_of_key_values(),
            has_subkeys: sk.detail.number_of_sub_keys() > 0,
        })
        .collect();

    Ok(RegistrySubkeysResponse {
        parent_path: key.path.clone(),
        subkeys,
    })
}

/// Get the values of a key at the given path
pub fn get_values(hive_path: &str, key_path: &str) -> DocumentResult<RegistryValuesResponse> {
    let mut parser = ParserBuilder::from_path(PathBuf::from(hive_path))
        .build()
        .map_err(|e| DocumentError::Parse(format!("Failed to open registry hive: {}", e)))?;

    let key = if key_path.is_empty() || key_path == "\\" || key_path == "/" {
        parser
            .get_root_key()
            .map_err(|e| DocumentError::Parse(format!("Failed to read root key: {}", e)))?
            .ok_or_else(|| DocumentError::Parse("Registry hive has no root key".to_string()))?
    } else {
        let clean_path = key_path.trim_start_matches('\\').trim_start_matches('/');
        parser
            .get_key(clean_path, false)
            .map_err(|e| {
                DocumentError::Parse(format!("Failed to navigate to key '{}': {}", key_path, e))
            })?
            .ok_or_else(|| DocumentError::Parse(format!("Key not found: {}", key_path)))?
    };

    let values: Vec<RegistryValue> = key
        .value_iter()
        .map(|val| {
            let (cell_value, _warnings) = val.get_content();
            let size = cell_value_size(&cell_value);
            RegistryValue {
                name: if val.detail.value_name().is_empty() {
                    "(Default)".to_string()
                } else {
                    val.detail.value_name().to_string()
                },
                data_type: data_type_name(&val.data_type).to_string(),
                data: format_cell_value(&cell_value),
                size,
            }
        })
        .collect();

    Ok(RegistryValuesResponse {
        key_path: key.path.clone(),
        values,
    })
}

/// Get detailed information about a specific key including both subkeys and values
pub fn get_key_info(hive_path: &str, key_path: &str) -> DocumentResult<RegistryKeyInfo> {
    let mut parser = ParserBuilder::from_path(PathBuf::from(hive_path))
        .build()
        .map_err(|e| DocumentError::Parse(format!("Failed to open registry hive: {}", e)))?;

    let key = if key_path.is_empty() || key_path == "\\" || key_path == "/" {
        parser
            .get_root_key()
            .map_err(|e| DocumentError::Parse(format!("Failed to read root key: {}", e)))?
            .ok_or_else(|| DocumentError::Parse("Registry hive has no root key".to_string()))?
    } else {
        let clean_path = key_path.trim_start_matches('\\').trim_start_matches('/');
        parser
            .get_key(clean_path, false)
            .map_err(|e| {
                DocumentError::Parse(format!("Failed to navigate to key '{}': {}", key_path, e))
            })?
            .ok_or_else(|| DocumentError::Parse(format!("Key not found: {}", key_path)))?
    };

    let pretty_path = key.get_pretty_path().to_string();
    let timestamp = Some(
        key.last_key_written_date_and_time()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    );
    let subkey_count = key.detail.number_of_sub_keys();
    let value_count = key.detail.number_of_key_values();

    // Get values
    let values: Vec<RegistryValue> = key
        .value_iter()
        .map(|val| {
            let (cell_value, _warnings) = val.get_content();
            let size = cell_value_size(&cell_value);
            RegistryValue {
                name: if val.detail.value_name().is_empty() {
                    "(Default)".to_string()
                } else {
                    val.detail.value_name().to_string()
                },
                data_type: data_type_name(&val.data_type).to_string(),
                data: format_cell_value(&cell_value),
                size,
            }
        })
        .collect();

    // Get subkeys
    let mut key_for_subkeys = key.clone();
    let sub_keys = key_for_subkeys.read_sub_keys(&mut parser);
    let subkeys: Vec<RegistryKey> = sub_keys
        .iter()
        .map(|sk| RegistryKey {
            name: sk.key_name.clone(),
            path: sk.path.clone(),
            timestamp: Some(
                sk.last_key_written_date_and_time()
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string(),
            ),
            subkey_count: sk.detail.number_of_sub_keys(),
            value_count: sk.detail.number_of_key_values(),
            has_subkeys: sk.detail.number_of_sub_keys() > 0,
        })
        .collect();

    Ok(RegistryKeyInfo {
        name: key.key_name.clone(),
        path: key.path.clone(),
        pretty_path,
        timestamp,
        subkey_count,
        value_count,
        values,
        subkeys,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Since we don't have a real registry hive in test_data, we test the helpers

    #[test]
    fn test_format_timestamp_valid() {
        // 2024-01-01 00:00:00 UTC
        // Unix: 1704067200s
        // FILETIME: (1704067200 + 11644473600) * 10_000_000
        let filetime = (1704067200u64 + 11_644_473_600u64) * 10_000_000u64;
        let result = format_timestamp(filetime);
        assert!(result.is_some());
        assert!(result.unwrap().contains("2024-01-01"));
    }

    #[test]
    fn test_format_timestamp_zero() {
        assert_eq!(format_timestamp(0), None);
    }

    #[test]
    fn test_data_type_name_sz() {
        assert_eq!(data_type_name(&CellKeyValueDataTypes::REG_SZ), "REG_SZ");
    }

    #[test]
    fn test_data_type_name_dword() {
        assert_eq!(
            data_type_name(&CellKeyValueDataTypes::REG_DWORD),
            "REG_DWORD"
        );
    }

    #[test]
    fn test_data_type_name_binary() {
        assert_eq!(
            data_type_name(&CellKeyValueDataTypes::REG_BIN),
            "REG_BINARY"
        );
    }

    #[test]
    fn test_data_type_name_multi_sz() {
        assert_eq!(
            data_type_name(&CellKeyValueDataTypes::REG_MULTI_SZ),
            "REG_MULTI_SZ"
        );
    }

    #[test]
    fn test_data_type_name_qword() {
        assert_eq!(
            data_type_name(&CellKeyValueDataTypes::REG_QWORD),
            "REG_QWORD"
        );
    }

    #[test]
    fn test_data_type_name_none() {
        assert_eq!(data_type_name(&CellKeyValueDataTypes::REG_NONE), "REG_NONE");
    }

    #[test]
    fn test_format_cell_value_string() {
        let val = CellValue::String("Hello World".to_string());
        assert_eq!(format_cell_value(&val), "Hello World");
    }

    #[test]
    fn test_format_cell_value_dword() {
        let val = CellValue::U32(42);
        let formatted = format_cell_value(&val);
        assert!(formatted.contains("0x0000002a"));
        assert!(formatted.contains("42"));
    }

    #[test]
    fn test_format_cell_value_qword() {
        let val = CellValue::U64(255);
        let formatted = format_cell_value(&val);
        assert!(formatted.contains("0x00000000000000ff"));
        assert!(formatted.contains("255"));
    }

    #[test]
    fn test_format_cell_value_multi_string() {
        let val = CellValue::MultiString(vec!["one".to_string(), "two".to_string()]);
        assert_eq!(format_cell_value(&val), "one; two");
    }

    #[test]
    fn test_format_cell_value_binary_short() {
        let val = CellValue::Binary(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let formatted = format_cell_value(&val);
        assert!(formatted.contains("dead"));
        assert!(formatted.contains("4 bytes"));
    }

    #[test]
    fn test_format_cell_value_binary_long() {
        let data: Vec<u8> = (0..128).collect();
        let val = CellValue::Binary(data);
        let formatted = format_cell_value(&val);
        assert!(formatted.contains("... (128 bytes)"));
    }

    #[test]
    fn test_format_cell_value_none() {
        let val = CellValue::None;
        assert_eq!(format_cell_value(&val), "(empty)");
    }

    #[test]
    fn test_format_cell_value_error() {
        let val = CellValue::Error;
        assert_eq!(format_cell_value(&val), "(error parsing value)");
    }

    #[test]
    fn test_cell_value_size_string() {
        assert_eq!(cell_value_size(&CellValue::String("Hi".to_string())), 4); // 2 chars * 2 bytes
    }

    #[test]
    fn test_cell_value_size_dword() {
        assert_eq!(cell_value_size(&CellValue::U32(0)), 4);
    }

    #[test]
    fn test_cell_value_size_qword() {
        assert_eq!(cell_value_size(&CellValue::U64(0)), 8);
    }

    #[test]
    fn test_cell_value_size_binary() {
        assert_eq!(cell_value_size(&CellValue::Binary(vec![1, 2, 3])), 3);
    }

    #[test]
    fn test_cell_value_size_none() {
        assert_eq!(cell_value_size(&CellValue::None), 0);
    }

    #[test]
    fn test_get_hive_info_nonexistent() {
        let result = get_hive_info("/nonexistent/file.dat");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_subkeys_nonexistent() {
        let result = get_subkeys("/nonexistent/file.dat", "\\");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_values_nonexistent() {
        let result = get_values("/nonexistent/file.dat", "\\");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_key_info_nonexistent() {
        let result = get_key_info("/nonexistent/file.dat", "\\");
        assert!(result.is_err());
    }
}
