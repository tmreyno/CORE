// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { generateReport, exportAsJson, exportAsMarkdown, type ReportInput } from "../generator";
import type { ForensicReport } from "../types";
import type { DiscoveredFile, ContainerInfo } from "../../types";

// =============================================================================
// Test Helpers
// =============================================================================

function makeFile(overrides: Partial<DiscoveredFile> = {}): DiscoveredFile {
  return {
    path: "/evidence/test.ad1",
    filename: "test.ad1",
    container_type: "ad1",
    size: 1024000,
    segment_count: 1,
    created: "2024-01-15T10:30:00Z",
    modified: "2024-01-15T10:30:00Z",
    ...overrides,
  };
}

function makeContainerInfo(overrides: Partial<ContainerInfo> = {}): ContainerInfo {
  return {
    container: "ad1",
    ...overrides,
  };
}

function makeInput(overrides: Partial<ReportInput> = {}): ReportInput {
  return {
    files: [makeFile()],
    fileInfoMap: new Map(),
    fileHashMap: new Map(),
    workingDirectory: "/evidence",
    ...overrides,
  };
}

// =============================================================================
// generateReport Tests
// =============================================================================

describe("Report Generator", () => {
  describe("generateReport", () => {
    it("should generate a report with correct schema version", () => {
      const report = generateReport(makeInput());
      expect(report.schemaVersion).toBe("1.0");
    });

    it("should include meta information", () => {
      const report = generateReport(makeInput({
        title: "Test Report",
        notes: "Some notes",
      }));
      expect(report.meta.generatedBy).toContain("FFX");
      expect(report.meta.title).toBe("Test Report");
      expect(report.meta.notes).toBe("Some notes");
      expect(report.meta.generatedAt).toBeTruthy();
      expect(report.meta.appVersion).toBeTruthy();
    });

    it("should include session information", () => {
      const files = [makeFile(), makeFile({ path: "/evidence/test2.e01", filename: "test2.e01" })];
      const hashMap = new Map([
        ["/evidence/test.ad1", { algorithm: "SHA-256", hash: "abc123" }],
      ]);
      const report = generateReport(makeInput({ files, fileHashMap: hashMap }));
      expect(report.session?.workingDirectory).toBe("/evidence");
      expect(report.session?.filesDiscovered).toBe(2);
      expect(report.session?.filesProcessed).toBe(1);
    });

    it("should generate evidence items for each file", () => {
      const files = [
        makeFile({ path: "/evidence/img1.ad1", filename: "img1.ad1" }),
        makeFile({ path: "/evidence/img2.e01", filename: "img2.e01", container_type: "e01" }),
      ];
      const report = generateReport(makeInput({ files }));
      expect(report.evidence).toHaveLength(2);
      expect(report.evidence[0].filename).toBe("img1.ad1");
      expect(report.evidence[1].filename).toBe("img2.e01");
    });

    it("should populate evidence item fields correctly", () => {
      const file = makeFile({
        path: "/evidence/test.ad1",
        filename: "test.ad1",
        container_type: "ad1",
        size: 2048000,
        segment_count: 3,
        created: "2024-06-01T08:00:00Z",
        modified: "2024-06-15T12:00:00Z",
      });
      const report = generateReport(makeInput({ files: [file] }));
      const item = report.evidence[0];
      expect(item.id).toBe("/evidence/test.ad1");
      expect(item.filename).toBe("test.ad1");
      expect(item.path).toBe("/evidence/test.ad1");
      expect(item.containerType).toBe("ad1");
      expect(item.size).toBe(2048000);
      expect(item.segmentCount).toBe(3);
    });

    // =========================================================================
    // Case Info Extraction
    // =========================================================================

    it("should extract case info from AD1 companion log", () => {
      const info = makeContainerInfo({
        ad1: {
          segment: { signature: "ADSEGMENTEDFILE", segment_index: 0, segment_number: 1, fragments_size: 0, header_size: 0 },
          logical: { signature: "AD", image_version: 3, zlib_chunk_size: 65536, logical_metadata_addr: 0, first_item_addr: 0, data_source_name_length: 0, ad_signature: "AD", data_source_name_addr: 0, attrguid_footer_addr: 0, locsguid_footer_addr: 0, data_source_name: "Test Source" },
          item_count: 100,
          companion_log: {
            case_number: "CASE-2024-001",
            evidence_number: "EV-001",
            examiner: "John Doe",
            notes: "Test acquisition",
          },
        },
      });
      const infoMap = new Map([["/evidence/test.ad1", info]]);
      const report = generateReport(makeInput({ fileInfoMap: infoMap }));
      expect(report.case.caseNumber).toBe("CASE-2024-001");
      expect(report.case.evidenceNumber).toBe("EV-001");
      expect(report.case.examiner).toBe("John Doe");
      expect(report.case.notes).toBe("Test acquisition");
    });

    it("should extract case info from E01 metadata", () => {
      const info = makeContainerInfo({
        container: "e01",
        e01: {
          format_version: "EWF-E01",
          segment_count: 5,
          sector_count: 1000000,
          bytes_per_sector: 512,
          chunk_count: 500,
          sectors_per_chunk: 64,
          total_size: 512000000,
          compression: "zlib",
          case_number: "E01-CASE-123",
          evidence_number: "E01-EV-42",
          examiner_name: "Jane Smith",
          notes: "Disk image",
        },
      });
      const file = makeFile({ path: "/evidence/disk.e01", filename: "disk.e01", container_type: "e01" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      expect(report.case.caseNumber).toBe("E01-CASE-123");
      expect(report.case.evidenceNumber).toBe("E01-EV-42");
      expect(report.case.examiner).toBe("Jane Smith");
    });

    it("should extract case info from UFED metadata", () => {
      const info = makeContainerInfo({
        container: "ufed",
        ufed: {
          format: "UFDR",
          size: 100000,
          associated_files: [],
          is_extraction_set: true,
          case_info: {
            case_identifier: "UFED-001",
            device_name: "iPhone 15",
            examiner_name: "Agent X",
            department: "Cyber Division",
            location: "Lab A",
          },
        },
      });
      const file = makeFile({ path: "/evidence/phone.ufd", filename: "phone.ufd", container_type: "ufed" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      expect(report.case.caseNumber).toBe("UFED-001");
      expect(report.case.department).toBe("Cyber Division");
      expect(report.case.location).toBe("Lab A");
    });

    it("should allow case info override to take precedence", () => {
      const info = makeContainerInfo({
        container: "e01",
        e01: {
          format_version: "EWF-E01",
          segment_count: 1, sector_count: 100, bytes_per_sector: 512,
          chunk_count: 10, sectors_per_chunk: 64, total_size: 51200,
          compression: "zlib",
          case_number: "AUTO-DETECTED",
        },
      });
      const infoMap = new Map([["/evidence/test.ad1", info]]);
      const report = generateReport(makeInput({
        fileInfoMap: infoMap,
        caseInfo: { caseNumber: "MANUAL-OVERRIDE" },
      }));
      expect(report.case.caseNumber).toBe("MANUAL-OVERRIDE");
    });

    // =========================================================================
    // Hash Record Generation
    // =========================================================================

    it("should generate hash records from computed hashes", () => {
      const hashMap = new Map([
        ["/evidence/test.ad1", { algorithm: "SHA-256", hash: "abcdef1234567890" }],
      ]);
      const report = generateReport(makeInput({ fileHashMap: hashMap }));
      expect(report.hashes).toHaveLength(1);
      expect(report.hashes[0].algorithm).toBe("SHA-256");
      expect(report.hashes[0].computedHash).toBe("ABCDEF1234567890");
      expect(report.hashes[0].source).toBe("computed");
    });

    it("should include stored hashes from E01 container", () => {
      const info = makeContainerInfo({
        container: "e01",
        e01: {
          format_version: "EWF-E01",
          segment_count: 1, sector_count: 100, bytes_per_sector: 512,
          chunk_count: 10, sectors_per_chunk: 64, total_size: 51200,
          compression: "zlib",
          stored_hashes: [
            { algorithm: "MD5", hash: "d41d8cd98f00b204e9800998ecf8427e", verified: null, timestamp: null, source: "container" },
            { algorithm: "SHA-1", hash: "da39a3ee5e6b4b0d3255bfef95601890afd80709", verified: null, timestamp: null, source: "container" },
          ],
        },
      });
      const file = makeFile({ path: "/evidence/disk.e01", filename: "disk.e01", container_type: "e01" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      // Should have both stored hashes as records (no computed hashes)
      expect(report.hashes.length).toBeGreaterThanOrEqual(2);
      const md5 = report.hashes.find(h => h.algorithm === "MD5");
      const sha1 = report.hashes.find(h => h.algorithm === "SHA-1");
      expect(md5?.storedHash).toBe("D41D8CD98F00B204E9800998ECF8427E");
      expect(sha1?.storedHash).toBe("DA39A3EE5E6B4B0D3255BFEF95601890AFD80709");
    });

    it("should match computed hash with stored hash for verification", () => {
      const info = makeContainerInfo({
        container: "e01",
        e01: {
          format_version: "EWF-E01",
          segment_count: 1, sector_count: 100, bytes_per_sector: 512,
          chunk_count: 10, sectors_per_chunk: 64, total_size: 51200,
          compression: "zlib",
          stored_hashes: [
            { algorithm: "SHA-256", hash: "abcdef", verified: true, timestamp: "2024-01-01T00:00:00Z", source: "container" },
          ],
        },
      });
      const file = makeFile({ path: "/evidence/disk.e01", filename: "disk.e01", container_type: "e01" });
      const infoMap = new Map([[file.path, info]]);
      const hashMap = new Map([[file.path, { algorithm: "SHA-256", hash: "abcdef", verified: true }]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap, fileHashMap: hashMap }));
      // Should have 1 hash record (computed + stored merged)
      const sha256Records = report.hashes.filter(h => h.algorithm === "SHA-256");
      expect(sha256Records).toHaveLength(1);
      expect(sha256Records[0].computedHash).toBe("ABCDEF");
      expect(sha256Records[0].storedHash).toBe("ABCDEF");
    });

    it("should handle files with no info and no hashes", () => {
      const report = generateReport(makeInput());
      expect(report.evidence).toHaveLength(1);
      expect(report.hashes).toHaveLength(0);
      expect(report.evidence[0].metadata.format).toBe("ad1");
    });

    // =========================================================================
    // Device & Extraction Info
    // =========================================================================

    it("should extract device info from UFED container", () => {
      const info = makeContainerInfo({
        container: "ufed",
        ufed: {
          format: "UFDR",
          size: 100000,
          associated_files: [],
          is_extraction_set: true,
          device_info: {
            vendor: "Apple",
            model: "iPhone 15",
            full_name: "Apple iPhone 15 Pro",
            serial_number: "ABC123",
            imei: "123456789012345",
            os_version: "17.2",
          },
        },
      });
      const file = makeFile({ path: "/evidence/phone.ufd", filename: "phone.ufd", container_type: "ufed" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      const device = report.evidence[0].device;
      expect(device).toBeDefined();
      expect(device?.vendor).toBe("Apple");
      expect(device?.model).toBe("iPhone 15");
      expect(device?.serialNumber).toBe("ABC123");
      expect(device?.imei).toBe("123456789012345");
    });

    it("should extract extraction info from UFED container", () => {
      const info = makeContainerInfo({
        container: "ufed",
        ufed: {
          format: "UFDR",
          size: 100000,
          associated_files: [],
          is_extraction_set: true,
          extraction_info: {
            acquisition_tool: "Cellebrite UFED",
            tool_version: "7.65",
            extraction_type: "Advanced Logical",
            connection_type: "USB",
            start_time: "2024-03-01T09:00:00Z",
            end_time: "2024-03-01T10:30:00Z",
          },
        },
      });
      const file = makeFile({ path: "/evidence/phone.ufd", filename: "phone.ufd", container_type: "ufed" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      const extraction = report.evidence[0].extraction;
      expect(extraction).toBeDefined();
      expect(extraction?.tool).toBe("Cellebrite UFED");
      expect(extraction?.toolVersion).toBe("7.65");
      expect(extraction?.extractionType).toBe("Advanced Logical");
    });

    it("should extract device info from E01 container", () => {
      const info = makeContainerInfo({
        container: "e01",
        e01: {
          format_version: "EWF-E01",
          segment_count: 1, sector_count: 100, bytes_per_sector: 512,
          chunk_count: 10, sectors_per_chunk: 64, total_size: 51200,
          compression: "zlib",
          model: "Samsung SSD 870",
          serial_number: "S6P2NS0T123456",
        },
      });
      const file = makeFile({ path: "/evidence/disk.e01", filename: "disk.e01", container_type: "e01" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      const device = report.evidence[0].device;
      expect(device?.model).toBe("Samsung SSD 870");
      expect(device?.serialNumber).toBe("S6P2NS0T123456");
    });

    // =========================================================================
    // Container Metadata
    // =========================================================================

    it("should generate AD1 metadata correctly", () => {
      const info = makeContainerInfo({
        ad1: {
          segment: { signature: "ADSEGMENTEDFILE", segment_index: 0, segment_number: 1, fragments_size: 0, header_size: 0 },
          logical: {
            signature: "AD", image_version: 3, zlib_chunk_size: 65536,
            logical_metadata_addr: 0, first_item_addr: 0, data_source_name_length: 10,
            ad_signature: "AD", data_source_name_addr: 0,
            attrguid_footer_addr: 0, locsguid_footer_addr: 0,
            data_source_name: "C:\\Evidence\\Disk1",
          },
          item_count: 500,
        },
      });
      const infoMap = new Map([["/evidence/test.ad1", info]]);
      const report = generateReport(makeInput({ fileInfoMap: infoMap }));
      const meta = report.evidence[0].metadata;
      expect(meta.format).toContain("AD1");
      expect(meta.itemCount).toBe(500);
      expect(meta.sourceDescription).toBe("C:\\Evidence\\Disk1");
    });

    it("should generate Archive metadata correctly", () => {
      const info = makeContainerInfo({
        container: "archive",
        archive: {
          format: "7z",
          segment_count: 1, total_size: 5000000,
          segment_names: ["test.7z"], segment_sizes: [5000000],
          first_segment: "test.7z", last_segment: "test.7z",
          is_multipart: false,
          entry_count: 42,
          encrypted_headers: false,
          aes_encrypted: true,
          version: "0.4",
        },
      });
      const file = makeFile({ path: "/evidence/test.7z", filename: "test.7z", container_type: "archive" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      const meta = report.evidence[0].metadata;
      expect(meta.format).toBe("7z");
      expect(meta.entryCount).toBe(42);
      expect(meta.encryption?.encrypted).toBe(true);
    });

    // =========================================================================
    // Multiple Files
    // =========================================================================

    it("should handle multiple files with mixed types", () => {
      const files = [
        makeFile({ path: "/ev/img1.ad1", filename: "img1.ad1", container_type: "ad1" }),
        makeFile({ path: "/ev/img2.e01", filename: "img2.e01", container_type: "e01" }),
        makeFile({ path: "/ev/phone.ufd", filename: "phone.ufd", container_type: "ufed" }),
      ];
      const hashMap = new Map([
        ["/ev/img1.ad1", { algorithm: "MD5", hash: "aaa" }],
        ["/ev/img2.e01", { algorithm: "SHA-256", hash: "bbb", verified: true }],
      ]);
      const report = generateReport(makeInput({ files, fileHashMap: hashMap, workingDirectory: "/ev" }));
      expect(report.evidence).toHaveLength(3);
      expect(report.hashes).toHaveLength(2);
      expect(report.session?.filesDiscovered).toBe(3);
      expect(report.session?.filesProcessed).toBe(2);
    });

    it("should handle empty file list", () => {
      const report = generateReport(makeInput({ files: [] }));
      expect(report.evidence).toHaveLength(0);
      expect(report.hashes).toHaveLength(0);
      expect(report.session?.filesDiscovered).toBe(0);
    });
  });

  // ===========================================================================
  // exportAsJson Tests
  // ===========================================================================

  describe("exportAsJson", () => {
    it("should produce valid JSON", () => {
      const report = generateReport(makeInput());
      const json = exportAsJson(report);
      expect(() => JSON.parse(json)).not.toThrow();
    });

    it("should pretty-print by default", () => {
      const report = generateReport(makeInput());
      const json = exportAsJson(report);
      expect(json).toContain("\n");
      expect(json).toContain("  ");
    });

    it("should produce compact JSON when pretty=false", () => {
      const report = generateReport(makeInput());
      const compact = exportAsJson(report, false);
      expect(compact).not.toContain("\n");
    });

    it("should preserve all report fields in JSON", () => {
      const report = generateReport(makeInput({ title: "Test", notes: "Note" }));
      const parsed: ForensicReport = JSON.parse(exportAsJson(report));
      expect(parsed.schemaVersion).toBe("1.0");
      expect(parsed.meta.title).toBe("Test");
      expect(parsed.meta.notes).toBe("Note");
    });
  });

  // ===========================================================================
  // exportAsMarkdown Tests
  // ===========================================================================

  describe("exportAsMarkdown", () => {
    it("should produce a markdown string with title header", () => {
      const report = generateReport(makeInput({ title: "Evidence Report" }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("# Evidence Report");
    });

    it("should use default title when none provided", () => {
      const report = generateReport(makeInput());
      const md = exportAsMarkdown(report);
      expect(md).toContain("# Forensic Evidence Report");
    });

    it("should include case information table when available", () => {
      const report = generateReport(makeInput({
        caseInfo: {
          caseNumber: "CASE-001",
          examiner: "Forensic Analyst",
          department: "Digital Forensics",
        },
      }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("## Case Information");
      expect(md).toContain("CASE-001");
      expect(md).toContain("Forensic Analyst");
      expect(md).toContain("Digital Forensics");
    });

    it("should include evidence items section", () => {
      const files = [
        makeFile({ path: "/ev/image.ad1", filename: "image.ad1", container_type: "ad1", size: 5242880 }),
      ];
      const report = generateReport(makeInput({ files }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("## Evidence Items");
      expect(md).toContain("### image.ad1");
      expect(md).toContain("**Type:** ad1");
    });

    it("should include hash verification section with status indicators", () => {
      const hashMap = new Map([
        ["/evidence/test.ad1", { algorithm: "SHA-256", hash: "abc", verified: true }],
      ]);
      const report = generateReport(makeInput({ fileHashMap: hashMap }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("## Hash Verification");
      expect(md).toContain("✓ VERIFIED");
    });

    it("should show MISMATCH status for failed verification", () => {
      const hashMap = new Map([
        ["/evidence/test.ad1", { algorithm: "MD5", hash: "xyz", verified: false }],
      ]);
      const report = generateReport(makeInput({ fileHashMap: hashMap }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("✗ MISMATCH");
    });

    it("should include device info in markdown", () => {
      const info = makeContainerInfo({
        container: "ufed",
        ufed: {
          format: "UFDR", size: 100, associated_files: [], is_extraction_set: true,
          device_info: { vendor: "Samsung", model: "Galaxy S23" },
        },
      });
      const file = makeFile({ path: "/ev/phone.ufd", filename: "phone.ufd", container_type: "ufed" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("#### Device Information");
      expect(md).toContain("Samsung");
      expect(md).toContain("Galaxy S23");
    });

    it("should include extraction info in markdown", () => {
      const info = makeContainerInfo({
        container: "ufed",
        ufed: {
          format: "UFDR", size: 100, associated_files: [], is_extraction_set: true,
          extraction_info: {
            acquisition_tool: "UFED Touch",
            extraction_type: "Physical",
            start_time: "2024-06-01T09:00:00Z",
          },
        },
      });
      const file = makeFile({ path: "/ev/phone.ufd", filename: "phone.ufd", container_type: "ufed" });
      const infoMap = new Map([[file.path, info]]);
      const report = generateReport(makeInput({ files: [file], fileInfoMap: infoMap }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("#### Extraction Information");
      expect(md).toContain("UFED Touch");
      expect(md).toContain("Physical");
    });

    it("should include notes section when present", () => {
      const report = generateReport(makeInput({
        caseInfo: { notes: "Case notes here" },
        notes: "Report-level notes",
      }));
      const md = exportAsMarkdown(report);
      expect(md).toContain("## Notes");
      expect(md).toContain("Case notes here");
      expect(md).toContain("Report-level notes");
    });

    it("should include footer with app name", () => {
      const report = generateReport(makeInput());
      const md = exportAsMarkdown(report);
      expect(md).toContain("*Report generated by");
    });
  });
});
