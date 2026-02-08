#!/usr/bin/env node
// =============================================================================
// CORE-FFX Container Testing Script
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Automated container testing script
 * Tests all container types: E01, AD1, Archives (ZIP, 7z, TAR), UFED
 * 
 * Usage: node test_all_containers.js
 */

import fs from 'fs';
import path from 'path';

// Test configuration
const CASE_DIR = '/Users/terryreynolds/1827-1001 Case With Data ';
const EVIDENCE_DIR = path.join(CASE_DIR, '1.Evidence');

// ANSI color codes for terminal output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
};

const log = {
  info: (msg) => console.log(`${colors.blue}ℹ${colors.reset} ${msg}`),
  success: (msg) => console.log(`${colors.green}✓${colors.reset} ${msg}`),
  error: (msg) => console.log(`${colors.red}✗${colors.reset} ${msg}`),
  warning: (msg) => console.log(`${colors.yellow}⚠${colors.reset} ${msg}`),
  header: (msg) => console.log(`\n${colors.bright}${colors.cyan}${msg}${colors.reset}`),
  subheader: (msg) => console.log(`${colors.bright}${msg}${colors.reset}`),
};

// Test results tracking
const results = {
  total: 0,
  passed: 0,
  failed: 0,
  skipped: 0,
  tests: [],
};

function addTestResult(name, status, message = '', duration = 0) {
  results.total++;
  results[status]++;
  results.tests.push({ name, status, message, duration });
  
  const statusIcon = {
    passed: '✓',
    failed: '✗',
    skipped: '⏭',
  }[status];
  
  const statusColor = {
    passed: colors.green,
    failed: colors.red,
    skipped: colors.yellow,
  }[status];
  
  const durationStr = duration > 0 ? ` (${duration}ms)` : '';
  console.log(`  ${statusColor}${statusIcon}${colors.reset} ${name}${durationStr}`);
  if (message) console.log(`    ${colors.reset}${message}${colors.reset}`);
}

// Find all container files
function findContainerFiles() {
  log.header('📁 Scanning Evidence Directory');
  log.info(`Path: ${EVIDENCE_DIR}`);
  
  if (!fs.existsSync(EVIDENCE_DIR)) {
    log.error(`Evidence directory not found: ${EVIDENCE_DIR}`);
    process.exit(1);
  }
  
  const files = fs.readdirSync(EVIDENCE_DIR);
  const containers = {
    e01: [],
    ad1: [],
    zip: [],
    '7z': [],
    tar: [],
    dmg: [],
    ufd: [],
    iso: [],
  };
  
  files.forEach(file => {
    const fullPath = path.join(EVIDENCE_DIR, file);
    const stat = fs.statSync(fullPath);
    if (!stat.isFile()) return;
    
    const ext = path.extname(file).toLowerCase();
    const size = stat.size;
    const sizeStr = formatBytes(size);
    
    // Categorize by extension
    if (/\.(e01|ex01|ewf)$/i.test(file)) {
      containers.e01.push({ file, path: fullPath, size, sizeStr });
    } else if (/\.ad1$/i.test(file)) {
      containers.ad1.push({ file, path: fullPath, size, sizeStr });
    } else if (/\.zip$/i.test(file)) {
      containers.zip.push({ file, path: fullPath, size, sizeStr });
    } else if (/\.7z$/i.test(file)) {
      containers['7z'].push({ file, path: fullPath, size, sizeStr });
    } else if (/\.tar$/i.test(file)) {
      containers.tar.push({ file, path: fullPath, size, sizeStr });
    } else if (/\.dmg$/i.test(file)) {
      containers.dmg.push({ file, path: fullPath, size, sizeStr });
    } else if (/\.(ufd|ufdr|ufdx)$/i.test(file)) {
      containers.ufd.push({ file, path: fullPath, size, sizeStr });
    } else if (/\.iso$/i.test(file)) {
      containers.iso.push({ file, path: fullPath, size, sizeStr });
    }
  });
  
  return containers;
}

function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// Test file existence and readability
function testFileAccess(container) {
  const start = Date.now();
  try {
    const stat = fs.statSync(container.path);
    if (!stat.isFile()) {
      addTestResult(`File Access: ${container.file}`, 'failed', 'Not a file', Date.now() - start);
      return false;
    }
    
    // Try to open file for reading
    const fd = fs.openSync(container.path, 'r');
    fs.closeSync(fd);
    
    addTestResult(`File Access: ${container.file}`, 'passed', `Size: ${container.sizeStr}`, Date.now() - start);
    return true;
  } catch (err) {
    addTestResult(`File Access: ${container.file}`, 'failed', err.message, Date.now() - start);
    return false;
  }
}

// Test file header (magic bytes)
function testFileHeader(container, expectedMagic) {
  const start = Date.now();
  try {
    const fd = fs.openSync(container.path, 'r');
    const buffer = Buffer.alloc(16);
    fs.readSync(fd, buffer, 0, 16, 0);
    fs.closeSync(fd);
    
    const header = buffer.toString('hex').toUpperCase();
    let valid = false;
    let detectedType = 'Unknown';
    
    // Check various magic bytes
    if (buffer.slice(0, 3).toString() === 'EVF' || buffer.slice(0, 3).toString() === 'LVF') {
      valid = true;
      detectedType = 'E01/L01 (EWF)';
    } else if (buffer.slice(0, 4).toString('hex') === '504b0304') {
      valid = true;
      detectedType = 'ZIP';
    } else if (buffer.slice(0, 6).toString('hex') === '377abcaf271c') {
      valid = true;
      detectedType = '7z';
    } else if (buffer.slice(0, 4).toString('hex') === '75737461' || buffer.slice(257, 262).toString() === 'ustar') {
      valid = true;
      detectedType = 'TAR';
    } else if (buffer.slice(0, 4).toString('hex') === '7801730d' || buffer.slice(0, 2).toString('hex') === '7801') {
      valid = true;
      detectedType = 'DMG (Apple)';
    }
    
    if (valid) {
      addTestResult(`Header Check: ${container.file}`, 'passed', `Type: ${detectedType}`, Date.now() - start);
    } else {
      addTestResult(`Header Check: ${container.file}`, 'warning', `Unknown format (${header.slice(0, 16)})`, Date.now() - start);
    }
    return valid;
  } catch (err) {
    addTestResult(`Header Check: ${container.file}`, 'failed', err.message, Date.now() - start);
    return false;
  }
}

// Main test runner
async function runAllTests() {
  console.log('\n' + '='.repeat(70));
  console.log('CORE-FFX Container Testing Suite');
  console.log('='.repeat(70));
  
  const containers = findContainerFiles();
  
  // Display summary
  log.header('📊 Container Summary');
  console.log(`  E01 Files:    ${containers.e01.length}`);
  console.log(`  AD1 Files:    ${containers.ad1.length}`);
  console.log(`  ZIP Archives: ${containers.zip.length}`);
  console.log(`  7z Archives:  ${containers['7z'].length}`);
  console.log(`  TAR Archives: ${containers.tar.length}`);
  console.log(`  DMG Images:   ${containers.dmg.length}`);
  console.log(`  UFED Files:   ${containers.ufd.length}`);
  console.log(`  ISO Images:   ${containers.iso.length}`);
  
  const totalContainers = Object.values(containers).reduce((sum, arr) => sum + arr.length, 0);
  console.log(`  ${colors.bright}Total: ${totalContainers}${colors.reset}`);
  
  // Test each container type
  log.header('🧪 Running Tests');
  
  // E01 Tests
  if (containers.e01.length > 0) {
    log.subheader('\n▶ E01/EWF Containers');
    containers.e01.forEach(container => {
      testFileAccess(container);
      testFileHeader(container, 'EVF');
    });
  }
  
  // AD1 Tests
  if (containers.ad1.length > 0) {
    log.subheader('\n▶ AD1 Containers');
    containers.ad1.forEach(container => {
      testFileAccess(container);
      testFileHeader(container, 'AD1');
    });
  }
  
  // ZIP Tests
  if (containers.zip.length > 0) {
    log.subheader('\n▶ ZIP Archives');
    containers.zip.forEach(container => {
      testFileAccess(container);
      testFileHeader(container, 'PK');
    });
  }
  
  // 7z Tests
  if (containers['7z'].length > 0) {
    log.subheader('\n▶ 7z Archives');
    containers['7z'].forEach(container => {
      testFileAccess(container);
      testFileHeader(container, '7z');
    });
  }
  
  // TAR Tests
  if (containers.tar.length > 0) {
    log.subheader('\n▶ TAR Archives');
    containers.tar.forEach(container => {
      testFileAccess(container);
      testFileHeader(container, 'ustar');
    });
  }
  
  // DMG Tests
  if (containers.dmg.length > 0) {
    log.subheader('\n▶ DMG (Apple Disk Images)');
    containers.dmg.forEach(container => {
      testFileAccess(container);
      testFileHeader(container, 'DMG');
    });
  }
  
  // UFED Tests
  if (containers.ufd.length > 0) {
    log.subheader('\n▶ UFED Mobile Extractions');
    containers.ufd.forEach(container => {
      testFileAccess(container);
    });
  }
  
  // ISO Tests
  if (containers.iso.length > 0) {
    log.subheader('\n▶ ISO Images');
    containers.iso.forEach(container => {
      testFileAccess(container);
    });
  }
  
  // Print final summary
  printSummary();
}

function printSummary() {
  log.header('📈 Test Results Summary');
  
  console.log(`  Total Tests:   ${results.total}`);
  console.log(`  ${colors.green}Passed:${colors.reset}        ${results.passed}`);
  console.log(`  ${colors.red}Failed:${colors.reset}        ${results.failed}`);
  console.log(`  ${colors.yellow}Skipped:${colors.reset}       ${results.skipped}`);
  
  const passRate = results.total > 0 ? ((results.passed / results.total) * 100).toFixed(1) : 0;
  console.log(`  ${colors.bright}Pass Rate:${colors.reset}     ${passRate}%`);
  
  if (results.failed > 0) {
    log.header('❌ Failed Tests');
    results.tests.filter(t => t.status === 'failed').forEach(test => {
      console.log(`  • ${test.name}`);
      console.log(`    ${colors.red}${test.message}${colors.reset}`);
    });
  }
  
  console.log('\n' + '='.repeat(70));
  
  if (results.failed === 0 && results.total > 0) {
    log.success('All tests passed! ✨');
  } else if (results.failed > 0) {
    log.error(`${results.failed} test(s) failed. See details above.`);
  } else {
    log.warning('No tests were run.');
  }
  
  console.log('='.repeat(70) + '\n');
  
  // Exit with error code if tests failed
  process.exit(results.failed > 0 ? 1 : 0);
}

// Run tests
runAllTests().catch(err => {
  log.error(`Test suite failed: ${err.message}`);
  console.error(err);
  process.exit(1);
});
