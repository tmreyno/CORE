// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { logger } from "./logger";
const log = logger.scope("OperationProfiler");

/**
 * Operation Timing Profiler
 * 
 * Instruments key operations to measure actual performance in the running app.
 * Can be enabled via dev tools to collect real timing data.
 * 
 * @example
 * ```tsx
 * import { operationProfiler } from '../utils/operationProfiler';
 * 
 * // Enable profiling
 * operationProfiler.enable();
 * 
 * // Use in components
 * const result = await operationProfiler.measure('tree-expand', async () => {
 *   return invoke('container_get_children', { ... });
 * });
 * 
 * // Get report
 * operationProfiler.printReport();
 * ```
 */

// =============================================================================
// Types
// =============================================================================

export interface OperationTiming {
  name: string;
  startTime: number;
  endTime: number;
  duration: number;
  metadata?: Record<string, unknown>;
}

export interface OperationStats {
  name: string;
  count: number;
  totalMs: number;
  avgMs: number;
  minMs: number;
  maxMs: number;
  p50Ms: number;
  p95Ms: number;
  p99Ms: number;
  durations: number[];
}

export interface ProfilerReport {
  enabled: boolean;
  startedAt: number;
  duration: number;
  operations: Record<string, OperationStats>;
  recentTimings: OperationTiming[];
}

// =============================================================================
// Categories for key operations
// =============================================================================

export const OperationCategories = {
  // Tree operations
  TREE_EXPAND: 'tree-expand',
  TREE_ROOT_LOAD: 'tree-root-load',
  TREE_SEARCH: 'tree-search',
  
  // Viewer operations
  VIEWER_INIT: 'viewer-init',
  VIEWER_HEX_CHUNK: 'viewer-hex-chunk',
  VIEWER_TEXT_LOAD: 'viewer-text-load',
  VIEWER_DOCUMENT_LOAD: 'viewer-document-load',
  VIEWER_IMAGE_LOAD: 'viewer-image-load',
  
  // Hash operations
  HASH_INIT: 'hash-init',
  HASH_PROGRESS: 'hash-progress',
  HASH_COMPLETE: 'hash-complete',
  
  // Container operations
  CONTAINER_INFO: 'container-info',
  CONTAINER_VERIFY: 'container-verify',
  CONTAINER_EXTRACT: 'container-extract',
  
  // File operations
  FILE_DISCOVER: 'file-discover',
  FILE_SCAN: 'file-scan',
} as const;

export type OperationCategory = typeof OperationCategories[keyof typeof OperationCategories];

// =============================================================================
// Profiler Implementation
// =============================================================================

class OperationProfiler {
  private enabled = false;
  private startedAt = 0;
  private timings: OperationTiming[] = [];
  private maxTimings = 1000; // Keep last N timings

  /**
   * Enable profiling
   */
  enable(): void {
    this.enabled = true;
    this.startedAt = performance.now();
    this.timings = [];
    log.info("Enabled - collecting timing data");
  }

  /**
   * Disable profiling
   */
  disable(): void {
    this.enabled = false;
    log.info("Disabled");
  }

  /**
   * Check if profiling is enabled
   */
  isEnabled(): boolean {
    return this.enabled;
  }

  /**
   * Clear all timing data
   */
  clear(): void {
    this.timings = [];
    this.startedAt = performance.now();
  }

  /**
   * Measure an async operation
   */
  async measure<T>(
    name: OperationCategory | string,
    fn: () => Promise<T>,
    metadata?: Record<string, unknown>
  ): Promise<T> {
    if (!this.enabled) {
      return fn();
    }

    const startTime = performance.now();
    try {
      const result = await fn();
      const endTime = performance.now();
      
      this.recordTiming({
        name,
        startTime,
        endTime,
        duration: endTime - startTime,
        metadata,
      });
      
      return result;
    } catch (error) {
      const endTime = performance.now();
      this.recordTiming({
        name,
        startTime,
        endTime,
        duration: endTime - startTime,
        metadata: { ...metadata, error: true },
      });
      throw error;
    }
  }

  /**
   * Measure a sync operation
   */
  measureSync<T>(
    name: OperationCategory | string,
    fn: () => T,
    metadata?: Record<string, unknown>
  ): T {
    if (!this.enabled) {
      return fn();
    }

    const startTime = performance.now();
    try {
      const result = fn();
      const endTime = performance.now();
      
      this.recordTiming({
        name,
        startTime,
        endTime,
        duration: endTime - startTime,
        metadata,
      });
      
      return result;
    } catch (error) {
      const endTime = performance.now();
      this.recordTiming({
        name,
        startTime,
        endTime,
        duration: endTime - startTime,
        metadata: { ...metadata, error: true },
      });
      throw error;
    }
  }

  /**
   * Start a timing (for operations that span multiple steps)
   */
  startTiming(name: OperationCategory | string): () => void {
    if (!this.enabled) {
      return () => {};
    }

    const startTime = performance.now();
    return (metadata?: Record<string, unknown>) => {
      const endTime = performance.now();
      this.recordTiming({
        name,
        startTime,
        endTime,
        duration: endTime - startTime,
        metadata,
      });
    };
  }

  /**
   * Record a timing entry
   */
  private recordTiming(timing: OperationTiming): void {
    this.timings.push(timing);
    
    // Keep only last N timings
    if (this.timings.length > this.maxTimings) {
      this.timings = this.timings.slice(-this.maxTimings);
    }
  }

  /**
   * Get statistics for an operation
   */
  getStats(name: string): OperationStats | null {
    const durations = this.timings
      .filter(t => t.name === name)
      .map(t => t.duration);
    
    if (durations.length === 0) {
      return null;
    }

    const sorted = [...durations].sort((a, b) => a - b);
    const totalMs = durations.reduce((a, b) => a + b, 0);

    return {
      name,
      count: durations.length,
      totalMs,
      avgMs: totalMs / durations.length,
      minMs: sorted[0],
      maxMs: sorted[sorted.length - 1],
      p50Ms: sorted[Math.floor(sorted.length * 0.5)],
      p95Ms: sorted[Math.floor(sorted.length * 0.95)] || sorted[sorted.length - 1],
      p99Ms: sorted[Math.floor(sorted.length * 0.99)] || sorted[sorted.length - 1],
      durations,
    };
  }

  /**
   * Get all operation names that have been recorded
   */
  getOperationNames(): string[] {
    return [...new Set(this.timings.map(t => t.name))];
  }

  /**
   * Get a full profiler report
   */
  getReport(): ProfilerReport {
    const operationNames = this.getOperationNames();
    const operations: Record<string, OperationStats> = {};
    
    for (const name of operationNames) {
      const stats = this.getStats(name);
      if (stats) {
        operations[name] = stats;
      }
    }

    return {
      enabled: this.enabled,
      startedAt: this.startedAt,
      duration: performance.now() - this.startedAt,
      operations,
      recentTimings: this.timings.slice(-50), // Last 50 timings
    };
  }

  /**
   * Print a formatted report to console
   */
  printReport(): void {
    const report = this.getReport();
    
    log.info('\n========================================');
    log.info('OPERATION PROFILER REPORT');
    log.info('========================================');
    log.info(`Profiling: ${report.enabled ? 'ENABLED' : 'DISABLED'}`);
    log.info(`Duration: ${(report.duration / 1000).toFixed(2)}s`);
    log.info('----------------------------------------\n');
    
    const sortedOps = Object.values(report.operations)
      .sort((a, b) => b.avgMs - a.avgMs);
    
    for (const op of sortedOps) {
      log.info(`📊 ${op.name}`);
      log.info(`   Count: ${op.count}`);
      log.info(`   Avg: ${op.avgMs.toFixed(2)}ms`);
      log.info(`   Min: ${op.minMs.toFixed(2)}ms | Max: ${op.maxMs.toFixed(2)}ms`);
      log.info(`   P50: ${op.p50Ms.toFixed(2)}ms | P95: ${op.p95Ms.toFixed(2)}ms | P99: ${op.p99Ms.toFixed(2)}ms`);
      
      // Performance grade
      const grade = op.avgMs < 16 ? '🟢 Excellent' :
                   op.avgMs < 50 ? '🟢 Good' :
                   op.avgMs < 100 ? '🟡 Acceptable' :
                   op.avgMs < 300 ? '🟠 Slow' : '🔴 Critical';
      log.info(`   Grade: ${grade}\n`);
    }
    
    log.info('========================================\n');
  }

  /**
   * Export timing data as JSON
   */
  exportJSON(): string {
    return JSON.stringify(this.getReport(), null, 2);
  }

  /**
   * Export timing data as CSV
   */
  exportCSV(): string {
    const lines = ['name,startTime,endTime,duration,metadata'];
    for (const t of this.timings) {
      lines.push(`${t.name},${t.startTime},${t.endTime},${t.duration},"${JSON.stringify(t.metadata || {})}"`);
    }
    return lines.join('\n');
  }
}

// =============================================================================
// Singleton Instance
// =============================================================================

export const operationProfiler = new OperationProfiler();

// Make available in dev tools console
if (typeof window !== 'undefined') {
  (window as unknown as { __profiler: OperationProfiler }).__profiler = operationProfiler;
}

// =============================================================================
// Convenience hooks for SolidJS
// =============================================================================

import { onMount, onCleanup } from "solid-js";

/**
 * Hook to automatically profile component mount time
 */
export function useProfileMount(componentName: string): void {
  const endTiming = operationProfiler.startTiming(`component-mount:${componentName}`);
  onMount(() => {
    endTiming();
  });
}

/**
 * Hook to run profiler and print report on unmount (useful for dev)
 */
export function useProfilerReport(): void {
  onMount(() => {
    if (!operationProfiler.isEnabled()) {
      operationProfiler.enable();
    }
  });
  
  onCleanup(() => {
    operationProfiler.printReport();
  });
}
