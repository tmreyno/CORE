// =============================================================================
// wizard/types — WIZARD_STEPS & getStepsForReportType tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { WIZARD_STEPS, getStepsForReportType } from "../types";
import type { WizardStepConfig } from "../types";

describe("WIZARD_STEPS", () => {
  it("has 8 steps", () => {
    expect(WIZARD_STEPS).toHaveLength(8);
  });

  it("starts with report_type and ends with export", () => {
    expect(WIZARD_STEPS[0].id).toBe("report_type");
    expect(WIZARD_STEPS[WIZARD_STEPS.length - 1].id).toBe("export");
  });

  it("every step has id and label", () => {
    for (const step of WIZARD_STEPS) {
      expect(step.id).toBeTruthy();
      expect(step.label).toBeTruthy();
    }
  });

  it("step IDs are unique", () => {
    const ids = WIZARD_STEPS.map(s => s.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe("getStepsForReportType", () => {
  function stepIds(steps: WizardStepConfig[]): string[] {
    return steps.map(s => s.id);
  }

  it("forensic_examination includes findings but not report_data", () => {
    const steps = stepIds(getStepsForReportType("forensic_examination"));
    expect(steps).toContain("findings");
    expect(steps).not.toContain("report_data");
  });

  it("chain_of_custody includes report_data but not evidence or findings", () => {
    const steps = stepIds(getStepsForReportType("chain_of_custody"));
    expect(steps).toContain("report_data");
    expect(steps).not.toContain("evidence");
    expect(steps).not.toContain("findings");
  });

  it("investigative_activity includes report_data and evidence, but not findings", () => {
    const steps = stepIds(getStepsForReportType("investigative_activity"));
    expect(steps).toContain("report_data");
    expect(steps).toContain("evidence");
    expect(steps).not.toContain("findings");
  });

  it("user_activity includes report_data", () => {
    const steps = stepIds(getStepsForReportType("user_activity"));
    expect(steps).toContain("report_data");
  });

  it("timeline includes report_data", () => {
    const steps = stepIds(getStepsForReportType("timeline"));
    expect(steps).toContain("report_data");
  });

  it("all report types always include report_type, case, examiner, preview, export", () => {
    const types = ["forensic_examination", "chain_of_custody", "investigative_activity", "user_activity", "timeline", "evidence_collection"] as const;
    for (const t of types) {
      const ids = stepIds(getStepsForReportType(t));
      expect(ids).toContain("report_type");
      expect(ids).toContain("case");
      expect(ids).toContain("examiner");
      expect(ids).toContain("preview");
      expect(ids).toContain("export");
    }
  });

  it("evidence_collection includes evidence and not report_data or findings", () => {
    const steps = stepIds(getStepsForReportType("evidence_collection"));
    expect(steps).toContain("evidence");
    expect(steps).not.toContain("report_data");
    expect(steps).not.toContain("findings");
  });
});
