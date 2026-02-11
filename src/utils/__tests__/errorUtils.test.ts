// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import { getErrorMessage } from "../errorUtils";

describe("errorUtils", () => {
  describe("getErrorMessage", () => {
    it("extracts message from Error instances", () => {
      expect(getErrorMessage(new Error("Something failed"))).toBe("Something failed");
    });

    it("extracts message from Error subclasses", () => {
      expect(getErrorMessage(new TypeError("Type mismatch"))).toBe("Type mismatch");
      expect(getErrorMessage(new RangeError("Out of range"))).toBe("Out of range");
    });

    it("returns string errors as-is", () => {
      expect(getErrorMessage("plain string error")).toBe("plain string error");
      expect(getErrorMessage("")).toBe("");
    });

    it("converts numbers to string", () => {
      expect(getErrorMessage(404)).toBe("404");
      expect(getErrorMessage(0)).toBe("0");
    });

    it("converts null and undefined to string", () => {
      expect(getErrorMessage(null)).toBe("null");
      expect(getErrorMessage(undefined)).toBe("undefined");
    });

    it("converts objects to string", () => {
      expect(getErrorMessage({ code: 500 })).toBe("[object Object]");
    });

    it("converts booleans to string", () => {
      expect(getErrorMessage(false)).toBe("false");
      expect(getErrorMessage(true)).toBe("true");
    });
  });
});
