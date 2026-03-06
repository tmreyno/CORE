// =============================================================================
// normalizeContainerFields — maps ContainerInfo to InfoField[] tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { normalizeContainerFields } from "../normalizeContainerFields";
import type { ContainerInfo, StoredHash } from "../../../types";

function emptyInfo(): ContainerInfo {
  return {} as ContainerInfo;
}

function findField(fields: { label: string; value: any }[], label: string) {
  return fields.find((f) => f.label === label);
}

describe("normalizeContainerFields", () => {
  // ── Empty info ──

  it("returns empty array for empty ContainerInfo", () => {
    expect(normalizeContainerFields(emptyInfo(), [])).toEqual([]);
  });

  // ── AD1 ──

  describe("AD1 branch", () => {
    const makeAd1Info = (overrides: Record<string, any> = {}): ContainerInfo =>
      ({
        ad1: {
          logical: { signature: "ADSEGMENTEDFILE", image_version: "2", zlib_chunk_size: 32768, data_source_name: "/dev/disk1" },
          segment: { segment_number: 3 },
          segment_files: ["file.ad1", "file.ad2", "file.ad3"],
          total_size: 1024000,
          item_count: 42,
          companion_log: { case_number: "C-001", evidence_number: "E-001", examiner: "Doe", acquisition_date: "2026-01-01", notes: "Test" },
          volume: { volume_label: "SYSTEM", filesystem: "NTFS", os_info: "Windows 10", block_size: 4096 },
          missing_segments: [],
          ...overrides,
        },
      }) as ContainerInfo;

    it("maps all AD1 fields", () => {
      const fields = normalizeContainerFields(makeAd1Info(), []);
      expect(findField(fields, "Format")?.value).toBe("AD1 (ADSEGMENTEDFILE)");
      expect(findField(fields, "Version")?.value).toBe("2");
      expect(findField(fields, "Segments")?.value).toBe("3 / 3");
      expect(findField(fields, "Items")?.value).toBe(42);
      expect(findField(fields, "Case #")?.value).toBe("C-001");
      expect(findField(fields, "Evidence #")?.value).toBe("E-001");
      expect(findField(fields, "Examiner")?.value).toBe("Doe");
      expect(findField(fields, "Filesystem")?.value).toBe("NTFS");
      expect(findField(fields, "Chunk Size")?.value).toBe(32768);
    });

    it("shows warning for missing segments", () => {
      const fields = normalizeContainerFields(
        makeAd1Info({ missing_segments: ["file.ad2", "file.ad3"] }),
        [],
      );
      const warning = findField(fields, "⚠ Incomplete");
      expect(warning).toBeDefined();
      expect(warning!.value).toContain("Missing 2 segment(s)");
      expect(warning!.format).toBe("warning");
    });

    it("does not show warning when no missing segments", () => {
      const fields = normalizeContainerFields(makeAd1Info(), []);
      expect(findField(fields, "⚠ Incomplete")).toBeUndefined();
    });

    it("marks Segments as incomplete when segments are missing", () => {
      const fields = normalizeContainerFields(
        makeAd1Info({ missing_segments: ["file.ad2"] }),
        [],
      );
      expect(findField(fields, "Segments")?.value).toContain("(incomplete)");
    });
  });

  // ── E01 ──

  describe("E01 branch", () => {
    const makeE01Info = (): ContainerInfo =>
      ({
        e01: {
          format_version: "EWF-E01",
          segment_count: 5,
          total_size: 500000000,
          compression: "deflate",
          bytes_per_sector: 512,
          sectors_per_chunk: 64,
          case_number: "CASE-42",
          evidence_number: "EV-01",
          examiner_name: "Smith",
          acquiry_date: "2026-02-15",
          system_date: "2026-02-15 10:00",
          model: "WD Blue",
          serial_number: "SN12345",
          description: "Hard drive image",
          notes: "E01 notes",
        },
      }) as ContainerInfo;

    it("maps all E01 fields", () => {
      const fields = normalizeContainerFields(makeE01Info(), []);
      expect(findField(fields, "Format")?.value).toBe("EWF-E01");
      expect(findField(fields, "Segments")?.value).toBe(5);
      expect(findField(fields, "Compression")?.value).toBe("deflate");
      expect(findField(fields, "Case #")?.value).toBe("CASE-42");
      expect(findField(fields, "Model")?.value).toBe("WD Blue");
      expect(findField(fields, "Serial #")?.value).toBe("SN12345");
    });
  });

  // ── L01 ──

  describe("L01 branch", () => {
    it("maps L01 fields (same structure as E01)", () => {
      const info = {
        l01: {
          format_version: "EWF-L01",
          segment_count: 2,
          total_size: 100000,
          compression: "none",
          bytes_per_sector: 512,
          sectors_per_chunk: 32,
          case_number: "L-CASE",
          evidence_number: "L-EV",
          examiner_name: "Jones",
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      expect(findField(fields, "Format")?.value).toBe("EWF-L01");
      expect(findField(fields, "Case #")?.value).toBe("L-CASE");
    });
  });

  // ── Raw ──

  describe("Raw branch", () => {
    it("maps basic raw fields", () => {
      const info = {
        raw: {
          segment_count: 1,
          total_size: 1000000,
          segment_names: ["image.dd"],
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      expect(findField(fields, "Format")?.value).toBe("Raw Image");
      expect(findField(fields, "Segments")?.value).toBe(1);
      // Single segment — no segment list
      expect(findField(fields, "Segment Files")).toBeUndefined();
    });

    it("lists segment files for multi-segment raw, truncated at 5", () => {
      const names = ["img.001", "img.002", "img.003", "img.004", "img.005", "img.006", "img.007"];
      const info = {
        raw: { segment_count: 7, total_size: 7000000, segment_names: names },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      const segField = findField(fields, "Segment Files");
      expect(segField).toBeDefined();
      expect(segField!.value).toContain("(+2 more)");
    });
  });

  // ── Archive ──

  describe("Archive branch", () => {
    it("maps archive fields with version", () => {
      const info = {
        archive: {
          format: "7z",
          version: "24.09",
          segment_count: 1,
          total_size: 50000,
          entry_count: 100,
          aes_encrypted: false,
          encrypted_headers: false,
          segment_names: [],
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      expect(findField(fields, "Format")?.value).toBe("7z v24.09");
      expect(findField(fields, "Entries")?.value).toBe(100);
    });

    it("shows encryption indicators", () => {
      const info = {
        archive: {
          format: "7z",
          segment_count: 1,
          total_size: 50000,
          entry_count: 10,
          aes_encrypted: true,
          encrypted_headers: true,
          segment_names: [],
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      expect(findField(fields, "AES Encrypted")?.value).toBe("Yes");
      expect(findField(fields, "Encrypted Headers")?.value).toBe("Filenames Hidden");
    });

    it("hides encryption fields when not encrypted", () => {
      const info = {
        archive: {
          format: "ZIP",
          segment_count: 1,
          total_size: 50000,
          entry_count: 10,
          aes_encrypted: false,
          encrypted_headers: false,
          segment_names: [],
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      // AES Encrypted value is undefined when not encrypted
      expect(findField(fields, "AES Encrypted")?.value).toBeUndefined();
    });

    it("shows header CRC valid/invalid", () => {
      const info = {
        archive: {
          format: "7z",
          segment_count: 1,
          total_size: 50000,
          entry_count: 10,
          aes_encrypted: false,
          encrypted_headers: false,
          start_header_crc_valid: true,
          segment_names: [],
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      expect(findField(fields, "Header CRC")?.value).toBe("✓ Valid");
    });

    it("shows CRC invalid with highlight", () => {
      const info = {
        archive: {
          format: "7z",
          segment_count: 1,
          total_size: 50000,
          entry_count: 10,
          aes_encrypted: false,
          encrypted_headers: false,
          start_header_crc_valid: false,
          segment_names: [],
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      const crc = findField(fields, "Header CRC");
      expect(crc?.value).toBe("✗ Invalid");
      expect(crc?.type).toBe("highlight");
    });
  });

  // ── UFED ──

  describe("UFED branch", () => {
    it("maps UFED fields", () => {
      const info = {
        ufed: {
          format: "UFDR",
          size: 2000000,
          associated_files: [{ filename: "report.xml" }],
          extraction_info: {
            extraction_type: "Full",
            acquisition_tool: "UFED",
            tool_version: "7.68",
            start_time: "2026-01-01",
            end_time: "2026-01-02",
            connection_type: "USB",
            guid: "abc-123",
          },
          case_info: {
            case_identifier: "UFD-001",
            device_name: "iPhone 14",
            examiner_name: "Lee",
            location: "Lab A",
          },
          device_info: {
            full_name: "Apple iPhone 14 Pro",
            model: "A2890",
            serial_number: "SN-UFD",
            imei: "123456789012345",
            imei2: "543210987654321",
            os_version: "iOS 17",
            vendor: "Apple",
          },
          evidence_number: "EV-UFD",
          device_hint: "iPhone",
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      expect(findField(fields, "Format")?.value).toBe("UFED (UFDR)");
      expect(findField(fields, "Extraction")?.value).toBe("Full");
      expect(findField(fields, "Tool")?.value).toBe("UFED v7.68");
      expect(findField(fields, "Case #")?.value).toBe("UFD-001");
      expect(findField(fields, "Device")?.value).toBe("Apple iPhone 14 Pro");
      expect(findField(fields, "IMEI")?.value).toContain("123456789012345");
      expect(findField(fields, "IMEI")?.value).toContain("543210987654321");
      expect(findField(fields, "OS")?.value).toBe("Apple iOS 17");
    });
  });

  // ── Companion log ──

  describe("Companion log branch", () => {
    it("maps companion log fields", () => {
      const info = {
        companion_log: {
          created_by: "FTK Imager 4.7",
          case_number: "CL-001",
          evidence_number: "CL-EV",
          examiner: "Brown",
          acquisition_started: "2026-03-01 09:00",
          unique_description: "/dev/sda",
          notes: "Companion notes",
        },
      } as ContainerInfo;
      const fields = normalizeContainerFields(info, []);
      expect(findField(fields, "Created By")?.value).toBe("FTK Imager 4.7");
      expect(findField(fields, "Case #")?.value).toBe("CL-001");
      expect(findField(fields, "Source")?.value).toBe("/dev/sda");
    });
  });

  // ── Stored Hashes ──

  describe("StoredHashes", () => {
    it("adds hash fields with algorithm label and source icon", () => {
      const hashes: StoredHash[] = [
        { algorithm: "md5", hash: "abc123", source: "container" } as StoredHash,
        { algorithm: "sha1", hash: "def456", source: "companion" } as StoredHash,
        { algorithm: "sha256", hash: "ghi789", source: "computed" } as StoredHash,
      ];
      const fields = normalizeContainerFields(emptyInfo(), hashes);
      expect(fields).toHaveLength(3);
      expect(fields[0].label).toContain("◆"); // container
      expect(fields[0].label).toContain("MD5");
      expect(fields[1].label).toContain("◇"); // companion
      expect(fields[1].label).toContain("SHA1");
      expect(fields[2].label).toContain("▣"); // computed
    });

    it("shows verify icon for verified/failed hashes", () => {
      const hashes: StoredHash[] = [
        { algorithm: "md5", hash: "abc", source: "container", verified: true } as StoredHash,
        { algorithm: "sha1", hash: "def", source: "container", verified: false } as StoredHash,
      ];
      const fields = normalizeContainerFields(emptyInfo(), hashes);
      expect(fields[0].label).toContain("✓");
      expect(fields[1].label).toContain("✗");
    });

    it("includes filename label for per-file hashes", () => {
      const hashes: StoredHash[] = [
        { algorithm: "md5", hash: "abc", source: "container", filename: "data.bin" } as StoredHash,
      ];
      const fields = normalizeContainerFields(emptyInfo(), hashes);
      expect(fields[0].label).toContain("(data.bin)");
    });

    it("marks hash fields as type hash", () => {
      const hashes: StoredHash[] = [
        { algorithm: "sha256", hash: "abc", source: "container" } as StoredHash,
      ];
      const fields = normalizeContainerFields(emptyInfo(), hashes);
      expect(fields[0].type).toBe("hash");
    });
  });
});
