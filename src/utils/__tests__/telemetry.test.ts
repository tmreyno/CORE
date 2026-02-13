// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import {
  generateId,
  getSeverityRank,
  detectCategory,
  sanitizeContext,
  logError,
  logInfo,
  logAuditAction,
  type AuditAction,
} from "../telemetry";

// =============================================================================
// generateId
// =============================================================================

describe("generateId", () => {
  it("returns a string", () => {
    expect(typeof generateId()).toBe("string");
  });

  it("returns unique values on successive calls", () => {
    const ids = new Set(Array.from({ length: 100 }, () => generateId()));
    expect(ids.size).toBe(100);
  });

  it("contains a timestamp and random segment separated by dash", () => {
    const id = generateId();
    // Format is "{base36-timestamp}-{base36-random}"
    expect(id).toMatch(/^[a-z0-9]+-[a-z0-9]+$/);
  });
});

// =============================================================================
// getSeverityRank
// =============================================================================

describe("getSeverityRank", () => {
  it("ranks debug lowest", () => {
    expect(getSeverityRank("debug")).toBe(0);
  });

  it("ranks info at 1", () => {
    expect(getSeverityRank("info")).toBe(1);
  });

  it("ranks warning at 2", () => {
    expect(getSeverityRank("warning")).toBe(2);
  });

  it("ranks error at 3", () => {
    expect(getSeverityRank("error")).toBe(3);
  });

  it("ranks fatal highest at 4", () => {
    expect(getSeverityRank("fatal")).toBe(4);
  });

  it("returns undefined for unknown severity", () => {
    expect(getSeverityRank("unknown" as any)).toBeUndefined();
  });
});

// =============================================================================
// detectCategory
// =============================================================================

describe("detectCategory", () => {
  it("detects network errors", () => {
    expect(detectCategory(new TypeError("Failed to fetch"))).toBe("network");
    expect(detectCategory(new Error("NetworkError occurred"))).toBe("network");
  });

  it("returns unknown for unrecognized network-related errors", () => {
    // "CORS" alone is not detected as network — doesn't contain "network" or "fetch"
    expect(detectCategory(new Error("CORS error"))).toBe("unknown");
  });

  it("detects filesystem errors", () => {
    expect(detectCategory(new Error("ENOENT: no such file"))).toBe("filesystem");
    expect(detectCategory(new Error("File not found"))).toBe("filesystem");
    expect(detectCategory(new Error("path resolution failed"))).toBe("filesystem");
  });

  it("returns unknown for permission errors without file/path keywords", () => {
    // "EACCES: permission denied" doesn't contain file, path, or enoent
    expect(detectCategory(new Error("EACCES: permission denied"))).toBe("unknown");
  });

  it("detects parser errors", () => {
    // detectCategory checks message for "parse" or "syntax" and name for "parse"
    expect(detectCategory(new Error("Parse error in module"))).toBe("parser");
    expect(detectCategory(new Error("syntax error at line 5"))).toBe("parser");
  });

  it("does not detect SyntaxError by name alone", () => {
    // SyntaxError name is "syntaxerror" lowercase — doesn't contain "parse"
    // message "Unexpected token" doesn't contain "parse" or "syntax"
    expect(detectCategory(new SyntaxError("Unexpected token"))).toBe("unknown");
  });

  it("detects database errors", () => {
    expect(detectCategory(new Error("SQLite error"))).toBe("database");
    expect(detectCategory(new Error("Database connection failed"))).toBe("database");
  });

  it("detects Tauri errors", () => {
    expect(detectCategory(new Error("tauri invoke failed"))).toBe("tauri");
    expect(detectCategory(new Error("invoke command error"))).toBe("tauri");
  });

  it("does not detect IPC as tauri without keyword", () => {
    // "IPC channel error" doesn't contain "tauri" or "invoke"
    expect(detectCategory(new Error("IPC channel error"))).toBe("unknown");
  });

  it("detects UI errors", () => {
    expect(detectCategory(new Error("Render error in component"))).toBe("ui");
    expect(detectCategory(new Error("Component mount failed"))).toBe("ui");
  });

  it("returns 'unknown' for generic errors", () => {
    expect(detectCategory(new Error("Something went wrong"))).toBe("unknown");
    expect(detectCategory(new Error(""))).toBe("unknown");
  });
});

// =============================================================================
// sanitizeContext
// =============================================================================

describe("sanitizeContext", () => {
  it("redacts keys containing 'password'", () => {
    const result = sanitizeContext({ password: "secret123" });
    expect(result.password).toBe("[REDACTED]");
  });

  it("redacts keys containing 'token'", () => {
    const result = sanitizeContext({ apiToken: "abc123", authToken: "xyz" });
    expect(result.apiToken).toBe("[REDACTED]");
    expect(result.authToken).toBe("[REDACTED]");
  });

  it("redacts keys containing 'secret'", () => {
    const result = sanitizeContext({ clientSecret: "s3cr3t" });
    expect(result.clientSecret).toBe("[REDACTED]");
  });

  it("redacts keys containing 'key'", () => {
    const result = sanitizeContext({ apiKey: "k3y" });
    expect(result.apiKey).toBe("[REDACTED]");
  });

  it("redacts keys containing 'auth'", () => {
    const result = sanitizeContext({ authorization: "Bearer abc" });
    expect(result.authorization).toBe("[REDACTED]");
  });

  it("redacts keys containing 'credential'", () => {
    const result = sanitizeContext({ userCredential: "cred" });
    expect(result.userCredential).toBe("[REDACTED]");
  });

  it("preserves non-sensitive keys", () => {
    const result = sanitizeContext({ filename: "test.ad1", size: 1024 });
    expect(result.filename).toBe("test.ad1");
    expect(result.size).toBe(1024);
  });

  it("handles nested objects recursively", () => {
    const result = sanitizeContext({
      config: { apiKey: "secret", host: "localhost" },
    });
    expect(result.config).toEqual({ apiKey: "[REDACTED]", host: "localhost" });
  });

  it("handles empty objects", () => {
    expect(sanitizeContext({})).toEqual({});
  });

  it("handles null and undefined values", () => {
    const result = sanitizeContext({ a: null, b: undefined, c: "test" });
    expect(result.a).toBeNull();
    expect(result.b).toBeUndefined();
    expect(result.c).toBe("test");
  });
});

// =============================================================================
// logError / logInfo
// =============================================================================

describe("logError", () => {
  beforeEach(() => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    vi.spyOn(console, "warn").mockImplementation(() => {});
    vi.spyOn(console, "info").mockImplementation(() => {});
    vi.spyOn(console, "debug").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("logs an error without throwing", () => {
    expect(() => {
      logError(new Error("test error"), { severity: "error", source: "test" });
    }).not.toThrow();
  });

  it("accepts string errors", () => {
    expect(() => {
      logError("something failed", { severity: "warning" });
    }).not.toThrow();
  });
});

describe("logInfo", () => {
  beforeEach(() => {
    vi.spyOn(console, "info").mockImplementation(() => {});
    vi.spyOn(console, "debug").mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("logs info messages without throwing", () => {
    expect(() => {
      logInfo("test info message", { context: { detail: "some detail" } });
    }).not.toThrow();
  });
});

// =============================================================================
// logAuditAction
// =============================================================================

describe("logAuditAction", () => {
  const mockLocalStorage: Record<string, string> = {};

  beforeEach(() => {
    // Mock localStorage
    vi.stubGlobal("localStorage", {
      getItem: vi.fn((key: string) => mockLocalStorage[key] ?? null),
      setItem: vi.fn((key: string, value: string) => {
        mockLocalStorage[key] = value;
      }),
      removeItem: vi.fn((key: string) => {
        delete mockLocalStorage[key];
      }),
    });

    // Enable audit logging in preferences
    mockLocalStorage["ffx-preferences"] = JSON.stringify({ auditLogging: true });
    // Clear audit log
    delete mockLocalStorage["ffx-audit-log"];
  });

  afterEach(() => {
    vi.restoreAllMocks();
    // Clean up
    for (const key of Object.keys(mockLocalStorage)) {
      delete mockLocalStorage[key];
    }
  });

  it("writes an audit entry to localStorage", () => {
    logAuditAction("file_opened", { filePath: "/evidence/test.ad1" });

    const stored = mockLocalStorage["ffx-audit-log"];
    expect(stored).toBeDefined();
    const entries = JSON.parse(stored);
    expect(entries.length).toBe(1);
    expect(entries[0].action).toBe("file_opened");
    expect(entries[0].details.filePath).toBe("/evidence/test.ad1");
  });

  it("appends multiple audit entries", () => {
    logAuditAction("file_opened", { filePath: "/test1.ad1" });
    logAuditAction("hash_verified", { algorithm: "SHA-256" });

    const stored = mockLocalStorage["ffx-audit-log"];
    const entries = JSON.parse(stored);
    expect(entries.length).toBe(2);
  });

  it("does not write when audit logging is disabled", () => {
    mockLocalStorage["ffx-preferences"] = JSON.stringify({ auditLogging: false });

    logAuditAction("file_opened", { filePath: "/test.ad1" });

    expect(mockLocalStorage["ffx-audit-log"]).toBeUndefined();
  });
});
