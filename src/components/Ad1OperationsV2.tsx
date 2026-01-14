// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Ad1OperationsV2 - Complete AD1 operations UI component
 * 
 * Demonstrates all V2 API capabilities:
 * - Container information display
 * - Hash verification (MD5/SHA1)
 * - File extraction with metadata
 * - Tree navigation with lazy loading
 */

import { Component, createSignal, For, Show } from 'solid-js';
import { 
  useAd1ContainerV2, 
  useAd1InfoV2, 
  formatBytes, 
  formatHashResult,
  ItemVerifyResult,
  ExtractionResult,
} from '../hooks/useAd1ContainerV2';
import { open } from '@tauri-apps/plugin-dialog';

const Ad1OperationsV2: Component = () => {
  const [containerPath, setContainerPath] = createSignal<string>('');
  const [activeTab, setActiveTab] = createSignal<'info' | 'verify' | 'extract'>('info');
  
  // Info tab state
  const info = useAd1InfoV2(containerPath(), false);
  const [includeTreeInfo, setIncludeTreeInfo] = createSignal(false);
  
  // Verify tab state
  const [hashType, setHashType] = createSignal<'md5' | 'sha1'>('md5');
  const [verifyResults, setVerifyResults] = createSignal<ItemVerifyResult[]>([]);
  const [verifyProgress, setVerifyProgress] = createSignal('');
  
  // Extract tab state
  const [outputDir, setOutputDir] = createSignal('');
  const [applyMetadata, setApplyMetadata] = createSignal(true);
  const [verifyHashes, setVerifyHashes] = createSignal(false);
  const [extractResult, setExtractResult] = createSignal<ExtractionResult | null>(null);
  const [extractProgress, setExtractProgress] = createSignal('');

  const container = () => containerPath() ? useAd1ContainerV2(containerPath()) : null;

  /**
   * Open container file picker
   */
  async function selectContainer() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'AD1 Container',
          extensions: ['ad1', 'AD1'],
        }],
      });

      if (selected && typeof selected === 'string') {
        setContainerPath(selected);
        // Clear previous results
        setVerifyResults([]);
        setExtractResult(null);
      }
    } catch (e) {
      console.error('Error selecting container:', e);
    }
  }

  /**
   * Select output directory for extraction
   */
  async function selectOutputDir() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === 'string') {
        setOutputDir(selected);
      }
    } catch (e) {
      console.error('Error selecting output directory:', e);
    }
  }

  /**
   * Get container info with optional tree
   */
  async function fetchInfo() {
    const c = container();
    if (!c) return;

    try {
      const result = await c.getInfo(includeTreeInfo());
      console.log('Container Info:', result);
    } catch (e) {
      console.error('Error fetching info:', e);
    }
  }

  /**
   * Verify all files in container
   */
  async function verifyAll() {
    const c = container();
    if (!c) return;

    setVerifyProgress('Starting verification...');
    setVerifyResults([]);

    try {
      const results = await c.verifyAll(hashType());
      setVerifyResults(results);
      
      const total = results.length;
      const passed = results.filter(r => r.result === 'Ok').length;
      const failed = results.filter(r => r.result === 'Mismatch').length;
      const noHash = results.filter(r => r.result === 'NotFound').length;
      
      setVerifyProgress(
        `Verification complete: ${passed} passed, ${failed} failed, ${noHash} no hash (${total} total)`
      );
    } catch (e) {
      setVerifyProgress(`Error: ${String(e)}`);
      console.error('Verification error:', e);
    }
  }

  /**
   * Extract all files
   */
  async function extractAllFiles() {
    const c = container();
    if (!c || !outputDir()) return;

    setExtractProgress('Starting extraction...');
    setExtractResult(null);

    try {
      const result = await c.extractAll(
        outputDir(),
        applyMetadata(),
        verifyHashes()
      );
      
      setExtractResult(result);
      setExtractProgress(
        `Extraction complete: ${result.total_files} files, ` +
        `${result.total_dirs} directories, ${formatBytes(result.total_bytes)}`
      );
    } catch (e) {
      setExtractProgress(`Error: ${String(e)}`);
      console.error('Extraction error:', e);
    }
  }

  return (
    <div class="ad1-operations-v2 p-4">
      {/* Header */}
      <div class="mb-4">
        <h2 class="text-2xl font-bold mb-2">AD1 Container Operations V2</h2>
        <p class="text-gray-600 mb-4">
          Based on libad1 implementation with lazy loading, hash verification, and extraction
        </p>
        
        {/* Container Selection */}
        <div class="flex gap-2 mb-4">
          <button
            onClick={selectContainer}
            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            Select Container
          </button>
          <Show when={containerPath()}>
            <div class="flex-1 p-2 bg-gray-100 rounded truncate">
              {containerPath()}
            </div>
          </Show>
        </div>
      </div>

      {/* Tabs */}
      <Show when={containerPath()}>
        <div class="tabs mb-4">
          <div class="flex gap-2 border-b">
            <button
              onClick={() => setActiveTab('info')}
              class={`px-4 py-2 ${
                activeTab() === 'info' 
                  ? 'border-b-2 border-blue-500 text-blue-500' 
                  : 'text-gray-600'
              }`}
            >
              Info
            </button>
            <button
              onClick={() => setActiveTab('verify')}
              class={`px-4 py-2 ${
                activeTab() === 'verify' 
                  ? 'border-b-2 border-blue-500 text-blue-500' 
                  : 'text-gray-600'
              }`}
            >
              Verify
            </button>
            <button
              onClick={() => setActiveTab('extract')}
              class={`px-4 py-2 ${
                activeTab() === 'extract' 
                  ? 'border-b-2 border-blue-500 text-blue-500' 
                  : 'text-gray-600'
              }`}
            >
              Extract
            </button>
          </div>
        </div>

        {/* Info Tab */}
        <Show when={activeTab() === 'info'}>
          <div class="info-tab">
            <div class="controls mb-4">
              <label class="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={includeTreeInfo()}
                  onChange={(e) => setIncludeTreeInfo(e.currentTarget.checked)}
                />
                <span>Include tree structure</span>
              </label>
              <button
                onClick={fetchInfo}
                class="mt-2 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                Refresh Info
              </button>
            </div>

            <Show when={!info.loading && info()}>
              {(infoData) => (
                <div class="info-display bg-gray-50 p-4 rounded">
                  <h3 class="text-lg font-bold mb-2">Container Information</h3>
                  
                  {/* Segment Header */}
                  <div class="mb-4">
                    <h4 class="font-semibold">Segment Header:</h4>
                    <div class="ml-4 text-sm">
                      <div>Signature: {infoData().segment_header.signature}</div>
                      <div>Segment: {infoData().segment_header.segment_number} / {infoData().segment_header.segment_index}</div>
                      <div>Header Size: {formatBytes(infoData().segment_header.header_size)}</div>
                    </div>
                  </div>

                  {/* Logical Header */}
                  <div class="mb-4">
                    <h4 class="font-semibold">Logical Header:</h4>
                    <div class="ml-4 text-sm">
                      <div>Image Version: {infoData().logical_header.image_version}</div>
                      <div>Data Source: {infoData().logical_header.data_source_name}</div>
                      <div>Chunk Size: {formatBytes(infoData().logical_header.zlib_chunk_size)}</div>
                    </div>
                  </div>

                  {/* Statistics */}
                  <div class="mb-4">
                    <h4 class="font-semibold">Statistics:</h4>
                    <div class="ml-4 text-sm">
                      <div>Total Items: {infoData().total_items}</div>
                      <div>Files: {infoData().file_count}</div>
                      <div>Directories: {infoData().dir_count}</div>
                      <div>Total Size: {formatBytes(infoData().total_size)}</div>
                    </div>
                  </div>

                  {/* Tree (if included) */}
                  <Show when={infoData().tree && infoData().tree!.length > 0}>
                    <div>
                      <h4 class="font-semibold mb-2">Tree Structure:</h4>
                      <div class="ml-4 text-sm font-mono bg-white p-2 rounded max-h-96 overflow-auto">
                        <For each={infoData().tree}>
                          {(item) => (
                            <div style={{ 'padding-left': `${item.depth * 16}px` }}>
                              {item.is_dir ? '📁' : '📄'} {item.name} 
                              {!item.is_dir && ` (${formatBytes(item.size)})`}
                            </div>
                          )}
                        </For>
                      </div>
                    </div>
                  </Show>
                </div>
              )}
            </Show>

            <Show when={info.loading}>
              <div class="text-center py-8">Loading...</div>
            </Show>

            <Show when={info.error}>
              <div class="bg-red-100 text-red-700 p-4 rounded">
                Error: {info.error}
              </div>
            </Show>
          </div>
        </Show>

        {/* Verify Tab */}
        <Show when={activeTab() === 'verify'}>
          <div class="verify-tab">
            <div class="controls mb-4">
              <div class="mb-2">
                <label class="mr-4">
                  <input
                    type="radio"
                    name="hashType"
                    value="md5"
                    checked={hashType() === 'md5'}
                    onChange={() => setHashType('md5')}
                  />
                  <span class="ml-1">MD5</span>
                </label>
                <label>
                  <input
                    type="radio"
                    name="hashType"
                    value="sha1"
                    checked={hashType() === 'sha1'}
                    onChange={() => setHashType('sha1')}
                  />
                  <span class="ml-1">SHA1</span>
                </label>
              </div>
              
              <button
                onClick={verifyAll}
                class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
              >
                Verify All Files
              </button>
            </div>

            <Show when={verifyProgress()}>
              <div class="mb-4 p-3 bg-blue-50 rounded">
                {verifyProgress()}
              </div>
            </Show>

            <Show when={verifyResults().length > 0}>
              <div class="results bg-gray-50 p-4 rounded max-h-96 overflow-auto">
                <h4 class="font-semibold mb-2">Verification Results:</h4>
                <div class="text-sm">
                  <For each={verifyResults()}>
                    {(result) => (
                      <div class="mb-2 p-2 bg-white rounded">
                        <div class="flex justify-between items-start">
                          <div class="flex-1">
                            <div class="font-mono">{result.path}</div>
                            <div class="text-xs text-gray-600">
                              {!result.is_dir && `Size: ${formatBytes(result.size)}`}
                            </div>
                          </div>
                          <div class="ml-4">
                            {formatHashResult(result.result)}
                          </div>
                        </div>
                        <Show when={result.result === 'Mismatch'}>
                          <div class="mt-1 text-xs">
                            <div>Stored: {result.stored_hash}</div>
                            <div>Computed: {result.computed_hash}</div>
                          </div>
                        </Show>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        </Show>

        {/* Extract Tab */}
        <Show when={activeTab() === 'extract'}>
          <div class="extract-tab">
            <div class="controls mb-4">
              <div class="mb-2">
                <button
                  onClick={selectOutputDir}
                  class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                >
                  Select Output Directory
                </button>
                <Show when={outputDir()}>
                  <div class="mt-2 p-2 bg-gray-100 rounded text-sm truncate">
                    {outputDir()}
                  </div>
                </Show>
              </div>

              <div class="mb-2">
                <label class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={applyMetadata()}
                    onChange={(e) => setApplyMetadata(e.currentTarget.checked)}
                  />
                  <span>Apply metadata (timestamps, attributes)</span>
                </label>
              </div>

              <div class="mb-4">
                <label class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={verifyHashes()}
                    onChange={(e) => setVerifyHashes(e.currentTarget.checked)}
                  />
                  <span>Verify hashes during extraction</span>
                </label>
              </div>

              <button
                onClick={extractAllFiles}
                disabled={!outputDir()}
                class="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Extract All Files
              </button>
            </div>

            <Show when={extractProgress()}>
              <div class="mb-4 p-3 bg-blue-50 rounded">
                {extractProgress()}
              </div>
            </Show>

            <Show when={extractResult()}>
              {(result) => (
                <div class="results bg-gray-50 p-4 rounded">
                  <h4 class="font-semibold mb-2">Extraction Results:</h4>
                  <div class="text-sm">
                    <div class="mb-2">
                      <strong>Total Files:</strong> {result().total_files}
                    </div>
                    <div class="mb-2">
                      <strong>Total Directories:</strong> {result().total_dirs}
                    </div>
                    <div class="mb-2">
                      <strong>Total Bytes:</strong> {formatBytes(result().total_bytes)}
                    </div>
                    
                    <Show when={verifyHashes()}>
                      <div class="mb-2">
                        <strong>Verified:</strong> {result().verified}
                      </div>
                    </Show>

                    <Show when={result().failed.length > 0}>
                      <div class="mt-4">
                        <strong class="text-red-600">Failed ({result().failed.length}):</strong>
                        <div class="ml-4 mt-1 max-h-32 overflow-auto">
                          <For each={result().failed}>
                            {(path) => <div class="text-xs">{path}</div>}
                          </For>
                        </div>
                      </div>
                    </Show>

                    <Show when={result().verification_failed.length > 0}>
                      <div class="mt-4">
                        <strong class="text-orange-600">
                          Verification Failed ({result().verification_failed.length}):
                        </strong>
                        <div class="ml-4 mt-1 max-h-32 overflow-auto">
                          <For each={result().verification_failed}>
                            {(path) => <div class="text-xs">{path}</div>}
                          </For>
                        </div>
                      </div>
                    </Show>
                  </div>
                </div>
              )}
            </Show>
          </div>
        </Show>
      </Show>
    </div>
  );
};

export default Ad1OperationsV2;
