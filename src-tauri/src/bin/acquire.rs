// =============================================================================
// CORE-FFX Acquire — Acquisition-only binary entry point
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// This binary provides a lightweight, acquisition-focused build of CORE-FFX.
//
// Build command:
//   cargo build --no-default-features --features acquire --bin core_acquire
//
// Included (acquire_specific):
//   - Evidence container inspection (AD1, EWF/E01/L01, UFED, RAW)
//   - Archive inspection and 7z creation
//   - EWF/E01 and L01 logical evidence imaging
//   - Hashing (batch, queue, verification)
//   - VFS (virtual filesystem mount for disk images)
//   - Drive/source discovery and read-only remounting
//   - File/directory discovery and scanning
//   - File export (copy with hashing and manifest)
//   - Basic session, file, and hash DB tracking
//   - Project load/save for tracking acquisitions
//   - Window management
//
// Excluded (review_specific / ai_helpers):
//   - Project database (bookmarks, tags, COC, annotations, etc.)
//   - Report generation (PDF, DOCX, HTML, Markdown)
//   - AI assistance
//   - Processed DB parsing (AXIOM, Cellebrite, Autopsy)
//   - Document/file viewers (spreadsheet, email, PST, binary, registry, etc.)
//   - Project comparison, templates, and timeline visualization

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Initialize logging/tracing system.
    // Control log level with RUST_LOG env var:
    //   RUST_LOG=debug ./core_acquire
    //   RUST_LOG=ffx_check_lib::ewf=trace ./core_acquire
    ffx_check_lib::logging::init();

    ffx_check_lib::run_acquire();
}
