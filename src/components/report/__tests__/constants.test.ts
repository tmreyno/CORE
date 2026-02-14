// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  CLASSIFICATIONS,
  SEVERITIES,
  EVIDENCE_TYPES,
  INVESTIGATION_TYPES,
  FINDING_CATEGORIES,
  CUSTODY_ACTIONS,
} from "../constants";

// =============================================================================
// CLASSIFICATIONS
// =============================================================================
describe("CLASSIFICATIONS", () => {
  it("has 5 classification levels", () => {
    expect(CLASSIFICATIONS).toHaveLength(5);
  });

  it("all have value, label, and color", () => {
    for (const c of CLASSIFICATIONS) {
      expect(c.value).toBeTruthy();
      expect(c.label).toBeTruthy();
      expect(c.color).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it("has unique values", () => {
    const values = CLASSIFICATIONS.map(c => c.value);
    expect(new Set(values).size).toBe(values.length);
  });

  it("includes expected classifications", () => {
    const values = CLASSIFICATIONS.map(c => c.value);
    expect(values).toContain("Public");
    expect(values).toContain("Confidential");
    expect(values).toContain("Restricted");
    expect(values).toContain("LawEnforcementSensitive");
  });
});

// =============================================================================
// SEVERITIES
// =============================================================================
describe("SEVERITIES", () => {
  it("has 5 severity levels", () => {
    expect(SEVERITIES).toHaveLength(5);
  });

  it("all have value, label, and hex color", () => {
    for (const s of SEVERITIES) {
      expect(s.value).toBeTruthy();
      expect(s.label).toBeTruthy();
      expect(s.color).toMatch(/^#[0-9a-f]{6}$/i);
    }
  });

  it("has unique values", () => {
    const values = SEVERITIES.map(s => s.value);
    expect(new Set(values).size).toBe(values.length);
  });

  it("includes Critical and Informational", () => {
    const values = SEVERITIES.map(s => s.value);
    expect(values).toContain("Critical");
    expect(values).toContain("Informational");
  });
});

// =============================================================================
// EVIDENCE_TYPES
// =============================================================================
describe("EVIDENCE_TYPES", () => {
  it("has 14 evidence types", () => {
    expect(EVIDENCE_TYPES).toHaveLength(14);
  });

  it("all have value and label", () => {
    for (const e of EVIDENCE_TYPES) {
      expect(e.value).toBeTruthy();
      expect(e.label).toBeTruthy();
    }
  });

  it("has unique values", () => {
    const values = EVIDENCE_TYPES.map(e => e.value);
    expect(new Set(values).size).toBe(values.length);
  });

  it("includes common forensic evidence types", () => {
    const values = EVIDENCE_TYPES.map(e => e.value);
    expect(values).toContain("HardDrive");
    expect(values).toContain("SSD");
    expect(values).toContain("MobilePhone");
    expect(values).toContain("ForensicImage");
    expect(values).toContain("Other");
  });
});

// =============================================================================
// INVESTIGATION_TYPES
// =============================================================================
describe("INVESTIGATION_TYPES", () => {
  it("has 8 investigation types", () => {
    expect(INVESTIGATION_TYPES).toHaveLength(8);
  });

  it("all have value and label", () => {
    for (const t of INVESTIGATION_TYPES) {
      expect(t.value).toBeTruthy();
      expect(t.label).toBeTruthy();
    }
  });

  it("has unique values", () => {
    const values = INVESTIGATION_TYPES.map(t => t.value);
    expect(new Set(values).size).toBe(values.length);
  });

  it("includes criminal and civil", () => {
    const values = INVESTIGATION_TYPES.map(t => t.value);
    expect(values).toContain("criminal");
    expect(values).toContain("civil");
    expect(values).toContain("incident_response");
  });
});

// =============================================================================
// FINDING_CATEGORIES
// =============================================================================
describe("FINDING_CATEGORIES", () => {
  it("has 12 categories", () => {
    expect(FINDING_CATEGORIES).toHaveLength(12);
  });

  it("all have value and label", () => {
    for (const c of FINDING_CATEGORIES) {
      expect(c.value).toBeTruthy();
      expect(c.label).toBeTruthy();
    }
  });

  it("has unique values", () => {
    const values = FINDING_CATEGORIES.map(c => c.value);
    expect(new Set(values).size).toBe(values.length);
  });

  it("includes key forensic categories", () => {
    const values = FINDING_CATEGORIES.map(c => c.value);
    expect(values).toContain("malware");
    expect(values).toContain("unauthorized_access");
    expect(values).toContain("evidence_destruction");
    expect(values).toContain("timeline");
  });
});

// =============================================================================
// CUSTODY_ACTIONS
// =============================================================================
describe("CUSTODY_ACTIONS", () => {
  it("has 7 actions", () => {
    expect(CUSTODY_ACTIONS).toHaveLength(7);
  });

  it("all have value and label", () => {
    for (const a of CUSTODY_ACTIONS) {
      expect(a.value).toBeTruthy();
      expect(a.label).toBeTruthy();
    }
  });

  it("has unique values", () => {
    const values = CUSTODY_ACTIONS.map(a => a.value);
    expect(new Set(values).size).toBe(values.length);
  });

  it("includes standard chain of custody actions", () => {
    const values = CUSTODY_ACTIONS.map(a => a.value);
    expect(values).toContain("received");
    expect(values).toContain("transferred");
    expect(values).toContain("analyzed");
    expect(values).toContain("released");
  });
});
