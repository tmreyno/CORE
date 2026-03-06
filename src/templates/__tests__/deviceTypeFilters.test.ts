// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getFilterMap, filterOptions } from "../deviceTypeFilters";
import type { InlineOption } from "../types";

// =============================================================================
// getFilterMap
// =============================================================================

describe("getFilterMap", () => {
  it("returns a filter map for known IDs", () => {
    const map = getFilterMap("device_to_storage_interface");
    expect(map).toBeDefined();
    expect(map!.laptop).toBeDefined();
    expect(Array.isArray(map!.laptop)).toBe(true);
  });

  it("returns undefined for unknown IDs", () => {
    expect(getFilterMap("nonexistent_map")).toBeUndefined();
  });

  it.each([
    "device_to_storage_interface",
    "device_to_form_factor",
    "device_to_acquisition_method",
    "form_factor_to_storage_interface",
    "device_to_connection_method",
  ])("has entries for %s", (id) => {
    const map = getFilterMap(id);
    expect(map).toBeDefined();
    expect(Object.keys(map!).length).toBeGreaterThan(0);
  });

  it("includes common device types in device_to_storage_interface", () => {
    const map = getFilterMap("device_to_storage_interface")!;
    expect(map.mobile_phone).toContain("ufs");
    expect(map.desktop_computer).toContain("sata");
    expect(map.virtual_machine).toContain("n_a");
  });
});

// =============================================================================
// filterOptions
// =============================================================================

describe("filterOptions", () => {
  // Sample options mimicking real schema structure.
  // "other" and "n_a" are placed before groups so they're standalone
  // (items after the last _group_ header are treated as its children).
  const sampleOptions: InlineOption[] = [
    { value: "other", label: "Other" },
    { value: "n_a", label: "N/A" },
    { value: "_group_common", label: "Common", disabled: true },
    { value: "sata", label: "SATA" },
    { value: "nvme_m2", label: "NVMe M.2" },
    { value: "_group_mobile", label: "Mobile", disabled: true },
    { value: "ufs", label: "UFS" },
    { value: "emmc", label: "eMMC" },
    { value: "_group_special", label: "Special", disabled: true },
    { value: "fc_san", label: "Fibre Channel SAN" },
    { value: "iscsi", label: "iSCSI" },
  ];

  it("returns all options when allowed is empty", () => {
    const result = filterOptions(sampleOptions, []);
    expect(result).toEqual(sampleOptions);
  });

  it("filters to allowed values plus other and n_a", () => {
    const result = filterOptions(sampleOptions, ["sata"]);
    const values = result.map((o) => o.value);
    expect(values).toContain("sata");
    expect(values).toContain("other");
    expect(values).toContain("n_a");
    expect(values).not.toContain("ufs");
    expect(values).not.toContain("emmc");
    expect(values).not.toContain("fc_san");
  });

  it("includes group header only when it has matching children", () => {
    const result = filterOptions(sampleOptions, ["sata"]);
    const values = result.map((o) => o.value);
    // _group_common should be included (has "sata")
    expect(values).toContain("_group_common");
    // _group_mobile should be excluded (no matching children)
    expect(values).not.toContain("_group_mobile");
    // _group_special should be excluded
    expect(values).not.toContain("_group_special");
  });

  it("includes group header when any child matches", () => {
    const result = filterOptions(sampleOptions, ["emmc"]);
    const values = result.map((o) => o.value);
    expect(values).toContain("_group_mobile");
    expect(values).toContain("emmc");
    expect(values).not.toContain("ufs"); // not in allowed
  });

  it("always includes other and n_a even if not in allowed", () => {
    const result = filterOptions(sampleOptions, ["sata"]);
    const values = result.map((o) => o.value);
    expect(values).toContain("other");
    expect(values).toContain("n_a");
  });

  it("handles options with no groups", () => {
    const flat: InlineOption[] = [
      { value: "a", label: "A" },
      { value: "b", label: "B" },
      { value: "c", label: "C" },
      { value: "other", label: "Other" },
    ];
    const result = filterOptions(flat, ["b"]);
    const values = result.map((o) => o.value);
    expect(values).toContain("b");
    expect(values).toContain("other");
    expect(values).not.toContain("a");
    expect(values).not.toContain("c");
  });

  it("handles empty options array", () => {
    expect(filterOptions([], ["sata"])).toEqual([]);
  });

  it("preserves order of options", () => {
    const result = filterOptions(sampleOptions, ["sata", "ufs"]);
    const values = result.map((o) => o.value);
    const sataIdx = values.indexOf("sata");
    const ufsIdx = values.indexOf("ufs");
    expect(sataIdx).toBeLessThan(ufsIdx);
  });
});
