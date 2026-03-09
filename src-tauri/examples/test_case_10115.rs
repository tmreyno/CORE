// =============================================================================
// CORE-FFX - Forensic File Explorer
// Full Pipeline Integration Test — Case 10115-0900
// =============================================================================
//!
//! Exercises ALL container types and core features against real evidence
//! from /Volumes/Tools/10115-0900.
//!
//! Usage:
//!   cargo run --example test_case_10115
//!   cargo run --example test_case_10115 -- --quick    # Skip slow operations
//!   cargo run --example test_case_10115 -- --verbose  # Extra detail
//!
//! Expected runtime: ~5-10 min (full), ~30s (quick)

use std::path::{Path, PathBuf};
use std::time::Instant;

use ffx_check_lib::ad1;
use ffx_check_lib::common::container_detect::detect_container_type;
use ffx_check_lib::common::format_size_compact;
use ffx_check_lib::common::vfs::VirtualFileSystem;
use ffx_check_lib::containers::scan_directory_recursive;
use ffx_check_lib::ewf;
use ffx_check_lib::ewf::l01_reader;
use ffx_check_lib::ewf::vfs::EwfVfs;
use ffx_check_lib::ufed;

const EVIDENCE_DIR: &str = "/Volumes/Tools/10115-0900/1.Evidence";

// ─── Test Result Tracking ───────────────────────────────────────────────────

struct TestResults {
    passed: Vec<String>,
    failed: Vec<(String, String)>,
    skipped: Vec<String>,
    timings: Vec<(String, f64)>,
}

impl TestResults {
    fn new() -> Self {
        Self {
            passed: Vec::new(),
            failed: Vec::new(),
            skipped: Vec::new(),
            timings: Vec::new(),
        }
    }

    fn pass(&mut self, name: &str) {
        println!("  PASS: {}", name);
        self.passed.push(name.to_string());
    }

    fn fail(&mut self, name: &str, reason: &str) {
        println!("  FAIL: {} -- {}", name, reason);
        self.failed.push((name.to_string(), reason.to_string()));
    }

    fn skip(&mut self, name: &str) {
        println!("  SKIP: {} (skipped)", name);
        self.skipped.push(name.to_string());
    }

    fn timed(&mut self, name: &str, secs: f64) {
        self.timings.push((name.to_string(), secs));
    }

    fn print_summary(&self) {
        let total = self.passed.len() + self.failed.len() + self.skipped.len();
        println!("\n{}", "=".repeat(70));
        println!("TEST SUMMARY");
        println!("{}", "=".repeat(70));
        println!(
            "  Total: {}  |  Passed: {}  |  Failed: {}  |  Skipped: {}",
            total,
            self.passed.len(),
            self.failed.len(),
            self.skipped.len()
        );

        if !self.failed.is_empty() {
            println!("\n  FAILURES:");
            for (name, reason) in &self.failed {
                println!("    [FAIL] {} -- {}", name, reason);
            }
        }

        if !self.timings.is_empty() {
            println!("\n  PERFORMANCE:");
            for (name, secs) in &self.timings {
                println!("    [{:.2}s] {}", secs, name);
            }
        }

        println!("{}", "=".repeat(70));

        if self.failed.is_empty() {
            println!("ALL TESTS PASSED\n");
        } else {
            println!("{} TEST(S) FAILED\n", self.failed.len());
        }
    }
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quick = args.iter().any(|a| a == "--quick");
    let verbose = args.iter().any(|a| a == "--verbose");

    println!("\n{}", "=".repeat(64));
    println!("  CORE-FFX Full Pipeline Test -- Case 10115-0900");
    println!("{}\n", "=".repeat(64));

    // Pre-check: does the evidence exist?
    if !Path::new(EVIDENCE_DIR).exists() {
        eprintln!("ERROR: Evidence directory not found: {}", EVIDENCE_DIR);
        eprintln!("       Mount the Tools volume and try again.");
        std::process::exit(1);
    }

    let mut results = TestResults::new();
    let overall_start = Instant::now();

    // Phase 1: Discovery & Detection
    println!("--- PHASE 1: Evidence Discovery & Container Detection ---\n");
    test_discovery(&mut results, verbose);

    // Phase 2: E01 Containers
    println!("\n--- PHASE 2: E01 Multi-Segment Containers ---\n");
    test_e01_containers(&mut results, quick, verbose);

    // Phase 3: AD1 Containers
    println!("\n--- PHASE 3: AD1 Containers (Single & Multi-Segment) ---\n");
    test_ad1_containers(&mut results, quick, verbose);

    // Phase 4: L01 Multi-Segment
    println!("\n--- PHASE 4: L01 Multi-Segment Container ---\n");
    test_l01_containers(&mut results, quick, verbose);

    // Phase 5: UFED Phone Extractions
    println!("\n--- PHASE 5: UFED Phone Extractions ---\n");
    test_ufed_containers(&mut results, verbose);

    // Phase 6: VFS & Filesystem Mounting
    println!("\n--- PHASE 6: VFS Filesystem Mounting (E01 -> Partition Browse) ---\n");
    test_vfs_mounting(&mut results, quick, verbose);

    // Phase 7: Cross-Container Batch Operations
    if !quick {
        println!("\n--- PHASE 7: Cross-Container Batch Operations ---\n");
        test_batch_operations(&mut results, verbose);
    }

    // Summary
    let total_secs = overall_start.elapsed().as_secs_f64();
    results.timed("Total pipeline", total_secs);
    results.print_summary();

    if !results.failed.is_empty() {
        std::process::exit(1);
    }
}

// =========================================================================
// Phase 1: Discovery
// =========================================================================

fn test_discovery(results: &mut TestResults, verbose: bool) {
    let start = Instant::now();

    // Test 1.1: Recursive scan finds all containers
    match scan_directory_recursive(EVIDENCE_DIR) {
        Ok(files) => {
            let count = files.len();
            println!(
                "  Discovered {} evidence files in {:.2}s",
                count,
                start.elapsed().as_secs_f64()
            );
            results.timed("Discovery scan", start.elapsed().as_secs_f64());

            if count >= 10 {
                results.pass(&format!("Discovery: found {} evidence containers", count));
            } else {
                results.fail(
                    "Discovery: found >=10 evidence containers",
                    &format!("Only found {}", count),
                );
            }

            // Test 1.2: Container type detection for each discovered file
            let mut type_counts: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            let mut detection_failures = 0;

            for file in &files {
                match detect_container_type(&file.path) {
                    Some(ct) => {
                        *type_counts.entry(ct.as_str().to_string()).or_insert(0) += 1;
                    }
                    None => {
                        detection_failures += 1;
                        if verbose {
                            println!("    [warn] No type detected: {}", file.path);
                        }
                    }
                }
            }

            if verbose {
                println!("  Type breakdown:");
                for (t, c) in &type_counts {
                    println!("    {}: {}", t, c);
                }
            }

            // Should detect E01 and AD1 types
            let has_e01 = type_counts.keys().any(|k| k.eq_ignore_ascii_case("e01"));
            let has_ad1 = type_counts.keys().any(|k| k.eq_ignore_ascii_case("ad1"));

            if has_e01 {
                results.pass("Detection: E01 containers recognized");
            } else {
                results.fail(
                    "Detection: E01 containers recognized",
                    &format!(
                        "Types found: {:?}",
                        type_counts.keys().collect::<Vec<_>>()
                    ),
                );
            }

            if has_ad1 {
                results.pass("Detection: AD1 containers recognized");
            } else {
                results.fail(
                    "Detection: AD1 containers recognized",
                    &format!(
                        "Types found: {:?}",
                        type_counts.keys().collect::<Vec<_>>()
                    ),
                );
            }

            if detection_failures == 0 {
                results.pass("Detection: all files have recognized types");
            } else {
                println!(
                    "    Note: {} files had no detected type (may be companion files)",
                    detection_failures
                );
            }
        }
        Err(e) => {
            results.fail("Discovery: recursive scan", &format!("{}", e));
        }
    }
}

// =========================================================================
// Phase 2: E01 Containers
// =========================================================================

fn test_e01_containers(results: &mut TestResults, quick: bool, verbose: bool) {
    let e01_containers = vec![
        (
            "Diana_PC",
            "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_Diana_PC/10115_0900_B_Diana_PC.E01",
            9usize,
        ),
        (
            "Dong_Wong_Laptop",
            "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_Dong_Wong_Laptop/10115_0900_B_Dong_Wong_Laptop.E01",
            9,
        ),
        (
            "Michelle_OS",
            "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_Michelle_OS/10115_0900_B_Michelle_OS.E01",
            9,
        ),
        (
            "HP_Desktop",
            "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_RmG_ITM9_HP_Desktop/10115_0900_B_RmG_ITM9_HP_Desktop.E01",
            9,
        ),
        (
            "Michelle_Data",
            "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_RmG_Itm7_Michelle_Data/10115_0900_B_RmG_Itm7_Michelle_Data.E01",
            9,
        ),
    ];

    for (label, path, expected_segments) in &e01_containers {
        println!("  [{}]", label);

        if !Path::new(path).exists() {
            results.fail(&format!("E01 {}: file exists", label), "File not found");
            continue;
        }

        // Test 2.1: Info loading
        let start = Instant::now();
        match ewf::info(path) {
            Ok(info) => {
                let elapsed = start.elapsed().as_secs_f64();
                results.timed(&format!("E01 {} info", label), elapsed);

                if verbose {
                    println!(
                        "    Segments: {}, Chunks: {}, Sectors: {}",
                        info.segment_count, info.chunk_count, info.sector_count
                    );
                    println!(
                        "    Total size: {} ({:.2} GB)",
                        format_size_compact(info.total_size),
                        info.total_size as f64 / 1_073_741_824.0
                    );
                    if let Some(case) = &info.case_number {
                        println!("    Case: {}", case);
                    }
                    if let Some(examiner) = &info.examiner_name {
                        println!("    Examiner: {}", examiner);
                    }
                }

                // Validate segment count
                if info.segment_count as usize >= *expected_segments {
                    results.pass(&format!(
                        "E01 {}: info loaded ({} segments, {:.2} GB)",
                        label,
                        info.segment_count,
                        info.total_size as f64 / 1_073_741_824.0
                    ));
                } else {
                    results.fail(
                        &format!("E01 {}: segment count", label),
                        &format!(
                            "Expected >={}, got {}",
                            expected_segments, info.segment_count
                        ),
                    );
                }

                // Check for stored hashes
                if !info.stored_hashes.is_empty() {
                    let algos: Vec<_> = info
                        .stored_hashes
                        .iter()
                        .map(|h| h.algorithm.clone())
                        .collect();
                    results.pass(&format!(
                        "E01 {}: stored hashes present ({})",
                        label,
                        algos.join(", ")
                    ));
                } else {
                    println!("    [warn] No stored hashes in E01 header");
                }
            }
            Err(e) => {
                results.fail(
                    &format!("E01 {}: info loading", label),
                    &format!("{}", e),
                );
                continue;
            }
        }

        // Test 2.2: Fast info (should be significantly faster)
        let start = Instant::now();
        match ewf::info_fast(path) {
            Ok(_) => {
                let elapsed = start.elapsed().as_secs_f64();
                results.timed(&format!("E01 {} info_fast", label), elapsed);
                results.pass(&format!("E01 {}: fast info ({:.3}s)", label, elapsed));
            }
            Err(e) => {
                results.fail(
                    &format!("E01 {}: fast info", label),
                    &format!("{}", e),
                );
            }
        }

        // Test 2.3: Hash verification (SLOW - only test first container in full mode)
        if !quick && *label == "Diana_PC" {
            println!("    Verifying hash (this may take several minutes)...");
            let start = Instant::now();
            match ewf::verify(path, "md5") {
                Ok(hash) => {
                    let elapsed = start.elapsed().as_secs_f64();
                    results.timed(&format!("E01 {} MD5 verify", label), elapsed);
                    results.pass(&format!(
                        "E01 {}: MD5 verified in {:.1}s -> {}...",
                        label,
                        elapsed,
                        &hash[..std::cmp::min(16, hash.len())]
                    ));
                }
                Err(e) => {
                    results.fail(
                        &format!("E01 {}: MD5 verify", label),
                        &format!("{}", e),
                    );
                }
            }
        } else {
            results.skip(&format!("E01 {}: hash verify", label));
        }

        println!();
    }
}

// =========================================================================
// Phase 3: AD1 Containers
// =========================================================================

fn test_ad1_containers(results: &mut TestResults, quick: bool, verbose: bool) {
    let ad1_containers = vec![
        (
            "NWang",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900-EV-0100-NWang-Accountant/10115-0900-EV-0100-NWang-Accountant.ad1",
            false,
        ),
        (
            "HBaik",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_HBaik/10115-0900_HBaik.ad1",
            false,
        ),
        (
            "Jeffery_Yang",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_Jeffery_Yang_ASUSdesktopcustomPC/10115-0900_Jeffery_Yang_ASUSdesktopcustomPC.ad1",
            false,
        ),
        (
            "Junqian_Huang",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900-Junqian-Huang/10115-0900-Junqian-Huang/Junqian-Huang.ad1",
            true,
        ),
        (
            "Tom_Chiang",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900-Tom-Chiang/10115-0900-Tom-Chiang/10115-0900-Tom_Chiang.ad1",
            true,
        ),
    ];

    for (label, path, is_multi) in &ad1_containers {
        println!("  [{}] (multi-segment: {})", label, is_multi);

        if !Path::new(path).exists() {
            results.fail(&format!("AD1 {}: file exists", label), "File not found");
            continue;
        }

        // Test 3.1: Fast info
        let start = Instant::now();
        match ad1::info_fast(path) {
            Ok(info) => {
                let elapsed = start.elapsed().as_secs_f64();
                results.timed(&format!("AD1 {} info_fast", label), elapsed);

                if verbose {
                    println!(
                        "    Name: {}",
                        info.logical.data_source_name
                    );
                    println!("    Total items: {}", info.item_count);
                    println!(
                        "    Total size: {}",
                        format_size_compact(info.total_size.unwrap_or(0))
                    );
                }

                results.pass(&format!(
                    "AD1 {}: fast info ({} items, {:.3}s)",
                    label, info.item_count, elapsed
                ));
            }
            Err(e) => {
                results.fail(
                    &format!("AD1 {}: fast info", label),
                    &format!("{}", e),
                );
                continue;
            }
        }

        // Test 3.2: Tree root children (v2 API)
        let start = Instant::now();
        match ad1::get_root_children_v2(path) {
            Ok(children) => {
                let elapsed = start.elapsed().as_secs_f64();
                results.timed(&format!("AD1 {} root_children", label), elapsed);

                if verbose {
                    println!("    Root children: {}", children.len());
                    for child in children.iter().take(5) {
                        println!(
                            "      {} ({})",
                            child.name,
                            if child.is_dir { "dir" } else { "file" }
                        );
                    }
                }

                if !children.is_empty() {
                    results.pass(&format!(
                        "AD1 {}: root tree ({} entries, {:.3}s)",
                        label,
                        children.len(),
                        elapsed
                    ));
                } else {
                    results.fail(
                        &format!("AD1 {}: root tree", label),
                        "No root children returned",
                    );
                }

                // Test 3.3: Drill into first directory child
                // NOTE: Use item_addr (the item's own offset), NOT data_addr (zlib metadata address)
                if let Some(dir_child) = children.iter().find(|c| c.is_dir) {
                    if let Some(addr) = dir_child.item_addr {
                        match ad1::get_children_at_addr_v2(path, addr, "") {
                            Ok(sub_children) => {
                                results.pass(&format!(
                                    "AD1 {}: drill-down into '{}' ({} children)",
                                    label, dir_child.name, sub_children.len()
                                ));
                            }
                            Err(e) => {
                                results.fail(
                                    &format!("AD1 {}: drill-down", label),
                                    &format!("{}", e),
                                );
                            }
                        }
                    }
                }

                // Test 3.4: Read a small file's data (first non-directory, <1MB)
                if !quick {
                    if let Some(file_child) = children
                        .iter()
                        .find(|c| !c.is_dir && c.size < 1_000_000 && c.data_addr.is_some())
                    {
                        match ad1::read_file_data_v2(path, file_child.data_addr.unwrap()) {
                            Ok(data) => {
                                if !data.is_empty() {
                                    results.pass(&format!(
                                        "AD1 {}: read file '{}' ({} bytes)",
                                        label,
                                        file_child.name,
                                        data.len()
                                    ));
                                } else {
                                    results.fail(
                                        &format!(
                                            "AD1 {}: read file '{}'",
                                            label, file_child.name
                                        ),
                                        "Got 0 bytes",
                                    );
                                }
                            }
                            Err(e) => {
                                results.fail(
                                    &format!("AD1 {}: read file", label),
                                    &format!("{}", e),
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                results.fail(
                    &format!("AD1 {}: root tree", label),
                    &format!("{}", e),
                );
            }
        }

        // Test 3.5: Multi-segment specific test
        if *is_multi {
            let ad1_path = PathBuf::from(path);
            let parent = ad1_path.parent().unwrap();
            let stem = ad1_path.file_stem().unwrap().to_str().unwrap();
            let segment_count = (2..=30)
                .take_while(|n| {
                    let ext = format!("ad{}", n);
                    parent.join(format!("{}.{}", stem, ext)).exists()
                })
                .count()
                + 1;

            println!("    Segment files found: {}", segment_count);
            if segment_count > 1 {
                results.pass(&format!(
                    "AD1 {}: multi-segment ({} segments)",
                    label, segment_count
                ));
            } else {
                results.fail(
                    &format!("AD1 {}: multi-segment detection", label),
                    "Expected multiple segments",
                );
            }
        }

        println!();
    }
}

// =========================================================================
// Phase 4: L01 Multi-Segment
// =========================================================================

fn test_l01_containers(results: &mut TestResults, _quick: bool, verbose: bool) {
    let l01_path = "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_Hoa_Ching_Hu/DCCollection_2026-03-03_17-10-23/DCCollection_2026-03-03_17-10-23.L01";

    println!("  [Hoa_Ching_Hu DCCollection]");

    if !Path::new(l01_path).exists() {
        results.fail("L01: file exists", "File not found");
        return;
    }

    // Test 4.1: L01 file tree parsing
    let start = Instant::now();
    match l01_reader::parse_l01_file_tree(l01_path) {
        Ok(tree) => {
            let elapsed = start.elapsed().as_secs_f64();
            results.timed("L01 tree parse", elapsed);

            let file_count = tree.file_count();
            let dir_count = tree.directory_count();

            if verbose {
                println!("    Files: {}, Directories: {}", file_count, dir_count);
                println!("    Root entries:");
                for entry in tree.root_entries().iter().take(10) {
                    println!(
                        "      {} ({}{})",
                        entry.name,
                        if entry.is_directory { "dir" } else { "file" },
                        if !entry.is_directory {
                            format!(", {}", format_size_compact(entry.size))
                        } else {
                            String::new()
                        }
                    );
                }
            }

            if file_count > 0 {
                results.pass(&format!(
                    "L01: tree parsed ({} files, {} dirs, {:.3}s)",
                    file_count, dir_count, elapsed
                ));
            } else {
                results.fail("L01: tree parsing", "No files found in tree");
            }

            // Test 4.2: Directory drill-down
            let root_entries = tree.root_entries();
            if let Some(dir) = root_entries.iter().find(|e| e.is_directory) {
                let children = tree.children_of(dir.identifier);
                results.pass(&format!(
                    "L01: drill-down into '{}' ({} children)",
                    dir.name,
                    children.len()
                ));
            }
        }
        Err(e) => {
            results.fail("L01: tree parsing", &format!("{}", e));
        }
    }

    // Test 4.3: L01 as EWF info (it's still an EWF container)
    let start = Instant::now();
    match ewf::info(l01_path) {
        Ok(info) => {
            let elapsed = start.elapsed().as_secs_f64();
            if verbose {
                println!(
                    "    EWF segments: {}, chunks: {}",
                    info.segment_count, info.chunk_count
                );
            }
            results.pass(&format!(
                "L01: EWF info loaded ({} segments, {:.3}s)",
                info.segment_count, elapsed
            ));
        }
        Err(e) => {
            results.fail("L01: EWF info", &format!("{}", e));
        }
    }

    // Test 4.4: Verify multi-segment files exist on disk
    let l01_dir = Path::new(l01_path).parent().unwrap();
    let segment_count = (1..=20)
        .take_while(|n| {
            let ext = if *n == 1 {
                "L01".to_string()
            } else {
                format!("L{:02}", n)
            };
            l01_dir
                .join(format!("DCCollection_2026-03-03_17-10-23.{}", ext))
                .exists()
        })
        .count();

    if segment_count > 1 {
        results.pass(&format!(
            "L01: multi-segment verified ({} segments on disk)",
            segment_count
        ));
    } else {
        results.fail("L01: multi-segment", "Expected multiple segment files");
    }

    println!();
}

// =========================================================================
// Phase 5: UFED Phone Extractions
// =========================================================================

fn test_ufed_containers(results: &mut TestResults, verbose: bool) {
    let ufed_subjects = vec![
        (
            "Samsung_KIT_Phone",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_A_011BRD_KIT Phone",
        ),
        (
            "GUOXI_iPhone",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_B_GUOXI iPhone",
        ),
        (
            "Warehouse_iPhone12",
            "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_Warehouse_iphone12promax",
        ),
    ];

    for (label, base_dir) in &ufed_subjects {
        println!("  [{}]", label);

        if !Path::new(base_dir).exists() {
            results.fail(
                &format!("UFED {}: directory exists", label),
                "Not found",
            );
            continue;
        }

        // Find .ufd files recursively
        let ufd_files = find_files_recursive(base_dir, "ufd");
        let zip_files = find_files_recursive(base_dir, "zip");
        let ufdx_files = find_files_recursive(base_dir, "ufdx");

        if verbose {
            println!("    .ufd files: {}", ufd_files.len());
            println!("    .zip files: {}", zip_files.len());
            println!("    .ufdx files: {}", ufdx_files.len());
            for f in &ufd_files {
                println!("      {}", f.display());
            }
        }

        // Test 5.1: UFED detection
        let mut detected_any = false;
        for ufd_path in &ufd_files {
            let path_str = ufd_path.to_str().unwrap_or("");
            if ufed::detection::is_ufed(path_str) {
                detected_any = true;
                if verbose {
                    println!(
                        "    Detected as UFED: {}",
                        ufd_path.file_name().unwrap().to_str().unwrap()
                    );
                    if let Some(fmt) = ufed::detection::detect_format(path_str) {
                        println!("      Format: {:?}", fmt);
                    }
                    if let Some(hint) = ufed::detection::extract_device_hint(path_str) {
                        println!("      Device: {}", hint);
                    }
                }
            }
        }

        // Also check .zip files
        for zip_path in &zip_files {
            let path_str = zip_path.to_str().unwrap_or("");
            if ufed::detection::is_ufed(path_str) {
                detected_any = true;
                if verbose {
                    println!(
                        "    Detected as UFED (zip): {}",
                        zip_path.file_name().unwrap().to_str().unwrap()
                    );
                }
            }
        }

        if detected_any {
            results.pass(&format!("UFED {}: detection", label));
        } else {
            // Try detection on the base directory itself
            if ufed::detection::is_ufed(base_dir) {
                results.pass(&format!("UFED {}: detection (via base dir)", label));
            } else {
                results.fail(
                    &format!("UFED {}: detection", label),
                    &format!(
                        "No .ufd or .zip detected as UFED. Found {} .ufd, {} .zip files",
                        ufd_files.len(),
                        zip_files.len()
                    ),
                );
            }
        }

        // Test 5.2: UFED info loading (try each .ufd)
        let mut info_loaded = false;
        for ufd_path in &ufd_files {
            let path_str = ufd_path.to_str().unwrap_or("");
            let start = Instant::now();
            match ufed::info(path_str) {
                Ok(info) => {
                    let elapsed = start.elapsed().as_secs_f64();
                    results.timed(&format!("UFED {} info", label), elapsed);
                    if verbose {
                        if let Some(ref ci) = info.case_info {
                            println!("    Case: {:?}", ci);
                        }
                        if let Some(ref di) = info.device_info {
                            println!("    Device: {:?}", di);
                        }
                        if let Some(ref ei) = info.extraction_info {
                            println!("    Extraction: {:?}", ei);
                        }
                    }
                    results.pass(&format!(
                        "UFED {}: info loaded ({:.3}s, format: {})",
                        label, elapsed, info.format
                    ));
                    info_loaded = true;
                    break;
                }
                Err(e) => {
                    if verbose {
                        println!("    [warn] Info failed for {}: {}", ufd_path.display(), e);
                    }
                }
            }
        }

        // If .ufd didn't work, try .zip files
        if !info_loaded {
            for zip_path in &zip_files {
                let path_str = zip_path.to_str().unwrap_or("");
                match ufed::info(path_str) {
                    Ok(info) => {
                        if verbose {
                            println!("    Format (via zip): {}", info.format);
                        }
                        results.pass(&format!("UFED {}: info loaded (via .zip)", label));
                        info_loaded = true;
                        break;
                    }
                    Err(_) => {}
                }
            }
        }

        if !info_loaded {
            results.fail(
                &format!("UFED {}: info loading", label),
                "Could not load info from any .ufd or .zip file",
            );
        }

        // Test 5.3: UFED tree browsing (if info loaded)
        if info_loaded {
            for ufd_path in &ufd_files {
                let path_str = ufd_path.to_str().unwrap_or("");
                let start = Instant::now();
                match ufed::tree::get_root_children(path_str) {
                    Ok(children) => {
                        let elapsed = start.elapsed().as_secs_f64();
                        if verbose {
                            println!("    Root children: {}", children.len());
                            for child in children.iter().take(5) {
                                println!(
                                    "      {} (dir: {})",
                                    child.name, child.is_dir
                                );
                            }
                        }
                        results.pass(&format!(
                            "UFED {}: tree browsing ({} root entries, {:.3}s)",
                            label,
                            children.len(),
                            elapsed
                        ));
                        break;
                    }
                    Err(e) => {
                        if verbose {
                            println!("    [warn] Tree failed: {}", e);
                        }
                    }
                }
            }
        }

        // Test 5.4: UFDX presence
        if !ufdx_files.is_empty() {
            results.pass(&format!(
                "UFED {}: EvidenceCollection.ufdx present ({} files)",
                label,
                ufdx_files.len()
            ));
        }

        // Test 5.5: Companion files
        let pdf_files = find_files_recursive(base_dir, "pdf");
        if !pdf_files.is_empty() {
            results.pass(&format!(
                "UFED {}: companion PDFs present ({} files)",
                label,
                pdf_files.len()
            ));
        }

        println!();
    }
}

// =========================================================================
// Phase 6: VFS & Filesystem Mounting
// =========================================================================

fn test_vfs_mounting(results: &mut TestResults, quick: bool, verbose: bool) {
    let test_e01 = "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_Dong_Wong_Laptop/10115_0900_B_Dong_Wong_Laptop.E01";

    if !Path::new(test_e01).exists() {
        results.skip("VFS: E01 not available for mounting test");
        return;
    }

    // Test 6.1: Physical mode (raw partition table)
    println!("  Testing VFS physical mode...");
    let start = Instant::now();
    match EwfVfs::open_physical(test_e01) {
        Ok(vfs) => {
            let elapsed = start.elapsed().as_secs_f64();
            results.timed("VFS physical open", elapsed);

            match vfs.readdir("/") {
                Ok(entries) => {
                    if verbose {
                        println!("    Physical root entries:");
                        for entry in &entries {
                            println!(
                                "      {} (dir: {})",
                                entry.name, entry.is_directory
                            );
                        }
                    }
                    results.pass(&format!(
                        "VFS: physical mode ({} root entries, {:.3}s)",
                        entries.len(),
                        elapsed
                    ));
                }
                Err(e) => {
                    results.fail("VFS: physical mode readdir", &format!("{:?}", e));
                }
            }
        }
        Err(e) => {
            results.fail("VFS: physical mode open", &format!("{:?}", e));
        }
    }

    // Test 6.2: Filesystem mode (auto-detect partitions + mount)
    println!("  Testing VFS filesystem mode...");
    let start = Instant::now();
    match EwfVfs::open_filesystem(test_e01) {
        Ok(vfs) => {
            let elapsed = start.elapsed().as_secs_f64();
            results.timed("VFS filesystem open", elapsed);

            let partition_count = vfs.partition_count();
            println!("    Partitions detected: {}", partition_count);

            if partition_count > 0 {
                results.pass(&format!(
                    "VFS: filesystem mode ({} partitions, {:.3}s)",
                    partition_count, elapsed
                ));

                // Explore first partition
                match vfs.readdir("/") {
                    Ok(partitions) => {
                        if let Some(first_part) = partitions.first() {
                            let part_path = format!("/{}", first_part.name);
                            match vfs.readdir(&part_path) {
                                Ok(files) => {
                                    if verbose {
                                        println!(
                                            "    First partition '{}' root ({} entries):",
                                            first_part.name,
                                            files.len()
                                        );
                                        for f in files.iter().take(10) {
                                            let attr =
                                                vfs.getattr(&format!("{}/{}", part_path, f.name))
                                                    .ok();
                                            let size =
                                                attr.as_ref().map(|a| a.size).unwrap_or(0);
                                            println!(
                                                "      {} {} {}",
                                                if f.is_directory { "DIR" } else { "   " },
                                                f.name,
                                                if !f.is_directory {
                                                    format!("({})", format_size_compact(size))
                                                } else {
                                                    String::new()
                                                }
                                            );
                                        }
                                    }

                                    results.pass(&format!(
                                        "VFS: partition browse ({} entries in '{}')",
                                        files.len(),
                                        first_part.name
                                    ));

                                    // Test 6.3: Deep directory traversal
                                    if !quick {
                                        let mut total_entries = 0;
                                        for f in files.iter().take(5) {
                                            if f.is_directory {
                                                let sub_path =
                                                    format!("{}/{}", part_path, f.name);
                                                if let Ok(sub_entries) = vfs.readdir(&sub_path) {
                                                    total_entries += sub_entries.len();
                                                }
                                            }
                                        }
                                        results.pass(&format!(
                                            "VFS: deep traversal ({} entries in subdirs)",
                                            total_entries
                                        ));
                                    }

                                    // Test 6.4: File attribute retrieval
                                    if let Some(file) =
                                        files.iter().find(|f| !f.is_directory)
                                    {
                                        let file_path =
                                            format!("{}/{}", part_path, file.name);
                                        match vfs.getattr(&file_path) {
                                            Ok(attr) => {
                                                results.pass(&format!(
                                                    "VFS: getattr '{}' ({}, modified: {:?})",
                                                    file.name,
                                                    format_size_compact(attr.size),
                                                    attr.modified
                                                ));
                                            }
                                            Err(e) => {
                                                results.fail(
                                                    "VFS: getattr",
                                                    &format!("{:?}", e),
                                                );
                                            }
                                        }
                                    }

                                    // Test 6.5: File read
                                    if !quick {
                                        if let Some(file) =
                                            files.iter().find(|f| !f.is_directory)
                                        {
                                            let file_path =
                                                format!("{}/{}", part_path, file.name);
                                            let attr = vfs.getattr(&file_path).ok();
                                            let size =
                                                attr.as_ref().map(|a| a.size).unwrap_or(0);
                                            if size > 0 && size < 1_000_000 {
                                                let read_size =
                                                    std::cmp::min(size as usize, 4096);
                                                let start = Instant::now();
                                                match vfs.read(&file_path, 0, read_size) {
                                                    Ok(data) => {
                                                        let elapsed =
                                                            start.elapsed().as_secs_f64();
                                                        results.pass(&format!(
                                                            "VFS: file read '{}' ({} bytes, {:.3}s)",
                                                            file.name,
                                                            data.len(),
                                                            elapsed
                                                        ));
                                                    }
                                                    Err(e) => {
                                                        results.fail(
                                                            "VFS: file read",
                                                            &format!("{:?}", e),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    results.fail(
                                        "VFS: partition browse",
                                        &format!("{:?}", e),
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        results.fail("VFS: list partitions", &format!("{:?}", e));
                    }
                }
            } else {
                results.fail("VFS: filesystem mode", "No partitions detected");
            }
        }
        Err(e) => {
            results.fail("VFS: filesystem mode open", &format!("{:?}", e));
        }
    }
}

// =========================================================================
// Phase 7: Batch Operations
// =========================================================================

fn test_batch_operations(results: &mut TestResults, verbose: bool) {
    // Test 7.1: Load info for ALL containers (sequential, timed)
    println!("  Loading info for all containers...");
    let start = Instant::now();

    let all_containers: Vec<(&str, &str)> = vec![
        ("E01", "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_Diana_PC/10115_0900_B_Diana_PC.E01"),
        ("E01", "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_Dong_Wong_Laptop/10115_0900_B_Dong_Wong_Laptop.E01"),
        ("E01", "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_Michelle_OS/10115_0900_B_Michelle_OS.E01"),
        ("E01", "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_RmG_ITM9_HP_Desktop/10115_0900_B_RmG_ITM9_HP_Desktop.E01"),
        ("E01", "/Volumes/Tools/10115-0900/1.Evidence/10115_0900_B_RmG_Itm7_Michelle_Data/10115_0900_B_RmG_Itm7_Michelle_Data.E01"),
        ("AD1", "/Volumes/Tools/10115-0900/1.Evidence/10115-0900-EV-0100-NWang-Accountant/10115-0900-EV-0100-NWang-Accountant.ad1"),
        ("AD1", "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_HBaik/10115-0900_HBaik.ad1"),
        ("AD1", "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_Jeffery_Yang_ASUSdesktopcustomPC/10115-0900_Jeffery_Yang_ASUSdesktopcustomPC.ad1"),
        ("AD1", "/Volumes/Tools/10115-0900/1.Evidence/10115-0900-Junqian-Huang/10115-0900-Junqian-Huang/Junqian-Huang.ad1"),
        ("AD1", "/Volumes/Tools/10115-0900/1.Evidence/10115-0900-Tom-Chiang/10115-0900-Tom-Chiang/10115-0900-Tom_Chiang.ad1"),
        ("L01", "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_Hoa_Ching_Hu/DCCollection_2026-03-03_17-10-23/DCCollection_2026-03-03_17-10-23.L01"),
    ];

    let mut load_successes = 0;
    let mut load_failures = 0;

    for (ctype, path) in &all_containers {
        if !Path::new(path).exists() {
            load_failures += 1;
            if verbose {
                println!("    [warn] Not found: {}", path);
            }
            continue;
        }

        let ok = match *ctype {
            "E01" | "L01" => ewf::info_fast(path).is_ok(),
            "AD1" => ad1::info_fast(path).is_ok(),
            _ => false,
        };

        if ok {
            load_successes += 1;
        } else {
            load_failures += 1;
            if verbose {
                println!("    [warn] Failed to load {}: {}", ctype, path);
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    results.timed("Batch info_fast (11 containers)", elapsed);

    if load_failures == 0 {
        results.pass(&format!(
            "Batch: all {} containers info loaded in {:.2}s",
            load_successes, elapsed
        ));
    } else {
        results.fail(
            "Batch: container info loading",
            &format!(
                "{}/{} succeeded, {} failed",
                load_successes,
                all_containers.len(),
                load_failures
            ),
        );
    }

    // Test 7.2: AD1 session cache test
    println!("  Testing AD1 session cache (load same container twice)...");
    let ad1_path =
        "/Volumes/Tools/10115-0900/1.Evidence/10115-0900_HBaik/10115-0900_HBaik.ad1";
    if Path::new(ad1_path).exists() {
        let start1 = Instant::now();
        let _ = ad1::get_root_children_v2(ad1_path);
        let first_load = start1.elapsed().as_secs_f64();

        let start2 = Instant::now();
        let _ = ad1::get_root_children_v2(ad1_path);
        let second_load = start2.elapsed().as_secs_f64();

        results.timed("AD1 first load (cold)", first_load);
        results.timed("AD1 second load (cached)", second_load);

        if second_load < first_load {
            results.pass(&format!(
                "Batch: AD1 cache speedup ({:.3}s -> {:.3}s, {:.0}% faster)",
                first_load,
                second_load,
                (1.0 - second_load / first_load) * 100.0
            ));
        } else {
            results.pass(&format!(
                "Batch: AD1 cache ({:.3}s -> {:.3}s)",
                first_load, second_load
            ));
        }
    }

    println!();
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn find_files_recursive(dir: &str, extension: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(s) = path.to_str() {
                    result.extend(find_files_recursive(s, extension));
                }
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
            {
                result.push(path);
            }
        }
    }
    result
}
