// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Test example for report generation
//!
//! Run with: cargo run --example test_report

use chrono::Utc;
use std::path::PathBuf;

// Import from the library
use ffx_check_lib::report::{
    types::*,
    OutputFormat,
    ReportGenerator,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Forensic Report Generation Test ===\n");

    // Create a sample forensic report
    let report = create_sample_report();
    
    println!("Created report: {}", report.metadata.title);
    println!("Case: {} - {}", report.case_info.case_number, 
             report.case_info.case_name.as_deref().unwrap_or("Unnamed"));
    println!("Examiner: {}", report.examiner.name);
    println!("Findings: {}", report.findings.len());
    println!("Evidence Items: {}", report.evidence_items.len());
    println!("Timeline Events: {}", report.timeline.len());
    println!();

    // Create output directory
    let output_dir = PathBuf::from("./test_output");
    std::fs::create_dir_all(&output_dir)?;

    // Initialize the report generator
    let generator = ReportGenerator::new()?;

    // Generate HTML report
    println!("Generating HTML report...");
    let html_path = output_dir.join("test_report.html");
    match generator.generate(&report, OutputFormat::Html, &html_path) {
        Ok(_) => println!("  ✓ HTML saved to: {}", html_path.display()),
        Err(e) => println!("  ✗ HTML failed: {}", e),
    }

    // Generate Markdown report
    println!("Generating Markdown report...");
    let md_path = output_dir.join("test_report.md");
    match generator.generate(&report, OutputFormat::Markdown, &md_path) {
        Ok(_) => println!("  ✓ Markdown saved to: {}", md_path.display()),
        Err(e) => println!("  ✗ Markdown failed: {}", e),
    }

    // Generate PDF report
    println!("Generating PDF report...");
    let pdf_path = output_dir.join("test_report.pdf");
    match generator.generate(&report, OutputFormat::Pdf, &pdf_path) {
        Ok(_) => println!("  ✓ PDF saved to: {}", pdf_path.display()),
        Err(e) => println!("  ✗ PDF failed: {}", e),
    }

    // Generate DOCX report
    println!("Generating DOCX report...");
    let docx_path = output_dir.join("test_report.docx");
    match generator.generate(&report, OutputFormat::Docx, &docx_path) {
        Ok(_) => println!("  ✓ DOCX saved to: {}", docx_path.display()),
        Err(e) => println!("  ✗ DOCX failed: {}", e),
    }

    // Generate Typst report (if feature enabled)
    println!("Generating Typst report...");
    let typst_path = output_dir.join("test_report.typ");
    match generator.generate(&report, OutputFormat::Typst, &typst_path) {
        Ok(_) => {
            println!("  ✓ Typst saved to: {}", typst_path.display());
            println!("    To compile to PDF: typst compile {} test_output/test_report_typst.pdf", typst_path.display());
        }
        Err(e) => println!("  ✗ Typst failed: {} (requires --features typst-reports)", e),
    }

    println!("\n=== Test Complete ===");
    println!("Output files are in: {}", output_dir.display());

    Ok(())
}

/// Create a sample forensic report for testing
fn create_sample_report() -> ForensicReport {
    ForensicReport {
        metadata: ReportMetadata {
            title: "Digital Forensic Examination Report".to_string(),
            report_number: "FR-2026-0001".to_string(),
            version: "1.0".to_string(),
            classification: Classification::LawEnforcementSensitive,
            generated_at: Utc::now(),
            generated_by: "FFX Forensic Toolkit".to_string(),
        },
        case_info: CaseInfo {
            case_number: "2026-CF-00123".to_string(),
            case_name: Some("State v. John Doe".to_string()),
            agency: Some("Metro Police Department".to_string()),
            requestor: Some("Det. Jane Smith".to_string()),
            request_date: Some(Utc::now()),
            exam_start_date: Some(Utc::now()),
            exam_end_date: None,
            investigation_type: Some("Fraud Investigation".to_string()),
            description: Some("Digital evidence examination pursuant to Search Warrant #2026-SW-456".to_string()),
        },
        examiner: ExaminerInfo {
            name: "Alex Johnson".to_string(),
            title: Some("Senior Digital Forensic Examiner".to_string()),
            organization: Some("Metro Police Forensic Lab".to_string()),
            email: Some("ajohnson@metro.gov".to_string()),
            phone: Some("(555) 123-4567".to_string()),
            certifications: vec![
                "EnCE (EnCase Certified Examiner)".to_string(),
                "GCFE (GIAC Certified Forensic Examiner)".to_string(),
                "ACE (AccessData Certified Examiner)".to_string(),
            ],
            badge_number: Some("F-1234".to_string()),
        },
        executive_summary: Some(
            "This report documents the forensic examination of digital evidence seized \
            pursuant to Search Warrant #2026-SW-456. The examination revealed several \
            artifacts of evidentiary value including documents, communications, and \
            internet history relevant to the investigation. Key findings include \
            evidence of document manipulation and communication records between \
            subjects of interest.".to_string()
        ),
        scope: Some(
            "The scope of this examination was limited to the recovery and analysis of \
            digital artifacts from the submitted evidence items. This examination focused \
            on document recovery, communication analysis, and timeline reconstruction.".to_string()
        ),
        methodology: Some(
            "Standard forensic methodology was employed throughout this examination:\n\
            1. Evidence intake and chain of custody documentation\n\
            2. Forensic imaging using write-blocking hardware\n\
            3. Hash verification of forensic images\n\
            4. Systematic artifact extraction and analysis\n\
            5. Documentation and report generation".to_string()
        ),
        evidence_items: vec![
            EvidenceItem {
                evidence_id: "E001".to_string(),
                description: "Dell Latitude laptop computer".to_string(),
                evidence_type: EvidenceType::Laptop,
                make: Some("Dell".to_string()),
                model: Some("Latitude 5540".to_string()),
                serial_number: Some("SN-DELL-2024-001".to_string()),
                capacity: Some("512 GB SSD".to_string()),
                condition: Some("Good condition, powered off when received".to_string()),
                received_date: Some(Utc::now()),
                submitted_by: Some("Det. Jane Smith".to_string()),
                acquisition_hashes: vec![
                    HashRecord {
                        item: "E001 - Full disk".to_string(),
                        algorithm: HashAlgorithm::MD5,
                        value: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
                        computed_at: Some(Utc::now()),
                        verified: Some(true),
                    },
                    HashRecord {
                        item: "E001 - Full disk".to_string(),
                        algorithm: HashAlgorithm::SHA256,
                        value: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
                        computed_at: Some(Utc::now()),
                        verified: Some(true),
                    },
                ],
                image_info: Some(ImageInfo {
                    format: "E01".to_string(),
                    file_names: vec!["E001_Dell_Latitude.E01".to_string()],
                    total_size: 512_000_000_000,
                    segments: Some(10),
                    compression: Some("Best".to_string()),
                    acquisition_tool: Some("FTK Imager 4.7".to_string()),
                    acquisition_date: Some(Utc::now()),
                }),
                notes: Some("Laptop was powered off when received. Battery at 45%.".to_string()),
                verification_hashes: vec![],
                acquisition_method: Some("Physical acquisition".to_string()),
                acquisition_tool: Some("FTK Imager 4.7".to_string()),
            },
            EvidenceItem {
                evidence_id: "E002".to_string(),
                description: "Samsung Galaxy S23 smartphone".to_string(),
                evidence_type: EvidenceType::MobilePhone,
                make: Some("Samsung".to_string()),
                model: Some("Galaxy S23".to_string()),
                serial_number: Some("IMEI-123456789012345".to_string()),
                capacity: Some("256 GB".to_string()),
                condition: Some("Screen intact, device locked".to_string()),
                received_date: Some(Utc::now()),
                submitted_by: Some("Det. Jane Smith".to_string()),
                acquisition_hashes: vec![],
                image_info: None,
                notes: Some("Device was in airplane mode. PIN lock enabled.".to_string()),
                verification_hashes: vec![],
                acquisition_method: Some("Logical extraction".to_string()),
                acquisition_tool: Some("Cellebrite UFED".to_string()),
            },
        ],
        chain_of_custody: vec![
            CustodyRecord {
                evidence_id: "E001".to_string(),
                timestamp: Utc::now(),
                released_by: "Det. Jane Smith".to_string(),
                received_by: "Alex Johnson".to_string(),
                purpose: Some("Forensic examination".to_string()),
                location: Some("Forensic Lab - Evidence Intake".to_string()),
                notes: None,
            },
        ],
        findings: vec![
            Finding {
                finding_id: "F001".to_string(),
                title: "Deleted Financial Documents Recovered".to_string(),
                category: FindingCategory::DeletedData,
                severity: FindingSeverity::High,
                description: "Analysis of unallocated space revealed 47 deleted Microsoft Excel \
                    spreadsheets containing financial records. These documents appear to have \
                    been deleted on 2025-12-15, approximately one week before the search warrant \
                    was executed. Document metadata indicates they were created by user 'JDoe'.".to_string(),
                supporting_evidence: vec!["E001".to_string()],
                related_files: vec![
                    "/Users/JDoe/Documents/Financials/Q4_2025_Report.xlsx".to_string(),
                    "/Users/JDoe/Documents/Financials/Transactions_Dec.xlsx".to_string(),
                ],
                timestamps: vec![Utc::now()],
                exhibits: vec![],
                notes: Some("Files recovered using signature-based carving from sectors 0x1A2B3C - 0x1A5F00".to_string()),
            },
            Finding {
                finding_id: "F002".to_string(),
                title: "Encrypted Communication Application".to_string(),
                category: FindingCategory::Communication,
                severity: FindingSeverity::Medium,
                description: "Signal messaging application was installed on both the laptop and \
                    mobile device. Message database was recovered from the mobile device backup. \
                    Analysis revealed communications with 3 contacts during the relevant time period.".to_string(),
                supporting_evidence: vec!["E001".to_string(), "E002".to_string()],
                related_files: vec![],
                timestamps: vec![],
                exhibits: vec![],
                notes: None,
            },
            Finding {
                finding_id: "F003".to_string(),
                title: "Browser History Analysis".to_string(),
                category: FindingCategory::InternetHistory,
                severity: FindingSeverity::Low,
                description: "Chrome browser history shows searches related to 'how to permanently \
                    delete files' and 'forensic file recovery' on 2025-12-14, the day before the \
                    deleted documents were removed.".to_string(),
                supporting_evidence: vec!["E001".to_string()],
                related_files: vec![
                    "/Users/JDoe/AppData/Local/Google/Chrome/User Data/Default/History".to_string(),
                ],
                timestamps: vec![Utc::now()],
                exhibits: vec![],
                notes: None,
            },
        ],
        timeline: vec![
            TimelineEvent {
                timestamp: Utc::now(),
                timestamp_type: "File Creation".to_string(),
                description: "Q4 Financial Report created".to_string(),
                source: "NTFS $MFT".to_string(),
                artifact: Some("Q4_2025_Report.xlsx".to_string()),
                evidence_id: Some("E001".to_string()),
                significance: Some("Original document creation".to_string()),
            },
            TimelineEvent {
                timestamp: Utc::now(),
                timestamp_type: "Web Search".to_string(),
                description: "Search: 'how to permanently delete files'".to_string(),
                source: "Chrome History".to_string(),
                artifact: None,
                evidence_id: Some("E001".to_string()),
                significance: Some("Evidence of anti-forensic intent".to_string()),
            },
            TimelineEvent {
                timestamp: Utc::now(),
                timestamp_type: "File Deletion".to_string(),
                description: "47 Excel files deleted from Documents folder".to_string(),
                source: "NTFS $MFT / $UsnJrnl".to_string(),
                artifact: None,
                evidence_id: Some("E001".to_string()),
                significance: Some("Mass deletion of financial records".to_string()),
            },
        ],
        hash_records: vec![
            HashRecord {
                item: "E001 - Forensic Image".to_string(),
                algorithm: HashAlgorithm::MD5,
                value: "d41d8cd98f00b204e9800998ecf8427e".to_string(),
                computed_at: Some(Utc::now()),
                verified: Some(true),
            },
            HashRecord {
                item: "E001 - Forensic Image".to_string(),
                algorithm: HashAlgorithm::SHA256,
                value: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
                computed_at: Some(Utc::now()),
                verified: Some(true),
            },
        ],
        tools: vec![
            ToolInfo {
                name: "FTK Imager".to_string(),
                version: "4.7.1.2".to_string(),
                vendor: Some("Exterro".to_string()),
                purpose: Some("Forensic imaging and hash verification".to_string()),
            },
            ToolInfo {
                name: "EnCase Forensic".to_string(),
                version: "23.4".to_string(),
                vendor: Some("OpenText".to_string()),
                purpose: Some("Evidence analysis and artifact extraction".to_string()),
            },
            ToolInfo {
                name: "FFX Forensic Toolkit".to_string(),
                version: "0.1.0".to_string(),
                vendor: Some("CORE Project".to_string()),
                purpose: Some("Report generation and timeline analysis".to_string()),
            },
        ],
        conclusions: Some(
            "Based on the forensic examination of the submitted evidence, the following \
            conclusions are supported:\n\n\
            1. Financial documents were systematically deleted from the laptop computer \
               approximately one week before the search warrant was executed.\n\n\
            2. Internet search history indicates the user researched file deletion methods \
               the day before the documents were removed.\n\n\
            3. Encrypted communication applications were in use on both devices during \
               the relevant time period.\n\n\
            These findings are presented for investigative purposes and are subject to \
            further analysis as needed.".to_string()
        ),
        appendices: vec![],
        signatures: vec![],
        notes: None,
    }
}
