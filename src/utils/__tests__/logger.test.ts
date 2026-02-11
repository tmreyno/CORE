// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { logger, type LogLevel } from "../logger";

describe("logger", () => {
  let logSpy: ReturnType<typeof vi.spyOn>;
  let warnSpy: ReturnType<typeof vi.spyOn>;
  let errorSpy: ReturnType<typeof vi.spyOn>;
  let savedLevel: LogLevel;

  beforeEach(() => {
    logSpy = vi.spyOn(console, "log").mockImplementation(() => {});
    warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});
    errorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    savedLevel = logger.getLevel();
    logger.setLevel("debug"); // Enable all levels for testing
  });

  afterEach(() => {
    logSpy.mockRestore();
    warnSpy.mockRestore();
    errorSpy.mockRestore();
    logger.setLevel(savedLevel);
  });

  // ===========================================================================
  // Basic logging methods
  // ===========================================================================

  describe("debug", () => {
    it("outputs via console.log when level is debug", () => {
      logger.debug("test message");
      expect(logSpy).toHaveBeenCalledTimes(1);
      expect(logSpy.mock.calls[0][0]).toContain("[DEBUG]");
      expect(logSpy.mock.calls[0][0]).toContain("test message");
    });

    it("passes additional args", () => {
      logger.debug("msg", { key: "value" }, 42);
      expect(logSpy).toHaveBeenCalledWith(
        expect.stringContaining("msg"),
        { key: "value" },
        42
      );
    });
  });

  describe("info", () => {
    it("outputs via console.log with [INFO] prefix", () => {
      logger.info("info message");
      expect(logSpy).toHaveBeenCalledTimes(1);
      expect(logSpy.mock.calls[0][0]).toContain("[INFO]");
      expect(logSpy.mock.calls[0][0]).toContain("info message");
    });
  });

  describe("warn", () => {
    it("outputs via console.warn with [WARN] prefix", () => {
      logger.warn("warning message");
      expect(warnSpy).toHaveBeenCalledTimes(1);
      expect(warnSpy.mock.calls[0][0]).toContain("[WARN]");
      expect(warnSpy.mock.calls[0][0]).toContain("warning message");
    });
  });

  describe("error", () => {
    it("outputs via console.error with [ERROR] prefix", () => {
      logger.error("error message");
      expect(errorSpy).toHaveBeenCalledTimes(1);
      expect(errorSpy.mock.calls[0][0]).toContain("[ERROR]");
      expect(errorSpy.mock.calls[0][0]).toContain("error message");
    });
  });

  // ===========================================================================
  // Message formatting
  // ===========================================================================

  describe("message formatting", () => {
    it("includes timestamp in HH:mm:ss.SSS format", () => {
      logger.info("timestamp test");
      const output = logSpy.mock.calls[0][0] as string;
      // Timestamp pattern: NN:NN:NN.NNN
      expect(output).toMatch(/\d{2}:\d{2}:\d{2}\.\d{3}/);
    });

    it("formats without context prefix for root logger", () => {
      logger.info("no context");
      const output = logSpy.mock.calls[0][0] as string;
      // Should NOT have [SomeContext] between [INFO] and message
      expect(output).toMatch(/\[INFO\] no context/);
    });
  });

  // ===========================================================================
  // Level filtering
  // ===========================================================================

  describe("level filtering", () => {
    it("suppresses debug when level is info", () => {
      logger.setLevel("info");
      logger.debug("should not appear");
      expect(logSpy).not.toHaveBeenCalled();
    });

    it("allows info when level is info", () => {
      logger.setLevel("info");
      logger.info("should appear");
      expect(logSpy).toHaveBeenCalledTimes(1);
    });

    it("suppresses debug and info when level is warn", () => {
      logger.setLevel("warn");
      logger.debug("no");
      logger.info("no");
      expect(logSpy).not.toHaveBeenCalled();
    });

    it("allows warn when level is warn", () => {
      logger.setLevel("warn");
      logger.warn("yes");
      expect(warnSpy).toHaveBeenCalledTimes(1);
    });

    it("suppresses debug, info, warn when level is error", () => {
      logger.setLevel("error");
      logger.debug("no");
      logger.info("no");
      logger.warn("no");
      expect(logSpy).not.toHaveBeenCalled();
      expect(warnSpy).not.toHaveBeenCalled();
    });

    it("allows error when level is error", () => {
      logger.setLevel("error");
      logger.error("yes");
      expect(errorSpy).toHaveBeenCalledTimes(1);
    });

    it("allows all levels when set to debug", () => {
      logger.setLevel("debug");
      logger.debug("d");
      logger.info("i");
      logger.warn("w");
      logger.error("e");
      expect(logSpy).toHaveBeenCalledTimes(2); // debug + info both use console.log
      expect(warnSpy).toHaveBeenCalledTimes(1);
      expect(errorSpy).toHaveBeenCalledTimes(1);
    });
  });

  // ===========================================================================
  // setLevel / getLevel
  // ===========================================================================

  describe("setLevel / getLevel", () => {
    it("returns the level that was set", () => {
      logger.setLevel("warn");
      expect(logger.getLevel()).toBe("warn");
    });

    it("persists across calls", () => {
      logger.setLevel("error");
      expect(logger.getLevel()).toBe("error");
      logger.setLevel("debug");
      expect(logger.getLevel()).toBe("debug");
    });
  });

  // ===========================================================================
  // isDebugEnabled
  // ===========================================================================

  describe("isDebugEnabled", () => {
    it("returns true when level is debug", () => {
      logger.setLevel("debug");
      expect(logger.isDebugEnabled()).toBe(true);
    });

    it("returns false when level is info", () => {
      logger.setLevel("info");
      expect(logger.isDebugEnabled()).toBe(false);
    });

    it("returns false when level is warn", () => {
      logger.setLevel("warn");
      expect(logger.isDebugEnabled()).toBe(false);
    });

    it("returns false when level is error", () => {
      logger.setLevel("error");
      expect(logger.isDebugEnabled()).toBe(false);
    });
  });

  // ===========================================================================
  // Scoped logger
  // ===========================================================================

  describe("scope", () => {
    it("creates a scoped logger with context prefix", () => {
      const log = logger.scope("TestScope");
      log.info("scoped message");
      const output = logSpy.mock.calls[0][0] as string;
      expect(output).toContain("[TestScope]");
      expect(output).toContain("scoped message");
    });

    it("scoped debug respects level filtering", () => {
      logger.setLevel("info");
      const log = logger.scope("TestScope");
      log.debug("should not appear");
      expect(logSpy).not.toHaveBeenCalled();
    });

    it("scoped warn uses console.warn", () => {
      const log = logger.scope("WarnScope");
      log.warn("warning");
      expect(warnSpy).toHaveBeenCalledTimes(1);
      expect(warnSpy.mock.calls[0][0]).toContain("[WarnScope]");
    });

    it("scoped error uses console.error", () => {
      const log = logger.scope("ErrorScope");
      log.error("error");
      expect(errorSpy).toHaveBeenCalledTimes(1);
      expect(errorSpy.mock.calls[0][0]).toContain("[ErrorScope]");
    });

    it("scoped logger passes additional args", () => {
      const log = logger.scope("Args");
      log.info("msg", { data: true });
      expect(logSpy).toHaveBeenCalledWith(
        expect.stringContaining("[Args]"),
        { data: true }
      );
    });

    it("multiple scopes are independent", () => {
      const log1 = logger.scope("Scope1");
      const log2 = logger.scope("Scope2");
      log1.info("first");
      log2.info("second");
      expect(logSpy.mock.calls[0][0]).toContain("[Scope1]");
      expect(logSpy.mock.calls[1][0]).toContain("[Scope2]");
    });
  });
});
