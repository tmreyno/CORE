// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  REPORT_TEMPLATES,
  getTemplateById,
  getDefaultTemplate,
} from "../templates";
import type { ReportTemplateType } from "../templates";

// =============================================================================
// REPORT_TEMPLATES
// =============================================================================
describe("REPORT_TEMPLATES", () => {
  it("contains 5 templates", () => {
    expect(REPORT_TEMPLATES).toHaveLength(5);
  });

  it("has expected template IDs", () => {
    const ids = REPORT_TEMPLATES.map(t => t.id);
    expect(ids).toContain("law_enforcement");
    expect(ids).toContain("corporate");
    expect(ids).toContain("academic");
    expect(ids).toContain("minimal");
    expect(ids).toContain("custom");
  });

  it("all templates have required fields", () => {
    for (const template of REPORT_TEMPLATES) {
      expect(template.id).toBeTruthy();
      expect(template.name).toBeTruthy();
      expect(template.description).toBeTruthy();
      expect(template.icon).toBeTruthy();
      expect(template.defaultClassification).toBeTruthy();
      expect(template.defaultSections).toBeDefined();
      expect(Array.isArray(template.requiredFields)).toBe(true);
    }
  });

  it("all templates have valid section definitions", () => {
    for (const template of REPORT_TEMPLATES) {
      expect(typeof template.defaultSections.executiveSummary).toBe("boolean");
      expect(typeof template.defaultSections.scope).toBe("boolean");
      expect(typeof template.defaultSections.methodology).toBe("boolean");
      expect(typeof template.defaultSections.chainOfCustody).toBe("boolean");
      expect(typeof template.defaultSections.timeline).toBe("boolean");
      expect(typeof template.defaultSections.conclusions).toBe("boolean");
      expect(typeof template.defaultSections.appendices).toBe("boolean");
    }
  });

  it("law_enforcement has all sections enabled", () => {
    const le = REPORT_TEMPLATES.find(t => t.id === "law_enforcement")!;
    expect(le.defaultSections.executiveSummary).toBe(true);
    expect(le.defaultSections.scope).toBe(true);
    expect(le.defaultSections.methodology).toBe(true);
    expect(le.defaultSections.chainOfCustody).toBe(true);
    expect(le.defaultSections.timeline).toBe(true);
    expect(le.defaultSections.conclusions).toBe(true);
    expect(le.defaultSections.appendices).toBe(true);
  });

  it("custom has all sections disabled", () => {
    const custom = REPORT_TEMPLATES.find(t => t.id === "custom")!;
    expect(custom.defaultSections.executiveSummary).toBe(false);
    expect(custom.defaultSections.scope).toBe(false);
    expect(custom.defaultSections.methodology).toBe(false);
    expect(custom.defaultSections.chainOfCustody).toBe(false);
    expect(custom.defaultSections.timeline).toBe(false);
    expect(custom.defaultSections.conclusions).toBe(false);
    expect(custom.defaultSections.appendices).toBe(false);
  });

  it("minimal has only executiveSummary and conclusions", () => {
    const minimal = REPORT_TEMPLATES.find(t => t.id === "minimal")!;
    expect(minimal.defaultSections.executiveSummary).toBe(true);
    expect(minimal.defaultSections.conclusions).toBe(true);
    expect(minimal.defaultSections.scope).toBe(false);
    expect(minimal.defaultSections.methodology).toBe(false);
    expect(minimal.defaultSections.chainOfCustody).toBe(false);
  });

  it("law_enforcement requires the most fields", () => {
    const le = REPORT_TEMPLATES.find(t => t.id === "law_enforcement")!;
    expect(le.requiredFields.length).toBeGreaterThanOrEqual(3);
    expect(le.requiredFields).toContain("case_number");
    expect(le.requiredFields).toContain("examiner_name");
  });

  it("custom requires no fields", () => {
    const custom = REPORT_TEMPLATES.find(t => t.id === "custom")!;
    expect(custom.requiredFields).toEqual([]);
  });

  it("has unique template IDs", () => {
    const ids = REPORT_TEMPLATES.map(t => t.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

// =============================================================================
// getTemplateById
// =============================================================================
describe("getTemplateById", () => {
  it("returns template for valid ID", () => {
    const template = getTemplateById("law_enforcement");
    expect(template).toBeDefined();
    expect(template!.id).toBe("law_enforcement");
    expect(template!.name).toBe("Law Enforcement");
  });

  it("returns each template by its ID", () => {
    const ids: ReportTemplateType[] = ["law_enforcement", "corporate", "academic", "minimal", "custom"];
    for (const id of ids) {
      const template = getTemplateById(id);
      expect(template).toBeDefined();
      expect(template!.id).toBe(id);
    }
  });

  it("returns undefined for unknown ID", () => {
    const template = getTemplateById("nonexistent" as ReportTemplateType);
    expect(template).toBeUndefined();
  });
});

// =============================================================================
// getDefaultTemplate
// =============================================================================
describe("getDefaultTemplate", () => {
  it("returns law_enforcement as default", () => {
    const template = getDefaultTemplate();
    expect(template.id).toBe("law_enforcement");
  });

  it("returns a valid template object", () => {
    const template = getDefaultTemplate();
    expect(template.name).toBeTruthy();
    expect(template.defaultSections).toBeDefined();
    expect(template.defaultClassification).toBeTruthy();
  });
});
