// =============================================================================
// useFormTemplate — pure function tests (evaluateCondition, buildDefaults, createRepeatableItem)
// =============================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";

let uuidCounter = 0;
vi.stubGlobal("crypto", {
  randomUUID: () => {
    uuidCounter++;
    return `uuid-${String(uuidCounter).padStart(4, "0")}`;
  },
});

import {
  evaluateCondition,
  buildDefaults,
  createRepeatableItem,
} from "../useFormTemplate";
import type { FieldCondition, SectionSchema, FormTemplate } from "../types";

beforeEach(() => {
  uuidCounter = 0;
});

// ── evaluateCondition ──

describe("evaluateCondition", () => {
  it("eq: returns true when values match", () => {
    const cond: FieldCondition = { field: "status", op: "eq", value: "active" };
    expect(evaluateCondition(cond, { status: "active" })).toBe(true);
  });

  it("eq: returns false when values differ", () => {
    const cond: FieldCondition = { field: "status", op: "eq", value: "active" };
    expect(evaluateCondition(cond, { status: "inactive" })).toBe(false);
  });

  it("neq: returns true when values differ", () => {
    const cond: FieldCondition = { field: "x", op: "neq", value: "a" };
    expect(evaluateCondition(cond, { x: "b" })).toBe(true);
  });

  it("neq: returns false when values match", () => {
    const cond: FieldCondition = { field: "x", op: "neq", value: "a" };
    expect(evaluateCondition(cond, { x: "a" })).toBe(false);
  });

  it("in: returns true when value is in array", () => {
    const cond: FieldCondition = { field: "role", op: "in", value: ["admin", "editor"] };
    expect(evaluateCondition(cond, { role: "admin" })).toBe(true);
  });

  it("in: returns false when value is not in array", () => {
    const cond: FieldCondition = { field: "role", op: "in", value: ["admin", "editor"] };
    expect(evaluateCondition(cond, { role: "viewer" })).toBe(false);
  });

  it("in: returns false when value is not an array", () => {
    const cond: FieldCondition = { field: "role", op: "in", value: "admin" };
    expect(evaluateCondition(cond, { role: "admin" })).toBe(false);
  });

  it("not_in: returns true when value is not in array", () => {
    const cond: FieldCondition = { field: "x", op: "not_in", value: ["a", "b"] };
    expect(evaluateCondition(cond, { x: "c" })).toBe(true);
  });

  it("not_in: returns false when value is in array", () => {
    const cond: FieldCondition = { field: "x", op: "not_in", value: ["a", "b"] };
    expect(evaluateCondition(cond, { x: "a" })).toBe(false);
  });

  it("not_in: returns true when value is not an array", () => {
    const cond: FieldCondition = { field: "x", op: "not_in", value: "a" };
    expect(evaluateCondition(cond, { x: "a" })).toBe(true);
  });

  it("truthy: returns true for truthy value", () => {
    const cond: FieldCondition = { field: "name", op: "truthy" };
    expect(evaluateCondition(cond, { name: "hello" })).toBe(true);
  });

  it("truthy: returns false for falsy value", () => {
    const cond: FieldCondition = { field: "name", op: "truthy" };
    expect(evaluateCondition(cond, { name: "" })).toBe(false);
    expect(evaluateCondition(cond, {})).toBe(false);
  });

  it("falsy: returns true for falsy value", () => {
    const cond: FieldCondition = { field: "x", op: "falsy" };
    expect(evaluateCondition(cond, { x: "" })).toBe(true);
    expect(evaluateCondition(cond, { x: 0 })).toBe(true);
    expect(evaluateCondition(cond, {})).toBe(true);
  });

  it("falsy: returns false for truthy value", () => {
    const cond: FieldCondition = { field: "x", op: "falsy" };
    expect(evaluateCondition(cond, { x: "yes" })).toBe(false);
  });

  it("unknown op defaults to true", () => {
    const cond = { field: "x", op: "banana" as any };
    expect(evaluateCondition(cond, { x: "anything" })).toBe(true);
  });

  it("itemData takes precedence over parent data", () => {
    const cond: FieldCondition = { field: "val", op: "eq", value: "item" };
    expect(evaluateCondition(cond, { val: "parent" }, { val: "item" })).toBe(true);
  });

  it("falls back to parent data when itemData lacks the field", () => {
    const cond: FieldCondition = { field: "val", op: "eq", value: "parent" };
    expect(evaluateCondition(cond, { val: "parent" }, {})).toBe(true);
  });
});

// ── buildDefaults ──

describe("buildDefaults", () => {
  it("returns empty object for template with no defaults", () => {
    const template: FormTemplate = {
      id: "test",
      name: "Test",
      version: "1.0.0",
      sections: [{
        id: "s1",
        title: "Section 1",
        fields: [
          { id: "f1", type: "text", label: "Field 1" },
          { id: "f2", type: "text", label: "Field 2" },
        ],
      }],
    } as FormTemplate;
    const data = buildDefaults(template);
    expect(data).toEqual({});
  });

  it("picks up field defaults", () => {
    const template = {
      id: "t",
      name: "T",
      version: "1",
      sections: [{
        id: "s1",
        title: "S1",
        fields: [
          { id: "name", type: "text", label: "Name", default: "John" },
          { id: "age", type: "number", label: "Age" },
          { id: "active", type: "checkbox", label: "Active", default: true },
        ],
      }],
    } as FormTemplate;
    const data = buildDefaults(template);
    expect(data.name).toBe("John");
    expect(data.age).toBeUndefined();
    expect(data.active).toBe(true);
  });

  it("initializes repeatable sections as empty arrays", () => {
    const template = {
      id: "t",
      name: "T",
      version: "1",
      sections: [{
        id: "items",
        title: "Items",
        repeatable: true,
        fields: [
          { id: "desc", type: "text", label: "Desc", default: "default" },
        ],
      }],
    } as FormTemplate;
    const data = buildDefaults(template);
    expect(data.items).toEqual([]);
    // Repeatable section fields should NOT be in top-level data
    expect(data.desc).toBeUndefined();
  });
});

// ── createRepeatableItem ──

describe("createRepeatableItem", () => {
  it("creates item with UUID and field defaults", () => {
    const section: SectionSchema = {
      id: "items",
      title: "Items",
      repeatable: true,
      fields: [
        { id: "desc", type: "text", label: "Desc", default: "untitled" },
        { id: "count", type: "number", label: "Count" },
      ],
    } as SectionSchema;
    const item = createRepeatableItem(section);
    expect(item.id).toBe("uuid-0001");
    expect(item.desc).toBe("untitled");
    expect(item.count).toBeUndefined();
  });

  it("skips heading, separator, and static fields", () => {
    const section: SectionSchema = {
      id: "items",
      title: "Items",
      fields: [
        { id: "h1", type: "heading", label: "Header" },
        { id: "s1", type: "separator", label: "" },
        { id: "st1", type: "static", label: "Info" },
        { id: "name", type: "text", label: "Name", default: "test" },
      ],
    } as SectionSchema;
    const item = createRepeatableItem(section);
    expect(item.h1).toBeUndefined();
    expect(item.s1).toBeUndefined();
    expect(item.st1).toBeUndefined();
    expect(item.name).toBe("test");
  });

  it("generates unique UUIDs per call", () => {
    const section = { id: "s", title: "S", fields: [] } as unknown as SectionSchema;
    const a = createRepeatableItem(section);
    const b = createRepeatableItem(section);
    expect(a.id).toBe("uuid-0001");
    expect(b.id).toBe("uuid-0002");
  });
});
