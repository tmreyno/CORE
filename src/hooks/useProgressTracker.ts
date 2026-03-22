// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Smoothed progress tracker with exponential moving average (EMA) for stable
 * speed and ETA calculations during long-running acquisition operations.
 *
 * The raw ETA from simple linear extrapolation can fluctuate wildly — this
 * tracker maintains a rolling speed estimate that converges quickly but
 * resists momentary spikes/dips.
 */

import { createSignal, createMemo, onCleanup } from "solid-js";
import { formatBytes } from "../utils";

export interface ProgressSnapshot {
  bytesProcessed: number;
  bytesTotal: number;
  percent: number;
}

export interface SmoothedStats {
  /** Smoothed bytes per second */
  speedBps: number | null;
  /** Smoothed ETA in milliseconds */
  etaMs: number | null;
  /** Elapsed time in milliseconds */
  elapsedMs: number;
  /** Formatted speed string e.g. "125.3 MB/s" */
  speedFormatted: string;
  /** Formatted ETA string e.g. "2m 15s" */
  etaFormatted: string;
  /** Formatted elapsed string e.g. "1m 30s" */
  elapsedFormatted: string;
}

/** EMA smoothing factor — higher = more responsive, lower = smoother */
const ALPHA = 0.3;
/** Minimum elapsed ms before showing speed/ETA */
const MIN_WARMUP_MS = 2000;
/** Minimum interval between speed samples (ms) to avoid noise */
const MIN_SAMPLE_INTERVAL_MS = 500;

/**
 * Creates a reactive progress tracker with smoothed speed and ETA.
 *
 * Usage:
 * ```tsx
 * const tracker = createProgressTracker();
 * // In your progress event handler:
 * tracker.update({ bytesProcessed: 5000, bytesTotal: 10000, percent: 50 });
 * // In your JSX:
 * <span>{tracker.stats().speedFormatted}</span>
 * <span>{tracker.stats().etaFormatted}</span>
 * ```
 */
export function createProgressTracker() {
  const [startTime] = createSignal(Date.now());
  const [lastSampleTime, setLastSampleTime] = createSignal(0);
  const [lastSampleBytes, setLastSampleBytes] = createSignal(0);
  const [smoothedSpeed, setSmoothedSpeed] = createSignal<number | null>(null);
  const [currentSnapshot, setCurrentSnapshot] = createSignal<ProgressSnapshot | null>(null);

  // Tick counter to force elapsed time updates
  const [tick, setTick] = createSignal(0);
  const timer = setInterval(() => setTick((t) => t + 1), 1000);
  onCleanup(() => clearInterval(timer));

  function update(snapshot: ProgressSnapshot) {
    const now = Date.now();
    setCurrentSnapshot(snapshot);

    const prevTime = lastSampleTime();
    const prevBytes = lastSampleBytes();
    const interval = now - prevTime;

    // Only sample speed when enough time has passed to avoid noise
    if (prevTime > 0 && interval >= MIN_SAMPLE_INTERVAL_MS && snapshot.bytesProcessed > prevBytes) {
      const instantSpeed = ((snapshot.bytesProcessed - prevBytes) / interval) * 1000;
      const prev = smoothedSpeed();
      if (prev === null) {
        setSmoothedSpeed(instantSpeed);
      } else {
        // Exponential moving average
        setSmoothedSpeed(ALPHA * instantSpeed + (1 - ALPHA) * prev);
      }
      setLastSampleTime(now);
      setLastSampleBytes(snapshot.bytesProcessed);
    } else if (prevTime === 0) {
      // First sample — just record the baseline
      setLastSampleTime(now);
      setLastSampleBytes(snapshot.bytesProcessed);
    }
  }

  function reset() {
    setLastSampleTime(0);
    setLastSampleBytes(0);
    setSmoothedSpeed(null);
    setCurrentSnapshot(null);
  }

  const stats = createMemo((): SmoothedStats => {
    // Touch tick to trigger re-computation every second
    tick();
    const snap = currentSnapshot();
    const start = startTime();
    const elapsed = Date.now() - start;
    const speed = smoothedSpeed();

    let speedBps: number | null = null;
    let etaMs: number | null = null;

    if (snap && elapsed >= MIN_WARMUP_MS && speed !== null && speed > 0) {
      speedBps = speed;
      if (snap.bytesTotal > 0 && snap.bytesProcessed < snap.bytesTotal) {
        const remaining = snap.bytesTotal - snap.bytesProcessed;
        etaMs = (remaining / speed) * 1000;
      }
    }

    return {
      speedBps,
      etaMs,
      elapsedMs: elapsed,
      speedFormatted: speedBps !== null ? `${formatBytes(speedBps)}/s` : "",
      etaFormatted: etaMs !== null ? formatEtaCompact(etaMs) : "",
      elapsedFormatted: formatElapsed(elapsed),
    };
  });

  return { update, reset, stats };
}

/** Format ETA with seconds precision for short durations, minutes for longer */
function formatEtaCompact(ms: number): string {
  if (ms <= 0) return "< 1s";
  const totalSec = Math.ceil(ms / 1000);
  if (totalSec < 60) return `${totalSec}s`;
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  if (min < 60) return sec > 0 ? `${min}m ${sec}s` : `${min}m`;
  const hr = Math.floor(min / 60);
  const remMin = min % 60;
  return remMin > 0 ? `${hr}h ${remMin}m` : `${hr}h`;
}

/** Format elapsed time */
function formatElapsed(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  if (totalSec < 60) return `${totalSec}s`;
  const min = Math.floor(totalSec / 60);
  const sec = totalSec % 60;
  if (min < 60) return `${min}m ${sec}s`;
  const hr = Math.floor(min / 60);
  const remMin = min % 60;
  return `${hr}h ${remMin}m`;
}
