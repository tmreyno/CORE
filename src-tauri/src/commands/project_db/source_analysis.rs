// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for persisted source byte-analysis records.

use super::with_project_db;
use crate::commands::hash::{open_hash_source, HashSourceInput};
use crate::common::{analyze_byte_source, SourceAnalysis, SourceAnalysisOptions};
use crate::project_db::{DbEvidenceFile, DbSourceAnalysisCategorySummary, DbSourceAnalysisRecord};
use sha2::{Digest, Sha256};

const DEFAULT_SOURCE_ANALYZER: &str = "core-source-analysis";
const MAX_SOURCE_ANALYZER_CHARS: usize = 128;
const MAX_SOURCE_ANALYSIS_RESPONSE_ROWS: usize = 10_000;
const MAX_SOURCE_ANALYSIS_FIELD_CHARS: usize = 4096;
const MAX_SOURCE_ANALYSIS_PREVIEW_CHARS: usize = 16_384;
const MAX_SOURCE_ANALYSIS_JSON_CHARS: usize = 65_536;
const MAX_SOURCE_ANALYSIS_JSON_DEPTH: usize = 4;
const MAX_SOURCE_ANALYSIS_JSON_ITEMS: usize = 256;
const SOURCE_ANALYSIS_TRUNCATED_SUFFIX: &str = "... [truncated]";

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbAnalyzeSourceRequest {
    pub source: HashSourceInput,
    pub options: Option<SourceAnalysisOptions>,
    pub evidence_file_id: Option<String>,
    pub evidence_file: Option<DbEvidenceFile>,
    pub analyzer: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbAnalyzeSourceResult {
    pub analysis: SourceAnalysis,
    pub record: DbSourceAnalysisRecord,
}

/// Read a persisted source-analysis record by ID.
#[tauri::command]
pub fn project_db_get_source_analysis(
    window: tauri::Window,
    id: String,
) -> Result<Option<DbSourceAnalysisRecord>, String> {
    with_project_db(window.label(), |db| db.get_source_analysis(&id))
        .map(|record| record.map(bounded_source_analysis_record_for_response))
}

/// List persisted source-analysis records.
#[tauri::command]
pub fn project_db_list_source_analyses(
    window: tauri::Window,
    limit: Option<i64>,
) -> Result<Vec<DbSourceAnalysisRecord>, String> {
    with_project_db(window.label(), |db| db.list_source_analyses(limit)).map(|records| {
        records
            .into_iter()
            .take(MAX_SOURCE_ANALYSIS_RESPONSE_ROWS)
            .map(bounded_source_analysis_record_for_response)
            .collect()
    })
}

/// Summarize persisted source analyses by primary signature category.
#[tauri::command]
pub fn project_db_summarize_source_analyses_by_category(
    window: tauri::Window,
) -> Result<Vec<DbSourceAnalysisCategorySummary>, String> {
    with_project_db(window.label(), |db| {
        db.summarize_source_analyses_by_category()
    })
}

/// Analyze a local file or supported container entry and persist the summary.
#[tauri::command]
pub async fn project_db_analyze_source_and_insert(
    window: tauri::Window,
    request: ProjectDbAnalyzeSourceRequest,
) -> Result<ProjectDbAnalyzeSourceResult, String> {
    let ProjectDbAnalyzeSourceRequest {
        source,
        options,
        evidence_file_id,
        evidence_file,
        analyzer,
    } = request;

    let resolved_evidence_id = evidence_file
        .as_ref()
        .map(|file| file.id.clone())
        .or(evidence_file_id);
    let analyzer = normalize_source_analyzer(analyzer);
    let analysis = tauri::async_runtime::spawn_blocking(move || {
        let byte_source = open_hash_source(&source)?;
        analyze_byte_source(byte_source.as_ref(), options.unwrap_or_default())
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Internal source analysis error: {e}"))??;
    let record = source_analysis_to_db_record(&analysis, resolved_evidence_id, analyzer)?;

    with_project_db(window.label(), |db| {
        if let Some(file) = &evidence_file {
            db.upsert_evidence_file(file)?;
        }
        db.upsert_source_analysis(&record)?;
        Ok(())
    })?;

    Ok(ProjectDbAnalyzeSourceResult { analysis, record })
}

fn source_analysis_to_db_record(
    analysis: &SourceAnalysis,
    evidence_file_id: Option<String>,
    analyzer: String,
) -> Result<DbSourceAnalysisRecord, String> {
    validate_source_analysis_bounds(analysis)?;

    let analyzer = normalize_source_analyzer(Some(analyzer));
    let source_ref_json = serde_json::to_string(&analysis.source_ref)
        .map_err(|e| format!("Failed to serialize source analysis ref: {e}"))?;
    let signatures_json = serde_json::to_string(&analysis.signatures)
        .map_err(|e| format!("Failed to serialize source signatures: {e}"))?;
    let entropy_windows_json = serde_json::to_string(&analysis.entropy_windows)
        .map_err(|e| format!("Failed to serialize source entropy windows: {e}"))?;
    let histogram_json = serde_json::to_string(&analysis.histogram)
        .map_err(|e| format!("Failed to serialize source histogram: {e}"))?;
    let indicators_json = serde_json::to_string(&analysis.indicators)
        .map_err(|e| format!("Failed to serialize source indicators: {e}"))?;
    let primary_signature = analysis.signatures.first();

    let record = DbSourceAnalysisRecord {
        id: source_analysis_id(
            &source_ref_json,
            analysis.offset,
            analysis.bytes_analyzed,
            &analyzer,
        ),
        evidence_file_id: evidence_file_id
            .map(|value| truncate_chars_with_suffix(&value, MAX_SOURCE_ANALYSIS_FIELD_CHARS)),
        source_id: truncate_chars_with_suffix(&analysis.source_id, MAX_SOURCE_ANALYSIS_FIELD_CHARS),
        source_ref_json: bounded_source_analysis_json_text(
            &source_ref_json,
            MAX_SOURCE_ANALYSIS_JSON_CHARS,
        ),
        total_size: source_analysis_u64_to_i64(analysis.total_size, "total_size")?,
        offset: source_analysis_u64_to_i64(analysis.offset, "offset")?,
        bytes_analyzed: source_analysis_usize_to_i64(analysis.bytes_analyzed, "bytes_analyzed")?,
        magic_hex: truncate_chars_with_suffix(&analysis.magic_hex, MAX_SOURCE_ANALYSIS_FIELD_CHARS),
        signature_count: source_analysis_usize_to_i64(
            analysis.signatures.len(),
            "signature_count",
        )?,
        primary_signature: primary_signature.map(|signature| {
            truncate_chars_with_suffix(&signature.description, MAX_SOURCE_ANALYSIS_FIELD_CHARS)
        }),
        primary_mime_type: primary_signature.map(|signature| {
            truncate_chars_with_suffix(&signature.mime_type, MAX_SOURCE_ANALYSIS_FIELD_CHARS)
        }),
        primary_category: primary_signature.map(|signature| {
            truncate_chars_with_suffix(&signature.category, MAX_SOURCE_ANALYSIS_FIELD_CHARS)
        }),
        entropy: analysis.entropy,
        printable_ratio: analysis.printable_ratio,
        is_likely_text: analysis.is_likely_text,
        ascii_preview: (!analysis.ascii_preview.is_empty()).then(|| {
            truncate_chars_with_suffix(&analysis.ascii_preview, MAX_SOURCE_ANALYSIS_PREVIEW_CHARS)
        }),
        signatures_json: Some(bounded_source_analysis_json_text(
            &signatures_json,
            MAX_SOURCE_ANALYSIS_JSON_CHARS,
        )),
        entropy_windows_json: Some(bounded_source_analysis_json_text(
            &entropy_windows_json,
            MAX_SOURCE_ANALYSIS_JSON_CHARS,
        )),
        histogram_json: Some(bounded_source_analysis_json_text(
            &histogram_json,
            MAX_SOURCE_ANALYSIS_JSON_CHARS,
        )),
        indicators_json: Some(bounded_source_analysis_json_text(
            &indicators_json,
            MAX_SOURCE_ANALYSIS_JSON_CHARS,
        )),
        analyzed_at: chrono::Utc::now().to_rfc3339(),
        analyzer,
    };

    Ok(record)
}

fn validate_source_analysis_bounds(analysis: &SourceAnalysis) -> Result<(), String> {
    source_analysis_u64_to_i64(analysis.total_size, "total_size")?;
    source_analysis_u64_to_i64(analysis.offset, "offset")?;
    let bytes_analyzed = source_analysis_usize_to_u64(analysis.bytes_analyzed, "bytes_analyzed")?;
    source_analysis_usize_to_i64(analysis.bytes_analyzed, "bytes_analyzed")?;
    source_analysis_usize_to_i64(analysis.signatures.len(), "signature_count")?;

    let analysis_end =
        checked_source_analysis_end(analysis.offset, bytes_analyzed, "analysis byte range")?;
    if analysis_end > analysis.total_size {
        return Err(format!(
            "Source analysis byte range exceeds source size: offset {} + {} bytes > {} bytes",
            analysis.offset, bytes_analyzed, analysis.total_size
        ));
    }

    for signature in &analysis.signatures {
        if signature.offset > analysis.total_size {
            return Err(format!(
                "Source signature offset exceeds source size: offset {} > {} bytes",
                signature.offset, analysis.total_size
            ));
        }
    }

    for window in &analysis.entropy_windows {
        let length = source_analysis_usize_to_u64(window.length, "entropy window length")?;
        let end = checked_source_analysis_end(window.offset, length, "entropy window range")?;
        if end > analysis.total_size {
            return Err(format!(
                "Source entropy window exceeds source size: offset {} + {} bytes > {} bytes",
                window.offset, length, analysis.total_size
            ));
        }
    }

    for indicator in &analysis.indicators {
        let length = source_analysis_usize_to_u64(indicator.length, "indicator length")?;
        if length == 0 {
            return Err("Source indicator length must be greater than zero".to_string());
        }
        let end = checked_source_analysis_end(indicator.offset, length, "indicator range")?;
        if end > analysis.total_size {
            return Err(format!(
                "Source indicator range exceeds source size: offset {} + {} bytes > {} bytes",
                indicator.offset, length, analysis.total_size
            ));
        }
    }

    Ok(())
}

fn source_analysis_u64_to_i64(value: u64, field_name: &str) -> Result<i64, String> {
    i64::try_from(value)
        .map_err(|_| format!("Source analysis {field_name} exceeds project DB range: {value}"))
}

fn source_analysis_usize_to_i64(value: usize, field_name: &str) -> Result<i64, String> {
    i64::try_from(value)
        .map_err(|_| format!("Source analysis {field_name} exceeds project DB range: {value}"))
}

fn source_analysis_usize_to_u64(value: usize, field_name: &str) -> Result<u64, String> {
    u64::try_from(value)
        .map_err(|_| format!("Source analysis {field_name} exceeds u64 range: {value}"))
}

fn checked_source_analysis_end(offset: u64, length: u64, field_name: &str) -> Result<u64, String> {
    offset.checked_add(length).ok_or_else(|| {
        format!("Source analysis {field_name} overflow: offset {offset} + {length} bytes")
    })
}

fn source_analysis_id(
    source_ref_json: &str,
    offset: u64,
    bytes_analyzed: usize,
    analyzer: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_ref_json.as_bytes());
    hasher.update(offset.to_le_bytes());
    hasher.update(bytes_analyzed.to_le_bytes());
    hasher.update(analyzer.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn normalize_source_analyzer(analyzer: Option<String>) -> String {
    let value = analyzer
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_SOURCE_ANALYZER);

    truncate_chars(value, MAX_SOURCE_ANALYZER_CHARS)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let end = value
        .char_indices()
        .nth(max_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    value[..end].to_string()
}

fn truncate_chars_with_suffix(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = SOURCE_ANALYSIS_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + SOURCE_ANALYSIS_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(SOURCE_ANALYSIS_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_source_analysis_record_for_response(
    mut record: DbSourceAnalysisRecord,
) -> DbSourceAnalysisRecord {
    record.id = truncate_chars_with_suffix(&record.id, MAX_SOURCE_ANALYSIS_FIELD_CHARS);
    record.evidence_file_id = record
        .evidence_file_id
        .map(|value| truncate_chars_with_suffix(&value, MAX_SOURCE_ANALYSIS_FIELD_CHARS));
    record.source_id =
        truncate_chars_with_suffix(&record.source_id, MAX_SOURCE_ANALYSIS_FIELD_CHARS);
    record.source_ref_json =
        bounded_source_analysis_json_text(&record.source_ref_json, MAX_SOURCE_ANALYSIS_JSON_CHARS);
    record.magic_hex =
        truncate_chars_with_suffix(&record.magic_hex, MAX_SOURCE_ANALYSIS_FIELD_CHARS);
    record.primary_signature = record
        .primary_signature
        .map(|value| truncate_chars_with_suffix(&value, MAX_SOURCE_ANALYSIS_FIELD_CHARS));
    record.primary_mime_type = record
        .primary_mime_type
        .map(|value| truncate_chars_with_suffix(&value, MAX_SOURCE_ANALYSIS_FIELD_CHARS));
    record.primary_category = record
        .primary_category
        .map(|value| truncate_chars_with_suffix(&value, MAX_SOURCE_ANALYSIS_FIELD_CHARS));
    record.ascii_preview = record
        .ascii_preview
        .map(|value| truncate_chars_with_suffix(&value, MAX_SOURCE_ANALYSIS_PREVIEW_CHARS));
    record.signatures_json = record
        .signatures_json
        .map(|value| bounded_source_analysis_json_text(&value, MAX_SOURCE_ANALYSIS_JSON_CHARS));
    record.entropy_windows_json = record
        .entropy_windows_json
        .map(|value| bounded_source_analysis_json_text(&value, MAX_SOURCE_ANALYSIS_JSON_CHARS));
    record.histogram_json = record
        .histogram_json
        .map(|value| bounded_source_analysis_json_text(&value, MAX_SOURCE_ANALYSIS_JSON_CHARS));
    record.indicators_json = record
        .indicators_json
        .map(|value| bounded_source_analysis_json_text(&value, MAX_SOURCE_ANALYSIS_JSON_CHARS));
    record.analyzed_at =
        truncate_chars_with_suffix(&record.analyzed_at, MAX_SOURCE_ANALYSIS_FIELD_CHARS);
    record.analyzer = truncate_chars_with_suffix(&record.analyzer, MAX_SOURCE_ANALYSIS_FIELD_CHARS);
    record
}

fn bounded_source_analysis_json_text(value: &str, max_chars: usize) -> String {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return if value.chars().count() <= max_chars {
            value.to_string()
        } else {
            truncate_chars_with_suffix(value, max_chars)
        };
    };
    let bounded = bounded_source_analysis_json_value(json, 0);
    let Ok(serialized) = serde_json::to_string(&bounded) else {
        return "{\"truncated\":true}".to_string();
    };

    if serialized.chars().count() <= max_chars {
        serialized
    } else {
        "{\"truncated\":true}".to_string()
    }
}

fn bounded_source_analysis_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_SOURCE_ANALYSIS_JSON_DEPTH {
        return serde_json::Value::String(SOURCE_ANALYSIS_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(value) => serde_json::Value::String(truncate_chars_with_suffix(
            &value,
            MAX_SOURCE_ANALYSIS_FIELD_CHARS,
        )),
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_SOURCE_ANALYSIS_JSON_ITEMS)
                .map(|value| bounded_source_analysis_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(object) => {
            let mut bounded = serde_json::Map::new();
            for (key, value) in object.into_iter().take(MAX_SOURCE_ANALYSIS_JSON_ITEMS) {
                bounded.insert(
                    truncate_chars_with_suffix(&key, MAX_SOURCE_ANALYSIS_FIELD_CHARS),
                    bounded_source_analysis_json_value(value, depth + 1),
                );
            }
            serde_json::Value::Object(bounded)
        }
        value @ (serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)) => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::{EntropyWindow, EvidenceSourceRef, SourceIndicator, SourceSignature};

    fn base_source_analysis() -> SourceAnalysis {
        SourceAnalysis {
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/file.bin".to_string(),
            },
            source_id: "/case/file.bin".to_string(),
            total_size: 128,
            offset: 0,
            bytes_analyzed: 32,
            magic_hex: "00 01".to_string(),
            signatures: Vec::new(),
            entropy: 1.0,
            entropy_windows: Vec::new(),
            histogram: vec![0; 256],
            printable_bytes: 0,
            nul_bytes: 0,
            high_bit_bytes: 0,
            printable_ratio: 0.0,
            is_likely_text: false,
            indicators: Vec::new(),
            ascii_preview: String::new(),
        }
    }

    #[test]
    fn source_analysis_to_db_record_preserves_summary_fields() {
        let analysis = SourceAnalysis {
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/file.pdf".to_string(),
            },
            source_id: "/case/file.pdf".to_string(),
            total_size: 128,
            offset: 0,
            bytes_analyzed: 32,
            magic_hex: "25 50 44 46".to_string(),
            signatures: vec![SourceSignature {
                offset: 0,
                description: "PDF Document".to_string(),
                mime_type: "application/pdf".to_string(),
                extensions: vec!["pdf".to_string()],
                category: "document".to_string(),
                confidence: "high".to_string(),
                magic_hex: "25 50 44 46".to_string(),
            }],
            entropy: 4.25,
            entropy_windows: Vec::new(),
            histogram: vec![0; 256],
            printable_bytes: 20,
            nul_bytes: 0,
            high_bit_bytes: 0,
            printable_ratio: 0.625,
            is_likely_text: true,
            indicators: Vec::new(),
            ascii_preview: "%PDF".to_string(),
        };

        let record = source_analysis_to_db_record(
            &analysis,
            Some("ev-1".to_string()),
            "test-analyzer".to_string(),
        )
        .unwrap();

        assert_eq!(record.evidence_file_id.as_deref(), Some("ev-1"));
        assert_eq!(record.source_id, "/case/file.pdf");
        assert_eq!(record.primary_signature.as_deref(), Some("PDF Document"));
        assert_eq!(record.primary_mime_type.as_deref(), Some("application/pdf"));
        assert_eq!(record.primary_category.as_deref(), Some("document"));
        assert_eq!(record.signature_count, 1);
        assert_eq!(record.bytes_analyzed, 32);
        assert_eq!(record.ascii_preview.as_deref(), Some("%PDF"));
        assert_eq!(record.analyzer, "test-analyzer");
        assert!(record.signatures_json.unwrap().contains("PDF Document"));
        assert_eq!(record.indicators_json.as_deref(), Some("[]"));
    }

    #[test]
    fn source_analysis_to_db_record_bounds_preview_and_json_fields() {
        let mut analysis = base_source_analysis();
        analysis.total_size = 1024;
        analysis.ascii_preview = "p".repeat(MAX_SOURCE_ANALYSIS_PREVIEW_CHARS + 32);
        analysis.signatures = (0..(MAX_SOURCE_ANALYSIS_JSON_ITEMS + 10))
            .map(|index| SourceSignature {
                offset: index as u64,
                description: format!("Signature {index}"),
                mime_type: "application/octet-stream".to_string(),
                extensions: vec!["bin".to_string()],
                category: "binary".to_string(),
                confidence: "low".to_string(),
                magic_hex: "00".to_string(),
            })
            .collect();

        let record = source_analysis_to_db_record(&analysis, None, "test-analyzer".to_string())
            .expect("source analysis conversion should succeed");

        let preview = record.ascii_preview.as_deref().unwrap();
        assert_eq!(preview.chars().count(), MAX_SOURCE_ANALYSIS_PREVIEW_CHARS);
        assert!(preview.ends_with(SOURCE_ANALYSIS_TRUNCATED_SUFFIX));

        let signatures: serde_json::Value =
            serde_json::from_str(record.signatures_json.as_deref().unwrap()).unwrap();
        assert_eq!(
            signatures.as_array().unwrap().len(),
            MAX_SOURCE_ANALYSIS_JSON_ITEMS
        );
    }

    #[test]
    fn bounded_source_analysis_record_for_response_caps_payloads_and_preserves_json() {
        let record = DbSourceAnalysisRecord {
            id: "analysis-1".to_string(),
            evidence_file_id: Some("ev-1".to_string()),
            source_id: "s".repeat(MAX_SOURCE_ANALYSIS_FIELD_CHARS + 32),
            source_ref_json: serde_json::json!({
                "kind": "localFile",
                "path": "x".repeat(MAX_SOURCE_ANALYSIS_JSON_CHARS + 32)
            })
            .to_string(),
            total_size: 42,
            offset: 0,
            bytes_analyzed: 42,
            magic_hex: "00".to_string(),
            signature_count: 1,
            primary_signature: Some("PDF Document".to_string()),
            primary_mime_type: Some("application/pdf".to_string()),
            primary_category: Some("document".to_string()),
            entropy: 1.0,
            printable_ratio: 0.5,
            is_likely_text: true,
            ascii_preview: Some("a".repeat(MAX_SOURCE_ANALYSIS_PREVIEW_CHARS + 32)),
            signatures_json: Some(
                serde_json::json!((0..(MAX_SOURCE_ANALYSIS_JSON_ITEMS + 10)).collect::<Vec<_>>())
                    .to_string(),
            ),
            entropy_windows_json: Some("[]".to_string()),
            histogram_json: Some("[]".to_string()),
            indicators_json: Some(
                serde_json::json!([{
                    "indicatorType": "email",
                    "value": "i".repeat(MAX_SOURCE_ANALYSIS_JSON_CHARS + 32),
                    "offset": 0,
                    "length": 8,
                    "confidence": "low"
                }])
                .to_string(),
            ),
            analyzed_at: "2026-02-16T10:00:00Z".to_string(),
            analyzer: "test-analyzer".to_string(),
        };

        let bounded = bounded_source_analysis_record_for_response(record);

        assert_eq!(
            bounded.source_id.chars().count(),
            MAX_SOURCE_ANALYSIS_FIELD_CHARS
        );
        assert!(bounded
            .source_id
            .ends_with(SOURCE_ANALYSIS_TRUNCATED_SUFFIX));
        assert_eq!(
            bounded.ascii_preview.as_deref().unwrap().chars().count(),
            MAX_SOURCE_ANALYSIS_PREVIEW_CHARS
        );
        assert!(serde_json::from_str::<serde_json::Value>(&bounded.source_ref_json).is_ok());
        let signatures: serde_json::Value =
            serde_json::from_str(bounded.signatures_json.as_deref().unwrap()).unwrap();
        assert_eq!(
            signatures.as_array().unwrap().len(),
            MAX_SOURCE_ANALYSIS_JSON_ITEMS
        );
        assert!(serde_json::from_str::<serde_json::Value>(
            bounded.indicators_json.as_deref().unwrap()
        )
        .is_ok());
    }

    #[test]
    fn source_analysis_to_db_record_normalizes_analyzer_label() {
        let mut analysis = SourceAnalysis {
            source_ref: EvidenceSourceRef::LocalFile {
                path: "/case/file.bin".to_string(),
            },
            source_id: "/case/file.bin".to_string(),
            total_size: 1,
            offset: 0,
            bytes_analyzed: 1,
            magic_hex: "00".to_string(),
            signatures: Vec::new(),
            entropy: 0.0,
            entropy_windows: Vec::new(),
            histogram: vec![0; 256],
            printable_bytes: 0,
            nul_bytes: 1,
            high_bit_bytes: 0,
            printable_ratio: 0.0,
            is_likely_text: false,
            indicators: Vec::new(),
            ascii_preview: String::new(),
        };

        let oversized = format!("  {}é  ", "a".repeat(MAX_SOURCE_ANALYZER_CHARS + 64));
        let record = source_analysis_to_db_record(&analysis, None, oversized).unwrap();

        assert_eq!(record.analyzer.chars().count(), MAX_SOURCE_ANALYZER_CHARS);
        assert!(record.analyzer.starts_with('a'));

        analysis.offset = 1;
        analysis.total_size = 2;
        let defaulted = source_analysis_to_db_record(&analysis, None, "   ".to_string()).unwrap();
        assert_eq!(defaulted.analyzer, DEFAULT_SOURCE_ANALYZER);
    }

    #[test]
    fn source_analysis_to_db_record_rejects_analysis_range_past_source_size() {
        let mut analysis = base_source_analysis();
        analysis.offset = 120;
        analysis.bytes_analyzed = 16;

        let err =
            source_analysis_to_db_record(&analysis, None, "test-analyzer".to_string()).unwrap_err();

        assert!(err.contains("byte range exceeds source size"));
    }

    #[test]
    fn source_analysis_to_db_record_rejects_unrepresentable_total_size() {
        let mut analysis = base_source_analysis();
        analysis.total_size = i64::MAX as u64 + 1;

        let err =
            source_analysis_to_db_record(&analysis, None, "test-analyzer".to_string()).unwrap_err();

        assert!(err.contains("total_size exceeds project DB range"));
    }

    #[test]
    fn source_analysis_to_db_record_rejects_signature_offset_past_source_size() {
        let mut analysis = base_source_analysis();
        analysis.signatures.push(SourceSignature {
            offset: 129,
            description: "Past EOF".to_string(),
            mime_type: "application/octet-stream".to_string(),
            extensions: Vec::new(),
            category: "unknown".to_string(),
            confidence: "low".to_string(),
            magic_hex: String::new(),
        });

        let err =
            source_analysis_to_db_record(&analysis, None, "test-analyzer".to_string()).unwrap_err();

        assert!(err.contains("signature offset exceeds source size"));
    }

    #[test]
    fn source_analysis_to_db_record_rejects_entropy_window_past_source_size() {
        let mut analysis = base_source_analysis();
        analysis.entropy_windows.push(EntropyWindow {
            offset: 120,
            length: 16,
            entropy: 4.0,
        });

        let err =
            source_analysis_to_db_record(&analysis, None, "test-analyzer".to_string()).unwrap_err();

        assert!(err.contains("entropy window exceeds source size"));
    }

    #[test]
    fn source_analysis_to_db_record_rejects_indicator_past_source_size() {
        let mut analysis = base_source_analysis();
        analysis.indicators.push(SourceIndicator {
            indicator_type: "email".to_string(),
            value: "admin@example.com".to_string(),
            offset: 120,
            length: 16,
            confidence: "medium".to_string(),
        });

        let err =
            source_analysis_to_db_record(&analysis, None, "test-analyzer".to_string()).unwrap_err();

        assert!(err.contains("indicator range exceeds source size"));
    }

    #[test]
    fn source_analysis_to_db_record_rejects_zero_length_indicator() {
        let mut analysis = base_source_analysis();
        analysis.indicators.push(SourceIndicator {
            indicator_type: "email".to_string(),
            value: "admin@example.com".to_string(),
            offset: 16,
            length: 0,
            confidence: "medium".to_string(),
        });

        let err =
            source_analysis_to_db_record(&analysis, None, "test-analyzer".to_string()).unwrap_err();

        assert!(err.contains("indicator length must be greater than zero"));
    }
}
