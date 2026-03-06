// =============================================================================
// fieldCounters — evidence info badge metadata count tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { countEwfFields, countAd1Fields, countUfedFields } from "../fieldCounters";
import type { ContainerInfo } from "../../../../../types/containerInfo";

function makeContainerInfo(overrides: Partial<ContainerInfo> = {}): ContainerInfo {
  return { container: "test", ...overrides };
}

describe("countEwfFields", () => {
  it("returns 0 when no e01 or l01 info present", () => {
    expect(countEwfFields(makeContainerInfo())).toBe(0);
  });

  it("returns base count of 3 for minimal e01 info (format, compression, size)", () => {
    expect(countEwfFields(makeContainerInfo({
      e01: {
        format_version: "EWF1",
        segment_count: 1,
        sector_count: 100,
        bytes_per_sector: 512,
        chunk_count: 10,
        sectors_per_chunk: 64,
        total_size: 51200,
        compression: "deflate",
      },
    }))).toBe(3);
  });

  it("counts case_number as +1", () => {
    expect(countEwfFields(makeContainerInfo({
      e01: {
        format_version: "EWF1", segment_count: 1, sector_count: 100,
        bytes_per_sector: 512, chunk_count: 10, sectors_per_chunk: 64,
        total_size: 51200, compression: "deflate",
        case_number: "2026-001",
      },
    }))).toBe(4);
  });

  it("counts all optional fields when present", () => {
    expect(countEwfFields(makeContainerInfo({
      e01: {
        format_version: "EWF1", segment_count: 1, sector_count: 100,
        bytes_per_sector: 512, chunk_count: 10, sectors_per_chunk: 64,
        total_size: 51200, compression: "deflate",
        case_number: "C1",
        examiner_name: "John",
        evidence_number: "EV-001",
        description: "Hard drive",
        model: "WD1000",
        serial_number: "SN123",
        stored_hashes: [{ algorithm: "MD5", hash: "abc" }],
      },
    }))).toBe(10); // 3 base + 7 optional
  });

  it("uses l01 when e01 is absent", () => {
    expect(countEwfFields(makeContainerInfo({
      l01: {
        format_version: "LEF1", segment_count: 1, sector_count: 50,
        bytes_per_sector: 512, chunk_count: 5, sectors_per_chunk: 64,
        total_size: 25600, compression: "none",
        examiner_name: "Jane",
      },
    }))).toBe(4); // 3 base + 1 examiner
  });

  it("treats empty stored_hashes as not present", () => {
    expect(countEwfFields(makeContainerInfo({
      e01: {
        format_version: "EWF1", segment_count: 1, sector_count: 100,
        bytes_per_sector: 512, chunk_count: 10, sectors_per_chunk: 64,
        total_size: 51200, compression: "deflate",
        stored_hashes: [],
      },
    }))).toBe(3); // empty array is falsy for .length check
  });
});

describe("countAd1Fields", () => {
  it("returns 0 when no ad1 info present", () => {
    expect(countAd1Fields(makeContainerInfo())).toBe(0);
  });

  it("returns 0 when ad1 has no companion_log", () => {
    expect(countAd1Fields(makeContainerInfo({
      ad1: {
        segment: {} as any,
        logical: {} as any,
        item_count: 10,
      },
    }))).toBe(0);
  });

  it("returns 0 when companion_log has no fields", () => {
    expect(countAd1Fields(makeContainerInfo({
      ad1: {
        segment: {} as any,
        logical: {} as any,
        item_count: 10,
        companion_log: {},
      },
    }))).toBe(0);
  });

  it("counts each present field", () => {
    expect(countAd1Fields(makeContainerInfo({
      ad1: {
        segment: {} as any,
        logical: {} as any,
        item_count: 10,
        companion_log: {
          case_number: "C1",
          examiner: "Doe",
          evidence_number: "EV1",
        },
      },
    }))).toBe(3);
  });

  it("counts all 6 fields when all present", () => {
    expect(countAd1Fields(makeContainerInfo({
      ad1: {
        segment: {} as any,
        logical: {} as any,
        item_count: 10,
        companion_log: {
          case_number: "C1",
          examiner: "Doe",
          evidence_number: "EV1",
          acquisition_date: "2026-01-01",
          source_device: "iPhone",
          acquisition_tool: "FTK Imager",
        },
      },
    }))).toBe(6);
  });
});

describe("countUfedFields", () => {
  it("returns 0 when no ufed info present", () => {
    expect(countUfedFields(makeContainerInfo())).toBe(0);
  });

  it("returns base count of 2 for minimal ufed info (format, size)", () => {
    expect(countUfedFields(makeContainerInfo({
      ufed: {
        format: "UFED",
        size: 5000000,
        associated_files: [],
        is_extraction_set: false,
      },
    }))).toBe(2);
  });

  it("adds 3 for device_info", () => {
    expect(countUfedFields(makeContainerInfo({
      ufed: {
        format: "UFED",
        size: 5000000,
        associated_files: [],
        is_extraction_set: false,
        device_info: { vendor: "Apple", model: "iPhone 14" },
      },
    }))).toBe(5); // 2 + 3
  });

  it("adds 2 for case_info", () => {
    expect(countUfedFields(makeContainerInfo({
      ufed: {
        format: "UFED",
        size: 5000000,
        associated_files: [],
        is_extraction_set: false,
        case_info: { case_identifier: "C1" },
      },
    }))).toBe(4); // 2 + 2
  });

  it("adds 3 for extraction_info", () => {
    expect(countUfedFields(makeContainerInfo({
      ufed: {
        format: "UFED",
        size: 5000000,
        associated_files: [],
        is_extraction_set: false,
        extraction_info: { extraction_type: "full" },
      },
    }))).toBe(5); // 2 + 3
  });

  it("adds all sections when all present", () => {
    expect(countUfedFields(makeContainerInfo({
      ufed: {
        format: "UFED",
        size: 5000000,
        associated_files: [],
        is_extraction_set: true,
        device_info: { vendor: "Samsung" },
        case_info: { case_identifier: "C2" },
        extraction_info: { extraction_type: "advanced" },
      },
    }))).toBe(10); // 2 + 3 + 2 + 3
  });
});
