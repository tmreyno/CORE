import { describe, expect, it, vi } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import {
  checkOllamaConnection,
  createEvidenceFromContainer,
  exportReportJson,
  extractEvidenceFromContainers,
  extractReportEvidenceFromProjectDb,
  generateAiNarrative,
  getAiProviders,
  getReportTemplate,
  importReportJson,
  isAiAvailable,
  type ContainerInfoInput,
} from "../api";

vi.mock("../../utils/platform", () => ({
  isTauri: false,
}));

const container: ContainerInfoInput = {
  container_type: "e01",
  path: "/evidence/disk.E01",
  filename: "disk.E01",
  size: 1024,
};

describe("report API browser runtime guards", () => {
  it("returns empty report evidence models without native project DB commands", async () => {
    await expect(extractEvidenceFromContainers([container])).resolves.toEqual([]);
    await expect(extractReportEvidenceFromProjectDb()).resolves.toMatchObject({
      evidenceItems: [],
      hashRecords: [],
      hashAlgorithmSummaries: [],
      verificationResultSummaries: [],
      artifacts: [],
      sourceAnalyses: [],
      annotations: [],
    });

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("rejects desktop-only report and AI generation calls", async () => {
      await expect(createEvidenceFromContainer(container, "EV-1")).rejects.toThrow("desktop app");
      await expect(
        generateAiNarrative("case facts", "executive_summary", "ollama", "model"),
      ).rejects.toThrow("desktop app");

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("returns safe browser defaults for templates and AI availability", async () => {
    await expect(getReportTemplate("forensic")).resolves.toBeNull();
    await expect(isAiAvailable()).resolves.toBe(false);
    await expect(getAiProviders()).resolves.toEqual([]);
    await expect(checkOllamaConnection()).resolves.toBe(false);

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("imports and exports report JSON without native commands", async () => {
    const report = { title: "Browser report", findings: [{ id: "f1" }] };

    const json = await exportReportJson(report);

    expect(json).toContain("Browser report");
    await expect(importReportJson(json)).resolves.toEqual(report);
    expect(mockInvoke).not.toHaveBeenCalled();
  });
});
