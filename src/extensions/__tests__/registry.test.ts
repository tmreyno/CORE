// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, afterEach } from "vitest";
import {
  registerExtension,
  unregisterExtension,
  enableExtension,
  disableExtension,
  getAllExtensions,
  getEnabledExtensions,
  getExtension,
  getExtensionState,
  getExtensionsByCategory,
} from "../registry";
import type { DatabaseProcessorExtension } from "../types";

// Mock the database hooks
vi.mock("../../hooks/useDatabase", () => ({
  getSetting: vi.fn().mockResolvedValue(null),
  setSetting: vi.fn().mockResolvedValue(undefined),
}));

describe("Extension Registry", () => {
  // Helper to create a test database processor extension
  function createTestDbProcessor(id: string): DatabaseProcessorExtension {
    return {
      manifest: {
        id,
        name: `Test ${id}`,
        version: "1.0.0",
        description: `Test extension ${id}`,
        author: "Test Author",
        category: "database-processor",
      },
      dbType: "test-db",
      filePatterns: ["*.testdb"],
      detect: vi.fn().mockResolvedValue(true),
      parse: vi.fn().mockResolvedValue({
        id: "test",
        name: "Test DB",
        path: "/test",
        type: "test-db",
        size: 0,
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
        version: "1.0",
      }),
      // Lifecycle hooks
      onLoad: vi.fn().mockResolvedValue(undefined),
      onEnable: vi.fn().mockResolvedValue(undefined),
      onDisable: vi.fn().mockResolvedValue(undefined),
    } as unknown as DatabaseProcessorExtension;
  }

  // Clean up after each test
  afterEach(async () => {
    const all = getAllExtensions();
    for (const ext of all) {
      try {
        await unregisterExtension(ext.manifest.id);
      } catch {
        // Ignore
      }
    }
  });

  describe("registerExtension", () => {
    it("should register a new extension", async () => {
      const ext = createTestDbProcessor("test-ext-1");
      
      await registerExtension(ext);
      
      const registered = getExtension("test-ext-1");
      expect(registered).toBeDefined();
      expect(registered?.manifest.name).toBe("Test test-ext-1");
    });

    it("should throw error for duplicate registration", async () => {
      const ext = createTestDbProcessor("duplicate-test");
      
      await registerExtension(ext);
      
      await expect(registerExtension(ext)).rejects.toThrow("Extension already registered");
    });
  });

  describe("unregisterExtension", () => {
    it("should unregister an extension", async () => {
      const ext = createTestDbProcessor("unregister-test");
      await registerExtension(ext);
      
      await unregisterExtension("unregister-test");
      
      expect(getExtension("unregister-test")).toBeUndefined();
    });

    it("should throw error for non-existent extension", async () => {
      await expect(unregisterExtension("non-existent")).rejects.toThrow("Extension not found");
    });
  });

  describe("enableExtension", () => {
    it("should enable an extension", async () => {
      const ext = createTestDbProcessor("enable-test");
      await registerExtension(ext);
      
      await enableExtension("enable-test");
      
      const state = getExtensionState("enable-test");
      expect(state?.enabled).toBe(true);
    });

    it("should be idempotent", async () => {
      const ext = createTestDbProcessor("idempotent-enable");
      await registerExtension(ext);
      
      await enableExtension("idempotent-enable");
      await enableExtension("idempotent-enable");
      
      const state = getExtensionState("idempotent-enable");
      expect(state?.enabled).toBe(true);
    });
  });

  describe("disableExtension", () => {
    it("should disable an extension", async () => {
      const ext = createTestDbProcessor("disable-test");
      await registerExtension(ext);
      await enableExtension("disable-test");
      
      await disableExtension("disable-test");
      
      const state = getExtensionState("disable-test");
      expect(state?.enabled).toBe(false);
    });

    it("should be idempotent", async () => {
      const ext = createTestDbProcessor("idempotent-disable");
      await registerExtension(ext);
      await enableExtension("idempotent-disable");
      
      await disableExtension("idempotent-disable");
      await disableExtension("idempotent-disable");
      
      const state = getExtensionState("idempotent-disable");
      expect(state?.enabled).toBe(false);
    });
  });

  describe("getAllExtensions", () => {
    it("should return all registered extensions", async () => {
      await registerExtension(createTestDbProcessor("all-1"));
      await registerExtension(createTestDbProcessor("all-2"));
      await registerExtension(createTestDbProcessor("all-3"));
      
      const all = getAllExtensions();
      
      expect(all.length).toBe(3);
    });
  });

  describe("getEnabledExtensions", () => {
    it("should return only enabled extensions", async () => {
      await registerExtension(createTestDbProcessor("enabled-1"));
      await registerExtension(createTestDbProcessor("enabled-2"));
      await registerExtension(createTestDbProcessor("disabled-1"));
      
      await enableExtension("enabled-1");
      await enableExtension("enabled-2");
      
      const enabled = getEnabledExtensions();
      
      expect(enabled.length).toBe(2);
      expect(enabled.some(e => e.manifest.id === "enabled-1")).toBe(true);
      expect(enabled.some(e => e.manifest.id === "enabled-2")).toBe(true);
      expect(enabled.some(e => e.manifest.id === "disabled-1")).toBe(false);
    });
  });

  describe("getExtensionsByCategory", () => {
    it("should filter enabled extensions by category", async () => {
      await registerExtension(createTestDbProcessor("db-filter-1"));
      await registerExtension(createTestDbProcessor("db-filter-2"));
      
      // Must enable to be included in category filter
      await enableExtension("db-filter-1");
      await enableExtension("db-filter-2");
      
      const dbProcessors = getExtensionsByCategory("database-processor");
      
      expect(dbProcessors.some(e => e.manifest.id === "db-filter-1")).toBe(true);
      expect(dbProcessors.some(e => e.manifest.id === "db-filter-2")).toBe(true);
    });
  });

  describe("error handling", () => {
    it("should throw for non-existent extension on enable", async () => {
      await expect(enableExtension("non-existent")).rejects.toThrow("Extension not found");
    });

    it("should throw for non-existent extension on disable", async () => {
      await expect(disableExtension("non-existent")).rejects.toThrow("Extension not found");
    });
  });
});
