// =============================================================================
// CORE-FFX - Forensic File Explorer
// Plist Viewer - Apple property list parsing for forensic analysis
// =============================================================================

use plist::Value;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::error::{DocumentError, DocumentResult};

pub(crate) const MAX_PLIST_PREVIEW_BYTES: u64 = 50 * 1024 * 1024;
const MAX_PLIST_FLAT_ENTRIES: usize = 10_000;
const MAX_PLIST_VALUE_DEPTH: usize = 128;
const MAX_PLIST_COLLECTION_ITEMS: usize = 10_000;

pub(crate) fn ensure_plist_preview_size_allowed(size: u64) -> DocumentResult<()> {
    if size > MAX_PLIST_PREVIEW_BYTES {
        return Err(DocumentError::Parse(format!(
            "Plist file too large for preview: {} bytes > {} bytes",
            size, MAX_PLIST_PREVIEW_BYTES
        )));
    }
    Ok(())
}

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
        convert_plist_value(value, 0)
    }
}

fn convert_plist_value(value: &Value, depth: usize) -> PlistValue {
    if depth > MAX_PLIST_VALUE_DEPTH {
        return PlistValue::String("(maximum plist depth reached)".to_string());
    }

    match value {
        Value::String(s) => PlistValue::String(s.clone()),
        Value::Integer(i) => PlistValue::Integer(i.as_signed().unwrap_or(0)),
        Value::Real(f) => PlistValue::Real(*f),
        Value::Boolean(b) => PlistValue::Boolean(*b),
        Value::Date(d) => PlistValue::Date(d.to_xml_format()),
        Value::Data(d) => {
            let preview: String = d.iter().take(32).map(|b| format!("{:02x}", b)).collect();
            if d.len() > 32 {
                PlistValue::Data(format!("{}... ({} bytes)", preview, d.len()))
            } else {
                PlistValue::Data(preview)
            }
        }
        Value::Array(arr) => PlistValue::Array(
            arr.iter()
                .take(MAX_PLIST_COLLECTION_ITEMS)
                .map(|value| convert_plist_value(value, depth + 1))
                .collect(),
        ),
        Value::Dictionary(dict) => PlistValue::Dictionary(
            dict.iter()
                .take(MAX_PLIST_COLLECTION_ITEMS)
                .map(|(key, value)| (key.clone(), convert_plist_value(value, depth + 1)))
                .collect(),
        ),
        _ => PlistValue::String("(unknown)".to_string()),
    }
}

/// Read and parse a plist file
pub fn read_plist(path: impl AsRef<Path>) -> DocumentResult<PlistInfo> {
    let path = path.as_ref();
    ensure_plist_preview_size_allowed(std::fs::metadata(path)?.len())?;
    let file = File::open(path)?;
    read_plist_from_reader(path.to_string_lossy(), file)
}

/// Read and parse a plist from any seekable byte stream.
pub fn read_plist_from_reader(
    source_id: impl Into<String>,
    mut reader: impl Read + Seek,
) -> DocumentResult<PlistInfo> {
    let source_id = source_id.into();
    let format = detect_plist_format_from_reader(&mut reader)?;
    reader.seek(SeekFrom::Start(0))?;

    let value: Value = plist::from_reader(reader)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse plist: {}", e)))?;

    let root_type = value_type_name(&value);
    let mut entries = Vec::new();
    flatten_plist(&value, "", &mut entries, 0);

    Ok(PlistInfo {
        path: source_id,
        format,
        root_type,
        entry_count: entries.len(),
        entries,
    })
}

/// Read plist and return structured value
pub fn read_plist_value(path: impl AsRef<Path>) -> DocumentResult<PlistValue> {
    let path = path.as_ref();
    ensure_plist_preview_size_allowed(std::fs::metadata(path)?.len())?;
    let file = File::open(path)?;

    let value: Value = plist::from_reader(file)
        .map_err(|e| DocumentError::Parse(format!("Failed to parse plist: {}", e)))?;

    Ok(PlistValue::from(&value))
}

/// Get value at a specific key path (e.g., "CFBundleIdentifier" or "nested/key")
pub fn get_plist_value_at_path(
    path: impl AsRef<Path>,
    key_path: &str,
) -> DocumentResult<Option<PlistValue>> {
    let path = path.as_ref();
    ensure_plist_preview_size_allowed(std::fs::metadata(path)?.len())?;
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

fn flatten_plist(value: &Value, prefix: &str, entries: &mut Vec<FlatPlistEntry>, depth: usize) {
    if depth > MAX_PLIST_VALUE_DEPTH || entries.len() >= MAX_PLIST_FLAT_ENTRIES {
        return;
    }

    match value {
        Value::Dictionary(dict) => {
            for (key, val) in dict {
                if entries.len() >= MAX_PLIST_FLAT_ENTRIES {
                    break;
                }
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
                flatten_plist(val, &path, entries, depth + 1);
            }
        }
        Value::Array(arr) => {
            for (idx, val) in arr.iter().enumerate() {
                if entries.len() >= MAX_PLIST_FLAT_ENTRIES {
                    break;
                }
                let path = format!("{}/{}", prefix, idx);
                entries.push(FlatPlistEntry {
                    key_path: path.clone(),
                    value_type: value_type_name(val),
                    value_preview: value_preview(val),
                });
                flatten_plist(val, &path, entries, depth + 1);
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
    }
    .to_string()
}

fn value_preview(value: &Value) -> String {
    match value {
        Value::String(s) => {
            if s.chars().count() > 50 {
                format!("{}...", s.chars().take(50).collect::<String>())
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

fn detect_plist_format_from_reader(reader: &mut (impl Read + Seek)) -> DocumentResult<String> {
    let original = reader.stream_position()?;
    let mut buf = Vec::new();
    reader.by_ref().take(8).read_to_end(&mut buf)?;
    reader.seek(SeekFrom::Start(original))?;

    if buf.len() >= 6 && &buf[..6] == b"bplist" {
        return Ok("Binary".to_string());
    }
    if buf.len() >= 5 && &buf[..5] == b"<?xml" {
        return Ok("XML".to_string());
    }
    Ok("Unknown".to_string())
}

/// Search plist for entries matching a pattern
pub fn search_plist(path: impl AsRef<Path>, pattern: &str) -> DocumentResult<Vec<FlatPlistEntry>> {
    let info = read_plist(path)?;
    let pattern_lower = pattern.to_lowercase();

    let results: Vec<FlatPlistEntry> = info
        .entries
        .into_iter()
        .filter(|e| {
            e.key_path.to_lowercase().contains(&pattern_lower)
                || e.value_preview.to_lowercase().contains(&pattern_lower)
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, SeekFrom, Write};

    struct OneByteCursor {
        data: Vec<u8>,
        position: u64,
    }

    impl OneByteCursor {
        fn new(data: &[u8]) -> Self {
            Self {
                data: data.to_vec(),
                position: 0,
            }
        }
    }

    impl Read for OneByteCursor {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let position = self.position as usize;
            if position >= self.data.len() || buf.is_empty() {
                return Ok(0);
            }
            buf[0] = self.data[position];
            self.position += 1;
            Ok(1)
        }
    }

    impl Seek for OneByteCursor {
        fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
            let len = self.data.len() as i64;
            let current = self.position as i64;
            let next = match pos {
                SeekFrom::Start(offset) => offset as i64,
                SeekFrom::End(offset) => len + offset,
                SeekFrom::Current(offset) => current + offset,
            };
            if next < 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid seek before start",
                ));
            }
            self.position = next as u64;
            Ok(self.position)
        }
    }

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

    #[test]
    fn read_plist_from_reader_parses_xml_source_metadata() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>com.core.ffx</string>
  <key>Enabled</key>
  <true/>
</dict>
</plist>"#;

        let info = read_plist_from_reader("container.ad1:Info.plist", Cursor::new(xml)).unwrap();

        assert_eq!(info.path, "container.ad1:Info.plist");
        assert_eq!(info.format, "XML");
        assert_eq!(info.root_type, "Dictionary(2)");
        assert!(info.entries.iter().any(|entry| {
            entry.key_path == "CFBundleIdentifier" && entry.value_preview == "com.core.ffx"
        }));
    }

    #[test]
    fn detect_plist_format_from_reader_preserves_position() {
        let mut reader = Cursor::new(b"bplist00payload");
        reader.set_position(2);

        let format = detect_plist_format_from_reader(&mut reader).unwrap();

        assert_eq!(format, "Unknown");
        assert_eq!(reader.position(), 2);
    }

    #[test]
    fn detect_plist_format_from_reader_assembles_short_reads() {
        let mut reader = OneByteCursor::new(b"bplist00payload");

        let format = detect_plist_format_from_reader(&mut reader).unwrap();

        assert_eq!(format, "Binary");
        assert_eq!(reader.position, 0);
    }

    #[test]
    fn read_plist_rejects_sparse_oversized_file_before_parse() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(b"<?xml").unwrap();
        tmp.as_file_mut()
            .set_len(MAX_PLIST_PREVIEW_BYTES + 1)
            .unwrap();

        let err = read_plist(tmp.path()).unwrap_err();

        assert!(err.to_string().contains("Plist file too large"));
    }

    #[test]
    fn value_preview_truncates_on_char_boundary() {
        let value = Value::String(format!("{}suffix", "é".repeat(51)));

        let preview = value_preview(&value);

        assert!(preview.ends_with("..."));
        assert_eq!(preview.trim_end_matches("...").chars().count(), 50);
    }

    #[test]
    fn flatten_plist_caps_flat_entries() {
        let mut dict = plist::Dictionary::new();
        for index in 0..(MAX_PLIST_FLAT_ENTRIES + 32) {
            dict.insert(format!("key{index}"), Value::String("value".to_string()));
        }
        let value = Value::Dictionary(dict);
        let mut entries = Vec::new();

        flatten_plist(&value, "", &mut entries, 0);

        assert_eq!(entries.len(), MAX_PLIST_FLAT_ENTRIES);
    }

    #[test]
    fn convert_plist_value_caps_collection_items() {
        let value = Value::Array(vec![Value::Boolean(true); MAX_PLIST_COLLECTION_ITEMS + 32]);

        let converted = convert_plist_value(&value, 0);

        match converted {
            PlistValue::Array(items) => assert_eq!(items.len(), MAX_PLIST_COLLECTION_ITEMS),
            other => panic!("expected capped array, got {other:?}"),
        }
    }

    #[test]
    fn convert_plist_value_stops_at_depth_limit() {
        let value = Value::Array(vec![Value::String("deep".to_string())]);

        let converted = convert_plist_value(&value, MAX_PLIST_VALUE_DEPTH + 1);

        assert!(matches!(
            converted,
            PlistValue::String(message) if message.contains("maximum plist depth")
        ));
    }
}
