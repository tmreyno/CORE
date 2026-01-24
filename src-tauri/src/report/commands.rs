// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for report generation
//!
//! These commands expose the report generation functionality to the frontend.

use tauri::State;
use parking_lot::Mutex;

use super::{
    ForensicReport, OutputFormat, ReportGenerator,
    types::*,
};
use crate::common::hex::format_size_compact;

/// State wrapper for the report generator
pub struct ReportState {
    generator: Mutex<ReportGenerator>,
}

impl ReportState {
    pub fn new() -> Result<Self, String> {
        let generator = ReportGenerator::new()
            .map_err(|e| e.to_string())?;
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
                generator: Mutex::new(ReportGenerator::new()
                    .expect("Report generator fallback also failed - fonts may be missing")),
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
    
    generator
        .template_engine()
        .render_html(&report)
        .map_err(|e| e.to_string())
}

/// Get available output formats
#[tauri::command]
pub fn get_output_formats() -> Vec<FormatInfo> {
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
            description: "Microsoft Word format - Best for editing and court submissions".to_string(),
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
            description: "Modern typesetting format - Coming soon".to_string(),
            extension: "typ".to_string(),
            supported: false,
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

/// Create a new empty report
#[tauri::command]
pub fn create_new_report(
    case_number: String,
    examiner_name: String,
) -> Result<ForensicReport, String> {
    ForensicReport::builder()
        .case_number(&case_number)
        .examiner_name(&examiner_name)
        .add_tool(ToolInfo {
            name: "FFX - Forensic File Xplorer".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            vendor: Some("FFX Team".to_string()),
            purpose: Some("Forensic image analysis and report generation".to_string()),
        })
        .build()
        .map_err(|e| e.to_string())
}

/// Validate a report
#[tauri::command]
pub fn validate_report(report: ForensicReport) -> Result<(), Vec<String>> {
    report.validate()
}

/// Add an evidence item to a report
#[tauri::command]
pub fn add_evidence_to_report(
    mut report: ForensicReport,
    evidence: EvidenceItem,
) -> ForensicReport {
    report.evidence_items.push(evidence);
    report
}

/// Add a finding to a report
#[tauri::command]
pub fn add_finding_to_report(
    mut report: ForensicReport,
    finding: Finding,
) -> ForensicReport {
    report.findings.push(finding);
    report
}

/// Export report to JSON (for saving/loading)
#[tauri::command]
pub fn export_report_json(report: ForensicReport) -> Result<String, String> {
    serde_json::to_string_pretty(&report)
        .map_err(|e| e.to_string())
}

/// Import report from JSON
#[tauri::command]
pub fn import_report_json(json: String) -> Result<ForensicReport, String> {
    serde_json::from_str(&json)
        .map_err(|e| e.to_string())
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
            acquisition_date: container.acquiry_date.as_ref().and_then(|d| 
                chrono::DateTime::parse_from_rfc3339(d).ok().map(|dt| dt.with_timezone(&chrono::Utc))
            ),
        });
        
        // Create evidence item
        let evidence_item = EvidenceItem {
            evidence_id: format!("E{:03}", index + 1),
            description: container.description.clone().unwrap_or_else(|| container.filename.clone()),
            evidence_type,
            make: None,
            model: container.model.clone(),
            serial_number: container.serial_number.clone(),
            capacity: container.total_size.or(Some(container.size)).map(format_size_compact),
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
    let mut item = items.into_iter().next().ok_or("Failed to create evidence")?;
    item.evidence_id = evidence_id;
    Ok(item)
}

/// Get a report template for different investigation types
#[tauri::command]
pub fn get_report_template(investigation_type: String) -> ForensicReport {
    let mut builder = ForensicReport::builder()
        .case_number("")
        .examiner_name("");
    
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
    
    // Build with minimal required fields
    builder.build().unwrap_or_else(|_| {
        // Return a truly minimal report if builder fails
        ForensicReport {
            metadata: ReportMetadata {
                title: "Forensic Examination Report".to_string(),
                report_number: "".to_string(),
                version: "1.0".to_string(),
                classification: Classification::Confidential,
                generated_at: chrono::Utc::now(),
                generated_by: "FFX Forensic File Xplorer".to_string(),
            },
            case_info: CaseInfo::default(),
            examiner: ExaminerInfo::default(),
            executive_summary: None,
            scope: None,
            methodology: Some(methodology.to_string()),
            evidence_items: vec![],
            chain_of_custody: vec![],
            findings: vec![],
            timeline: vec![],
            hash_records: vec![],
            tools: vec![],
            conclusions: None,
            appendices: vec![],
            signatures: vec![],
            notes: None,
        }
    })
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
                id: "ollama".to_string(),
                name: "Ollama (Local)".to_string(),
                description: "Run LLMs locally - requires Ollama installed".to_string(),
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
            AiProviderInfo {
                id: "openai".to_string(),
                name: "OpenAI".to_string(),
                description: "Cloud-based GPT models - requires API key".to_string(),
                requires_api_key: true,
                default_model: "gpt-4o-mini".to_string(),
                available_models: vec![
                    "gpt-4o".to_string(),
                    "gpt-4o-mini".to_string(),
                    "gpt-4-turbo".to_string(),
                    "gpt-3.5-turbo".to_string(),
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
