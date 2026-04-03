// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { LoggerLike } from "@core-suite/desktop-hooks";
import { logger } from "../utils/logger";

function adaptLoggerMethod(
  method: (message: string, ...args: unknown[]) => void
): (...args: unknown[]) => void {
  return (...args: unknown[]) => {
    if (args.length === 0) {
      method("");
      return;
    }

    const [message, ...rest] = args;
    method(typeof message === "string" ? message : String(message), ...rest);
  };
}

export function createDesktopHookLogger(scopeName: string): LoggerLike {
  const scopedLog = logger.scope(scopeName);

  return {
    debug: adaptLoggerMethod(scopedLog.debug),
    warn: adaptLoggerMethod(scopedLog.warn),
    error: adaptLoggerMethod(scopedLog.error),
  };
}