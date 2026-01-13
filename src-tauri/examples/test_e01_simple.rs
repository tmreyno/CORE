// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Simple E01 test example
use ffx_check_lib::ewf;

fn main() {
    let path = std::env::args().nth(1).unwrap_or("/Users/terryreynolds/Downloads/2020JimmyWilson.E01".to_string());
    println!("Testing E01 info for: {}", path);
    
    match ewf::info(&path) {
        Ok(info) => {
            println!("Success!");
            println!("Segments: {}", info.segment_count);
            println!("Chunks: {}", info.chunk_count);
            println!("Sectors: {}", info.sector_count);
            
            // Header info
            if let Some(case) = &info.case_number { println!("Case Number: {}", case); }
            if let Some(evidence) = &info.evidence_number { println!("Evidence Number: {}", evidence); }
            if let Some(desc) = &info.description { println!("Description: {}", desc); }
            if let Some(examiner) = &info.examiner_name { println!("Examiner: {}", examiner); }
            if let Some(notes) = &info.notes { println!("Notes: {}", notes); }
            if let Some(acq_date) = &info.acquiry_date { println!("Acquisition Date: {}", acq_date); }
            if let Some(sys_date) = &info.system_date { println!("System Date: {}", sys_date); }
            
            println!("Stored hashes: {}", info.stored_hashes.len());
            for h in &info.stored_hashes {
                println!("  Algorithm: {}", h.algorithm);
                println!("  Hash: '{}'", h.hash);
                println!("  Timestamp: {:?}", h.timestamp);
                println!("  Source: {:?}", h.source);
                println!("---");
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
