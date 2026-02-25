// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  REPORT_PRESETS,
  getPresetById,
  getDefaultPreset,
} from "../constants";
import type { ReportPreset } from "../constants";

// =============================================================================
// REPORT_PRESETS
// =============================================================================
describe("REPORT_PRESETS", () => {
  it("contains 5 presets", () => {
    expect(REPORT_PRESETS).toHaveLength(5);
  });

  it("has expected preset IDs", () => {
    const ids = REPORT_PRESETS.map(p => p.id);
    expect(ids).toContain("law_enforcement");
    expect(ids).toContain("corporate");
    expect(ids).toContain("academic");
    expect(ids).toContain("minimal");
    expect(ids).toContain("custom");
  });

  it("all presets have required fields", () => {
    for (const preset of REPORT_PRESETS) {
      expect(preset.id).toBeTruthy();
      expect(preset.name).toBeTruthy();
      expect(preset.description).toBeTruthy();
      expect(preset.icon).toBeTruthy();
      expect(preset.defaultClassification).toBeTruthy();
      expect(preset.defaultSections).toBeDefined();
    }
  });

  it("all presets have valid section definitions", () => {
    for (const preset of REPORT_PRESETS) {
      expect(typeof preset.defaultSections.executiveSummary).toBe("boolean");
      expect(typeof preset.defaultSections.scope).toBe("boolean");
      expect(typeof preset.defaultSections.methodology).toBe("boolean");
      expect(typeof preset.defaultSections.chainOfCustody).toBe("boolean");
      expect(typeof preset.defaultSections.timeline).toBe("boolean");
      expect(typeof preset.defaultSections.conclusions).toBe("boolean");
      expect(typeof preset.defaultSections.appendices).toBe("boolean");
    }
  });

  it("law_enforcement has all sections enabled", () => {
    const le = REPORT_PRESETS.find(p => p.id === "law_enforcement")!;
    expect(le.defaultSections.executiveSummary).toBe(true);
    expect(le.defaultSections.scope).toBe(true);
    expect(le.defaultSections.methodology).toBe(true);
    expect(le.defaultSections.chainOfCustody).toBe(true);
    expect(le.defaultSections.timeline).toBe(true);
    expect(le.defaultSections.conclusions).toBe(true);
    expect(le.defaultSections.appendices).toBe(true);
  });

  it("custom has all sections disabled", () => {
    const custom = REPORT_PRESETS.find(p => p.id === "custom")!;
    expect(custom.defaultSections.executiveSummary).toBe(false);
    expect(custom.defaultSections.scope).toBe(false);
    expect(custom.defaultSections.methodology).toBe(false);
    expect(custom.defaultSections.chainOfCustody).toBe(false);
    expect(custom.defaultSections.timeline).toBe(false);
    expect(custom.defaultSections.conclusions).toBe(false);
    expect(custom.defaultSections.appendices).toBe(false);
  });

  it("minimal has only executiveSummary and conclusions", () => {
    const minimal = REPORT_PRESETS.find(p => p.id === "minimal")!;
    expect(minimal.defaultSections.executiveSummary).toBe(true);
    expect(minimal.defaultSections.conclusions).toBe(true);
    expect(minimal.defaultSections.scope).toBe(false);
    expect(minimal.defaultSections.methodology).toBe(false);
    expect(minimal.defaultSections.chainOfCustody).toBe(false);
  });

  it("has unique preset IDs", () => {
    const ids = REPORT_PRESETS.map(p => p.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

// =============================================================================
// getPresetById
// =============================================================================
describe("getPresetById", () => {
  it("returns preset for valid ID", () => {
    const preset = getPresetById("law_enforcement");
    expect(preset).toBeDefined();
    expect(preset!.id).toBe("law_enforcement");
    expect(preset!.name).toBe("Law Enforcement");
  });

  it("returns each preset by its ID", () => {
    const ids: ReportPreset[] = ["law_enforcement", "corporate", "academic", "minimal", "custom"];
    for (const id of ids) {
      const preset = getPresetById(id);
      expect(preset).toBeDefined();
      expect(preset!.id).toBe(id);
    }
  });

  it("returns undefined for unknown ID", () => {
    const preset = getPresetById("nonexistent" as ReportPreset);
    expect(preset).toBeUndefined();
  });
});

// =============================================================================
// getDefaultPreset
// =============================================================================
describe("getDefaultPreset", () => {
  it("returns law_enforcement as default", () => {
    const preset = getDefaultPreset();
    expect(preset.id).toBe("law_enforcement");
  });

  it("returns a valid preset object", () => {
    const preset = getDefaultPreset();
    expect(preset.name).toBeTruthy();
    expect(preset.defaultSections).toBeDefined();
    expect(preset.defaultClassification).toBeTruthy();
  });
});
