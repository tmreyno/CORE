// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Test parser.rs parsing
use ffx_check_lib::ewf::parser;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let path = std::env::args()
        .nth(1)
        .unwrap_or("/Users/terryreynolds/Downloads/4Dell Latitude CPi.E01".to_string());
    println!("Testing parser for: {}", path);

    match parser::parse_ewf_file(&path) {
        Ok(info) => {
            println!("Success!");
            println!("Sections found:");
            for s in &info.sections {
                println!(
                    "  {} at offset {} ({} bytes)",
                    s.section_type, s.file_offset, s.section_size
                );
            }
            println!("\nCase Info:");
            println!("  Case Number: {:?}", info.case_info.case_number);
            println!("  Evidence Number: {:?}", info.case_info.evidence_number);
            println!("  Description: {:?}", info.case_info.description);
            println!("  Examiner: {:?}", info.case_info.examiner);
            println!("  Notes: {:?}", info.case_info.notes);
            println!("  Acquisition Date: {:?}", info.case_info.acquisition_date);
            println!("Hashes: {:?}", info.hashes);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
