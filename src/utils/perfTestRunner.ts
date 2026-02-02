// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Performance Test Runner
 * 
 * Run from browser dev console:
 *   await window.__runPerfTests()
 * 
 * Or import and run:
 *   import { runPerformanceTests } from './utils/perfTestRunner';
 *   await runPerformanceTests();
 */

import { invoke } from "@tauri-apps/api/core";

interface TimingResult {
  operation: string;
  durationMs: number;
  success: boolean;
  details?: string;
}

interface TestReport {
  timestamp: string;
  totalDuration: number;
  results: TimingResult[];
  summary: {
    treeExpansion: { avg: number; min: number; max: number; count: number };
    viewerLoad: { avg: number; min: number; max: number; count: number };
    hashInit: { avg: number; min: number; max: number; count: number };
  };
}

/**
 * Measure a single operation
 */
async function measureOp<T>(
  name: string,
  fn: () => Promise<T>
): Promise<{ result: T | null; timing: TimingResult }> {
  const start = performance.now();
  let success = true;
  let result: T | null = null;
  let details: string | undefined;
  
  try {
    result = await fn();
  } catch (err) {
    success = false;
    details = String(err);
  }
  
  return {
    result,
    timing: {
      operation: name,
      durationMs: performance.now() - start,
      success,
      details,
    },
  };
}

/**
 * Calculate statistics from durations
 */
function calcStats(durations: number[]) {
  if (durations.length === 0) return { avg: 0, min: 0, max: 0, count: 0 };
  return {
    avg: durations.reduce((a, b) => a + b, 0) / durations.length,
    min: Math.min(...durations),
    max: Math.max(...durations),
    count: durations.length,
  };
}

/**
 * Run performance tests against the current evidence directory
 */
export async function runPerformanceTests(
  containerPath?: string
): Promise<TestReport> {
  const startTime = performance.now();
  const results: TimingResult[] = [];
  
  console.log('\n🔬 CORE-FFX Performance Test Runner');
  console.log('====================================\n');
  
  // Try to get a container path if not provided
  if (!containerPath) {
    console.log('⚠️  No container path provided. Provide a path to test real operations.');
    console.log('   Usage: await window.__runPerfTests("/path/to/container.ad1")\n');
    
    // Run synthetic tests only
    console.log('Running synthetic timing tests...\n');
    
    // Test IPC overhead
    for (let i = 0; i < 5; i++) {
      const { timing } = await measureOp('ipc-ping', async () => {
        // This tests raw IPC overhead without any real work
        return await invoke<boolean>('plugin:dialog|exists', { path: '/' });
      });
      results.push(timing);
      console.log(`  IPC Ping ${i + 1}: ${timing.durationMs.toFixed(2)}ms`);
    }
  } else {
    console.log(`Testing container: ${containerPath}\n`);
    
    // =====================================================
    // Test 1: Container Info Loading
    // =====================================================
    console.log('📦 Testing Container Info Loading...');
    
    for (let i = 0; i < 3; i++) {
      const { timing } = await measureOp('container-info', async () => {
        return await invoke('logical_info', { path: containerPath });
      });
      results.push(timing);
      console.log(`  Load ${i + 1}: ${timing.durationMs.toFixed(2)}ms ${timing.success ? '✓' : '✗'}`);
    }
    
    // =====================================================
    // Test 2: Tree Expansion (Root Children) - V2 API (fast, cached)
    // =====================================================
    console.log('\n🌳 Testing Tree Root Expansion (V2 API)...');
    
    for (let i = 0; i < 5; i++) {
      const { timing } = await measureOp('tree-root-v2', async () => {
        return await invoke('container_get_root_children_v2', { 
          containerPath 
        });
      });
      results.push(timing);
      console.log(`  Expand ${i + 1}: ${timing.durationMs.toFixed(2)}ms ${timing.success ? '✓' : '✗'}`);
    }
    
    // =====================================================
    // Test 3: Tree Node Expansion (by address - V2 API)
    // =====================================================
    console.log('\n🌿 Testing Tree Node Expansion (by address - V2 API)...');
    
    // First get root to find a directory to expand
    const { result: rootChildren } = await measureOp('get-root-v2', async () => {
      return await invoke<Array<{ path: string; is_dir: boolean; first_child_addr?: number }>>('container_get_root_children_v2', { 
        containerPath 
      });
    });
    
    if (rootChildren && rootChildren.length > 0) {
      const firstDir = rootChildren.find(c => c.is_dir && c.first_child_addr);
      if (firstDir && firstDir.first_child_addr) {
        for (let i = 0; i < 5; i++) {
          const { timing } = await measureOp('tree-expand-addr', async () => {
            return await invoke('container_get_children_at_addr_v2', { 
              containerPath, 
              addr: firstDir.first_child_addr,
              parentPath: firstDir.path
            });
          });
          results.push(timing);
          console.log(`  Expand addr ${i + 1}: ${timing.durationMs.toFixed(2)}ms ${timing.success ? '✓' : '✗'}`);
        }
      }
    }
    
    // =====================================================
    // Test 4: Hex Viewer Chunk Loading
    // =====================================================
    console.log('\n📄 Testing Hex Viewer Chunk Loading...');
    
    for (let i = 0; i < 10; i++) {
      const offset = i * 256;
      const { timing } = await measureOp('hex-chunk', async () => {
        return await invoke('read_file_chunk', { 
          path: containerPath, 
          offset,
          size: 256
        });
      });
      results.push(timing);
      console.log(`  Chunk ${i + 1} (offset ${offset}): ${timing.durationMs.toFixed(2)}ms ${timing.success ? '✓' : '✗'}`);
    }
    
    // =====================================================
    // Test 5: Hash Initiation (setup only, don't actually hash)
    // =====================================================
    console.log('\n#️⃣ Testing Hash Command Availability...');
    
    // We can't actually start hashing without risking a long operation,
    // but we can test the command is responsive
    const { timing: hashCheckTiming } = await measureOp('hash-check', async () => {
      // Test by checking if the hash command exists (will error but fast)
      try {
        await invoke('batch_hash', { 
          paths: [], 
          algorithm: 'SHA-256' 
        });
      } catch {
        // Expected - empty paths
      }
      return true;
    });
    results.push(hashCheckTiming);
    console.log(`  Hash command check: ${hashCheckTiming.durationMs.toFixed(2)}ms`);
  }
  
  // Calculate summary statistics
  const treeExpansionResults = results.filter(r => 
    r.operation.includes('tree') && r.success
  ).map(r => r.durationMs);
  
  const viewerLoadResults = results.filter(r => 
    (r.operation.includes('hex') || r.operation.includes('container-info')) && r.success
  ).map(r => r.durationMs);
  
  const hashResults = results.filter(r => 
    r.operation.includes('hash') && r.success
  ).map(r => r.durationMs);
  
  const report: TestReport = {
    timestamp: new Date().toISOString(),
    totalDuration: performance.now() - startTime,
    results,
    summary: {
      treeExpansion: calcStats(treeExpansionResults),
      viewerLoad: calcStats(viewerLoadResults),
      hashInit: calcStats(hashResults),
    },
  };
  
  // Print summary
  console.log('\n====================================');
  console.log('📊 SUMMARY');
  console.log('====================================');
  console.log(`Total test time: ${report.totalDuration.toFixed(2)}ms\n`);
  
  if (report.summary.treeExpansion.count > 0) {
    console.log('Tree Expansion:');
    console.log(`  Avg: ${report.summary.treeExpansion.avg.toFixed(2)}ms`);
    console.log(`  Min: ${report.summary.treeExpansion.min.toFixed(2)}ms`);
    console.log(`  Max: ${report.summary.treeExpansion.max.toFixed(2)}ms`);
  }
  
  if (report.summary.viewerLoad.count > 0) {
    console.log('\nViewer/Info Loading:');
    console.log(`  Avg: ${report.summary.viewerLoad.avg.toFixed(2)}ms`);
    console.log(`  Min: ${report.summary.viewerLoad.min.toFixed(2)}ms`);
    console.log(`  Max: ${report.summary.viewerLoad.max.toFixed(2)}ms`);
  }
  
  if (report.summary.hashInit.count > 0) {
    console.log('\nHash Check:');
    console.log(`  Avg: ${report.summary.hashInit.avg.toFixed(2)}ms`);
  }
  
  // Performance grades
  console.log('\n🎯 Performance Grades:');
  const gradeOp = (avgMs: number) => {
    if (avgMs < 20) return '🟢 Excellent (<20ms)';
    if (avgMs < 50) return '🟢 Good (<50ms)';
    if (avgMs < 100) return '🟡 Acceptable (<100ms)';
    if (avgMs < 300) return '🟠 Slow (<300ms)';
    return '🔴 Needs Optimization (>300ms)';
  };
  
  if (report.summary.treeExpansion.count > 0) {
    console.log(`  Tree Expansion: ${gradeOp(report.summary.treeExpansion.avg)}`);
  }
  if (report.summary.viewerLoad.count > 0) {
    console.log(`  Viewer Loading: ${gradeOp(report.summary.viewerLoad.avg)}`);
  }
  
  console.log('\n====================================\n');
  
  return report;
}

// =============================================================================
// Hash Performance Tests
// =============================================================================

interface HashTestResult {
  algorithm: string;
  fileSize: number;
  durationMs: number;
  throughputMBps: number;
  success: boolean;
  error?: string;
}

interface HashPerfReport {
  timestamp: string;
  totalDuration: number;
  results: HashTestResult[];
  summary: {
    [algorithm: string]: {
      avgThroughputMBps: number;
      minThroughputMBps: number;
      maxThroughputMBps: number;
      avgDurationMs: number;
      totalBytesHashed: number;
    };
  };
}

/**
 * Run hash performance tests
 * 
 * Tests different hash algorithms on files of various sizes
 * Measures throughput in MB/s
 */
export async function runHashPerformanceTests(
  testFilePath?: string
): Promise<HashPerfReport> {
  const startTime = performance.now();
  const results: HashTestResult[] = [];
  
  console.log('\n🔐 CORE-FFX Hash Performance Test');
  console.log('====================================\n');
  
  const algorithms = ['MD5', 'SHA-1', 'SHA-256', 'BLAKE3', 'XXH3'];
  
  if (!testFilePath) {
    console.log('⚠️  No test file provided.');
    console.log('   Usage: await window.__runHashTests("/path/to/file")');
    console.log('   For best results, use a file 10MB+ in size.\n');
    
    // Create a synthetic test using in-memory data
    console.log('Running synthetic tests with IPC overhead measurement...\n');
    
    for (const algo of algorithms) {
      const { timing } = await measureOp(`hash-init-${algo}`, async () => {
        try {
          await invoke('batch_hash', { 
            files: [], 
            algorithm: algo 
          });
        } catch {
          // Expected - empty files list
        }
        return true;
      });
      console.log(`  ${algo} command init: ${timing.durationMs.toFixed(2)}ms`);
    }
    
    return {
      timestamp: new Date().toISOString(),
      totalDuration: performance.now() - startTime,
      results: [],
      summary: {},
    };
  }
  
  // Get file info first
  console.log(`Testing file: ${testFilePath}\n`);
  
  let fileSize = 0;
  try {
    // Try mmap viewer file size API
    fileSize = await invoke<number>('mmap_hex_get_file_size', { path: testFilePath });
    console.log(`File size: ${formatBytes(fileSize)} (${fileSize} bytes)\n`);
  } catch {
    // Try to get size from hash result later
    console.log('File size: (will determine from hash operation)\n');
  }
  
  // Test each algorithm
  for (const algo of algorithms) {
    console.log(`\n#️⃣ Testing ${algo}...`);
    
    // Warm-up run (not counted)
    try {
      await invoke('batch_hash', {
        files: [{ path: testFilePath, containerType: 'file' }],
        algorithm: algo,
      });
    } catch (e) {
      console.log(`  ⚠️ Warm-up failed: ${e}`);
    }
    
    // Timed runs
    const runs = 3;
    for (let i = 0; i < runs; i++) {
      const start = performance.now();
      let success = true;
      let error: string | undefined;
      let backendThroughput: number | undefined;
      
      try {
        const hashResults = await invoke<Array<{
          path: string;
          algorithm: string;
          hash: string | null;
          error: string | null;
          durationMs: number | null;
          throughputMbs: number | null;
        }>>('batch_hash', {
          files: [{ path: testFilePath, containerType: 'file' }],
          algorithm: algo,
        });
        
        if (hashResults.length > 0 && hashResults[0].throughputMbs) {
          backendThroughput = hashResults[0].throughputMbs;
        }
      } catch (e) {
        success = false;
        error = String(e);
      }
      
      const durationMs = performance.now() - start;
      // Use backend-reported throughput if available, otherwise calculate from file size
      const throughputMBps = backendThroughput || (fileSize > 0 ? (fileSize / (1024 * 1024)) / (durationMs / 1000) : 0);
      
      results.push({
        algorithm: algo,
        fileSize,
        durationMs,
        throughputMBps,
        success,
        error,
      });
      
      if (success) {
        console.log(`  Run ${i + 1}: ${durationMs.toFixed(0)}ms (${throughputMBps.toFixed(1)} MB/s)`);
      } else {
        console.log(`  Run ${i + 1}: FAILED - ${error}`);
      }
    }
  }
  
  // Calculate summary statistics per algorithm
  const summary: HashPerfReport['summary'] = {};
  
  for (const algo of algorithms) {
    const algoResults = results.filter(r => r.algorithm === algo && r.success);
    if (algoResults.length === 0) continue;
    
    const throughputs = algoResults.map(r => r.throughputMBps);
    const durations = algoResults.map(r => r.durationMs);
    
    summary[algo] = {
      avgThroughputMBps: throughputs.reduce((a, b) => a + b, 0) / throughputs.length,
      minThroughputMBps: Math.min(...throughputs),
      maxThroughputMBps: Math.max(...throughputs),
      avgDurationMs: durations.reduce((a, b) => a + b, 0) / durations.length,
      totalBytesHashed: algoResults.reduce((a, r) => a + r.fileSize, 0),
    };
  }
  
  // Print summary
  console.log('\n====================================');
  console.log('📊 HASH PERFORMANCE SUMMARY');
  console.log('====================================\n');
  
  console.log('| Algorithm | Avg Speed | Min | Max | Avg Time |');
  console.log('|-----------|-----------|-----|-----|----------|');
  
  for (const algo of algorithms) {
    const s = summary[algo];
    if (!s) {
      console.log(`| ${algo.padEnd(9)} | FAILED | - | - | - |`);
      continue;
    }
    console.log(
      `| ${algo.padEnd(9)} | ${s.avgThroughputMBps.toFixed(1).padStart(6)} MB/s | ` +
      `${s.minThroughputMBps.toFixed(1).padStart(3)} | ${s.maxThroughputMBps.toFixed(1).padStart(3)} | ` +
      `${s.avgDurationMs.toFixed(0).padStart(5)}ms |`
    );
  }
  
  // Performance grades for hash algorithms
  console.log('\n🎯 Hash Algorithm Grades (for this file size):');
  const gradeHash = (mbps: number) => {
    if (mbps > 1000) return '🟢 Excellent (>1 GB/s)';
    if (mbps > 500) return '🟢 Good (>500 MB/s)';
    if (mbps > 200) return '🟡 Acceptable (>200 MB/s)';
    if (mbps > 50) return '🟠 Slow (>50 MB/s)';
    return '🔴 Very Slow (<50 MB/s)';
  };
  
  for (const algo of algorithms) {
    const s = summary[algo];
    if (!s) continue;
    console.log(`  ${algo}: ${gradeHash(s.avgThroughputMBps)}`);
  }
  
  // Recommendations
  console.log('\n💡 Recommendations:');
  const sortedAlgos = algorithms
    .filter(a => summary[a])
    .sort((a, b) => (summary[b]?.avgThroughputMBps || 0) - (summary[a]?.avgThroughputMBps || 0));
  
  if (sortedAlgos.length > 0) {
    console.log(`  Fastest: ${sortedAlgos[0]} (${summary[sortedAlgos[0]]?.avgThroughputMBps.toFixed(1)} MB/s)`);
    if (sortedAlgos.includes('SHA-256')) {
      console.log(`  Best for forensics: SHA-256 (${summary['SHA-256']?.avgThroughputMBps.toFixed(1)} MB/s)`);
    }
    if (sortedAlgos.includes('BLAKE3')) {
      console.log(`  Best modern: BLAKE3 (${summary['BLAKE3']?.avgThroughputMBps.toFixed(1)} MB/s)`);
    }
  }
  
  console.log('\n====================================\n');
  
  const report: HashPerfReport = {
    timestamp: new Date().toISOString(),
    totalDuration: performance.now() - startTime,
    results,
    summary,
  };
  
  return report;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// Expose to window for dev tools usage
if (typeof window !== 'undefined') {
  (window as unknown as { 
    __runPerfTests: typeof runPerformanceTests;
    __runHashTests: typeof runHashPerformanceTests;
  }).__runPerfTests = runPerformanceTests;
  (window as unknown as { __runHashTests: typeof runHashPerformanceTests }).__runHashTests = runHashPerformanceTests;
  console.log('[CORE-FFX] Performance tests available:');
  console.log('  await window.__runPerfTests("/path/to/container.ad1")');
  console.log('  await window.__runHashTests("/path/to/file")');
}
