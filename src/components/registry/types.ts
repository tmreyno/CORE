// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Registry viewer types — matching Rust structs.
 */

import type { RegistryMetadataSection } from "../../types/viewerMetadata";

export interface RegistryKey {
  name: string;
  path: string;
  timestamp: string | null;
  subkeyCount: number;
  valueCount: number;
  hasSubkeys: boolean;
}

export interface RegistryValue {
  name: string;
  dataType: string;
  data: string;
  size: number;
}

export interface RegistryHiveInfo {
  path: string;
  rootKeyName: string;
  rootKeyPath: string;
  rootTimestamp: string | null;
  totalKeys: number;
  totalValues: number;
  rootSubkeyCount: number;
  rootValueCount: number;
}

export interface RegistrySubkeysResponse {
  parentPath: string;
  subkeys: RegistryKey[];
}

export interface RegistryKeyInfo {
  name: string;
  path: string;
  prettyPath: string;
  timestamp: string | null;
  subkeyCount: number;
  valueCount: number;
  values: RegistryValue[];
  subkeys: RegistryKey[];
}

export interface RegistryViewerProps {
  /** Path to the registry hive file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: RegistryMetadataSection) => void;
}

export interface TreeNode {
  key: RegistryKey;
  children: TreeNode[];
  loaded: boolean;
  expanded: boolean;
  depth: number;
}
