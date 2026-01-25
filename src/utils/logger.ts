// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Centralized Logging Utility
 * 
 * Provides consistent logging with debug messages only shown in development mode.
 * Use this instead of direct console.log/warn/error calls.
 * 
 * @example
 * ```tsx
 * import { logger } from '../utils/logger';
 * 
 * logger.debug('Loading project', { path });
 * logger.info('Project loaded successfully');
 * logger.warn('Cache expired, reloading');
 * logger.error('Failed to load project', error);
 * ```
 */

// =============================================================================
// Configuration
// =============================================================================

// Safely check for development mode (works in browser and Vite)
const isDev = typeof import.meta !== 'undefined' && import.meta.env?.DEV === true;

/** Log level configuration */
export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

/** Current minimum log level (can be adjusted at runtime) */
let minLevel: LogLevel = isDev ? 'debug' : 'info';

const levelPriority: Record<LogLevel, number> = {
  debug: 0,
  info: 1,
  warn: 2,
  error: 3,
};

// =============================================================================
// Logger Implementation
// =============================================================================

/**
 * Format a log message with optional context
 */
function formatMessage(level: LogLevel, message: string, context?: string): string {
  const timestamp = new Date().toISOString().slice(11, 23); // HH:mm:ss.SSS
  const prefix = context ? `[${context}]` : '';
  return `${timestamp} [${level.toUpperCase()}]${prefix} ${message}`;
}

/**
 * Check if a log level should be output
 */
function shouldLog(level: LogLevel): boolean {
  return levelPriority[level] >= levelPriority[minLevel];
}

/**
 * Centralized logger with conditional debug output
 */
export const logger = {
  /**
   * Debug-level logging (only in development)
   * Use for detailed debugging information
   */
  debug: (message: string, ...args: unknown[]): void => {
    if (shouldLog('debug')) {
      console.log(formatMessage('debug', message), ...args);
    }
  },

  /**
   * Info-level logging
   * Use for general information about app state/progress
   */
  info: (message: string, ...args: unknown[]): void => {
    if (shouldLog('info')) {
      console.log(formatMessage('info', message), ...args);
    }
  },

  /**
   * Warning-level logging
   * Use for potentially problematic situations
   */
  warn: (message: string, ...args: unknown[]): void => {
    if (shouldLog('warn')) {
      console.warn(formatMessage('warn', message), ...args);
    }
  },

  /**
   * Error-level logging
   * Use for error conditions
   */
  error: (message: string, ...args: unknown[]): void => {
    if (shouldLog('error')) {
      console.error(formatMessage('error', message), ...args);
    }
  },

  /**
   * Create a scoped logger with a context prefix
   * @example
   * const log = logger.scope('ProjectLoader');
   * log.debug('Loading...'); // outputs: [ProjectLoader] Loading...
   */
  scope: (context: string) => ({
    debug: (message: string, ...args: unknown[]): void => {
      if (shouldLog('debug')) {
        console.log(formatMessage('debug', message, context), ...args);
      }
    },
    info: (message: string, ...args: unknown[]): void => {
      if (shouldLog('info')) {
        console.log(formatMessage('info', message, context), ...args);
      }
    },
    warn: (message: string, ...args: unknown[]): void => {
      if (shouldLog('warn')) {
        console.warn(formatMessage('warn', message, context), ...args);
      }
    },
    error: (message: string, ...args: unknown[]): void => {
      if (shouldLog('error')) {
        console.error(formatMessage('error', message, context), ...args);
      }
    },
  }),

  /**
   * Set the minimum log level
   */
  setLevel: (level: LogLevel): void => {
    minLevel = level;
  },

  /**
   * Get the current log level
   */
  getLevel: (): LogLevel => minLevel,

  /**
   * Check if debug logging is enabled
   */
  isDebugEnabled: (): boolean => shouldLog('debug'),
};

export default logger;
