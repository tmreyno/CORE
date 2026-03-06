// =============================================================================
// formDataConversion — evidence collection data conversion tests
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock crypto.randomUUID for deterministic IDs
let uuidCounter = 0;
vi.stubGlobal("crypto", {
  randomUUID: () => {
    uuidCounter++;
    return `uuid-${String(uuidCounter).padStart(4, "0")}`;
  },
});

import { generateId, evidenceToFormData, formDataToEvidence } from "../formDataConversion";

beforeEach(() => {
  uuidCounter = 0;
});

describe("generateId", () => {
  it("delegates to crypto.randomUUID", () => {
    expect(generateId()).toBe("uuid-0001");
    expect(generateId()).toBe("uuid-0002");
  });
});

describe("evidenceToFormData", () => {
  it("converts minimal evidence data to form data", () => {
    const fd = evidenceToFormData({
      collection_date: "2026-03-01",
      collecting_officer: "Doe",
      authorization: "Warrant #123",
      collected_items: [],
    });

    expect(fd.collection_date).toBe("2026-03-01");
    expect(fd.collecting_officer).toBe("Doe");
    expect(fd.authorization).toBe("Warrant #123");
    expect(fd.collected_items).toEqual([]);
  });

  it("defaults undefined fields to empty strings", () => {
    const fd = evidenceToFormData({
      collection_date: "",
      collecting_officer: "",
      authorization: "",
      collected_items: [],
    });

    expect(fd.system_date_time).toBe("");
    expect(fd.documentation_notes).toBe("");
    expect(fd.conditions).toBe("");
  });

  it("defaults undefined witnesses to empty array", () => {
    const fd = evidenceToFormData({
      collection_date: "",
      collecting_officer: "",
      authorization: "",
      collected_items: [],
    });

    expect(fd.witnesses).toEqual([]);
  });

  it("passes through witnesses array", () => {
    const fd = evidenceToFormData({
      collection_date: "",
      collecting_officer: "",
      authorization: "",
      witnesses: ["Alice", "Bob"],
      collected_items: [],
    });

    expect(fd.witnesses).toEqual(["Alice", "Bob"]);
  });

  it("converts collected items with defaults", () => {
    const fd = evidenceToFormData({
      collection_date: "",
      collecting_officer: "",
      authorization: "",
      collected_items: [{
        id: "item-1",
        item_number: "EV-001",
        description: "Laptop",
        condition: "good",
        packaging: "anti-static bag",
      }],
    });

    const items = fd.collected_items as any[];
    expect(items).toHaveLength(1);
    expect(items[0].id).toBe("item-1");
    expect(items[0].item_number).toBe("EV-001");
    expect(items[0].description).toBe("Laptop");
    // Defaults should be applied for missing fields
    expect(items[0].device_type).toBe("desktop_computer");
    expect(items[0].storage_interface).toBe("sata");
    expect(items[0].brand).toBe("");
  });
});

describe("formDataToEvidence", () => {
  it("converts minimal form data to evidence", () => {
    const result = formDataToEvidence({
      collection_date: "2026-03-01",
      collecting_officer: "Doe",
      authorization: "Warrant #123",
    });

    expect(result.collection_date).toBe("2026-03-01");
    expect(result.collecting_officer).toBe("Doe");
    expect(result.authorization).toBe("Warrant #123");
    expect(result.collected_items).toEqual([]);
  });

  it("parses witnesses from array", () => {
    const result = formDataToEvidence({
      witnesses: ["Alice", "Bob"],
    });

    expect(result.witnesses).toEqual(["Alice", "Bob"]);
  });

  it("parses witnesses from comma-separated string", () => {
    const result = formDataToEvidence({
      witnesses: "Alice, Bob, Charlie",
    });

    expect(result.witnesses).toEqual(["Alice", "Bob", "Charlie"]);
  });

  it("filters empty witness entries", () => {
    const result = formDataToEvidence({
      witnesses: "Alice,, ,Bob",
    });

    expect(result.witnesses).toEqual(["Alice", "Bob"]);
  });

  it("handles non-array/non-string witnesses as empty", () => {
    const result = formDataToEvidence({
      witnesses: 42 as any,
    });

    expect(result.witnesses).toEqual([]);
  });

  it("converts collected items from form data", () => {
    const result = formDataToEvidence({
      collected_items: [{
        id: "item-1",
        item_number: "EV-001",
        description: "Phone",
        device_type: "mobile_phone",
        condition: "fair",
      }],
    });

    expect(result.collected_items).toHaveLength(1);
    expect(result.collected_items[0].id).toBe("item-1");
    expect(result.collected_items[0].device_type).toBe("mobile_phone");
    expect(result.collected_items[0].condition).toBe("fair");
  });

  it("generates UUID for item without ID", () => {
    const result = formDataToEvidence({
      collected_items: [{
        description: "USB Drive",
      }],
    });

    expect(result.collected_items[0].id).toBe("uuid-0001");
  });

  it("applies default device_type and storage_interface", () => {
    const result = formDataToEvidence({
      collected_items: [{}],
    });

    expect(result.collected_items[0].device_type).toBe("desktop_computer");
    expect(result.collected_items[0].storage_interface).toBe("sata");
    expect(result.collected_items[0].condition).toBe("good");
  });

  it("round-trips through evidenceToFormData and back", () => {
    const original = {
      collection_date: "2026-03-01",
      collecting_officer: "Doe",
      authorization: "Warrant",
      witnesses: ["Alice", "Bob"],
      collected_items: [{
        id: "item-1",
        item_number: "EV-001",
        description: "Laptop",
        device_type: "laptop",
        storage_interface: "nvme",
        condition: "good",
        packaging: "sealed bag",
      }],
    };

    const fd = evidenceToFormData(original as any);
    const result = formDataToEvidence(fd);

    expect(result.collection_date).toBe("2026-03-01");
    expect(result.collecting_officer).toBe("Doe");
    expect(result.witnesses).toEqual(["Alice", "Bob"]);
    expect(result.collected_items[0].id).toBe("item-1");
    expect(result.collected_items[0].description).toBe("Laptop");
    expect(result.collected_items[0].device_type).toBe("laptop");
  });
});
