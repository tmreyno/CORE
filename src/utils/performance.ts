// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Performance monitoring utilities for tracking render performance,
 * memory usage, and identifying bottlenecks in the application.
 */

import { createSignal, onMount, onCleanup, createEffect, Accessor } from "solid-js";

// ============================================================================
// Types
// ============================================================================

export interface RenderMetrics {
  /** Component name */
  name: string;
  /** Number of renders */
  renderCount: number;
  /** Last render duration in ms */
  lastRenderDuration: number;
  /** Average render duration in ms */
  avgRenderDuration: number;
  /** Total render time in ms */
  totalRenderTime: number;
  /** Timestamp of last render */
  lastRenderTimestamp: number;
}

export interface MemoryMetrics {
  /** Used JS heap size in bytes */
  usedJSHeapSize: number;
  /** Total JS heap size in bytes */
  totalJSHeapSize: number;
  /** JS heap size limit in bytes */
  jsHeapSizeLimit: number;
  /** Formatted used heap */
  usedHeapFormatted: string;
  /** Formatted total heap */
  totalHeapFormatted: string;
}

export interface PerformanceEntry {
  timestamp: number;
  type: "render" | "effect" | "memo" | "action";
  name: string;
  duration: number;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Performance Store
// ============================================================================

const performanceStore = {
  metrics: new Map<string, RenderMetrics>(),
  entries: [] as PerformanceEntry[],
  maxEntries: 1000,
  enabled: false,
};

/**
 * Enable or disable performance monitoring
 */
export function setPerformanceMonitoringEnabled(enabled: boolean): void {
  performanceStore.enabled = enabled;
  if (!enabled) {
    performanceStore.metrics.clear();
    performanceStore.entries = [];
  }
}

/**
 * Check if performance monitoring is enabled
 */
export function isPerformanceMonitoringEnabled(): boolean {
  return performanceStore.enabled;
}

// ============================================================================
// Render Tracking
// ============================================================================

/**
 * Track component render performance
 * 
 * @example
 * ```tsx
 * const MyComponent = () => {
 *   const tracker = useRenderTracker("MyComponent");
 *   
 *   return <div>Content</div>;
 * };
 * ```
 */
export function useRenderTracker(componentName: string) {
  const [renderCount, setRenderCount] = createSignal(0);
  const startTime = performance.now();

  onMount(() => {
    if (!performanceStore.enabled) return;

    const duration = performance.now() - startTime;
    const existing = performanceStore.metrics.get(componentName);

    if (existing) {
      existing.renderCount++;
      existing.lastRenderDuration = duration;
      existing.totalRenderTime += duration;
      existing.avgRenderDuration = existing.totalRenderTime / existing.renderCount;
      existing.lastRenderTimestamp = Date.now();
    } else {
      performanceStore.metrics.set(componentName, {
        name: componentName,
        renderCount: 1,
        lastRenderDuration: duration,
        avgRenderDuration: duration,
        totalRenderTime: duration,
        lastRenderTimestamp: Date.now(),
      });
    }

    setRenderCount(c => c + 1);

    // Log entry
    addPerformanceEntry({
      timestamp: Date.now(),
      type: "render",
      name: componentName,
      duration,
    });
  });

  return {
    renderCount,
    getMetrics: () => performanceStore.metrics.get(componentName),
  };
}

/**
 * Track effect execution time
 */
export function trackEffect<T>(name: string, fn: () => T): T {
  if (!performanceStore.enabled) return fn();

  const start = performance.now();
  const result = fn();
  const duration = performance.now() - start;

  addPerformanceEntry({
    timestamp: Date.now(),
    type: "effect",
    name,
    duration,
  });

  return result;
}

/**
 * Track memo computation time
 */
export function trackMemo<T>(name: string, fn: () => T): T {
  if (!performanceStore.enabled) return fn();

  const start = performance.now();
  const result = fn();
  const duration = performance.now() - start;

  addPerformanceEntry({
    timestamp: Date.now(),
    type: "memo",
    name,
    duration,
  });

  return result;
}

/**
 * Track action execution time
 */
export async function trackAction<T>(
  name: string, 
  fn: () => Promise<T>
): Promise<T> {
  if (!performanceStore.enabled) return fn();

  const start = performance.now();
  try {
    const result = await fn();
    const duration = performance.now() - start;

    addPerformanceEntry({
      timestamp: Date.now(),
      type: "action",
      name,
      duration,
    });

    return result;
  } catch (error) {
    const duration = performance.now() - start;
    
    addPerformanceEntry({
      timestamp: Date.now(),
      type: "action",
      name,
      duration,
      metadata: { error: String(error) },
    });

    throw error;
  }
}

// ============================================================================
// Memory Monitoring
// ============================================================================

/**
 * Get current memory metrics
 * Returns null if memory API is not available
 */
export function getMemoryMetrics(): MemoryMetrics | null {
  // @ts-expect-error - memory API not in types
  const memory = performance.memory;
  if (!memory) return null;

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
    return `${(bytes / 1073741824).toFixed(2)} GB`;
  };

  return {
    usedJSHeapSize: memory.usedJSHeapSize,
    totalJSHeapSize: memory.totalJSHeapSize,
    jsHeapSizeLimit: memory.jsHeapSizeLimit,
    usedHeapFormatted: formatBytes(memory.usedJSHeapSize),
    totalHeapFormatted: formatBytes(memory.totalJSHeapSize),
  };
}

/**
 * Hook for monitoring memory usage
 * Updates at specified interval (default: 5 seconds)
 */
export function useMemoryMonitor(intervalMs: number = 5000) {
  const [metrics, setMetrics] = createSignal<MemoryMetrics | null>(null);

  onMount(() => {
    // Initial measurement
    setMetrics(getMemoryMetrics());

    // Set up interval
    const interval = setInterval(() => {
      setMetrics(getMemoryMetrics());
    }, intervalMs);

    onCleanup(() => clearInterval(interval));
  });

  return metrics;
}

// ============================================================================
// Performance Entries
// ============================================================================

function addPerformanceEntry(entry: PerformanceEntry): void {
  performanceStore.entries.push(entry);
  
  // Trim if over max
  if (performanceStore.entries.length > performanceStore.maxEntries) {
    performanceStore.entries = performanceStore.entries.slice(-performanceStore.maxEntries);
  }
}

/**
 * Get all performance entries
 */
export function getPerformanceEntries(): PerformanceEntry[] {
  return [...performanceStore.entries];
}

/**
 * Get performance entries by type
 */
export function getPerformanceEntriesByType(type: PerformanceEntry["type"]): PerformanceEntry[] {
  return performanceStore.entries.filter(e => e.type === type);
}

/**
 * Get render metrics for all tracked components
 */
export function getAllRenderMetrics(): RenderMetrics[] {
  return Array.from(performanceStore.metrics.values());
}

/**
 * Get render metrics for a specific component
 */
export function getRenderMetrics(componentName: string): RenderMetrics | undefined {
  return performanceStore.metrics.get(componentName);
}

/**
 * Clear all performance data
 */
export function clearPerformanceData(): void {
  performanceStore.metrics.clear();
  performanceStore.entries = [];
}

// ============================================================================
// Performance Analysis
// ============================================================================

export interface PerformanceSummary {
  totalRenders: number;
  totalEffects: number;
  totalMemos: number;
  totalActions: number;
  avgRenderDuration: number;
  avgEffectDuration: number;
  slowestComponent: string | null;
  mostRenderedComponent: string | null;
  memoryTrend: "stable" | "increasing" | "decreasing" | "unknown";
}

/**
 * Get a summary of performance metrics
 */
export function getPerformanceSummary(): PerformanceSummary {
  const entries = performanceStore.entries;
  const metrics = Array.from(performanceStore.metrics.values());

  const renders = entries.filter(e => e.type === "render");
  const effects = entries.filter(e => e.type === "effect");
  const memos = entries.filter(e => e.type === "memo");
  const actions = entries.filter(e => e.type === "action");

  const avgRenderDuration = renders.length > 0
    ? renders.reduce((sum, e) => sum + e.duration, 0) / renders.length
    : 0;

  const avgEffectDuration = effects.length > 0
    ? effects.reduce((sum, e) => sum + e.duration, 0) / effects.length
    : 0;

  const slowest = metrics.length > 0
    ? metrics.reduce((max, m) => m.avgRenderDuration > max.avgRenderDuration ? m : max)
    : null;

  const mostRendered = metrics.length > 0
    ? metrics.reduce((max, m) => m.renderCount > max.renderCount ? m : max)
    : null;

  return {
    totalRenders: renders.length,
    totalEffects: effects.length,
    totalMemos: memos.length,
    totalActions: actions.length,
    avgRenderDuration,
    avgEffectDuration,
    slowestComponent: slowest?.name ?? null,
    mostRenderedComponent: mostRendered?.name ?? null,
    memoryTrend: "unknown",
  };
}

// ============================================================================
// FPS Monitor
// ============================================================================

/**
 * Hook for monitoring frames per second
 */
export function useFPSMonitor() {
  const [fps, setFps] = createSignal(60);
  const [isLowFPS, setIsLowFPS] = createSignal(false);

  onMount(() => {
    let frameCount = 0;
    let lastTime = performance.now();
    let animationId: number;

    const measureFPS = () => {
      frameCount++;
      const currentTime = performance.now();
      
      if (currentTime - lastTime >= 1000) {
        const currentFps = Math.round(frameCount * 1000 / (currentTime - lastTime));
        setFps(currentFps);
        setIsLowFPS(currentFps < 30);
        frameCount = 0;
        lastTime = currentTime;
      }
      
      animationId = requestAnimationFrame(measureFPS);
    };

    animationId = requestAnimationFrame(measureFPS);

    onCleanup(() => cancelAnimationFrame(animationId));
  });

  return { fps, isLowFPS };
}

// ============================================================================
// Debug Panel Component Helper
// ============================================================================

/**
 * Format duration for display
 */
export function formatDuration(ms: number): string {
  if (ms < 1) return `${(ms * 1000).toFixed(0)}μs`;
  if (ms < 1000) return `${ms.toFixed(2)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

/**
 * Get performance grade based on metrics
 */
export function getPerformanceGrade(avgRenderMs: number): "excellent" | "good" | "fair" | "poor" {
  if (avgRenderMs < 5) return "excellent";
  if (avgRenderMs < 16) return "good"; // 60fps threshold
  if (avgRenderMs < 33) return "fair"; // 30fps threshold
  return "poor";
}
