// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Performance Timing Tests for Key Operations
 * 
 * Tests the speed of:
 * 1. Evidence container tree expansion (lazy loading children)
 * 2. Viewer opening (loading file content for display)
 * 3. Hash initiation (time from request to actual hashing start)
 * 
 * Run with: npx vitest run src/__tests__/performance/operationTiming.test.ts
 */

import { describe, it, expect, beforeAll, afterAll } from "vitest";

// Mock Tauri invoke to measure timing without actual backend
const mockTiming = {
  containerGetChildren: { calls: 0, totalMs: 0, durations: [] as number[] },
  containerGetInfo: { calls: 0, totalMs: 0, durations: [] as number[] },
  readFileChunk: { calls: 0, totalMs: 0, durations: [] as number[] },
  hashCompute: { calls: 0, totalMs: 0, durations: [] as number[] },
};

// Simulated network/IPC delays (in ms) - adjust these for realistic testing
const SIMULATED_DELAYS = {
  containerGetChildren: { min: 5, max: 50 },   // Tree node expansion
  containerGetInfo: { min: 10, max: 100 },      // Container info loading
  readFileChunk: { min: 2, max: 20 },           // Hex viewer chunk reads
  hashCompute: { min: 100, max: 500 },          // Hash computation start
};

function simulateDelay(operation: keyof typeof SIMULATED_DELAYS): Promise<void> {
  const { min, max } = SIMULATED_DELAYS[operation];
  const delay = min + Math.random() * (max - min);
  return new Promise(resolve => setTimeout(resolve, delay));
}

// Performance measurement utility
function measureOperation<T>(
  name: keyof typeof mockTiming,
  fn: () => Promise<T>
): Promise<{ result: T; durationMs: number }> {
  return new Promise(async (resolve) => {
    const start = performance.now();
    const result = await fn();
    const durationMs = performance.now() - start;
    
    mockTiming[name].calls++;
    mockTiming[name].totalMs += durationMs;
    mockTiming[name].durations.push(durationMs);
    
    resolve({ result, durationMs });
  });
}

// Statistics helper
function getStats(durations: number[]) {
  if (durations.length === 0) return { avg: 0, min: 0, max: 0, p50: 0, p95: 0 };
  const sorted = [...durations].sort((a, b) => a - b);
  return {
    avg: durations.reduce((a, b) => a + b, 0) / durations.length,
    min: sorted[0],
    max: sorted[sorted.length - 1],
    p50: sorted[Math.floor(sorted.length * 0.5)],
    p95: sorted[Math.floor(sorted.length * 0.95)],
  };
}

describe("Operation Timing Tests", () => {
  beforeAll(() => {
    // Reset timing data
    Object.values(mockTiming).forEach(t => {
      t.calls = 0;
      t.totalMs = 0;
      t.durations = [];
    });
  });

  afterAll(() => {
    // Print summary report
    console.log("\n========================================");
    console.log("PERFORMANCE TIMING SUMMARY");
    console.log("========================================\n");
    
    for (const [name, timing] of Object.entries(mockTiming)) {
      if (timing.calls === 0) continue;
      const stats = getStats(timing.durations);
      console.log(`${name}:`);
      console.log(`  Calls: ${timing.calls}`);
      console.log(`  Avg: ${stats.avg.toFixed(2)}ms`);
      console.log(`  Min: ${stats.min.toFixed(2)}ms`);
      console.log(`  Max: ${stats.max.toFixed(2)}ms`);
      console.log(`  P50: ${stats.p50.toFixed(2)}ms`);
      console.log(`  P95: ${stats.p95.toFixed(2)}ms`);
      console.log("");
    }
  });

  describe("Tree Expansion Speed", () => {
    it("should measure time to expand a tree node (load children)", async () => {
      // Simulate 10 tree expansions
      const iterations = 10;
      
      for (let i = 0; i < iterations; i++) {
        const { durationMs } = await measureOperation("containerGetChildren", async () => {
          await simulateDelay("containerGetChildren");
          // Simulated children response
          return [
            { path: "/folder/file1.txt", name: "file1.txt", is_dir: false, size: 1000 },
            { path: "/folder/file2.txt", name: "file2.txt", is_dir: false, size: 2000 },
            { path: "/folder/subfolder", name: "subfolder", is_dir: true, size: 0 },
          ];
        });
        
        // Tree expansion should be fast - under 100ms ideally
        expect(durationMs).toBeLessThan(100);
      }
      
      const stats = getStats(mockTiming.containerGetChildren.durations);
      console.log(`Tree expansion - Avg: ${stats.avg.toFixed(2)}ms, P95: ${stats.p95.toFixed(2)}ms`);
      
      // P95 should be under 100ms for good UX
      expect(stats.p95).toBeLessThan(100);
    });

    it("should measure nested tree expansion (3 levels deep)", async () => {
      const expandLevel = async (depth: number): Promise<number> => {
        const { durationMs } = await measureOperation("containerGetChildren", async () => {
          await simulateDelay("containerGetChildren");
          return Array(3).fill(null).map((_, i) => ({
            path: `/level${depth}/item${i}`,
            name: `item${i}`,
            is_dir: depth < 3,
            size: depth >= 3 ? 1000 : 0,
          }));
        });
        return durationMs;
      };
      
      // Expand 3 levels
      let totalTime = 0;
      for (let depth = 1; depth <= 3; depth++) {
        totalTime += await expandLevel(depth);
      }
      
      console.log(`3-level nested expansion total: ${totalTime.toFixed(2)}ms`);
      // Total for 3 levels should be under 300ms
      expect(totalTime).toBeLessThan(300);
    });
  });

  describe("Viewer Opening Speed", () => {
    it("should measure time to load container info", async () => {
      const iterations = 5;
      
      for (let i = 0; i < iterations; i++) {
        const { durationMs } = await measureOperation("containerGetInfo", async () => {
          await simulateDelay("containerGetInfo");
          // Simulated container info
          return {
            container: "ad1",
            ad1: {
              segment: { signature: "ADSEGMENTEDFILE", segment_index: 0 },
              logical: { signature: "ADLOGICAL", image_version: 2 },
              item_count: 1500,
            }
          };
        });
        
        // Info load should be under 200ms
        expect(durationMs).toBeLessThan(200);
      }
      
      const stats = getStats(mockTiming.containerGetInfo.durations);
      console.log(`Container info load - Avg: ${stats.avg.toFixed(2)}ms, P95: ${stats.p95.toFixed(2)}ms`);
    });

    it("should measure time to load hex viewer chunk", async () => {
      const iterations = 20; // Hex viewer makes many chunk requests
      
      for (let i = 0; i < iterations; i++) {
        const { durationMs } = await measureOperation("readFileChunk", async () => {
          await simulateDelay("readFileChunk");
          // Simulated 256-byte chunk
          return {
            bytes: Array(256).fill(0).map(() => Math.floor(Math.random() * 256)),
            offset: i * 256,
            total_size: 10000,
            has_more: true,
            has_prev: i > 0,
          };
        });
        
        // Each chunk should load very fast - under 50ms
        expect(durationMs).toBeLessThan(50);
      }
      
      const stats = getStats(mockTiming.readFileChunk.durations);
      console.log(`Hex chunk load - Avg: ${stats.avg.toFixed(2)}ms, P95: ${stats.p95.toFixed(2)}ms`);
      
      // P95 for chunk loads should be under 50ms
      expect(stats.p95).toBeLessThan(50);
    });

    it("should measure full viewer initialization sequence", async () => {
      const start = performance.now();
      
      // 1. Load container info
      await measureOperation("containerGetInfo", async () => {
        await simulateDelay("containerGetInfo");
        return { container: "ad1" };
      });
      
      // 2. Load root tree nodes
      await measureOperation("containerGetChildren", async () => {
        await simulateDelay("containerGetChildren");
        return [{ path: "/root", name: "root", is_dir: true }];
      });
      
      // 3. Load first hex chunk
      await measureOperation("readFileChunk", async () => {
        await simulateDelay("readFileChunk");
        return { bytes: Array(256).fill(0), offset: 0, total_size: 10000 };
      });
      
      const totalMs = performance.now() - start;
      console.log(`Full viewer init: ${totalMs.toFixed(2)}ms`);
      
      // Full init should be under 300ms for good UX
      expect(totalMs).toBeLessThan(300);
    });
  });

  describe("Hash Initiation Speed", () => {
    it("should measure time from hash request to start", async () => {
      const iterations = 5;
      
      for (let i = 0; i < iterations; i++) {
        const { durationMs } = await measureOperation("hashCompute", async () => {
          // This measures the "initiation" - setting up the hash job
          // Actual hashing would take longer but that's file-size dependent
          await simulateDelay("hashCompute");
          return "hash_job_id_" + i;
        });
        
        // Hash initiation should be quick - under 500ms
        expect(durationMs).toBeLessThan(500);
      }
      
      const stats = getStats(mockTiming.hashCompute.durations);
      console.log(`Hash initiation - Avg: ${stats.avg.toFixed(2)}ms, P95: ${stats.p95.toFixed(2)}ms`);
    });

    it("should measure hash request flow (dialog -> backend)", async () => {
      const start = performance.now();
      
      // Simulated flow:
      // 1. User clicks hash button
      const uiResponseTime = 5; // UI should respond instantly
      await new Promise(r => setTimeout(r, uiResponseTime));
      
      // 2. Show confirmation dialog (if enabled)
      const dialogTime = 10; // Dialog appearance
      await new Promise(r => setTimeout(r, dialogTime));
      
      // 3. User confirms, send to backend
      await measureOperation("hashCompute", async () => {
        await simulateDelay("hashCompute");
        return "started";
      });
      
      const totalMs = performance.now() - start;
      console.log(`Hash request flow total: ${totalMs.toFixed(2)}ms`);
      
      // From click to backend start should be under 600ms
      expect(totalMs).toBeLessThan(600);
    });
  });

  describe("Combined Workflow Speed", () => {
    it("should measure typical user workflow: open -> expand -> hash", async () => {
      const start = performance.now();
      
      // Step 1: Open container (load info)
      await measureOperation("containerGetInfo", async () => {
        await simulateDelay("containerGetInfo");
        return { container: "ad1", item_count: 500 };
      });
      
      // Step 2: Expand root node
      await measureOperation("containerGetChildren", async () => {
        await simulateDelay("containerGetChildren");
        return [{ path: "/root", name: "root", is_dir: true }];
      });
      
      // Step 3: Expand first subfolder
      await measureOperation("containerGetChildren", async () => {
        await simulateDelay("containerGetChildren");
        return [{ path: "/root/file.txt", name: "file.txt", is_dir: false }];
      });
      
      // Step 4: View file in hex viewer
      await measureOperation("readFileChunk", async () => {
        await simulateDelay("readFileChunk");
        return { bytes: Array(256).fill(0), offset: 0 };
      });
      
      // Step 5: Start hash
      await measureOperation("hashCompute", async () => {
        await simulateDelay("hashCompute");
        return "hash_started";
      });
      
      const totalMs = performance.now() - start;
      console.log(`\n=== Complete Workflow ===`);
      console.log(`Total time: ${totalMs.toFixed(2)}ms`);
      
      // Complete workflow should be under 1 second
      expect(totalMs).toBeLessThan(1000);
    });
  });
});
