// =============================================================================
// CORE-FFX - Forensic File Explorer
// Plist Viewer - Apple property list parsing for forensic analysis
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use plist::Value;

use super::error::{DocumentError, DocumentResult};

/// Plist value representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PlistValue {
    String(String),
    Integer(i64),
    Real(f64),
    Boolean(bool),
    Date(String),
    Data(String), // base64 or hex
    Array(Vec<PlistValue>),
    Dictionary(Vec<(String, PlistValue)>),
}

/// Flattened plist entry for searching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlatPlistEntry {
    pub key_path: String,
    pub value_type: String,
    pub value_preview: String,
}

/// Parsed plist information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlistInfo {
    pub path: String,
    pub format: String,
    pub root_type: String,
    pub entry_count: usize,
    pub entries: Vec<FlatPlistEntry>,
}

impl From<&Value> for PlistValue {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(s) => PlistValue::String(s.clone()),
            Value::Integer(i) => PlistValue::Integer(i.as_signed().unwrap_or(0)),
            Value::Real(f) => PlistValue::Real(*f),
            Value::Boolean(b) => PlistValue::Boolean(*b),
            Value::Date(d) => PlistValue::Date(d.to_xml_format()),
            Value::Data(d) => {
                // Convert to hex string preview
                let preview: String = d.iter()
                    .take(32)
                    .map(|b| format!("{:02x}", b))
                    .collect();
                if d.len() > 32 {
                    PlistValue::Data(format!("{}... ({} bytes)", preview, d.len()))
                } else {
                    PlistValue::Data(preview)
                }
            }
            Value::Array(arr) => {
                PlistValue::Array(arr.iter().map(PlistValue::from).collect())
            }
            Value::Dictionary(dict) => {
                PlistValue::Dictionary(
                    dict.iter()
                        .map(|(k, v)| (k.clone(), PlistValue::from(v)))
                        .collect()
                )
            }
            _ => PlistValue::String("(unknown)".to_string()),
        }
    }
}

/// Read and parse a plist file
pub fn read_plist(path: impl AsRef<Path>) -> DocumentResult<PlistInfo> {
    let path = path.as_ref();
    let file = File::open(path)?;
    
    let value: Value = plist::from_reader(file)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse plist: {}", e)))?;
    
    let root_type = value_type_name(&value);
    let mut entries = Vec::new();
    flatten_plist(&value, "", &mut entries);
    
    Ok(PlistInfo {
        path: path.to_string_lossy().to_string(),
        format: detect_plist_format(path),
        root_type,
        entry_count: entries.len(),
        entries,
    })
}

/// Read plist and return structured value
pub fn read_plist_value(path: impl AsRef<Path>) -> DocumentResult<PlistValue> {
    let path = path.as_ref();
    let file = File::open(path)?;
    
    let value: Value = plist::from_reader(file)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse plist: {}", e)))?;
    
    Ok(PlistValue::from(&value))
}

/// Get value at a specific key path (e.g., "CFBundleIdentifier" or "nested/key")
pub fn get_plist_value_at_path(path: impl AsRef<Path>, key_path: &str) -> DocumentResult<Option<PlistValue>> {
    let path = path.as_ref();
    let file = File::open(path)?;
    
    let value: Value = plist::from_reader(file)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse plist: {}", e)))?;
    
    let result = navigate_to_key(&value, key_path);
    Ok(result.map(PlistValue::from))
}

fn navigate_to_key<'a>(value: &'a Value, key_path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = key_path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current = value;
    
    for part in parts {
        match current {
            Value::Dictionary(dict) => {
                current = dict.get(part)?;
            }
            Value::Array(arr) => {
                let idx: usize = part.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    
    Some(current)
}

fn flatten_plist(value: &Value, prefix: &str, entries: &mut Vec<FlatPlistEntry>) {
    match value {
        Value::Dictionary(dict) => {
            for (key, val) in dict {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}/{}", prefix, key)
                };
                entries.push(FlatPlistEntry {
                    key_path: path.clone(),
                    value_type: value_type_name(val),
                    value_preview: value_preview(val),
                });
                flatten_plist(val, &path, entries);
            }
        }
        Value::Array(arr) => {
            for (idx, val) in arr.iter().enumerate() {
                let path = format!("{}/{}", prefix, idx);
                entries.push(FlatPlistEntry {
                    key_path: path.clone(),
                    value_type: value_type_name(val),
                    value_preview: value_preview(val),
                });
                flatten_plist(val, &path, entries);
            }
        }
        _ => {}
    }
}

fn value_type_name(value: &Value) -> String {
    match value {
        Value::String(_) => "String",
        Value::Integer(_) => "Integer",
        Value::Real(_) => "Real",
        Value::Boolean(_) => "Boolean",
        Value::Date(_) => "Date",
        Value::Data(_) => "Data",
        Value::Array(arr) => return format!("Array({})", arr.len()),
        Value::Dictionary(dict) => return format!("Dictionary({})", dict.len()),
        _ => "Unknown",
    }.to_string()
}

fn value_preview(value: &Value) -> String {
    match value {
        Value::String(s) => {
            if s.len() > 50 {
                format!("{}...", &s[..50])
            } else {
                s.clone()
            }
        }
        Value::Integer(i) => format!("{}", i.as_signed().unwrap_or(0)),
        Value::Real(f) => format!("{}", f),
        Value::Boolean(b) => format!("{}", b),
        Value::Date(d) => d.to_xml_format(),
        Value::Data(d) => {
            let preview: String = d.iter().take(16).map(|b| format!("{:02x}", b)).collect();
            if d.len() > 16 {
                format!("{}... ({} bytes)", preview, d.len())
            } else {
                preview
            }
        }
        Value::Array(arr) => format!("[{} items]", arr.len()),
        Value::Dictionary(dict) => format!("{{ {} keys }}", dict.len()),
        _ => "(unknown)".to_string(),
    }
}

fn detect_plist_format(path: &Path) -> String {
    if let Ok(file) = File::open(path) {
        let mut buf = [0u8; 8];
        use std::io::Read;
        if let Ok(mut file) = Ok::<_, std::io::Error>(file) {
            if file.read_exact(&mut buf).is_ok() {
                if &buf[..6] == b"bplist" {
                    return "Binary".to_string();
                }
                if &buf[..5] == b"<?xml" {
                    return "XML".to_string();
                }
            }
        }
    }
    "Unknown".to_string()
}

/// Search plist for entries matching a pattern
pub fn search_plist(path: impl AsRef<Path>, pattern: &str) -> DocumentResult<Vec<FlatPlistEntry>> {
    let info = read_plist(path)?;
    let pattern_lower = pattern.to_lowercase();
    
    let results: Vec<FlatPlistEntry> = info.entries
        .into_iter()
        .filter(|e| {
            e.key_path.to_lowercase().contains(&pattern_lower) ||
            e.value_preview.to_lowercase().contains(&pattern_lower)
        })
        .collect();
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plist_value_from_string() {
        let value = Value::String("test".to_string());
        let plist_value = PlistValue::from(&value);
        assert!(matches!(plist_value, PlistValue::String(s) if s == "test"));
    }
    
    #[test]
    fn test_value_type_name() {
        assert_eq!(value_type_name(&Value::String("".to_string())), "String");
        assert_eq!(value_type_name(&Value::Boolean(true)), "Boolean");
    }
}
