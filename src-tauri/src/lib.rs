// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! FFX - Forensic File Xplorer Library
//!
//! This library provides parsing, verification, and extraction capabilities
//! for forensic disk image and logical evidence container formats.
//!
//! ## Supported Formats
//!
//! | Format | Module    | Description                                    |
//! |--------|-----------|------------------------------------------------|
//! | AD1    | `ad1`     | AccessData Logical Image (FTK)                 |
//! | E01    | `ewf`     | Expert Witness Format / EnCase (physical)      |
//! | L01    | `ewf`     | EnCase Logical Evidence File                   |
//! | Ex01   | `ewf`     | Expert Witness Format v2 (physical)            |
//! | Lx01   | `ewf`     | EnCase Logical Evidence v2                     |
//! | RAW    | `raw`     | dd-style images (.dd, .raw, .img, .001)        |
//! | 7z     | `archive` | 7-Zip archives (metadata only)                 |
//! | ZIP    | `archive` | ZIP/ZIP64 archives (metadata only)             |
//! | RAR    | `archive` | RAR4/RAR5 archives (metadata only)             |
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  Tauri Commands (commands/)                                 │
//! │   - container: AD1 operations (V1 and V2)                   │
//! │   - lazy_loading: Unified lazy loading                      │
//! │   - archive: ZIP, 7z, RAR, TAR operations                   │
//! │   - ewf: E01/L01 operations                                 │
//! │   - raw: Raw disk image operations                          │
//! │   - vfs: Virtual filesystem mounting                        │
//! │   - hash: Batch hashing operations                          │
//! │   - system: System stats monitoring                         │
//! │   - analysis: Hex dump, entropy analysis                    │
//! │   - database: SQLite persistence                            │
//! │   - project: Project file handling                          │
//! │   - viewer: File viewer operations                          │
//! │   - discovery: Evidence file discovery                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Container Abstraction (containers.rs)                      │
//! │   - ContainerInfo, ContainerKind                            │
//! │   - Auto-detection by signature and extension               │
//! │   - Unified info/verify/extract interface                   │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Format Parsers                                             │
//! │   ┌─────────┬─────────┬─────────┬─────────┬─────────┐      │
//! │   │  ad1.rs │  ewf.rs │  l01.rs │  raw.rs │archive.rs│     │
//! │   │  ufed.rs│         │         │         │         │      │
//! │   └─────────┴─────────┴─────────┴─────────┴─────────┘      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Common Utilities (common/)                                 │
//! │   - hash.rs:     Streaming hashers (SHA, BLAKE3, XXH3)     │
//! │   - binary.rs:   Little-endian binary reading              │
//! │   - segments.rs: Multi-segment file discovery              │
//! │   - io_pool.rs:  File handle pooling                       │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Tauri Integration
//!
//! All parsing operations run on blocking threads to prevent UI freezing.
//! Progress events are emitted via Tauri's event system:
//!
//! - `scan-file-found`: File discovered during directory scan
//! - `verify-progress`: Hash verification progress (current, total, percent)
//! - `segment-verify-progress`: Per-segment verification progress
//!
//! ## Performance Considerations
//!
//! - **Streaming I/O**: Large files use buffered/memory-mapped I/O
//! - **Parallel hashing**: BLAKE3 uses rayon for parallel processing
//! - **Pipelined I/O**: Separate reader/hasher threads for throughput
//! - **LRU caching**: AD1 decompressed blocks cached for random access
//!
//! ## Forensic Standards Compliance
//!
//! This library prioritizes forensic correctness:
//! - Read-only operations (never modifies source files)
//! - Hash verification against stored checksums
//! - Companion log parsing for chain-of-custody metadata
//! - Byte-accurate extraction preserving timestamps

// =============================================================================
// Module Declarations
// =============================================================================

pub mod ad1;        // AccessData Logical Image (FTK)
pub mod archive;    // Archive formats (7z, ZIP, RAR, etc.)
pub mod commands;   // Tauri command handlers (organized by feature)
pub mod common;     // Shared utilities (hash, binary, segments)
pub mod containers; // Container abstraction layer
pub mod database;   // SQLite persistence layer
pub mod ewf;        // Expert Witness Format (E01/L01/Ex01/Lx01) parser
pub mod formats;    // Centralized format definitions and detection
pub mod logging;    // Logging and tracing configuration
pub mod processed;  // Processed forensic databases (AXIOM, PA, etc.)
pub mod project;    // Project file handling (.ffxproj)
pub mod raw;        // Raw disk images (.dd, .raw, .img, .001, etc.)
pub mod report;     // Forensic report generation (PDF, DOCX, HTML)
pub mod ufed;       // UFED containers (UFD, UFDR, UFDX)
pub mod viewer;     // Hex/text file viewer

// =============================================================================
// Application Entry Point
// =============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(report::commands::ReportState::default())
        .setup(|app| {
            // Initialize database early (in background thread to not block startup)
            std::thread::spawn(|| {
                let _ = database::get_db();  // This triggers lazy initialization
                tracing::info!("Database initialized");
            });
            
            // Start background system stats monitoring
            commands::system::start_system_stats_monitor(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Container commands (V1)
            commands::logical_info,
            commands::logical_info_fast,
            commands::container_get_tree,
            commands::container_get_children,
            commands::container_get_children_at_addr,
            commands::container_read_entry,
            commands::container_read_entry_by_addr,
            commands::container_read_entry_chunk,
            commands::container_get_entry_info,
            commands::logical_verify,
            commands::ad1_hash_segments,
            commands::logical_extract,
            
            // Container commands (V2 - based on libad1)
            commands::container_get_root_children_v2,
            commands::container_get_children_at_addr_v2,
            commands::container_read_file_data_v2,
            commands::container_get_item_info_v2,
            commands::container_verify_item_hash_v2,
            commands::container_verify_all_v2,
            commands::container_get_info_v2,
            commands::container_extract_all_v2,
            commands::container_extract_item_v2,
            
            // Lazy loading commands
            commands::lazy_get_container_summary,
            commands::lazy_get_root_children,
            commands::lazy_get_children,
            commands::lazy_get_settings,
            commands::lazy_update_settings,
            
            // Archive commands
            commands::archive_get_tree,
            commands::archive_get_metadata,
            commands::archive_extract_entry,
            
            // UFED commands
            commands::ufed_get_tree,
            commands::ufed_get_children,
            commands::ufed_get_entry_count,
            
            // EWF/E01 commands
            commands::e01_v3_info,
            commands::e01_v3_verify,
            commands::e01_verify_segments,
            commands::e01_read_at,
            commands::e01_media_info,
            
            // RAW commands
            commands::raw_info,
            commands::raw_verify,
            commands::raw_verify_segments,
            
            // VFS commands
            commands::vfs_mount_image,
            commands::vfs_list_dir,
            commands::vfs_read_file,
            commands::vfs_get_attr,
            
            // Hash commands
            commands::batch_hash,
            
            // System commands
            commands::get_system_stats,
            
            // Analysis commands
            commands::read_file_bytes,
            commands::hex_dump,
            commands::detect_file_type,
            commands::analyze_file_entropy,
            commands::analyze_entropy_blocks,
            commands::compare_hashes,
            commands::verify_file_hash,
            
            // Discovery commands
            commands::path_exists,
            commands::path_is_directory,
            commands::discover_evidence_files,
            commands::scan_for_processed_databases,
            commands::scan_directory,
            commands::scan_directory_recursive,
            commands::scan_directory_streaming,
            commands::find_case_documents,
            commands::find_coc_forms,
            commands::find_case_document_folders,
            commands::discover_case_documents,
            
            // Database commands
            commands::db_get_or_create_session,
            commands::db_get_recent_sessions,
            commands::db_get_last_session,
            commands::db_upsert_file,
            commands::db_get_files_for_session,
            commands::db_get_file_by_path,
            commands::db_insert_hash,
            commands::db_get_hashes_for_file,
            commands::db_get_latest_hash,
            commands::db_insert_verification,
            commands::db_get_verifications_for_file,
            commands::db_save_open_tabs,
            commands::db_get_open_tabs,
            commands::db_set_setting,
            commands::db_get_setting,
            
            // Project commands
            commands::project_get_default_path,
            commands::project_check_exists,
            commands::project_save,
            commands::project_load,
            commands::project_create,
            
            // Viewer commands
            commands::viewer_read_chunk,
            commands::viewer_detect_type,
            commands::viewer_parse_header,
            commands::viewer_read_text,
            
            // Report generation commands
            report::commands::generate_report,
            report::commands::preview_report,
            report::commands::get_output_formats,
            report::commands::create_new_report,
            report::commands::validate_report,
            report::commands::add_evidence_to_report,
            report::commands::add_finding_to_report,
            report::commands::export_report_json,
            report::commands::import_report_json,
            report::commands::extract_evidence_from_containers,
            report::commands::create_evidence_from_container,
            report::commands::get_report_template,
            
            // AI commands
            report::commands::ai_commands::is_ai_available,
            report::commands::ai_commands::get_ai_providers,
            report::commands::ai_commands::generate_ai_narrative,
            report::commands::ai_commands::check_ollama_connection,
            
            // Processed database commands
            processed::commands::scan_processed_databases,
            processed::commands::get_processed_db_details,
            processed::commands::is_processed_database,
            processed::commands::get_processed_db_summary,
            
            // AXIOM-specific commands
            processed::commands::get_axiom_case_info,
            processed::commands::get_axiom_artifact_categories,
            processed::commands::query_axiom_artifacts_cmd,
            processed::commands::list_axiom_db_tables
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
