// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for report generation
//!
//! These commands expose the report generation functionality to the frontend.

use parking_lot::Mutex;
use std::collections::BTreeMap;
use tauri::State;

use core_types::evidence::{DbCollectedItem, DbEvidenceCollection};
use core_types::evidence_collection_contract::EVIDENCE_COLLECTION_PACKAGE_VERSION;
use core_types::mobile::{
    MobileEvidenceCollectionPackage, MobileEvidenceCollectionPackageCollection, MobileProject,
};

use super::{types::*, ForensicReport, OutputFormat, ReportGenerator};
use crate::common::hex::format_size_compact;
use crate::project_db::{
    DbAnnotation, DbArtifactEvidenceSummary, DbArtifactExtractorSummary, DbEvidenceFile,
    DbHashAlgorithmSummary, DbNormalizedArtifact, DbProjectHash, DbSourceAnalysisCategorySummary,
    DbSourceAnalysisRecord, DbVerificationResultSummary, ProjectDatabase,
};

const MAX_REPORT_DB_DETAIL_ROWS: i64 = 10_000;
const MAX_REPORT_FIELD_CHARS: usize = 4_096;
const MAX_REPORT_PREVIEW_CHARS: usize = 16_384;
const MAX_REPORT_JSON_CHARS: usize = 65_536;
const MAX_REPORT_JSON_DEPTH: usize = 4;
const MAX_REPORT_JSON_ITEMS: usize = 256;
const MAX_REPORT_METADATA_ENTRIES: usize = 256;
const MAX_REPORT_SOURCE_INDICATORS: usize = 512;
const MAX_REPORT_INDICATOR_VALUE_CHARS: usize = 2_048;
const REPORT_TRUNCATED_SUFFIX: &str = "... [truncated]";

/// State wrapper for the report generator
pub struct ReportState {
    generator: Mutex<ReportGenerator>,
}

impl ReportState {
    pub fn new() -> Result<Self, String> {
        let generator = ReportGenerator::new().map_err(|e| e.to_string())?;
        Ok(Self {
            generator: Mutex::new(generator),
        })
    }
}

impl Default for ReportState {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::error!("Failed to create report state: {}", e);
            // Create with a placeholder generator that will error on use
            Self {
                generator: Mutex::new(
                    ReportGenerator::new()
                        .expect("Report generator fallback also failed - fonts may be missing"),
                ),
            }
        })
    }
}

/// Generate a report in the specified format
#[tauri::command]
pub async fn generate_report(
    report: ForensicReport,
    format: OutputFormat,
    output_path: String,
    state: State<'_, ReportState>,
) -> Result<String, String> {
    let generator = state.generator.lock();

    generator
        .generate(&report, format, &output_path)
        .map_err(|e| e.to_string())?;

    Ok(output_path)
}

/// Generate a report preview (HTML)
#[tauri::command]
pub async fn preview_report(
    report: ForensicReport,
    state: State<'_, ReportState>,
) -> Result<String, String> {
    let generator = state.generator.lock();

    Ok(generator.render_preview_html(&report))
}

/// Get available output formats
#[tauri::command]
pub fn get_output_formats() -> Vec<FormatInfo> {
    let typst_supported = OutputFormat::Typst.is_supported();

    vec![
        FormatInfo {
            format: OutputFormat::Pdf,
            name: "PDF".to_string(),
            description: "Portable Document Format - Best for printing and sharing".to_string(),
            extension: "pdf".to_string(),
            supported: true,
        },
        FormatInfo {
            format: OutputFormat::Docx,
            name: "Word Document".to_string(),
            description: "Microsoft Word format - Best for editing and court submissions"
                .to_string(),
            extension: "docx".to_string(),
            supported: true,
        },
        FormatInfo {
            format: OutputFormat::Html,
            name: "HTML".to_string(),
            description: "Web page format - Best for browser viewing".to_string(),
            extension: "html".to_string(),
            supported: true,
        },
        FormatInfo {
            format: OutputFormat::Markdown,
            name: "Markdown".to_string(),
            description: "Plain text with formatting - Best for version control".to_string(),
            extension: "md".to_string(),
            supported: true,
        },
        FormatInfo {
            format: OutputFormat::Typst,
            name: "Typst".to_string(),
            description: if typst_supported {
                "Modern typesetting format - Best for high-quality source reports".to_string()
            } else {
                "Modern typesetting format - Requires the typst-reports feature".to_string()
            },
            extension: "typ".to_string(),
            supported: typst_supported,
        },
    ]
}

/// Information about an output format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FormatInfo {
    pub format: OutputFormat,
    pub name: String,
    pub description: String,
    pub extension: String,
    pub supported: bool,
}

/// Export report to JSON (for saving/loading)
#[tauri::command]
pub fn export_report_json(report: ForensicReport) -> Result<String, String> {
    serde_json::to_string_pretty(&report).map_err(|e| e.to_string())
}

/// Import report from JSON
#[tauri::command]
pub fn import_report_json(json: String) -> Result<ForensicReport, String> {
    serde_json::from_str(&json).map_err(|e| e.to_string())
}

/// Container info from the frontend (simplified for report extraction)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInfoInput {
    pub container_type: String,
    pub path: String,
    pub filename: String,
    pub size: u64,
    // EWF fields
    pub case_number: Option<String>,
    pub evidence_number: Option<String>,
    pub examiner_name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub acquiry_date: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub total_size: Option<u64>,
    // Hash info
    pub stored_hashes: Option<Vec<StoredHashInput>>,
    pub computed_hash: Option<StoredHashInput>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredHashInput {
    pub algorithm: String,
    pub hash: String,
    pub verified: Option<bool>,
}

/// Project database evidence facts prepared for report generation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDbReportEvidence {
    pub evidence_items: Vec<EvidenceItem>,
    pub hash_records: Vec<HashRecord>,
    pub hash_algorithm_summaries: Vec<ReportHashAlgorithmSummary>,
    pub verification_result_summaries: Vec<ReportVerificationResultSummary>,
    pub artifacts: Vec<DbNormalizedArtifact>,
    pub artifact_summaries: Vec<ReportArtifactSummary>,
    pub artifact_categories: Vec<ReportArtifactCategorySummary>,
    pub artifact_evidence_summaries: Vec<ReportArtifactEvidenceSummary>,
    pub artifact_extractor_summaries: Vec<ReportArtifactExtractorSummary>,
    pub source_analyses: Vec<DbSourceAnalysisRecord>,
    pub source_analysis_summaries: Vec<ReportSourceAnalysisSummary>,
    pub source_analysis_category_summaries: Vec<ReportSourceAnalysisCategorySummary>,
    pub annotations: Vec<DbAnnotation>,
}

/// Count and coverage rollup for stored hashes using one algorithm.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportHashAlgorithmSummary {
    pub algorithm: HashAlgorithm,
    pub algorithm_label: String,
    pub count: i64,
    pub evidence_file_count: i64,
    pub source_count: i64,
    pub latest_computed_at: Option<String>,
}

/// Count and coverage rollup for hash verification results.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportVerificationResultSummary {
    pub result: String,
    pub count: i64,
    pub hash_count: i64,
    pub latest_verified_at: Option<String>,
}

/// Report-ready summary of a normalized artifact.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArtifactSummary {
    pub id: String,
    pub evidence_file_id: Option<String>,
    pub source_id: String,
    pub source_ref: Option<serde_json::Value>,
    pub name: String,
    pub category: String,
    pub type_description: String,
    pub mime_type: Option<String>,
    pub size: i64,
    pub size_display: String,
    pub confidence: String,
    pub is_text: bool,
    pub preview: Option<String>,
    pub metadata: BTreeMap<String, String>,
    pub extractor: String,
    pub extracted_at: String,
}

/// Count of normalized artifacts in one report category.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArtifactCategorySummary {
    pub category: String,
    pub count: usize,
}

/// Count and size rollup for normalized artifacts tied to one evidence file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArtifactEvidenceSummary {
    pub evidence_file_id: Option<String>,
    pub count: i64,
    pub total_size: i64,
    pub total_size_display: String,
    pub text_count: i64,
    pub category_count: i64,
    pub latest_extracted_at: Option<String>,
}

/// Count and size rollup for normalized artifacts produced by one extractor.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportArtifactExtractorSummary {
    pub extractor: String,
    pub count: i64,
    pub total_size: i64,
    pub total_size_display: String,
    pub text_count: i64,
    pub category_count: i64,
    pub evidence_file_count: i64,
    pub latest_extracted_at: Option<String>,
}

/// Report-ready summary of a persisted source byte-analysis record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSourceAnalysisSummary {
    pub id: String,
    pub evidence_file_id: Option<String>,
    pub source_id: String,
    pub source_ref: Option<serde_json::Value>,
    pub total_size: i64,
    pub total_size_display: String,
    pub offset: i64,
    pub bytes_analyzed: i64,
    pub bytes_analyzed_display: String,
    pub magic_hex: String,
    pub signature_count: i64,
    pub primary_signature: Option<String>,
    pub primary_mime_type: Option<String>,
    pub primary_category: String,
    pub entropy: f64,
    pub printable_ratio: f64,
    pub is_likely_text: bool,
    pub indicators: Vec<ReportSourceIndicator>,
    pub indicator_count: usize,
    pub preview: Option<String>,
    pub analyzed_at: String,
    pub analyzer: String,
}

/// Text-like indicators extracted during source byte analysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSourceIndicator {
    pub indicator_type: String,
    pub value: String,
    pub offset: u64,
    pub length: usize,
    pub confidence: String,
}

/// Count and entropy rollup for persisted source analyses by category.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSourceAnalysisCategorySummary {
    pub category: String,
    pub count: i64,
    pub evidence_file_count: i64,
    pub avg_entropy: f64,
    pub text_like_count: i64,
    pub latest_analyzed_at: Option<String>,
}

fn format_signed_byte_count(value: i64, field_name: &str) -> String {
    match u64::try_from(value) {
        Ok(value) => format_size_compact(value),
        Err(_) => format!("invalid {field_name}: {value} bytes"),
    }
}

/// Extract evidence items from container info
///
/// This command takes container information from the frontend and converts
/// it into properly formatted EvidenceItem structures for the report.
#[tauri::command]
pub fn extract_evidence_from_containers(
    containers: Vec<ContainerInfoInput>,
) -> Result<Vec<EvidenceItem>, String> {
    let mut evidence_items = Vec::new();

    for (index, container) in containers.iter().enumerate() {
        // Determine evidence type from container type
        let evidence_type = match container.container_type.to_lowercase().as_str() {
            "e01" | "ex01" | "ewf" => EvidenceType::ForensicImage,
            "l01" | "lx01" => EvidenceType::ForensicImage,
            "ad1" => EvidenceType::ForensicImage,
            "raw" | "dd" | "img" => EvidenceType::ForensicImage,
            "ufed" | "ufdx" | "ufd" | "ufdr" => EvidenceType::MobilePhone,
            "zip" | "7z" | "tar" | "gz" => EvidenceType::Other,
            _ => EvidenceType::Other,
        };

        // Build hash records from stored and computed
        let mut acquisition_hashes = Vec::new();

        if let Some(ref hashes) = container.stored_hashes {
            for h in hashes {
                acquisition_hashes.push(HashRecord {
                    item: container.filename.clone(),
                    algorithm: parse_hash_algorithm(&h.algorithm),
                    value: h.hash.clone(),
                    computed_at: None,
                    verified: h.verified,
                });
            }
        }

        if let Some(ref h) = container.computed_hash {
            acquisition_hashes.push(HashRecord {
                item: container.filename.clone(),
                algorithm: parse_hash_algorithm(&h.algorithm),
                value: h.hash.clone(),
                computed_at: Some(chrono::Utc::now()),
                verified: h.verified,
            });
        }

        // Build image info
        let image_info = Some(ImageInfo {
            format: container.container_type.clone(),
            file_names: vec![container.filename.clone()],
            total_size: container.total_size.unwrap_or(container.size),
            segments: None,
            compression: None,
            acquisition_tool: Some("FFX - Forensic File Xplorer".to_string()),
            acquisition_date: container.acquiry_date.as_ref().and_then(|d| {
                chrono::DateTime::parse_from_rfc3339(d)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
        });

        // Create evidence item
        let evidence_item = EvidenceItem {
            evidence_id: format!("E{:03}", index + 1),
            description: container
                .description
                .clone()
                .unwrap_or_else(|| container.filename.clone()),
            evidence_type,
            make: None,
            model: container.model.clone(),
            serial_number: container.serial_number.clone(),
            capacity: container
                .total_size
                .or(Some(container.size))
                .map(format_size_compact),
            condition: None,
            received_date: None,
            submitted_by: None,
            acquisition_hashes,
            verification_hashes: Vec::new(),
            image_info,
            notes: container.notes.clone(),
            acquisition_method: None,
            acquisition_tool: None,
        };

        evidence_items.push(evidence_item);
    }

    Ok(evidence_items)
}

/// Extract report-ready evidence facts from the active project database.
#[tauri::command]
pub fn extract_report_evidence_from_project_db(
    window: tauri::Window,
) -> Result<ProjectDbReportEvidence, String> {
    crate::commands::project_db::with_project_db(window.label(), project_db_report_evidence)
}

fn project_db_report_evidence(db: &ProjectDatabase) -> rusqlite::Result<ProjectDbReportEvidence> {
    let files = db.get_evidence_files_limited(Some(MAX_REPORT_DB_DETAIL_ROWS))?;
    let mut evidence_items = Vec::with_capacity(files.len());
    let mut hash_records = Vec::new();
    let artifacts = db.list_artifacts(Some(MAX_REPORT_DB_DETAIL_ROWS))?;
    let source_analyses = db.list_source_analyses(Some(MAX_REPORT_DB_DETAIL_ROWS))?;
    let annotations = db.get_all_annotations_limited(Some(MAX_REPORT_DB_DETAIL_ROWS))?;
    let hash_algorithm_summaries = db
        .summarize_hashes_by_algorithm()?
        .iter()
        .map(report_hash_algorithm_summary_from_project_db)
        .collect();
    let verification_result_summaries = db
        .summarize_verifications_by_result()?
        .iter()
        .map(report_verification_result_summary_from_project_db)
        .collect();
    let artifact_evidence_summaries = db
        .summarize_artifacts_by_evidence()?
        .iter()
        .map(report_artifact_evidence_summary_from_project_db)
        .collect();
    let artifact_extractor_summaries = db
        .summarize_artifacts_by_extractor()?
        .iter()
        .map(report_artifact_extractor_summary_from_project_db)
        .collect();
    let source_analysis_category_summaries = db
        .summarize_source_analyses_by_category()?
        .iter()
        .map(report_source_analysis_category_summary_from_project_db)
        .collect();

    for (index, file) in files.iter().enumerate() {
        let hashes = db.get_hashes_for_file(&file.id)?;
        let file_artifacts = db.list_artifacts_for_evidence(&file.id)?;

        evidence_items.push(evidence_item_from_project_db(
            index,
            file,
            &hashes,
            file_artifacts.len(),
        ));

        hash_records.extend(
            hashes
                .iter()
                .map(|hash| hash_record_from_project_db(&file.filename, hash)),
        );
    }

    Ok(ProjectDbReportEvidence {
        artifact_summaries: artifact_summaries_from_project_db(&artifacts),
        artifact_categories: artifact_category_summaries(&artifacts),
        artifact_evidence_summaries,
        artifact_extractor_summaries,
        evidence_items,
        hash_records,
        hash_algorithm_summaries,
        verification_result_summaries,
        artifacts: bounded_report_artifacts(&artifacts),
        source_analysis_summaries: source_analysis_summaries_from_project_db(&source_analyses),
        source_analysis_category_summaries,
        source_analyses: bounded_report_source_analyses(&source_analyses),
        annotations: bounded_report_annotations(&annotations),
    })
}

fn bounded_report_artifacts(artifacts: &[DbNormalizedArtifact]) -> Vec<DbNormalizedArtifact> {
    artifacts
        .iter()
        .take(MAX_REPORT_DB_DETAIL_ROWS as usize)
        .cloned()
        .map(|mut artifact| {
            artifact.id = truncate_report_text(&artifact.id, MAX_REPORT_FIELD_CHARS);
            artifact.evidence_file_id =
                truncate_report_option(artifact.evidence_file_id, MAX_REPORT_FIELD_CHARS);
            artifact.source_id = truncate_report_text(&artifact.source_id, MAX_REPORT_FIELD_CHARS);
            artifact.source_ref_json =
                bounded_json_text(&artifact.source_ref_json, MAX_REPORT_JSON_CHARS);
            artifact.name = truncate_report_text(&artifact.name, MAX_REPORT_FIELD_CHARS);
            artifact.extension = truncate_report_option(artifact.extension, MAX_REPORT_FIELD_CHARS);
            artifact.mime_type = truncate_report_option(artifact.mime_type, MAX_REPORT_FIELD_CHARS);
            artifact.type_description =
                truncate_report_text(&artifact.type_description, MAX_REPORT_FIELD_CHARS);
            artifact.category = truncate_report_text(&artifact.category, MAX_REPORT_FIELD_CHARS);
            artifact.confidence =
                truncate_report_text(&artifact.confidence, MAX_REPORT_FIELD_CHARS);
            artifact.content_preview =
                truncate_report_option(artifact.content_preview, MAX_REPORT_PREVIEW_CHARS);
            artifact.metadata_json = artifact
                .metadata_json
                .map(|metadata| bounded_json_text(&metadata, MAX_REPORT_JSON_CHARS));
            artifact.extracted_at =
                truncate_report_text(&artifact.extracted_at, MAX_REPORT_FIELD_CHARS);
            artifact.extractor = truncate_report_text(&artifact.extractor, MAX_REPORT_FIELD_CHARS);
            artifact
        })
        .collect()
}

fn bounded_report_source_analyses(
    records: &[DbSourceAnalysisRecord],
) -> Vec<DbSourceAnalysisRecord> {
    records
        .iter()
        .take(MAX_REPORT_DB_DETAIL_ROWS as usize)
        .cloned()
        .map(|mut record| {
            record.id = truncate_report_text(&record.id, MAX_REPORT_FIELD_CHARS);
            record.evidence_file_id =
                truncate_report_option(record.evidence_file_id, MAX_REPORT_FIELD_CHARS);
            record.source_id = truncate_report_text(&record.source_id, MAX_REPORT_FIELD_CHARS);
            record.source_ref_json =
                bounded_json_text(&record.source_ref_json, MAX_REPORT_JSON_CHARS);
            record.magic_hex = truncate_report_text(&record.magic_hex, MAX_REPORT_FIELD_CHARS);
            record.primary_signature =
                truncate_report_option(record.primary_signature, MAX_REPORT_FIELD_CHARS);
            record.primary_mime_type =
                truncate_report_option(record.primary_mime_type, MAX_REPORT_FIELD_CHARS);
            record.primary_category =
                truncate_report_option(record.primary_category, MAX_REPORT_FIELD_CHARS);
            record.ascii_preview =
                truncate_report_option(record.ascii_preview, MAX_REPORT_PREVIEW_CHARS);
            record.signatures_json = record
                .signatures_json
                .map(|json| bounded_json_text(&json, MAX_REPORT_JSON_CHARS));
            record.entropy_windows_json = record
                .entropy_windows_json
                .map(|json| bounded_json_text(&json, MAX_REPORT_JSON_CHARS));
            record.histogram_json = record
                .histogram_json
                .map(|json| bounded_json_text(&json, MAX_REPORT_JSON_CHARS));
            record.indicators_json = record
                .indicators_json
                .map(|json| bounded_json_text(&json, MAX_REPORT_JSON_CHARS));
            record.analyzed_at = truncate_report_text(&record.analyzed_at, MAX_REPORT_FIELD_CHARS);
            record.analyzer = truncate_report_text(&record.analyzer, MAX_REPORT_FIELD_CHARS);
            record
        })
        .collect()
}

fn bounded_report_annotations(annotations: &[DbAnnotation]) -> Vec<DbAnnotation> {
    annotations
        .iter()
        .take(MAX_REPORT_DB_DETAIL_ROWS as usize)
        .cloned()
        .map(|mut annotation| {
            annotation.id = truncate_report_text(&annotation.id, MAX_REPORT_FIELD_CHARS);
            annotation.file_path =
                truncate_report_text(&annotation.file_path, MAX_REPORT_FIELD_CHARS);
            annotation.container_path =
                truncate_report_option(annotation.container_path, MAX_REPORT_FIELD_CHARS);
            annotation.annotation_type =
                truncate_report_text(&annotation.annotation_type, MAX_REPORT_FIELD_CHARS);
            annotation.label = truncate_report_text(&annotation.label, MAX_REPORT_FIELD_CHARS);
            annotation.content =
                truncate_report_option(annotation.content, MAX_REPORT_PREVIEW_CHARS);
            annotation.color = truncate_report_option(annotation.color, MAX_REPORT_FIELD_CHARS);
            annotation.created_by =
                truncate_report_text(&annotation.created_by, MAX_REPORT_FIELD_CHARS);
            annotation.created_at =
                truncate_report_text(&annotation.created_at, MAX_REPORT_FIELD_CHARS);
            annotation.modified_at =
                truncate_report_text(&annotation.modified_at, MAX_REPORT_FIELD_CHARS);
            annotation
        })
        .collect()
}

fn truncate_report_option(value: Option<String>, max_chars: usize) -> Option<String> {
    value.map(|value| truncate_report_text(&value, max_chars))
}

fn truncate_report_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let suffix_chars = REPORT_TRUNCATED_SUFFIX.chars().count();
    let keep_chars = max_chars.saturating_sub(suffix_chars);
    let end = value
        .char_indices()
        .nth(keep_chars)
        .map(|(idx, _)| idx)
        .unwrap_or(value.len());
    let mut truncated = String::with_capacity(end + REPORT_TRUNCATED_SUFFIX.len());
    truncated.push_str(&value[..end]);
    truncated.push_str(REPORT_TRUNCATED_SUFFIX);
    truncated
}

fn bounded_json_text(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let Ok(json) = serde_json::from_str::<serde_json::Value>(value) else {
        return truncate_report_text(value, max_chars);
    };

    let bounded = bounded_json_value(json, 0);
    let Ok(serialized) = serde_json::to_string(&bounded) else {
        return "{\"truncated\":true}".to_string();
    };

    if serialized.chars().count() <= max_chars {
        serialized
    } else {
        "{\"truncated\":true}".to_string()
    }
}

fn bounded_json_value(value: serde_json::Value, depth: usize) -> serde_json::Value {
    if depth >= MAX_REPORT_JSON_DEPTH {
        return serde_json::Value::String(REPORT_TRUNCATED_SUFFIX.to_string());
    }

    match value {
        serde_json::Value::String(value) => {
            serde_json::Value::String(truncate_report_text(&value, MAX_REPORT_FIELD_CHARS))
        }
        serde_json::Value::Array(values) => serde_json::Value::Array(
            values
                .into_iter()
                .take(MAX_REPORT_JSON_ITEMS)
                .map(|value| bounded_json_value(value, depth + 1))
                .collect(),
        ),
        serde_json::Value::Object(object) => {
            let mut bounded = serde_json::Map::new();
            for (key, value) in object.into_iter().take(MAX_REPORT_JSON_ITEMS) {
                bounded.insert(
                    truncate_report_text(&key, MAX_REPORT_FIELD_CHARS),
                    bounded_json_value(value, depth + 1),
                );
            }
            serde_json::Value::Object(bounded)
        }
        value @ (serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)) => value,
    }
}

fn evidence_item_from_project_db(
    index: usize,
    file: &DbEvidenceFile,
    hashes: &[DbProjectHash],
    artifact_count: usize,
) -> EvidenceItem {
    let total_size = u64::try_from(file.total_size).unwrap_or(0);
    let capacity = format_signed_byte_count(file.total_size, "evidence total size");
    let acquisition_hashes = hashes
        .iter()
        .map(|hash| hash_record_from_project_db(&file.filename, hash))
        .collect();

    let notes = if artifact_count > 0 {
        Some(format!("{artifact_count} normalized artifact(s) extracted"))
    } else {
        None
    };

    EvidenceItem {
        evidence_id: format!("E{:03}", index + 1),
        description: file.filename.clone(),
        evidence_type: evidence_type_from_container(&file.container_type),
        make: None,
        model: None,
        serial_number: None,
        capacity: Some(capacity),
        condition: None,
        received_date: parse_project_datetime(file.created.as_deref()),
        submitted_by: None,
        acquisition_hashes,
        verification_hashes: Vec::new(),
        image_info: Some(ImageInfo {
            format: file.container_type.clone(),
            file_names: vec![file.filename.clone()],
            total_size,
            segments: u32::try_from(file.segment_count).ok(),
            compression: None,
            acquisition_tool: Some("FFX - Forensic File Xplorer".to_string()),
            acquisition_date: parse_project_datetime(file.created.as_deref()),
        }),
        notes,
        acquisition_method: None,
        acquisition_tool: Some("FFX - Forensic File Xplorer".to_string()),
    }
}

fn hash_record_from_project_db(item: &str, hash: &DbProjectHash) -> HashRecord {
    HashRecord {
        item: item.to_string(),
        algorithm: parse_hash_algorithm(&hash.algorithm),
        value: hash.hash_value.clone(),
        computed_at: parse_project_datetime(Some(&hash.computed_at)),
        verified: None,
    }
}

fn report_hash_algorithm_summary_from_project_db(
    summary: &DbHashAlgorithmSummary,
) -> ReportHashAlgorithmSummary {
    let algorithm = parse_hash_algorithm(&summary.algorithm);
    let algorithm_label = algorithm.as_str().to_string();
    ReportHashAlgorithmSummary {
        algorithm,
        algorithm_label,
        count: summary.count,
        evidence_file_count: summary.evidence_file_count,
        source_count: summary.source_count,
        latest_computed_at: summary.latest_computed_at.clone(),
    }
}

fn report_verification_result_summary_from_project_db(
    summary: &DbVerificationResultSummary,
) -> ReportVerificationResultSummary {
    ReportVerificationResultSummary {
        result: summary.result.clone(),
        count: summary.count,
        hash_count: summary.hash_count,
        latest_verified_at: summary.latest_verified_at.clone(),
    }
}

fn artifact_summaries_from_project_db(
    artifacts: &[DbNormalizedArtifact],
) -> Vec<ReportArtifactSummary> {
    artifacts
        .iter()
        .map(artifact_summary_from_project_db)
        .collect()
}

fn artifact_summary_from_project_db(artifact: &DbNormalizedArtifact) -> ReportArtifactSummary {
    ReportArtifactSummary {
        id: artifact.id.clone(),
        evidence_file_id: artifact.evidence_file_id.clone(),
        source_id: artifact.source_id.clone(),
        source_ref: parse_artifact_source_ref(&artifact.source_ref_json),
        name: artifact.name.clone(),
        category: artifact.category.clone(),
        type_description: artifact.type_description.clone(),
        mime_type: artifact.mime_type.clone(),
        size: artifact.size,
        size_display: format_signed_byte_count(artifact.size, "artifact size"),
        confidence: artifact.confidence.clone(),
        is_text: artifact.is_text,
        preview: artifact.content_preview.clone(),
        metadata: parse_artifact_metadata(artifact.metadata_json.as_deref()),
        extractor: artifact.extractor.clone(),
        extracted_at: artifact.extracted_at.clone(),
    }
}

fn parse_artifact_source_ref(source_ref_json: &str) -> Option<serde_json::Value> {
    match serde_json::from_str(source_ref_json) {
        Ok(value) => Some(value),
        Err(error) => Some(serde_json::json!({
            "invalidSourceRef": true,
            "parseError": truncate_report_text(&error.to_string(), MAX_REPORT_FIELD_CHARS),
            "raw": truncate_report_text(source_ref_json, MAX_REPORT_FIELD_CHARS),
        })),
    }
}

fn parse_artifact_metadata(metadata_json: Option<&str>) -> BTreeMap<String, String> {
    let Some(metadata_json) = metadata_json else {
        return BTreeMap::new();
    };

    match serde_json::from_str::<BTreeMap<String, String>>(metadata_json) {
        Ok(metadata) => metadata
            .into_iter()
            .take(MAX_REPORT_METADATA_ENTRIES)
            .map(|(key, value)| {
                (
                    truncate_report_text(&key, MAX_REPORT_FIELD_CHARS),
                    truncate_report_text(&value, MAX_REPORT_FIELD_CHARS),
                )
            })
            .collect(),
        Err(map_error) => match serde_json::from_str::<serde_json::Value>(metadata_json) {
            Ok(serde_json::Value::Object(object)) => object
                .iter()
                .take(MAX_REPORT_METADATA_ENTRIES)
                .map(|(key, value)| {
                    (
                        truncate_report_text(key, MAX_REPORT_FIELD_CHARS),
                        truncate_report_text(
                            &metadata_value_to_string(value),
                            MAX_REPORT_FIELD_CHARS,
                        ),
                    )
                })
                .collect(),
            Ok(_) => artifact_metadata_parse_error(
                "artifact metadata JSON is not an object",
                metadata_json,
            ),
            Err(value_error) => {
                artifact_metadata_parse_error(&format!("{map_error}; {value_error}"), metadata_json)
            }
        },
    }
}

fn artifact_metadata_parse_error(error: &str, raw: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "metadata.parseError".to_string(),
            truncate_report_text(error, MAX_REPORT_FIELD_CHARS),
        ),
        (
            "metadata.raw".to_string(),
            truncate_report_text(raw, MAX_REPORT_FIELD_CHARS),
        ),
    ])
}

fn metadata_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
    }
}

fn artifact_category_summaries(
    artifacts: &[DbNormalizedArtifact],
) -> Vec<ReportArtifactCategorySummary> {
    let mut counts = BTreeMap::<String, usize>::new();
    for artifact in artifacts {
        *counts.entry(artifact.category.clone()).or_default() += 1;
    }

    counts
        .into_iter()
        .map(|(category, count)| ReportArtifactCategorySummary { category, count })
        .collect()
}

fn report_artifact_evidence_summary_from_project_db(
    summary: &DbArtifactEvidenceSummary,
) -> ReportArtifactEvidenceSummary {
    ReportArtifactEvidenceSummary {
        evidence_file_id: summary.evidence_file_id.clone(),
        count: summary.count,
        total_size: summary.total_size,
        total_size_display: format_signed_byte_count(summary.total_size, "artifact total size"),
        text_count: summary.text_count,
        category_count: summary.category_count,
        latest_extracted_at: summary.latest_extracted_at.clone(),
    }
}

fn report_artifact_extractor_summary_from_project_db(
    summary: &DbArtifactExtractorSummary,
) -> ReportArtifactExtractorSummary {
    ReportArtifactExtractorSummary {
        extractor: summary.extractor.clone(),
        count: summary.count,
        total_size: summary.total_size,
        total_size_display: format_signed_byte_count(summary.total_size, "artifact total size"),
        text_count: summary.text_count,
        category_count: summary.category_count,
        evidence_file_count: summary.evidence_file_count,
        latest_extracted_at: summary.latest_extracted_at.clone(),
    }
}

fn source_analysis_summaries_from_project_db(
    records: &[DbSourceAnalysisRecord],
) -> Vec<ReportSourceAnalysisSummary> {
    records
        .iter()
        .map(source_analysis_summary_from_project_db)
        .collect()
}

fn source_analysis_summary_from_project_db(
    record: &DbSourceAnalysisRecord,
) -> ReportSourceAnalysisSummary {
    let indicators = parse_source_indicators(record.indicators_json.as_deref());
    ReportSourceAnalysisSummary {
        id: record.id.clone(),
        evidence_file_id: record.evidence_file_id.clone(),
        source_id: record.source_id.clone(),
        source_ref: parse_artifact_source_ref(&record.source_ref_json),
        total_size: record.total_size,
        total_size_display: format_signed_byte_count(record.total_size, "source total size"),
        offset: record.offset,
        bytes_analyzed: record.bytes_analyzed,
        bytes_analyzed_display: format_signed_byte_count(record.bytes_analyzed, "bytes analyzed"),
        magic_hex: record.magic_hex.clone(),
        signature_count: record.signature_count,
        primary_signature: record.primary_signature.clone(),
        primary_mime_type: record.primary_mime_type.clone(),
        primary_category: record
            .primary_category
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        entropy: record.entropy,
        printable_ratio: record.printable_ratio,
        is_likely_text: record.is_likely_text,
        indicator_count: indicators.len(),
        indicators,
        preview: record.ascii_preview.clone(),
        analyzed_at: record.analyzed_at.clone(),
        analyzer: record.analyzer.clone(),
    }
}

fn parse_source_indicators(indicators_json: Option<&str>) -> Vec<ReportSourceIndicator> {
    let Some(indicators_json) = indicators_json else {
        return Vec::new();
    };
    let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(indicators_json) else {
        return Vec::new();
    };

    values
        .iter()
        .take(MAX_REPORT_SOURCE_INDICATORS)
        .filter_map(parse_source_indicator_value)
        .collect()
}

fn parse_source_indicator_value(value: &serde_json::Value) -> Option<ReportSourceIndicator> {
    let object = value.as_object()?;
    let indicator_type = object.get("indicatorType")?.as_str()?.trim();
    let indicator_value = object.get("value")?.as_str()?.trim();
    let offset = object.get("offset")?.as_u64()?;
    let length = usize::try_from(object.get("length")?.as_u64()?).ok()?;
    let confidence = object.get("confidence")?.as_str()?.trim();

    if indicator_type.is_empty()
        || indicator_value.is_empty()
        || length == 0
        || confidence.is_empty()
    {
        return None;
    }

    Some(ReportSourceIndicator {
        indicator_type: truncate_report_text(indicator_type, MAX_REPORT_FIELD_CHARS),
        value: truncate_report_text(indicator_value, MAX_REPORT_INDICATOR_VALUE_CHARS),
        offset,
        length,
        confidence: truncate_report_text(confidence, MAX_REPORT_FIELD_CHARS),
    })
}

fn report_source_analysis_category_summary_from_project_db(
    summary: &DbSourceAnalysisCategorySummary,
) -> ReportSourceAnalysisCategorySummary {
    ReportSourceAnalysisCategorySummary {
        category: summary.category.clone(),
        count: summary.count,
        evidence_file_count: summary.evidence_file_count,
        avg_entropy: summary.avg_entropy,
        text_like_count: summary.text_like_count,
        latest_analyzed_at: summary.latest_analyzed_at.clone(),
    }
}

fn evidence_type_from_container(container_type: &str) -> EvidenceType {
    match container_type.to_lowercase().as_str() {
        "e01" | "ex01" | "ewf" | "l01" | "lx01" | "ad1" | "raw" | "dd" | "img" => {
            EvidenceType::ForensicImage
        }
        "ufed" | "ufdx" | "ufd" | "ufdr" => EvidenceType::MobilePhone,
        _ => EvidenceType::Other,
    }
}

fn parse_project_datetime(value: Option<&str>) -> Option<chrono::DateTime<chrono::Utc>> {
    value.and_then(|value| {
        chrono::DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    })
}

/// Parse hash algorithm string to enum
fn parse_hash_algorithm(s: &str) -> HashAlgorithm {
    match s.to_lowercase().as_str() {
        "md5" => HashAlgorithm::MD5,
        "sha1" | "sha-1" => HashAlgorithm::SHA1,
        "sha256" | "sha-256" => HashAlgorithm::SHA256,
        "sha512" | "sha-512" => HashAlgorithm::SHA512,
        "blake2" | "blake2b" => HashAlgorithm::Blake2b,
        "blake3" => HashAlgorithm::Blake3,
        _ => HashAlgorithm::SHA256, // Default to SHA256
    }
}

/// Create evidence item from a single container
#[tauri::command]
pub fn create_evidence_from_container(
    container: ContainerInfoInput,
    evidence_id: String,
) -> Result<EvidenceItem, String> {
    let items = extract_evidence_from_containers(vec![container])?;
    let mut item = items
        .into_iter()
        .next()
        .ok_or("Failed to create evidence")?;
    item.evidence_id = evidence_id;
    Ok(item)
}

/// Get a report template for different investigation types
#[tauri::command]
pub fn get_report_template(investigation_type: String) -> ForensicReport {
    let mut builder = ForensicReport::builder().case_number("").examiner_name("");

    // Add type-specific methodology
    let methodology = match investigation_type.as_str() {
        "computer" => {
            r#"The examination was conducted using forensically sound practices and industry-standard tools. The evidence was acquired using write-blocking hardware to prevent any modification to the original media. A forensic image was created and verified using cryptographic hash values.

The examination process included:
1. Physical inspection of evidence items
2. Forensic imaging with hash verification
3. File system analysis
4. Artifact extraction and analysis
5. Timeline analysis
6. Documentation of findings"#
        }
        "mobile" => {
            r#"The mobile device examination was conducted using forensically sound practices. The device was placed in airplane mode or a faraday bag to prevent remote modification. Data was extracted using industry-standard mobile forensic tools.

The examination process included:
1. Device identification and photography
2. Logical and/or physical extraction
3. Application data analysis
4. Communication analysis (calls, messages)
5. Location data analysis
6. Media file examination
7. Documentation of findings"#
        }
        "network" => {
            r#"The network forensic examination was conducted using industry-standard analysis tools. Network captures were analyzed for relevant traffic patterns and communications.

The examination process included:
1. Packet capture analysis
2. Protocol analysis
3. Traffic pattern identification
4. Communication reconstruction
5. Malware traffic analysis
6. Documentation of findings"#
        }
        _ => {
            r#"The examination was conducted using forensically sound practices and industry-standard tools. Evidence integrity was maintained throughout the process using cryptographic hash verification.

The examination process included:
1. Evidence acquisition with verification
2. Data analysis using appropriate tools
3. Documentation of findings"#
        }
    };

    builder = builder.methodology(methodology);

    builder
        .build()
        .expect("report template builder should include required case and examiner fields")
}

#[cfg(feature = "ai-assistant")]
pub mod ai_commands {
    use crate::report::ai::{AiAssistant, AiProvider};
    use crate::report::NarrativeType;

    /// AI provider info for frontend
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct AiProviderInfo {
        pub id: String,
        pub name: String,
        pub description: String,
        pub requires_api_key: bool,
        pub default_model: String,
        pub available_models: Vec<String>,
    }

    /// Get available AI providers
    #[tauri::command]
    pub fn get_ai_providers() -> Vec<AiProviderInfo> {
        vec![
            AiProviderInfo {
                id: "openai".to_string(),
                name: "OpenAI".to_string(),
                description:
                    "Recommended for the strongest report-writing quality via the Responses API. Uses a typed key or OPENAI_API_KEY."
                        .to_string(),
                requires_api_key: true,
                default_model: "gpt-5".to_string(),
                available_models: vec![
                    "gpt-5".to_string(),
                    "gpt-5-mini".to_string(),
                    "gpt-4.1".to_string(),
                    "gpt-4.1-mini".to_string(),
                ],
            },
            AiProviderInfo {
                id: "ollama".to_string(),
                name: "Ollama (Local)".to_string(),
                description: "Run local models on the workstation with Ollama installed."
                    .to_string(),
                requires_api_key: false,
                default_model: "llama3.2".to_string(),
                available_models: vec![
                    "llama3.2".to_string(),
                    "llama3.1".to_string(),
                    "mistral".to_string(),
                    "codellama".to_string(),
                    "phi3".to_string(),
                    "gemma2".to_string(),
                ],
            },
        ]
    }

    /// Generate AI narrative for a report section
    #[tauri::command]
    pub async fn generate_ai_narrative(
        context: String,
        narrative_type: String,
        provider: String,
        model: String,
        api_key: Option<String>,
    ) -> Result<String, String> {
        let narrative_type = match narrative_type.as_str() {
            "executive_summary" => NarrativeType::ExecutiveSummary,
            "finding" => NarrativeType::FindingDescription,
            "timeline" => NarrativeType::TimelineNarrative,
            "evidence" => NarrativeType::EvidenceDescription,
            "methodology" => NarrativeType::Methodology,
            "conclusion" => NarrativeType::Conclusion,
            _ => return Err(format!("Unknown narrative type: {}", narrative_type)),
        };

        let provider_enum = match provider.as_str() {
            "ollama" => AiProvider::Ollama {
                model: model.clone(),
                base_url: None,
            },
            "openai" => AiProvider::OpenAi {
                model: model.clone(),
                api_key,
            },
            _ => return Err(format!("Unknown provider: {}", provider)),
        };

        let ai = AiAssistant::new(provider_enum);

        ai.generate_narrative(&context, narrative_type)
            .await
            .map_err(|e| e.to_string())
    }

    /// Check if Ollama is running and accessible
    #[tauri::command]
    pub async fn check_ollama_connection() -> Result<bool, String> {
        // Try to connect to Ollama API
        let client = reqwest::Client::new();
        match client
            .get("http://localhost:11434/api/version")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Check if AI assistant is available
    #[tauri::command]
    pub fn is_ai_available() -> bool {
        true
    }
}

#[cfg(not(feature = "ai-assistant"))]
pub mod ai_commands {
    /// AI provider info for frontend (stub)
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct AiProviderInfo {
        pub id: String,
        pub name: String,
        pub description: String,
        pub requires_api_key: bool,
        pub default_model: String,
        pub available_models: Vec<String>,
    }

    /// Check if AI assistant is available
    #[tauri::command]
    pub fn is_ai_available() -> bool {
        false
    }

    /// Get available AI providers (stub - returns empty)
    #[tauri::command]
    pub fn get_ai_providers() -> Vec<AiProviderInfo> {
        vec![]
    }

    /// Generate AI narrative (stub - returns error)
    #[tauri::command]
    pub async fn generate_ai_narrative(
        _context: String,
        _narrative_type: String,
        _provider: String,
        _model: String,
        _api_key: Option<String>,
    ) -> Result<String, String> {
        Err("AI assistant is not enabled. Rebuild with 'ai-assistant' feature.".to_string())
    }

    /// Check if Ollama is running (stub - returns false)
    #[tauri::command]
    pub async fn check_ollama_connection() -> Result<bool, String> {
        Ok(false)
    }
}

// =============================================================================
// Evidence Collection Multi-Format Export
// =============================================================================

/// Export evidence collection data in the specified format.
///
/// Supported `format` values: `"pdf"`, `"csv"`, `"xlsx"`, `"html"`, `"json"`
#[tauri::command]
pub async fn export_evidence_collection(
    data: super::types::EvidenceCollectionData,
    case_number: String,
    format: String,
    output_path: String,
    state: State<'_, ReportState>,
) -> Result<String, String> {
    match format.to_lowercase().as_str() {
        "csv" => {
            super::evidence_collection_export::export_csv(&data, &output_path)
                .map_err(|e| e.to_string())?;
        }
        "xlsx" => {
            super::evidence_collection_export::export_xlsx(&data, &case_number, &output_path)
                .map_err(|e| e.to_string())?;
        }
        "html" => {
            super::evidence_collection_export::export_html(&data, &case_number, &output_path)
                .map_err(|e| e.to_string())?;
        }
        "json" => {
            let package = build_evidence_collection_package(&data, &case_number, "CORE-FFX")?;
            let json = serde_json::to_string_pretty(&package).map_err(|e| e.to_string())?;
            std::fs::write(&output_path, json).map_err(|e| format!("Failed to write file: {e}"))?;
        }
        "pdf" => {
            // Build a minimal ForensicReport to feed the PDF renderer
            let report = super::types::ForensicReport {
                case_info: super::types::CaseInfo {
                    case_number: case_number.clone(),
                    ..Default::default()
                },
                evidence_collection: Some(data),
                report_type: Some("evidence_collection".to_string()),
                ..Default::default()
            };
            let generator = state.generator.lock();
            generator
                .generate(&report, super::OutputFormat::Pdf, &output_path)
                .map_err(|e| e.to_string())?;
        }
        _ => {
            return Err(format!(
                "Unsupported export format: '{}'. Use pdf, csv, xlsx, html, or json.",
                format
            ))
        }
    }
    Ok(output_path)
}

fn build_evidence_collection_package(
    data: &super::types::EvidenceCollectionData,
    case_number: &str,
    source_app: &str,
) -> Result<MobileEvidenceCollectionPackage, String> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let project_id = uuid::Uuid::new_v4().to_string();
    let collection_id = uuid::Uuid::new_v4().to_string();
    let witnesses_json = if data.witnesses.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&data.witnesses).map_err(|e| e.to_string())?)
    };

    let items = data
        .collected_items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item_id = if item.id.is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                item.id.clone()
            };
            let item_number = if item.item_number.is_empty() {
                (index + 1).to_string()
            } else {
                item.item_number.clone()
            };
            let photo_refs_json = if item.photo_refs.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&item.photo_refs).map_err(|e| e.to_string())?)
            };

            Ok(DbCollectedItem {
                id: item_id,
                collection_id: collection_id.clone(),
                coc_item_id: None,
                evidence_file_id: None,
                source_id: None,
                source_ref_json: None,
                item_number,
                description: item.description.clone(),
                found_location: item.found_location.clone(),
                item_type: if item.item_type.is_empty() {
                    item.device_type.clone()
                } else {
                    item.item_type.clone()
                },
                make: item.make.clone(),
                model: item.model.clone(),
                serial_number: item.serial_number.clone(),
                condition: item.condition.clone(),
                packaging: item.packaging.clone(),
                packaging_type: None,
                packaging_detail: None,
                photo_refs_json,
                notes: item.notes.clone(),
                item_collection_datetime: item.item_collection_datetime.clone(),
                item_system_datetime: item.item_system_datetime.clone(),
                item_collecting_officer: item.item_collecting_officer.clone(),
                item_authorization: item.item_authorization.clone(),
                device_type: (!item.device_type.is_empty()).then(|| item.device_type.clone()),
                device_type_other: item.device_type_other.clone(),
                storage_interface: item.storage_interface.clone(),
                storage_interface_other: item.storage_interface_other.clone(),
                brand: item.brand.clone(),
                color: item.color.clone(),
                imei: item.imei.clone(),
                other_identifiers: item.other_identifiers.clone(),
                building: item.building.clone(),
                room: item.room.clone(),
                location_other: item.location_other.clone(),
                image_format: item.image_format.clone(),
                image_format_other: item.image_format_other.clone(),
                acquisition_method: item.acquisition_method.clone(),
                acquisition_method_other: item.acquisition_method_other.clone(),
                hash_algorithm: None,
                hash_value: None,
                hash_computed_at: None,
                storage_notes: item.storage_notes.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(MobileEvidenceCollectionPackage {
        export_version: EVIDENCE_COLLECTION_PACKAGE_VERSION.to_string(),
        exported_at: timestamp.clone(),
        source_app: source_app.to_string(),
        project: MobileProject {
            id: project_id,
            case_number: case_number.to_string(),
            case_title: case_number.to_string(),
            examiner_name: data.collecting_officer.clone(),
            organization: String::new(),
            created_at: timestamp.clone(),
            modified_at: timestamp.clone(),
            status: "active".to_string(),
        },
        collections: vec![MobileEvidenceCollectionPackageCollection {
            collection: DbEvidenceCollection {
                id: collection_id,
                case_number: case_number.to_string(),
                collection_date: data.collection_date.clone(),
                collection_location: data.collection_location.clone(),
                collecting_officer: data.collecting_officer.clone(),
                authorization: data.authorization.clone(),
                authorization_date: data.authorization_date.clone(),
                authorizing_authority: data.authorizing_authority.clone(),
                witnesses_json,
                documentation_notes: data.documentation_notes.clone(),
                conditions: data.conditions.clone(),
                status: "draft".to_string(),
                created_at: timestamp.clone(),
                modified_at: timestamp,
                item_count: items.len() as i64,
            },
            items,
        }],
        coc_items: vec![],
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // =========================================================================
    // get_output_formats
    // =========================================================================

    #[test]
    fn test_get_output_formats_returns_all_formats() {
        let formats = get_output_formats();
        assert_eq!(formats.len(), 5);
    }

    #[test]
    fn test_get_output_formats_contains_pdf() {
        let formats = get_output_formats();
        let pdf = formats.iter().find(|f| f.extension == "pdf").unwrap();
        assert_eq!(pdf.name, "PDF");
        assert!(pdf.supported);
        assert!(matches!(pdf.format, OutputFormat::Pdf));
    }

    #[test]
    fn test_get_output_formats_contains_docx() {
        let formats = get_output_formats();
        let docx = formats.iter().find(|f| f.extension == "docx").unwrap();
        assert_eq!(docx.name, "Word Document");
        assert!(docx.supported);
    }

    #[test]
    fn test_get_output_formats_contains_html() {
        let formats = get_output_formats();
        let html = formats.iter().find(|f| f.extension == "html").unwrap();
        assert_eq!(html.name, "HTML");
        assert!(html.supported);
    }

    #[test]
    fn test_get_output_formats_contains_markdown() {
        let formats = get_output_formats();
        let md = formats.iter().find(|f| f.extension == "md").unwrap();
        assert_eq!(md.name, "Markdown");
        assert!(md.supported);
    }

    #[test]
    fn test_get_output_formats_typst_support_matches_feature_flag() {
        let formats = get_output_formats();
        let typst = formats.iter().find(|f| f.extension == "typ").unwrap();
        assert_eq!(typst.name, "Typst");
        #[cfg(feature = "typst-reports")]
        assert!(typst.supported);
        #[cfg(not(feature = "typst-reports"))]
        assert!(!typst.supported);
    }

    // =========================================================================
    // export_report_json / import_report_json
    // =========================================================================

    #[test]
    fn test_export_report_json_produces_valid_json() {
        let report = ForensicReport::default();
        let json = export_report_json(report).unwrap();
        assert!(json.contains("metadata"));
        assert!(json.contains("case_info"));
        assert!(json.contains("examiner"));
    }

    #[test]
    fn test_import_report_json_roundtrip() {
        let original = ForensicReport::builder()
            .case_number("2026-001")
            .examiner_name("Jane Doe")
            .build()
            .unwrap();

        let json = export_report_json(original).unwrap();
        let imported = import_report_json(json).unwrap();

        assert_eq!(imported.case_info.case_number, "2026-001");
        assert_eq!(imported.examiner.name, "Jane Doe");
    }

    #[test]
    fn test_import_report_json_invalid_json_returns_error() {
        let result = import_report_json("not valid json".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_export_report_json_pretty_printed() {
        let report = ForensicReport::default();
        let json = export_report_json(report).unwrap();
        // Pretty-printed JSON has newlines
        assert!(json.contains('\n'));
    }

    // =========================================================================
    // parse_hash_algorithm
    // =========================================================================

    #[test]
    fn test_parse_hash_algorithm_md5() {
        assert!(matches!(parse_hash_algorithm("md5"), HashAlgorithm::MD5));
        assert!(matches!(parse_hash_algorithm("MD5"), HashAlgorithm::MD5));
    }

    #[test]
    fn test_parse_hash_algorithm_sha1() {
        assert!(matches!(parse_hash_algorithm("sha1"), HashAlgorithm::SHA1));
        assert!(matches!(parse_hash_algorithm("SHA-1"), HashAlgorithm::SHA1));
        assert!(matches!(parse_hash_algorithm("sha-1"), HashAlgorithm::SHA1));
    }

    #[test]
    fn test_parse_hash_algorithm_sha256() {
        assert!(matches!(
            parse_hash_algorithm("sha256"),
            HashAlgorithm::SHA256
        ));
        assert!(matches!(
            parse_hash_algorithm("SHA-256"),
            HashAlgorithm::SHA256
        ));
        assert!(matches!(
            parse_hash_algorithm("sha-256"),
            HashAlgorithm::SHA256
        ));
    }

    #[test]
    fn test_parse_hash_algorithm_sha512() {
        assert!(matches!(
            parse_hash_algorithm("sha512"),
            HashAlgorithm::SHA512
        ));
        assert!(matches!(
            parse_hash_algorithm("SHA-512"),
            HashAlgorithm::SHA512
        ));
    }

    #[test]
    fn test_parse_hash_algorithm_blake() {
        assert!(matches!(
            parse_hash_algorithm("blake2"),
            HashAlgorithm::Blake2b
        ));
        assert!(matches!(
            parse_hash_algorithm("blake2b"),
            HashAlgorithm::Blake2b
        ));
        assert!(matches!(
            parse_hash_algorithm("blake3"),
            HashAlgorithm::Blake3
        ));
    }

    #[test]
    fn test_parse_hash_algorithm_unknown_defaults_to_sha256() {
        assert!(matches!(
            parse_hash_algorithm("unknown"),
            HashAlgorithm::SHA256
        ));
        assert!(matches!(
            parse_hash_algorithm("crc32"),
            HashAlgorithm::SHA256
        ));
    }

    // =========================================================================
    // extract_evidence_from_containers
    // =========================================================================

    #[test]
    fn test_extract_evidence_empty_containers() {
        let result = extract_evidence_from_containers(vec![]).unwrap();
        assert!(result.is_empty());
    }

    fn make_container(container_type: &str, filename: &str) -> ContainerInfoInput {
        ContainerInfoInput {
            container_type: container_type.to_string(),
            path: format!("/evidence/{}", filename),
            filename: filename.to_string(),
            size: 100,
            case_number: None,
            evidence_number: None,
            examiner_name: None,
            description: None,
            notes: None,
            acquiry_date: None,
            model: None,
            serial_number: None,
            total_size: None,
            stored_hashes: None,
            computed_hash: None,
        }
    }

    #[test]
    fn test_extract_evidence_single_e01() {
        let mut container = make_container("e01", "disk.E01");
        container.description = Some("Suspect hard drive".to_string());
        container.model = Some("WD10EZEX".to_string());
        container.serial_number = Some("WD-ABC123".to_string());
        container.total_size = Some(500_000_000_000);
        container.stored_hashes = Some(vec![StoredHashInput {
            algorithm: "md5".to_string(),
            hash: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
            verified: Some(true),
        }]);

        let items = extract_evidence_from_containers(vec![container]).unwrap();
        assert_eq!(items.len(), 1);

        let item = &items[0];
        assert_eq!(item.evidence_id, "E001");
        assert_eq!(item.description, "Suspect hard drive");
        assert!(matches!(item.evidence_type, EvidenceType::ForensicImage));
        assert_eq!(item.model.as_deref(), Some("WD10EZEX"));
        assert_eq!(item.serial_number.as_deref(), Some("WD-ABC123"));
        assert_eq!(item.acquisition_hashes.len(), 1);
        assert_eq!(
            item.acquisition_hashes[0].value,
            "d41d8cd98f00b204e9800998ecf8427e"
        );
        assert!(item.image_info.is_some());
    }

    #[test]
    fn test_extract_evidence_ufed_type() {
        let items =
            extract_evidence_from_containers(vec![make_container("ufed", "phone.ufdr")]).unwrap();
        assert!(matches!(items[0].evidence_type, EvidenceType::MobilePhone));
    }

    #[test]
    fn test_extract_evidence_archive_type_is_other() {
        let items =
            extract_evidence_from_containers(vec![make_container("zip", "backup.zip")]).unwrap();
        assert!(matches!(items[0].evidence_type, EvidenceType::Other));
    }

    #[test]
    fn test_extract_evidence_multiple_containers_get_sequential_ids() {
        let items = extract_evidence_from_containers(vec![
            make_container("e01", "disk1.E01"),
            make_container("e01", "disk2.E01"),
            make_container("e01", "disk3.E01"),
        ])
        .unwrap();

        assert_eq!(items[0].evidence_id, "E001");
        assert_eq!(items[1].evidence_id, "E002");
        assert_eq!(items[2].evidence_id, "E003");
    }

    #[test]
    fn test_extract_evidence_description_falls_back_to_filename() {
        let items =
            extract_evidence_from_containers(vec![make_container("ad1", "logical.ad1")]).unwrap();
        assert_eq!(items[0].description, "logical.ad1");
    }

    #[test]
    fn test_extract_evidence_with_computed_hash() {
        let mut container = make_container("e01", "disk.E01");
        container.computed_hash = Some(StoredHashInput {
            algorithm: "sha256".to_string(),
            hash: "abcdef1234567890".to_string(),
            verified: Some(true),
        });

        let items = extract_evidence_from_containers(vec![container]).unwrap();
        assert_eq!(items[0].acquisition_hashes.len(), 1);
        assert_eq!(items[0].acquisition_hashes[0].value, "abcdef1234567890");
        assert!(items[0].acquisition_hashes[0].computed_at.is_some());
    }

    #[test]
    fn test_project_db_evidence_item_uses_hashes_and_artifact_count() {
        let file = DbEvidenceFile {
            id: "ev_1".to_string(),
            path: "/case/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "e01".to_string(),
            total_size: 1_048_576,
            segment_count: 2,
            discovered_at: "2026-02-16T10:00:00Z".to_string(),
            created: Some("2026-02-16T10:00:00Z".to_string()),
            modified: None,
        };
        let hashes = vec![DbProjectHash {
            id: "hash_1".to_string(),
            file_id: "ev_1".to_string(),
            source_id: None,
            source_ref_json: None,
            algorithm: "SHA-256".to_string(),
            hash_value: "abcdef123456".to_string(),
            computed_at: "2026-02-16T10:01:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        }];

        let item = evidence_item_from_project_db(0, &file, &hashes, 3);

        assert_eq!(item.evidence_id, "E001");
        assert!(matches!(item.evidence_type, EvidenceType::ForensicImage));
        assert_eq!(item.acquisition_hashes.len(), 1);
        assert_eq!(item.acquisition_hashes[0].value, "abcdef123456");
        assert_eq!(
            item.notes.as_deref(),
            Some("3 normalized artifact(s) extracted")
        );
        assert_eq!(item.image_info.as_ref().unwrap().segments, Some(2));
    }

    #[test]
    fn test_project_db_evidence_item_marks_negative_size_capacity_invalid() {
        let file = DbEvidenceFile {
            id: "ev_1".to_string(),
            path: "/case/evidence/disk.E01".to_string(),
            filename: "disk.E01".to_string(),
            container_type: "e01".to_string(),
            total_size: -1,
            segment_count: 1,
            discovered_at: "2026-02-16T10:00:00Z".to_string(),
            created: Some("2026-02-16T10:00:00Z".to_string()),
            modified: None,
        };

        let item = evidence_item_from_project_db(0, &file, &[], 0);

        assert_eq!(
            item.capacity.as_deref(),
            Some("invalid evidence total size: -1 bytes")
        );
        assert_eq!(item.image_info.as_ref().unwrap().total_size, 0);
    }

    #[test]
    fn test_hash_record_from_project_db_parses_timestamp() {
        let hash = DbProjectHash {
            id: "hash_1".to_string(),
            file_id: "ev_1".to_string(),
            source_id: None,
            source_ref_json: None,
            algorithm: "BLAKE3".to_string(),
            hash_value: "abc".to_string(),
            computed_at: "2026-02-16T10:01:00Z".to_string(),
            segment_index: None,
            segment_name: None,
            source: "computed".to_string(),
        };

        let record = hash_record_from_project_db("disk.E01", &hash);

        assert_eq!(record.item, "disk.E01");
        assert!(matches!(record.algorithm, HashAlgorithm::Blake3));
        assert!(record.computed_at.is_some());
    }

    #[test]
    fn test_hash_algorithm_summary_from_project_db_formats_report_fields() {
        let db_summary = DbHashAlgorithmSummary {
            algorithm: "SHA-256".to_string(),
            count: 4,
            evidence_file_count: 2,
            source_count: 3,
            latest_computed_at: Some("2026-02-16T10:04:00Z".to_string()),
        };

        let summary = report_hash_algorithm_summary_from_project_db(&db_summary);

        assert!(matches!(summary.algorithm, HashAlgorithm::SHA256));
        assert_eq!(summary.algorithm_label, "SHA-256");
        assert_eq!(summary.count, 4);
        assert_eq!(summary.evidence_file_count, 2);
        assert_eq!(summary.source_count, 3);
        assert_eq!(
            summary.latest_computed_at.as_deref(),
            Some("2026-02-16T10:04:00Z")
        );
    }

    #[test]
    fn test_verification_result_summary_from_project_db_formats_report_fields() {
        let db_summary = DbVerificationResultSummary {
            result: "mismatch".to_string(),
            count: 2,
            hash_count: 2,
            latest_verified_at: Some("2026-02-16T10:06:00Z".to_string()),
        };

        let summary = report_verification_result_summary_from_project_db(&db_summary);

        assert_eq!(summary.result, "mismatch");
        assert_eq!(summary.count, 2);
        assert_eq!(summary.hash_count, 2);
        assert_eq!(
            summary.latest_verified_at.as_deref(),
            Some("2026-02-16T10:06:00Z")
        );
    }

    fn make_artifact(id: &str, category: &str, name: &str) -> DbNormalizedArtifact {
        DbNormalizedArtifact {
            id: id.to_string(),
            evidence_file_id: Some("ev_1".to_string()),
            source_id: format!("ad1:/case/logical.ad1:/{}", name),
            source_ref_json:
                r#"{"kind":"containerEntry","containerPath":"/case/logical.ad1","entryPath":"/docs/notes.txt","containerType":"ad1"}"#
                    .to_string(),
            name: name.to_string(),
            extension: Some("txt".to_string()),
            size: 2048,
            mime_type: Some("text/plain".to_string()),
            type_description: "Text Document".to_string(),
            category: category.to_string(),
            confidence: "high".to_string(),
            is_text: true,
            content_preview: Some("preview text".to_string()),
            metadata_json: Some(
                r#"{"image.width":"640","image.height":480,"sourceId":"ad1:/case/logical.ad1:/docs/notes.txt"}"#
                    .to_string(),
            ),
            extracted_at: "2026-02-16T10:02:00Z".to_string(),
            extractor: "artifact_extract_source".to_string(),
        }
    }

    fn make_source_analysis(
        id: &str,
        category: Option<&str>,
        signature: Option<&str>,
    ) -> DbSourceAnalysisRecord {
        DbSourceAnalysisRecord {
            id: id.to_string(),
            evidence_file_id: Some("ev_1".to_string()),
            source_id: "ad1:/case/logical.ad1:/docs/notes.txt".to_string(),
            source_ref_json:
                r#"{"kind":"containerEntry","containerPath":"/case/logical.ad1","entryPath":"/docs/notes.txt","containerType":"ad1"}"#
                    .to_string(),
            total_size: 2048,
            offset: 0,
            bytes_analyzed: 512,
            magic_hex: "25 50 44 46".to_string(),
            signature_count: if signature.is_some() { 1 } else { 0 },
            primary_signature: signature.map(str::to_string),
            primary_mime_type: Some("application/pdf".to_string()),
            primary_category: category.map(str::to_string),
            entropy: 4.25,
            printable_ratio: 0.75,
            is_likely_text: true,
            ascii_preview: Some("%PDF".to_string()),
            signatures_json: Some(r#"[{"description":"PDF Document"}]"#.to_string()),
            entropy_windows_json: Some("[]".to_string()),
            histogram_json: Some("[0,1]".to_string()),
            indicators_json: Some(
                r#"[{"indicatorType":"email","value":"admin@example.com","offset":16,"length":17,"confidence":"medium"}]"#
                    .to_string(),
            ),
            analyzed_at: "2026-02-16T10:04:00Z".to_string(),
            analyzer: "core-source-analysis".to_string(),
        }
    }

    #[test]
    fn test_project_db_report_evidence_includes_unlinked_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("report-artifacts.ffxdb");
        let db = ProjectDatabase::open(&db_path).unwrap();

        db.upsert_evidence_file(&DbEvidenceFile {
            id: "ev_1".to_string(),
            path: "/case/logical.ad1".to_string(),
            filename: "logical.ad1".to_string(),
            container_type: "ad1".to_string(),
            total_size: 4096,
            segment_count: 1,
            discovered_at: "2026-02-16T10:00:00Z".to_string(),
            created: None,
            modified: None,
        })
        .unwrap();

        let linked = make_artifact("artifact_linked", "text", "linked.txt");
        db.upsert_artifact(&linked).unwrap();

        let mut unlinked = make_artifact("artifact_unlinked", "text", "unlinked.log");
        unlinked.evidence_file_id = None;
        unlinked.source_id = "/case/loose/unlinked.log".to_string();
        unlinked.source_ref_json =
            r#"{"kind":"localFile","path":"/case/loose/unlinked.log"}"#.to_string();
        unlinked.extracted_at = "2026-02-16T10:03:00Z".to_string();
        db.upsert_artifact(&unlinked).unwrap();

        db.upsert_source_analysis(&make_source_analysis(
            "analysis_1",
            Some("document"),
            Some("PDF Document"),
        ))
        .unwrap();
        db.insert_annotation(&DbAnnotation {
            id: "ann_hex_magic".to_string(),
            file_path: "ad1:/case/logical.ad1:/docs/report.pdf".to_string(),
            container_path: Some("/case/logical.ad1".to_string()),
            annotation_type: "hex-magic".to_string(),
            offset_start: Some(0),
            offset_end: Some(16),
            line_start: None,
            line_end: None,
            label: "Magic Bytes".to_string(),
            content: Some("Initial signature bytes: 25 50 44 46".to_string()),
            color: Some("#38bdf8".to_string()),
            created_by: "hex-viewer".to_string(),
            created_at: "2026-02-16T10:05:00Z".to_string(),
            modified_at: "2026-02-16T10:05:00Z".to_string(),
        })
        .unwrap();

        let evidence = project_db_report_evidence(&db).unwrap();

        assert_eq!(evidence.artifacts.len(), 2);
        assert!(evidence
            .artifacts
            .iter()
            .any(|artifact| artifact.id == "artifact_linked"));
        assert!(evidence
            .artifacts
            .iter()
            .any(|artifact| artifact.id == "artifact_unlinked"));
        assert_eq!(evidence.artifact_summaries.len(), 2);
        assert_eq!(evidence.artifact_categories.len(), 1);
        assert_eq!(evidence.artifact_categories[0].category, "text");
        assert_eq!(evidence.artifact_categories[0].count, 2);
        assert_eq!(
            evidence.evidence_items[0].notes.as_deref(),
            Some("1 normalized artifact(s) extracted")
        );
        assert!(evidence
            .artifact_evidence_summaries
            .iter()
            .any(|summary| summary.evidence_file_id.is_none() && summary.count == 1));
        assert_eq!(evidence.source_analyses.len(), 1);
        assert_eq!(evidence.source_analysis_summaries.len(), 1);
        assert_eq!(
            evidence.source_analysis_summaries[0]
                .primary_signature
                .as_deref(),
            Some("PDF Document")
        );
        assert_eq!(
            evidence.source_analysis_summaries[0].primary_category,
            "document"
        );
        assert_eq!(evidence.source_analysis_category_summaries.len(), 1);
        assert_eq!(
            evidence.source_analysis_category_summaries[0].category,
            "document"
        );
        assert_eq!(evidence.source_analysis_category_summaries[0].count, 1);
        assert_eq!(evidence.annotations.len(), 1);
        assert_eq!(evidence.annotations[0].annotation_type, "hex-magic");
    }

    #[test]
    fn test_artifact_summary_from_project_db_preserves_report_fields() {
        let artifact = make_artifact("artifact_1", "text", "notes.txt");

        let summary = artifact_summary_from_project_db(&artifact);

        assert_eq!(summary.id, "artifact_1");
        assert_eq!(summary.evidence_file_id.as_deref(), Some("ev_1"));
        assert_eq!(summary.name, "notes.txt");
        assert_eq!(summary.category, "text");
        assert_eq!(summary.type_description, "Text Document");
        assert_eq!(summary.mime_type.as_deref(), Some("text/plain"));
        assert_eq!(summary.size, 2048);
        assert_eq!(summary.size_display, "2.00 KB");
        assert_eq!(summary.preview.as_deref(), Some("preview text"));
        assert_eq!(
            summary
                .source_ref
                .as_ref()
                .and_then(|value| value.get("containerType"))
                .and_then(serde_json::Value::as_str),
            Some("ad1")
        );
        assert_eq!(
            summary.metadata.get("image.width").map(String::as_str),
            Some("640")
        );
        assert_eq!(
            summary.metadata.get("image.height").map(String::as_str),
            Some("480")
        );
        assert_eq!(summary.extractor, "artifact_extract_source");
        assert!(summary.is_text);
    }

    #[test]
    fn test_artifact_summary_handles_invalid_metadata_json() {
        let mut artifact = make_artifact("artifact_1", "text", "notes.txt");
        artifact.metadata_json = Some("{not-valid-json".to_string());
        artifact.source_ref_json = "{not-valid-json".to_string();

        let summary = artifact_summary_from_project_db(&artifact);

        assert!(summary
            .metadata
            .get("metadata.parseError")
            .is_some_and(|error| !error.is_empty()));
        assert_eq!(
            summary.metadata.get("metadata.raw").map(String::as_str),
            Some("{not-valid-json")
        );
        assert_eq!(
            summary
                .source_ref
                .as_ref()
                .and_then(|value| value.get("invalidSourceRef"))
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            summary
                .source_ref
                .as_ref()
                .and_then(|value| value.get("raw"))
                .and_then(serde_json::Value::as_str),
            Some("{not-valid-json")
        );
    }

    #[test]
    fn test_artifact_summary_marks_non_object_metadata_json() {
        let metadata = parse_artifact_metadata(Some("[1,2,3]"));

        assert_eq!(
            metadata.get("metadata.parseError").map(String::as_str),
            Some("artifact metadata JSON is not an object")
        );
        assert_eq!(
            metadata.get("metadata.raw").map(String::as_str),
            Some("[1,2,3]")
        );
    }

    #[test]
    fn test_artifact_summary_caps_metadata_entries_and_values() {
        let mut object = serde_json::Map::new();
        for index in 0..(MAX_REPORT_METADATA_ENTRIES + 25) {
            object.insert(
                format!("key-{index}"),
                serde_json::Value::String("m".repeat(MAX_REPORT_FIELD_CHARS + 25)),
            );
        }
        let metadata_json = serde_json::Value::Object(object).to_string();

        let metadata = parse_artifact_metadata(Some(&metadata_json));

        assert_eq!(metadata.len(), MAX_REPORT_METADATA_ENTRIES);
        assert!(metadata
            .values()
            .all(|value| value.chars().count() == MAX_REPORT_FIELD_CHARS));
        assert!(metadata
            .values()
            .all(|value| value.ends_with(REPORT_TRUNCATED_SUFFIX)));
    }

    #[test]
    fn test_bounded_report_artifacts_truncate_payload_fields_with_valid_json() {
        let mut artifact = make_artifact("artifact_1", "text", "notes.txt");
        artifact.content_preview = Some("p".repeat(MAX_REPORT_PREVIEW_CHARS + 25));
        artifact.source_ref_json = serde_json::json!({
            "kind": "containerEntry",
            "entryPath": "x".repeat(MAX_REPORT_JSON_CHARS + 25),
        })
        .to_string();
        artifact.metadata_json = Some(
            serde_json::json!({
                "source": "m".repeat(MAX_REPORT_JSON_CHARS + 25),
            })
            .to_string(),
        );

        let artifacts = bounded_report_artifacts(&[artifact]);

        assert_eq!(artifacts.len(), 1);
        let bounded = &artifacts[0];
        let preview = bounded.content_preview.as_deref().unwrap();
        assert_eq!(preview.chars().count(), MAX_REPORT_PREVIEW_CHARS);
        assert!(preview.ends_with(REPORT_TRUNCATED_SUFFIX));
        assert!(serde_json::from_str::<serde_json::Value>(&bounded.source_ref_json).is_ok());
        assert!(serde_json::from_str::<serde_json::Value>(
            bounded.metadata_json.as_deref().unwrap()
        )
        .is_ok());
    }

    #[test]
    fn test_artifact_summary_marks_negative_size_display_invalid() {
        let mut artifact = make_artifact("artifact_1", "text", "notes.txt");
        artifact.size = -8;

        let summary = artifact_summary_from_project_db(&artifact);

        assert_eq!(summary.size, -8);
        assert_eq!(summary.size_display, "invalid artifact size: -8 bytes");
    }

    #[test]
    fn test_artifact_category_summaries_counts_by_category() {
        let artifacts = vec![
            make_artifact("artifact_1", "text", "a.txt"),
            make_artifact("artifact_2", "text", "b.txt"),
            make_artifact("artifact_3", "image", "c.jpg"),
        ];

        let categories = artifact_category_summaries(&artifacts);

        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].category, "image");
        assert_eq!(categories[0].count, 1);
        assert_eq!(categories[1].category, "text");
        assert_eq!(categories[1].count, 2);
    }

    #[test]
    fn test_artifact_evidence_summary_from_project_db_formats_report_fields() {
        let db_summary = DbArtifactEvidenceSummary {
            evidence_file_id: Some("ev_1".to_string()),
            count: 3,
            total_size: 4096,
            text_count: 2,
            category_count: 2,
            latest_extracted_at: Some("2026-02-16T10:03:00Z".to_string()),
        };

        let summary = report_artifact_evidence_summary_from_project_db(&db_summary);

        assert_eq!(summary.evidence_file_id.as_deref(), Some("ev_1"));
        assert_eq!(summary.count, 3);
        assert_eq!(summary.total_size, 4096);
        assert_eq!(summary.total_size_display, "4.00 KB");
        assert_eq!(summary.text_count, 2);
        assert_eq!(summary.category_count, 2);
        assert_eq!(
            summary.latest_extracted_at.as_deref(),
            Some("2026-02-16T10:03:00Z")
        );
    }

    #[test]
    fn test_artifact_evidence_summary_marks_negative_total_size_display_invalid() {
        let db_summary = DbArtifactEvidenceSummary {
            evidence_file_id: Some("ev_1".to_string()),
            count: 3,
            total_size: -4096,
            text_count: 2,
            category_count: 2,
            latest_extracted_at: None,
        };

        let summary = report_artifact_evidence_summary_from_project_db(&db_summary);

        assert_eq!(summary.total_size, -4096);
        assert_eq!(
            summary.total_size_display,
            "invalid artifact total size: -4096 bytes"
        );
    }

    #[test]
    fn test_artifact_extractor_summary_from_project_db_formats_report_fields() {
        let db_summary = DbArtifactExtractorSummary {
            extractor: "core-artifact-extractor".to_string(),
            count: 5,
            total_size: 8192,
            text_count: 3,
            category_count: 4,
            evidence_file_count: 2,
            latest_extracted_at: Some("2026-02-16T10:04:00Z".to_string()),
        };

        let summary = report_artifact_extractor_summary_from_project_db(&db_summary);

        assert_eq!(summary.extractor, "core-artifact-extractor");
        assert_eq!(summary.count, 5);
        assert_eq!(summary.total_size, 8192);
        assert_eq!(summary.total_size_display, "8.00 KB");
        assert_eq!(summary.text_count, 3);
        assert_eq!(summary.category_count, 4);
        assert_eq!(summary.evidence_file_count, 2);
        assert_eq!(
            summary.latest_extracted_at.as_deref(),
            Some("2026-02-16T10:04:00Z")
        );
    }

    #[test]
    fn test_artifact_extractor_summary_marks_negative_total_size_display_invalid() {
        let db_summary = DbArtifactExtractorSummary {
            extractor: "core-artifact-extractor".to_string(),
            count: 5,
            total_size: -8192,
            text_count: 3,
            category_count: 4,
            evidence_file_count: 2,
            latest_extracted_at: None,
        };

        let summary = report_artifact_extractor_summary_from_project_db(&db_summary);

        assert_eq!(summary.total_size, -8192);
        assert_eq!(
            summary.total_size_display,
            "invalid artifact total size: -8192 bytes"
        );
    }

    #[test]
    fn test_source_analysis_summary_from_project_db_formats_report_fields() {
        let record = make_source_analysis("analysis_1", Some("document"), Some("PDF Document"));

        let summary = source_analysis_summary_from_project_db(&record);

        assert_eq!(summary.id, "analysis_1");
        assert_eq!(summary.evidence_file_id.as_deref(), Some("ev_1"));
        assert_eq!(summary.source_id, "ad1:/case/logical.ad1:/docs/notes.txt");
        assert_eq!(
            summary
                .source_ref
                .as_ref()
                .and_then(|value| value.get("containerType"))
                .and_then(serde_json::Value::as_str),
            Some("ad1")
        );
        assert_eq!(summary.total_size_display, "2.00 KB");
        assert_eq!(summary.bytes_analyzed_display, "512 bytes");
        assert_eq!(summary.primary_signature.as_deref(), Some("PDF Document"));
        assert_eq!(
            summary.primary_mime_type.as_deref(),
            Some("application/pdf")
        );
        assert_eq!(summary.primary_category, "document");
        assert!(summary.is_likely_text);
        assert_eq!(summary.preview.as_deref(), Some("%PDF"));
    }

    #[test]
    fn test_source_analysis_summary_marks_negative_byte_displays_invalid() {
        let mut record = make_source_analysis("analysis_1", Some("document"), Some("PDF Document"));
        record.total_size = -2048;
        record.bytes_analyzed = -512;

        let summary = source_analysis_summary_from_project_db(&record);

        assert_eq!(summary.total_size, -2048);
        assert_eq!(
            summary.total_size_display,
            "invalid source total size: -2048 bytes"
        );
        assert_eq!(summary.bytes_analyzed, -512);
        assert_eq!(
            summary.bytes_analyzed_display,
            "invalid bytes analyzed: -512 bytes"
        );
    }

    #[test]
    fn test_source_analysis_indicators_keep_valid_entries_when_some_are_malformed() {
        let indicators = parse_source_indicators(Some(
            r#"[
                {"indicatorType":"email","value":"admin@example.com","offset":16,"length":17,"confidence":"medium"},
                {"indicatorType":"url","value":"https://example.test","offset":-1,"length":20,"confidence":"low"},
                {"indicatorType":"empty","value":"","offset":32,"length":0,"confidence":"low"},
                {"indicatorType":"ip","value":"192.0.2.1","offset":64,"length":9,"confidence":"high"}
            ]"#,
        ));

        assert_eq!(indicators.len(), 2);
        assert_eq!(indicators[0].indicator_type, "email");
        assert_eq!(indicators[0].offset, 16);
        assert_eq!(indicators[0].length, 17);
        assert_eq!(indicators[1].indicator_type, "ip");
        assert_eq!(indicators[1].offset, 64);
    }

    #[test]
    fn test_source_analysis_indicators_reject_invalid_json() {
        assert!(parse_source_indicators(Some("{not-json")).is_empty());
        assert!(parse_source_indicators(None).is_empty());
    }

    #[test]
    fn test_source_analysis_indicators_cap_count_and_values() {
        let mut indicators = Vec::new();
        for index in 0..(MAX_REPORT_SOURCE_INDICATORS + 10) {
            indicators.push(serde_json::json!({
                "indicatorType": "email",
                "value": format!("{}@example.test", "v".repeat(MAX_REPORT_INDICATOR_VALUE_CHARS + index + 1)),
                "offset": index,
                "length": 17,
                "confidence": "medium"
            }));
        }
        let indicators_json = serde_json::to_string(&indicators).unwrap();

        let parsed = parse_source_indicators(Some(&indicators_json));

        assert_eq!(parsed.len(), MAX_REPORT_SOURCE_INDICATORS);
        assert_eq!(
            parsed[0].value.chars().count(),
            MAX_REPORT_INDICATOR_VALUE_CHARS
        );
        assert!(parsed[0].value.ends_with(REPORT_TRUNCATED_SUFFIX));
    }

    #[test]
    fn test_bounded_report_source_analyses_truncate_raw_payload_fields() {
        let mut record = make_source_analysis("analysis_1", Some("document"), Some("PDF Document"));
        record.ascii_preview = Some("a".repeat(MAX_REPORT_PREVIEW_CHARS + 25));
        record.indicators_json = Some(
            serde_json::json!([{
                "indicatorType": "email",
                "value": "i".repeat(MAX_REPORT_JSON_CHARS + 25),
                "offset": 1,
                "length": 17,
                "confidence": "medium"
            }])
            .to_string(),
        );

        let records = bounded_report_source_analyses(&[record]);

        let preview = records[0].ascii_preview.as_deref().unwrap();
        assert_eq!(preview.chars().count(), MAX_REPORT_PREVIEW_CHARS);
        assert!(preview.ends_with(REPORT_TRUNCATED_SUFFIX));
        assert!(serde_json::from_str::<serde_json::Value>(
            records[0].indicators_json.as_deref().unwrap()
        )
        .is_ok());
    }

    #[test]
    fn test_source_analysis_summary_defaults_unknown_category() {
        let record = make_source_analysis("analysis_1", None, None);

        let summary = source_analysis_summary_from_project_db(&record);

        assert_eq!(summary.primary_category, "unknown");
        assert_eq!(summary.signature_count, 0);
        assert!(summary.primary_signature.is_none());
    }

    #[test]
    fn test_source_analysis_category_summary_from_project_db_formats_report_fields() {
        let db_summary = DbSourceAnalysisCategorySummary {
            category: "document".to_string(),
            count: 4,
            evidence_file_count: 2,
            avg_entropy: 4.75,
            text_like_count: 3,
            latest_analyzed_at: Some("2026-02-16T10:04:00Z".to_string()),
        };

        let summary = report_source_analysis_category_summary_from_project_db(&db_summary);

        assert_eq!(summary.category, "document");
        assert_eq!(summary.count, 4);
        assert_eq!(summary.evidence_file_count, 2);
        assert_eq!(summary.avg_entropy, 4.75);
        assert_eq!(summary.text_like_count, 3);
        assert_eq!(
            summary.latest_analyzed_at.as_deref(),
            Some("2026-02-16T10:04:00Z")
        );
    }

    #[test]
    fn test_bounded_report_annotations_cap_rows_and_content() {
        let annotations: Vec<DbAnnotation> = (0..(MAX_REPORT_DB_DETAIL_ROWS as usize + 1))
            .map(|index| DbAnnotation {
                id: format!("ann_{index}"),
                file_path: format!("/case/file-{index}.bin"),
                container_path: None,
                annotation_type: "hex-review".to_string(),
                offset_start: Some(0),
                offset_end: Some(16),
                line_start: None,
                line_end: None,
                label: "Magic Bytes".to_string(),
                content: Some("c".repeat(MAX_REPORT_PREVIEW_CHARS + 25)),
                color: Some("#38bdf8".to_string()),
                created_by: "hex-viewer".to_string(),
                created_at: "2026-02-16T10:05:00Z".to_string(),
                modified_at: "2026-02-16T10:05:00Z".to_string(),
            })
            .collect();

        let bounded = bounded_report_annotations(&annotations);

        assert_eq!(bounded.len(), MAX_REPORT_DB_DETAIL_ROWS as usize);
        let content = bounded[0].content.as_deref().unwrap();
        assert_eq!(content.chars().count(), MAX_REPORT_PREVIEW_CHARS);
        assert!(content.ends_with(REPORT_TRUNCATED_SUFFIX));
    }

    #[test]
    fn test_extract_evidence_image_info_populated() {
        let mut container = make_container("e01", "disk.E01");
        container.size = 1_000_000;
        container.total_size = Some(500_000_000_000);

        let items = extract_evidence_from_containers(vec![container]).unwrap();
        let info = items[0].image_info.as_ref().unwrap();
        assert_eq!(info.format, "e01");
        assert_eq!(info.file_names, vec!["disk.E01"]);
        assert_eq!(info.total_size, 500_000_000_000);
        assert_eq!(
            info.acquisition_tool.as_deref(),
            Some("FFX - Forensic File Xplorer")
        );
    }

    // =========================================================================
    // create_evidence_from_container
    // =========================================================================

    #[test]
    fn test_create_evidence_from_container_uses_custom_id() {
        let mut container = make_container("ad1", "test.ad1");
        container.description = Some("Test container".to_string());

        let item = create_evidence_from_container(container, "CUSTOM-001".to_string()).unwrap();
        assert_eq!(item.evidence_id, "CUSTOM-001");
        assert_eq!(item.description, "Test container");
    }

    // =========================================================================
    // get_report_template
    // =========================================================================

    #[test]
    fn test_get_report_template_computer() {
        let report = get_report_template("computer".to_string());
        let methodology = report.methodology.unwrap();
        assert!(methodology.contains("forensically sound"));
        assert!(methodology.contains("write-blocking"));
    }

    #[test]
    fn test_get_report_template_mobile() {
        let report = get_report_template("mobile".to_string());
        let methodology = report.methodology.unwrap();
        assert!(methodology.contains("mobile device"));
        assert!(methodology.contains("airplane mode"));
    }

    #[test]
    fn test_get_report_template_network() {
        let report = get_report_template("network".to_string());
        let methodology = report.methodology.unwrap();
        assert!(methodology.contains("network forensic"));
        assert!(methodology.contains("Packet capture"));
    }

    #[test]
    fn test_get_report_template_unknown_type_uses_generic() {
        let report = get_report_template("other".to_string());
        let methodology = report.methodology.unwrap();
        assert!(methodology.contains("forensically sound"));
        assert!(!methodology.contains("write-blocking"));
        assert!(!methodology.contains("mobile device"));
    }

    #[test]
    fn test_get_report_template_has_metadata() {
        let report = get_report_template("computer".to_string());
        assert!(report
            .metadata
            .title
            .starts_with("Forensic Examination Report"));
        assert_eq!(report.case_info.case_number, "");
        assert_eq!(report.examiner.name, "");
        assert!(report.methodology.is_some());
    }

    // =========================================================================
    // FormatInfo serialization
    // =========================================================================

    #[test]
    fn test_format_info_serialization() {
        let info = FormatInfo {
            format: OutputFormat::Pdf,
            name: "PDF".to_string(),
            description: "Portable Document Format".to_string(),
            extension: "pdf".to_string(),
            supported: true,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"PDF\""));
        assert!(json.contains("\"supported\":true"));
    }

    // =========================================================================
    // ContainerInfoInput / StoredHashInput serialization
    // =========================================================================

    #[test]
    fn test_container_info_input_deserialization() {
        let json = r#"{
            "container_type": "e01",
            "path": "/test.E01",
            "filename": "test.E01",
            "size": 1000,
            "case_number": null,
            "evidence_number": null,
            "examiner_name": null,
            "description": null,
            "notes": null,
            "acquiry_date": null,
            "model": null,
            "serial_number": null,
            "total_size": null,
            "stored_hashes": null,
            "computed_hash": null
        }"#;
        let parsed: ContainerInfoInput = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.container_type, "e01");
        assert_eq!(parsed.filename, "test.E01");
        assert_eq!(parsed.size, 1000);
    }

    #[test]
    fn test_stored_hash_input_roundtrip() {
        let input = StoredHashInput {
            algorithm: "sha256".to_string(),
            hash: "abc123".to_string(),
            verified: Some(true),
        };
        let json = serde_json::to_string(&input).unwrap();
        let parsed: StoredHashInput = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.algorithm, "sha256");
        assert_eq!(parsed.hash, "abc123");
        assert_eq!(parsed.verified, Some(true));
    }

    // =========================================================================
    // Evidence type mapping
    // =========================================================================

    #[test]
    fn test_evidence_type_mapping_forensic_formats() {
        let forensic_types = [
            "e01", "ex01", "ewf", "l01", "lx01", "ad1", "raw", "dd", "img",
        ];
        for t in &forensic_types {
            let items =
                extract_evidence_from_containers(vec![make_container(t, &format!("test.{}", t))])
                    .unwrap();
            assert!(
                matches!(items[0].evidence_type, EvidenceType::ForensicImage),
                "Expected ForensicImage for type '{}', got {:?}",
                t,
                items[0].evidence_type
            );
        }
    }

    #[test]
    fn test_evidence_type_mapping_mobile_formats() {
        let mobile_types = ["ufed", "ufdx", "ufd", "ufdr"];
        for t in &mobile_types {
            let items =
                extract_evidence_from_containers(vec![make_container(t, &format!("test.{}", t))])
                    .unwrap();
            assert!(
                matches!(items[0].evidence_type, EvidenceType::MobilePhone),
                "Expected MobilePhone for type '{}', got {:?}",
                t,
                items[0].evidence_type
            );
        }
    }
}
