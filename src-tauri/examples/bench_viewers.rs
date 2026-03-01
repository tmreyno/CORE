// =============================================================================
// CORE-FFX - Viewer Performance Benchmark
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================
//
// Benchmark viewers and container operations
// Run with: cargo run --example bench_viewers -- [test_case]
//
// Test cases:
//   all        - Run all tests (default)
//   viewers    - Test document viewers only
//   containers - Test container operations only

use std::path::Path;
use std::time::{Duration, Instant};

// Test file paths - update these to match your system
const AD1_FILE: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_GRCDH2/02606-0900_1E_GRCDH2_IMG1.ad1";
const E01_FILE: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/PC-MUS-001.E01";
const UFED_DIR: &str =
    "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/02606-0900_1E_00216P";
const ZIP_FILE: &str = "/Users/terryreynolds/1827-1001 Case With Data /1.Evidence/takeout-20220222T154448Z/takeout-20220222T154448Z-001.zip";

struct BenchResult {
    name: String,
    duration: Duration,
    success: bool,
    details: String,
}

impl BenchResult {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            duration: Duration::ZERO,
            success: false,
            details: String::new(),
        }
    }

    fn pass(mut self, duration: Duration, details: &str) -> Self {
        self.duration = duration;
        self.success = true;
        self.details = details.to_string();
        self
    }

    fn fail(mut self, err: &str) -> Self {
        self.success = false;
        self.details = err.to_string();
        self
    }
}

fn format_duration(d: Duration) -> String {
    if d.as_secs() >= 1 {
        format!("{:.2}s", d.as_secs_f64())
    } else if d.as_millis() >= 1 {
        format!("{}ms", d.as_millis())
    } else {
        format!("{}µs", d.as_micros())
    }
}

fn bench_container_detection() -> Vec<BenchResult> {
    use ffx_check_lib::common::detect_container_type;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  CONTAINER TYPE DETECTION                                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    let containers = vec![
        ("AD1", AD1_FILE),
        ("E01", E01_FILE),
        ("UFED", UFED_DIR),
        ("ZIP", ZIP_FILE),
    ];

    for (name, path) in containers {
        let result = BenchResult::new(&format!("Detect {}", name));

        if !Path::new(path).exists() {
            results.push(result.fail("File not found"));
            println!("  ⏭️  {} - SKIP (file not found)", name);
            continue;
        }

        let start = Instant::now();
        let detected = detect_container_type(path);
        let duration = start.elapsed();

        let details = format!("{:?}", detected);
        println!(
            "  ✅ {} - {} -> {:?}",
            name,
            format_duration(duration),
            detected
        );
        results.push(result.pass(duration, &details));
    }

    results
}

fn bench_ad1_operations() -> Vec<BenchResult> {
    use ffx_check_lib::ad1;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  AD1 CONTAINER OPERATIONS                                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    if !Path::new(AD1_FILE).exists() {
        println!("  ⏭️  AD1 file not found - skipping all AD1 tests");
        return results;
    }

    // Test: Get info (fast)
    {
        let result = BenchResult::new("AD1 Info Fast");
        let start = Instant::now();
        match ad1::info_fast(AD1_FILE) {
            Ok(info) => {
                let duration = start.elapsed();
                let details = format!(
                    "items={}, version={}",
                    info.item_count, info.logical.image_version
                );
                println!(
                    "  ✅ Info Fast - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Info Fast - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test: Get full info
    {
        let result = BenchResult::new("AD1 Info Full");
        let start = Instant::now();
        match ad1::info(AD1_FILE, false) {
            Ok(info) => {
                let duration = start.elapsed();
                let details = format!("items={}", info.item_count);
                println!(
                    "  ✅ Info Full - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Info Full - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test: Build tree
    {
        let result = BenchResult::new("AD1 Get Tree");
        let start = Instant::now();
        match ad1::get_tree(AD1_FILE) {
            Ok(tree) => {
                let duration = start.elapsed();
                let details = format!("{} entries", tree.len());
                println!(
                    "  ✅ Get Tree - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Get Tree - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test: Get root children (lazy loading first level)
    {
        let result = BenchResult::new("AD1 Get Root Children (V2)");
        let start = Instant::now();
        match ad1::get_root_children_v2(AD1_FILE) {
            Ok(children) => {
                let duration = start.elapsed();
                let details = format!("{} items", children.len());
                println!(
                    "  ✅ Get Root Children V2 - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Get Root Children V2 - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test: Get children at address (lazy loading - subsequent levels)
    {
        let result = BenchResult::new("AD1 Children@Addr (V2)");
        let start = Instant::now();
        // Use address 0 (root) for this test
        match ad1::get_children_at_addr_v2(AD1_FILE, 0, "/") {
            Ok(children) => {
                let duration = start.elapsed();
                let details = format!("{} items", children.len());
                println!(
                    "  ✅ Get Children@Addr V2 - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Get Children@Addr V2 - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    results
}

fn bench_e01_operations() -> Vec<BenchResult> {
    use ffx_check_lib::ewf;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  E01/EWF CONTAINER OPERATIONS                                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    if !Path::new(E01_FILE).exists() {
        println!("  ⏭️  E01 file not found - skipping all E01 tests");
        return results;
    }

    // Test: Get info fast
    {
        let result = BenchResult::new("E01 Info Fast");
        let start = Instant::now();
        match ewf::info_fast(E01_FILE) {
            Ok(info) => {
                let duration = start.elapsed();
                let size = format_size(info.total_size);
                let details = format!("total_size={}, segments={}", size, info.segment_count);
                println!(
                    "  ✅ Info Fast - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Info Fast - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test: Full info
    {
        let result = BenchResult::new("E01 Info Full");
        let start = Instant::now();
        match ewf::info(E01_FILE) {
            Ok(info) => {
                let duration = start.elapsed();
                let size = format_size(info.total_size);
                let details = format!("total_size={}", size);
                println!(
                    "  ✅ Info Full - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Info Full - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    results
}

fn bench_archive_operations() -> Vec<BenchResult> {
    use ffx_check_lib::archive;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  ARCHIVE OPERATIONS                                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    if !Path::new(ZIP_FILE).exists() {
        println!("  ⏭️  ZIP file not found - skipping archive tests");
        return results;
    }

    // Test: List archive
    {
        let result = BenchResult::new("Archive List ZIP");
        let start = Instant::now();
        match archive::list_zip_entries(ZIP_FILE) {
            Ok(entries) => {
                let duration = start.elapsed();
                let details = format!("{} entries", entries.len());
                println!(
                    "  ✅ List ZIP - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ List ZIP - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test: Get entry count
    {
        let result = BenchResult::new("Archive Entry Count");
        let start = Instant::now();
        match archive::get_zip_entry_count(ZIP_FILE) {
            Ok(count) => {
                let duration = start.elapsed();
                let details = format!("{} entries", count);
                println!(
                    "  ✅ Entry Count - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Entry Count - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test: Get root entries (lazy)
    {
        let result = BenchResult::new("Archive Root Entries");
        let start = Instant::now();
        match archive::get_zip_root_entries(ZIP_FILE) {
            Ok(entries) => {
                let duration = start.elapsed();
                let details = format!("{} root entries", entries.len());
                println!(
                    "  ✅ Root Entries - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Root Entries - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    results
}

fn bench_document_viewers() -> Vec<BenchResult> {
    use ffx_check_lib::viewer::document::universal;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  DOCUMENT VIEWER OPERATIONS                                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    // Test format detection speed
    let test_files = vec![
        ("Markdown", "/Users/terryreynolds/GitHub/CORE-1/README.md"),
        ("JSON", "/Users/terryreynolds/GitHub/CORE-1/package.json"),
        (
            "Rust",
            "/Users/terryreynolds/GitHub/CORE-1/src-tauri/src/lib.rs",
        ),
        (
            "TOML",
            "/Users/terryreynolds/GitHub/CORE-1/src-tauri/Cargo.toml",
        ),
    ];

    println!("  === Format Detection ===\n");
    for (name, path) in &test_files {
        let result = BenchResult::new(&format!("Detect {}", name));

        if !Path::new(path).exists() {
            results.push(result.fail("File not found"));
            println!("    ⏭️  {} - SKIP", name);
            continue;
        }

        let start = Instant::now();
        let format = universal::UniversalFormat::from_path(path);
        let duration = start.elapsed();

        let details = format!("{:?}", format);
        println!(
            "    ✅ {} - {} -> {:?}",
            name,
            format_duration(duration),
            format
        );
        results.push(result.pass(duration, &details));
    }

    // Test file info extraction
    println!("\n  === File Info Extraction ===\n");
    for (name, path) in &test_files {
        let result = BenchResult::new(&format!("Info {}", name));

        if !Path::new(path).exists() {
            results.push(result.fail("File not found"));
            continue;
        }

        let start = Instant::now();
        let info = universal::FileInfo::from_path(path);
        let duration = start.elapsed();

        match info {
            Ok(info) => {
                let details = format!("size={}, mime={}", format_size(info.size), info.mime_type);
                println!(
                    "    ✅ {} - {} ({})",
                    name,
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("    ❌ {} - FAIL: {}", name, e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    results
}

fn bench_spreadsheet_viewer() -> Vec<BenchResult> {
    use ffx_check_lib::viewer::document::spreadsheet;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  SPREADSHEET VIEWER OPERATIONS                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    // Look for Excel files in the evidence folder
    let xlsx_files =
        find_files_with_extension("/Users/terryreynolds/1827-1001 Case With Data ", "xlsx", 1);
    let csv_files =
        find_files_with_extension("/Users/terryreynolds/1827-1001 Case With Data ", "csv", 1);

    // Test XLSX
    if let Some(xlsx_path) = xlsx_files.first() {
        println!("  Testing XLSX: {}\n", xlsx_path);

        let result = BenchResult::new("XLSX Info");
        let start = Instant::now();
        match spreadsheet::read_spreadsheet_info(xlsx_path) {
            Ok(info) => {
                let duration = start.elapsed();
                let details = format!("{} sheets", info.total_sheets);
                println!(
                    "    ✅ Get Info - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));

                // Test reading first sheet
                if !info.sheets.is_empty() {
                    let result = BenchResult::new("XLSX Read Sheet");
                    let sheet_name = &info.sheets[0].name;
                    let start = Instant::now();
                    match spreadsheet::read_sheet(xlsx_path, sheet_name, 0, 100) {
                        Ok(rows) => {
                            let duration = start.elapsed();
                            let details = format!("{} rows", rows.len());
                            println!(
                                "    ✅ Read Sheet - {} ({})",
                                format_duration(duration),
                                details
                            );
                            results.push(result.pass(duration, &details));
                        }
                        Err(e) => {
                            println!("    ❌ Read Sheet - FAIL: {}", e);
                            results.push(result.fail(&e.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                println!("    ❌ Get Info - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    } else {
        println!("  ⏭️  No XLSX files found");
    }

    // Test CSV
    if let Some(csv_path) = csv_files.first() {
        println!("\n  Testing CSV: {}\n", csv_path);

        let result = BenchResult::new("CSV Read");
        let start = Instant::now();
        match spreadsheet::read_csv(csv_path, Some(100)) {
            Ok((headers, rows)) => {
                let duration = start.elapsed();
                let details = format!("{} cols, {} rows", headers.len(), rows.len());
                println!(
                    "    ✅ Read CSV - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("    ❌ Read CSV - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    } else {
        println!("  ⏭️  No CSV files found");
    }

    results
}

fn bench_hex_viewer() -> Vec<BenchResult> {
    use ffx_check_lib::viewer;

    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  HEX VIEWER OPERATIONS                                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut results = Vec::new();

    let test_file = "/Users/terryreynolds/GitHub/CORE-1/src-tauri/Cargo.toml";

    if !Path::new(test_file).exists() {
        println!("  ⏭️  Test file not found");
        return results;
    }

    // Test reading chunk
    {
        let result = BenchResult::new("Hex Read 64KB");
        let start = Instant::now();
        match viewer::read_file_chunk(test_file, 0, Some(65536)) {
            Ok(data) => {
                let duration = start.elapsed();
                let details = format!("{} bytes", data.bytes.len());
                println!(
                    "  ✅ Read 64KB - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Read 64KB - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    // Test header parsing
    {
        let result = BenchResult::new("Parse Header");
        let start = Instant::now();
        match viewer::parse_file_header(test_file) {
            Ok(metadata) => {
                let duration = start.elapsed();
                let details = format!(
                    "format={}, {} regions",
                    metadata.format,
                    metadata.regions.len()
                );
                println!(
                    "  ✅ Parse Header - {} ({})",
                    format_duration(duration),
                    details
                );
                results.push(result.pass(duration, &details));
            }
            Err(e) => {
                println!("  ❌ Parse Header - FAIL: {}", e);
                results.push(result.fail(&e.to_string()));
            }
        }
    }

    results
}

// Helper: Format file size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// Helper: Find files with extension
fn find_files_with_extension(dir: &str, ext: &str, max: usize) -> Vec<String> {
    let mut found = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if found.len() >= max {
                break;
            }
            let path = entry.path();
            if path.is_file() {
                if let Some(e) = path.extension() {
                    if e.to_string_lossy().eq_ignore_ascii_case(ext) {
                        found.push(path.to_string_lossy().to_string());
                    }
                }
            } else if path.is_dir() {
                // Recurse (limited depth)
                let sub =
                    find_files_with_extension(&path.to_string_lossy(), ext, max - found.len());
                found.extend(sub);
            }
        }
    }
    found
}

fn print_summary(results: &[BenchResult]) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  BENCHMARK SUMMARY                                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let passed = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();
    let total_time: Duration = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.duration)
        .sum();

    println!("  Total Tests:  {}", results.len());
    println!("  Passed:       {} ✅", passed);
    println!("  Failed:       {} ❌", failed);
    println!("  Total Time:   {}", format_duration(total_time));

    // Performance breakdown
    println!("\n  === Performance Breakdown (slowest first) ===\n");

    let mut sorted: Vec<_> = results.iter().filter(|r| r.success).collect();
    sorted.sort_by(|a, b| b.duration.cmp(&a.duration));

    println!("  {:40} {:>12}", "Operation", "Time");
    println!("  {:-<40} {:-<12}", "", "");

    for r in sorted.iter().take(15) {
        let status = if r.duration.as_millis() > 1000 {
            "🐌"
        } else if r.duration.as_millis() > 100 {
            "⚠️ "
        } else {
            "⚡"
        };
        println!(
            "  {} {:38} {:>10}",
            status,
            r.name,
            format_duration(r.duration)
        );
    }

    // Performance ratings
    println!("\n  Legend: ⚡ <100ms (fast) | ⚠️  100ms-1s (acceptable) | 🐌 >1s (slow)");

    // Failed operations
    if failed > 0 {
        println!("\n  === Failed Operations ===\n");
        for r in results.iter().filter(|r| !r.success) {
            println!("  ❌ {}: {}", r.name, r.details);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let test_case = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    println!("\n");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         CORE-FFX VIEWER PERFORMANCE BENCHMARK                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("\n  Running: {} tests\n", test_case);

    let mut all_results = Vec::new();

    match test_case {
        "containers" => {
            all_results.extend(bench_container_detection());
            all_results.extend(bench_ad1_operations());
            all_results.extend(bench_e01_operations());
            all_results.extend(bench_archive_operations());
        }
        "viewers" => {
            all_results.extend(bench_document_viewers());
            all_results.extend(bench_spreadsheet_viewer());
            all_results.extend(bench_hex_viewer());
        }
        "ad1" => {
            all_results.extend(bench_ad1_operations());
        }
        "e01" => {
            all_results.extend(bench_e01_operations());
        }
        _ => {
            // Run all
            all_results.extend(bench_container_detection());
            all_results.extend(bench_ad1_operations());
            all_results.extend(bench_e01_operations());
            all_results.extend(bench_archive_operations());
            all_results.extend(bench_document_viewers());
            all_results.extend(bench_spreadsheet_viewer());
            all_results.extend(bench_hex_viewer());
        }
    }

    print_summary(&all_results);
}
