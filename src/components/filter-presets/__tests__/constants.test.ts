// =============================================================================
// filter-presets/constants — DEFAULT_QUICK_FILTERS tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { DEFAULT_QUICK_FILTERS } from "../constants";

describe("DEFAULT_QUICK_FILTERS", () => {
  it("has 8 filters", () => {
    expect(DEFAULT_QUICK_FILTERS).toHaveLength(8);
  });

  it("every filter has id, name, extensions, and icon", () => {
    for (const f of DEFAULT_QUICK_FILTERS) {
      expect(f.id).toBeTruthy();
      expect(f.name).toBeTruthy();
      expect(f.extensions.length).toBeGreaterThan(0);
      expect(f.icon).toBeTruthy();
    }
  });

  it("all IDs are unique", () => {
    const ids = DEFAULT_QUICK_FILTERS.map(f => f.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("all extensions start with a dot", () => {
    for (const f of DEFAULT_QUICK_FILTERS) {
      for (const ext of f.extensions) {
        expect(ext).toMatch(/^\./);
      }
    }
  });

  it("includes expected categories", () => {
    const ids = DEFAULT_QUICK_FILTERS.map(f => f.id);
    expect(ids).toContain("documents");
    expect(ids).toContain("images");
    expect(ids).toContain("videos");
    expect(ids).toContain("audio");
    expect(ids).toContain("archives");
    expect(ids).toContain("code");
    expect(ids).toContain("databases");
    expect(ids).toContain("emails");
  });

  it("documents filter includes common doc extensions", () => {
    const docs = DEFAULT_QUICK_FILTERS.find(f => f.id === "documents")!;
    expect(docs.extensions).toContain(".pdf");
    expect(docs.extensions).toContain(".docx");
    expect(docs.extensions).toContain(".txt");
  });

  it("images filter includes common image extensions", () => {
    const imgs = DEFAULT_QUICK_FILTERS.find(f => f.id === "images")!;
    expect(imgs.extensions).toContain(".jpg");
    expect(imgs.extensions).toContain(".png");
    expect(imgs.extensions).toContain(".gif");
  });

  it("archives filter includes forensic formats", () => {
    const arch = DEFAULT_QUICK_FILTERS.find(f => f.id === "archives")!;
    expect(arch.extensions).toContain(".ad1");
    expect(arch.extensions).toContain(".e01");
    expect(arch.extensions).toContain(".l01");
  });

  it("emails filter includes PST and EML", () => {
    const emails = DEFAULT_QUICK_FILTERS.find(f => f.id === "emails")!;
    expect(emails.extensions).toContain(".eml");
    expect(emails.extensions).toContain(".pst");
    expect(emails.extensions).toContain(".mbox");
  });
});
