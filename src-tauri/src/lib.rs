// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

#![recursion_limit = "1024"]

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
pub mod archive;    // Archive formats (7z, ZIP, RAR, etc.) - READ ONLY
pub mod commands;   // Tauri command handlers (organized by feature)
pub mod common;     // Shared utilities (hash, binary, segments)
pub mod containers; // Container abstraction layer
pub mod database;   // SQLite persistence layer
pub mod ewf;        // Expert Witness Format (E01/L01/Ex01/Lx01) parser
pub mod formats;    // Centralized format definitions and detection
pub mod logging;    // Logging and tracing configuration
pub mod processed;  // Processed forensic databases (AXIOM, PA, etc.)
pub mod project;    // Project file handling (.cffx)
pub mod project_recovery;  // Project backup, recovery, and version history
pub mod project_statistics; // Project analytics and insights
pub mod session_analytics;  // Session tracking and analytics
pub mod workspace_profiles; // Workspace profiles for different scenarios
pub mod project_templates;  // Project templates for rapid initialization
pub mod activity_timeline;  // Enhanced activity timeline and visualization
pub mod project_comparison; // Project comparison and merge
pub mod raw;        // Raw disk images (.dd, .raw, .img, .001, etc.)
pub mod report;     // Forensic report generation (PDF, DOCX, HTML)
pub mod ufed;       // UFED containers (UFD, UFDR, UFDX)
pub mod viewer;     // Hex/text file viewer

use tracing::info;

// =============================================================================
// Application Entry Point
// =============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let run_start = std::time::Instant::now();
    info!("Tauri run() started");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(report::commands::ReportState::default())
        .manage(commands::index::IndexCacheState(std::sync::Arc::new(std::sync::Mutex::new(None))))
        .manage(commands::index::IndexWorkerState(std::sync::Arc::new(std::sync::Mutex::new(None))))
        .manage(commands::mmap_hex::MmapViewerState(std::sync::Arc::new(std::sync::Mutex::new(None))))
        .manage(commands::parallel_extract::ParallelExtractorState(std::sync::Arc::new(tokio::sync::Mutex::new(None))))
        .manage(commands::deduplication::DeduplicationState(std::sync::Arc::new(tokio::sync::Mutex::new(None))))
        .manage(commands::streaming_extract::StreamingExtractorState::default())
        .setup(move |app| {
            info!(elapsed_ms = run_start.elapsed().as_millis(), "setup() callback");
            
            // Pre-warm rayon thread pool in background (first use is slow)
            std::thread::spawn(|| {
                let rayon_start = std::time::Instant::now();
                // Force rayon to initialize its thread pool by doing a trivial parallel operation
                let _: Vec<_> = (0..rayon::current_num_threads()).into_iter().collect();
                rayon::scope(|_| {});  // This actually initializes the pool
                info!(elapsed_ms = rayon_start.elapsed().as_millis(), 
                    threads = rayon::current_num_threads(),
                    "Rayon thread pool warmed");
            });
            
            // Initialize system stats in background (expensive sysinfo refresh)
            commands::system::init_system_stats_background();
            
            // Initialize database early (in background thread to not block startup)
            std::thread::spawn(|| {
                let db_start = std::time::Instant::now();
                let _ = database::get_db();  // This triggers lazy initialization
                info!(elapsed_ms = db_start.elapsed().as_millis(), "Database initialized");
            });
            
            // Start background system stats monitoring
            commands::system::start_system_stats_monitor(app.handle().clone());
            info!(elapsed_ms = run_start.elapsed().as_millis(), "setup() complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Container commands (V1) - Note: get_children removed, use V2 APIs
            commands::logical_info,
            commands::logical_info_fast,
            commands::get_stored_hashes_only,
            commands::container_get_tree,
            commands::container_read_entry,
            commands::container_read_entry_by_addr,
            commands::container_read_entry_chunk,
            commands::container_extract_entry_to_temp,
            commands::container_get_entry_info,
            commands::logical_verify,
            commands::ad1_hash_segments,
            commands::logical_extract,
            
            // Container commands (V2 - based on libad1, ~8000x faster)
            commands::container_get_root_children_v2,
            commands::container_get_children_at_addr_v2,
            commands::container_read_file_data_v2,
            commands::container_get_item_info_v2,
            commands::container_get_item_metadata_v2,
            commands::container_get_items_metadata_v2,
            commands::container_get_status_v2,
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
            
            // Unified container commands (NEW - replaces container_*, archive_*, ufed_*)
            commands::unified_get_summary,
            commands::unified_detect_type,
            commands::unified_get_entry_count,
            commands::unified_get_children,
            commands::unified_get_children_typed,
            commands::unified_get_settings,
            commands::unified_update_settings,
            commands::unified_get_tree,
            
            // Archive commands (inspection only - no creation)
            commands::archive::metadata::archive_get_tree,
            commands::archive::metadata::archive_get_metadata,
            commands::archive::extraction::archive_extract_entry,
            commands::archive::extraction::archive_read_entry_chunk,
            commands::archive::nested::nested_archive_read_entry_chunk,
            commands::archive::nested::nested_container_get_tree,
            commands::archive::nested::nested_container_get_info,
            commands::archive::nested::nested_container_clear_cache,
            
            // Archive creation commands (sevenzip-ffi)
            commands::create_7z_archive,
            commands::archive::tools::test_7z_archive,
            commands::estimate_archive_size,
            commands::cancel_archive_creation,
            
            // NEW: Advanced archive features
            commands::archive::tools::repair_7z_archive,
            commands::archive::tools::validate_7z_archive,
            commands::archive::tools::get_last_archive_error,
            commands::archive::tools::clear_last_archive_error,
            commands::archive::tools::encrypt_data_native,
            commands::archive::tools::decrypt_data_native,
            commands::archive::tools::extract_split_7z_archive,
            
            // UFED commands
            commands::ufed_get_tree,
            commands::ufed_get_children,
            commands::ufed_get_entry_count,
            
            // EWF/E01 commands
            commands::e01_v3_info,
            commands::e01_v3_verify,
            commands::e01_read_at,
            commands::e01_media_info,
            
            // RAW commands
            commands::raw_info,
            commands::raw_verify,
            
            // VFS commands
            commands::vfs_mount_image,
            commands::vfs_list_dir,
            commands::vfs_read_file,
            commands::vfs_get_attr,
            
            // Hash commands
            commands::batch_hash,
            commands::batch_hash_smart,
            commands::hash_queue_get_stats,
            commands::hash_queue_get_items,
            commands::hash_queue_pause,
            commands::hash_queue_resume,
            commands::hash_queue_clear_completed,
            commands::hash_cache_stats,
            commands::hash_cache_clear,
            commands::hash_cache_invalidate_path,
            commands::hash_cache_get,
            
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
            commands::viewer_read_binary_base64,
            
            // Search commands
            commands::search::search_container,
            commands::search::search_all_containers,
            
            // Index cache commands
            commands::index::index_cache_init,
            commands::index::index_cache_has_index,
            commands::index::index_cache_get_summary,
            commands::index::index_cache_store,
            commands::index::index_cache_load,
            commands::index::index_cache_invalidate,
            commands::index::index_cache_stats,
            commands::index::index_cache_clear,
            commands::index::index_worker_start,
            commands::index::index_worker_cancel,
            commands::index::index_worker_get_active,
            commands::index::index_worker_is_indexing,
            
            // Memory-mapped hex viewer commands
            commands::mmap_hex_init,
            commands::mmap_hex_get_file_size,
            commands::mmap_hex_get_page,
            commands::mmap_hex_get_pages_window,
            commands::mmap_hex_close_file,
            commands::mmap_hex_get_cache_stats,
            commands::mmap_hex_clear_caches,
            
            // Parallel extraction commands
            commands::parallel_extract_init,
            commands::parallel_extract_batch,
            commands::parallel_extract_cancel,
            commands::parallel_extract_get_active,
            
            // Deduplication commands
            commands::dedup_init,
            commands::dedup_scan_files,
            commands::dedup_get_statistics,
            commands::dedup_get_duplicate_groups,
            commands::dedup_get_group_files,
            commands::dedup_export_json,
            commands::dedup_clear,
            
            // Streaming extraction commands
            commands::stream_extract_init,
            commands::stream_extract_start,
            commands::stream_extract_get_progress,
            commands::stream_extract_cancel,
            commands::stream_extract_get_active,
            commands::stream_extract_get_job_statuses,
            
            // Recovery & notification commands
            commands::recovery_save_operation,
            commands::recovery_load_operation,
            commands::recovery_get_interrupted,
            commands::recovery_get_by_state,
            commands::recovery_update_progress,
            commands::recovery_update_state,
            commands::recovery_mark_failed,
            commands::recovery_delete_operation,
            commands::recovery_cleanup_old,
            commands::recovery_get_stats,
            commands::recovery_create_operation,
            commands::notification_show,
            commands::notification_info,
            commands::notification_success,
            commands::notification_warning,
            commands::notification_error,
            commands::notification_set_enabled,
            commands::notification_operation_completed,
            commands::notification_operation_failed,
            commands::notification_progress_milestone,
            commands::notification_recovery_available,
            
            // Observability commands
            commands::observability::get_metrics,
            commands::observability::get_metric,
            commands::observability::increment_counter,
            commands::observability::set_gauge,
            commands::observability::record_histogram,
            commands::observability::export_metrics,
            commands::observability::reset_metrics,
            commands::observability::get_system_uptime,
            commands::observability::get_metrics_count,
            commands::observability::get_health,
            commands::observability::get_health_with_thresholds,
            commands::observability::is_system_healthy,
            commands::observability::init_tracing,
            commands::observability::get_default_log_dir,
            commands::observability::parse_log_level,
            commands::observability::get_system_status,
            
            // Profiler commands
            commands::profiler_start,
            commands::profiler_start_custom,
            commands::profiler_stop,
            commands::profiler_stop_with_flamegraph,
            commands::profiler_is_active,
            commands::profiler_get_stats,
            commands::profiler_get_history,
            commands::profiler_clear_history,
            commands::profiler_get_hot_paths,
            commands::profiler_get_summary,
            
            // Memory profiler commands
            commands::memory_start_tracking,
            commands::memory_stop_tracking,
            commands::memory_is_active,
            commands::memory_take_snapshot,
            commands::memory_get_stats,
            commands::memory_get_snapshots,
            commands::memory_clear_snapshots,
            commands::memory_detect_leaks,
            commands::memory_get_current_usage,
            commands::memory_format_bytes,
            commands::memory_get_summary,
            
            // Regression testing commands (Phase 16)
            commands::regression_record_baseline,
            commands::regression_run_test,
            commands::regression_compare_results,
            commands::regression_get_baselines,
            commands::regression_get_baseline,
            commands::regression_delete_baseline,
            commands::regression_detect_regressions,
            commands::regression_get_history,
            commands::regression_export_report,
            commands::regression_clear_history,
            commands::regression_get_summary,
            commands::regression_analyze_trends,
            commands::regression_set_threshold,
            commands::regression_get_thresholds,
            commands::regression_save_baselines,
            commands::regression_load_baselines,
            
            // Project advanced commands (recovery, backup, statistics)
            commands::project_create_backup,
            commands::project_create_version,
            commands::project_list_versions,
            commands::project_check_recovery,
            commands::project_recover_autosave,
            commands::project_clear_autosave,
            commands::project_check_health,
            commands::project_compute_statistics,
            
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
            processed::commands::list_axiom_db_tables,
            
            // Document commands (unified read/write)
            viewer::document::commands::document_read,
            viewer::document::commands::document_read_bytes,
            viewer::document::commands::document_render_html,
            viewer::document::commands::document_extract_text,
            viewer::document::commands::document_get_metadata,
            viewer::document::commands::document_detect_format,
            viewer::document::commands::document_is_supported,
            viewer::document::commands::document_supported_extensions,
            viewer::document::commands::document_read_batch,
            viewer::document::commands::document_search_text,
            viewer::document::commands::document_convert,
            
            // Universal viewer commands (read-only)
            viewer::document::commands::universal_get_info,
            viewer::document::commands::universal_get_viewer_hint,
            viewer::document::commands::universal_detect_format,
            viewer::document::commands::universal_is_supported,
            viewer::document::commands::universal_supported_extensions,
            viewer::document::commands::universal_read_data_url,
            viewer::document::commands::universal_read_text,
            viewer::document::commands::universal_get_image_dimensions,
            viewer::document::commands::universal_create_thumbnail,
            viewer::document::commands::universal_read_bytes,
            
            // Spreadsheet commands (native viewer)
            viewer::document::commands::spreadsheet_info,
            viewer::document::commands::spreadsheet_read_sheet,
            
            // Email viewer commands
            viewer::document::commands::email_parse_eml,
            viewer::document::commands::email_parse_mbox,
            
            // Plist viewer commands
            viewer::document::commands::plist_read,
            viewer::document::commands::plist_read_value,
            viewer::document::commands::plist_get_value_at_path,
            viewer::document::commands::plist_search,
            
            // EXIF metadata commands
            viewer::document::commands::exif_extract,
            viewer::document::commands::exif_has_data,
            
            // Binary analysis commands
            viewer::document::commands::binary_analyze,
            viewer::document::commands::binary_detect_format,
            
            // Workspace profile commands
            commands::project_extended::profile_list,
            commands::project_extended::profile_get,
            commands::project_extended::profile_get_active,
            commands::project_extended::profile_set_active,
            commands::project_extended::profile_add,
            commands::project_extended::profile_update,
            commands::project_extended::profile_delete,
            commands::project_extended::profile_clone,
            commands::project_extended::profile_export,
            commands::project_extended::profile_import,
            
            // Template commands
            commands::project_extended::template_list,
            commands::project_extended::template_list_by_category,
            commands::project_extended::template_get,
            commands::project_extended::template_apply,
            commands::project_extended::template_create_from_project,
            commands::project_extended::template_export,
            commands::project_extended::template_import,
            commands::project_extended::template_delete,
            
            // Timeline visualization commands
            commands::project_extended::timeline_compute_visualization,
            commands::project_extended::timeline_export,
            commands::project_extended::timeline_export_json,
            
            // Project comparison commands
            commands::project_extended::project_compare,
            commands::project_extended::project_merge,
            commands::project_extended::project_sync_bookmarks,
            commands::project_extended::project_sync_notes,
            
            // Export command (unified copy/export with options)
            commands::export_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
