// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getErrorMessage } from "./errorUtils";

describe("getErrorMessage", () => {
  it("extracts message from Error instance", () => {
    const error = new Error("Something went wrong");
    expect(getErrorMessage(error)).toBe("Something went wrong");
  });

  it("extracts message from Error subclass", () => {
    const error = new TypeError("Invalid type");
    expect(getErrorMessage(error)).toBe("Invalid type");
  });

  it("returns string directly when thrown as string", () => {
    expect(getErrorMessage("string error")).toBe("string error");
  });

  it("converts number to string", () => {
    expect(getErrorMessage(42)).toBe("42");
  });

  it("converts null to string", () => {
    expect(getErrorMessage(null)).toBe("null");
  });

  it("converts undefined to string", () => {
    expect(getErrorMessage(undefined)).toBe("undefined");
  });

  it("converts boolean to string", () => {
    expect(getErrorMessage(false)).toBe("false");
  });

  it("converts object to string", () => {
    expect(getErrorMessage({ code: 404 })).toBe("[object Object]");
  });

  it("handles empty Error message", () => {
    expect(getErrorMessage(new Error(""))).toBe("");
  });

  it("handles Error with complex message", () => {
    const msg = "Connection refused: ECONNREFUSED 127.0.0.1:3000";
    expect(getErrorMessage(new Error(msg))).toBe(msg);
  });
});
