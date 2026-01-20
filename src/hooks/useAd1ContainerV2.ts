// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAd1ContainerV2 - React/Solid hook for AD1 Container V2 API
 * 
 * Based on libad1 C implementation with improvements for:
 * - Lazy loading
 * - Hash verification
 * - File extraction
 * - Container information
 */

import { invoke } from '@tauri-apps/api/core';
import { createSignal, createResource } from 'solid-js';
import { formatBytes } from '../utils';

// Re-export for backwards compatibility
export { formatBytes };

export interface TreeEntryV2 {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  item_type: number;
  first_child_addr?: number | null;
  data_addr?: number | null;
  item_addr?: number | null;
  compressed_size?: number | null;
  data_end_addr?: number | null;
  metadata_addr?: number | null;
  md5_hash?: string | null;
  sha1_hash?: string | null;
  created?: string | null;
  accessed?: string | null;
  modified?: string | null;
  attributes?: string[] | null;
  child_count?: number | null;
}

export interface ItemVerifyResult {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  hash_type: 'Md5' | 'Sha1';
  result: 'Ok' | 'Mismatch' | 'NotFound' | { Error: string };
  stored_hash?: string;
  computed_hash?: string;
}

export interface ExtractionResult {
  total_files: number;
  total_dirs: number;
  total_bytes: number;
  failed: string[];
  verified: number;
  verification_failed: string[];
}

export interface Ad1InfoV2 {
  segment_header: {
    signature: string;
    segment_index: number;
    segment_number: number;
    fragments_size: number;
    header_size: number;
  };
  logical_header: {
    signature: string;
    image_version: number;
    zlib_chunk_size: number;
    logical_metadata_addr: number;
    first_item_addr: number;
    data_source_name_length: number;
    ad_signature: string;
    data_source_name_addr: number;
    attrguid_footer_addr: number;
    locsguid_footer_addr: number;
    data_source_name: string;
  };
  total_items: number;
  total_size: number;
  file_count: number;
  dir_count: number;
  tree?: TreeItem[];
}

export interface TreeItem {
  name: string;
  is_dir: boolean;
  size: number;
  depth: number;
  path: string;
  children?: TreeItem[];
}

/**
 * Hook for AD1 Container V2 operations
 */
export function useAd1ContainerV2(containerPath: string) {
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * Get root children (fast, no full tree parsing)
   */
  async function getRootChildren(): Promise<TreeEntryV2[]> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<TreeEntryV2[]>('container_get_root_children_v2', {
        containerPath,
      });
      return result;
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Get children at specific address (lazy loading)
   */
  async function getChildrenAtAddr(
    addr: number,
    parentPath: string
  ): Promise<TreeEntryV2[]> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<TreeEntryV2[]>('container_get_children_at_addr_v2', {
        containerPath,
        addr,
        parentPath,
      });
      return result;
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Read file data by item address
   */
  async function readFileData(itemAddr: number): Promise<Uint8Array> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<number[]>('container_read_file_data_v2', {
        containerPath,
        itemAddr,
      });
      return new Uint8Array(result);
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Get item information by address
   */
  async function getItemInfo(addr: number): Promise<TreeEntryV2> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<TreeEntryV2>('container_get_item_info_v2', {
        containerPath,
        addr,
      });
      return result;
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Verify item hash
   */
  async function verifyItemHash(itemAddr: number): Promise<boolean> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<boolean>('container_verify_item_hash_v2', {
        containerPath,
        itemAddr,
      });
      return result;
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Verify all items in container
   */
  async function verifyAll(hashType: 'md5' | 'sha1'): Promise<ItemVerifyResult[]> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<ItemVerifyResult[]>('container_verify_all_v2', {
        containerPath,
        hashType,
      });
      return result;
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Get container information
   */
  async function getInfo(includeTree: boolean = false): Promise<Ad1InfoV2> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<Ad1InfoV2>('container_get_info_v2', {
        containerPath,
        includeTree,
      });
      return result;
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Extract all files
   */
  async function extractAll(
    outputDir: string,
    applyMetadata: boolean = true,
    verifyHashes: boolean = false
  ): Promise<ExtractionResult> {
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<ExtractionResult>('container_extract_all_v2', {
        containerPath,
        outputDir,
        applyMetadata,
        verifyHashes,
      });
      return result;
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  /**
   * Extract single item by address
   */
  async function extractItem(
    itemAddr: number,
    outputPath: string
  ): Promise<void> {
    setIsLoading(true);
    setError(null);
    
    try {
      await invoke<void>('container_extract_item_v2', {
        containerPath,
        itemAddr,
        outputPath,
      });
    } catch (e) {
      const errorMsg = String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  }

  return {
    isLoading,
    error,
    getRootChildren,
    getChildrenAtAddr,
    readFileData,
    getItemInfo,
    verifyItemHash,
    verifyAll,
    getInfo,
    extractAll,
    extractItem,
  };
}

/**
 * Resource for container info (auto-fetching)
 */
export function useAd1InfoV2(containerPath: string, includeTree: boolean = false) {
  const [info] = createResource(
    () => ({ containerPath, includeTree }),
    async ({ containerPath, includeTree }) => {
      return await invoke<Ad1InfoV2>('container_get_info_v2', {
        containerPath,
        includeTree,
      });
    }
  );

  return info;
}

/**
 * Format hash verification result for display
 */
export function formatHashResult(result: ItemVerifyResult['result']): string {
  if (result === 'Ok') return '✅ Match';
  if (result === 'Mismatch') return '❌ Mismatch';
  if (result === 'NotFound') return '⚠️ No Hash';
  if (typeof result === 'object' && 'Error' in result) {
    return `❌ Error: ${result.Error}`;
  }
  return '❓ Unknown';
}
