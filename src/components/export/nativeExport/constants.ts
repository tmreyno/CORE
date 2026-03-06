// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { CompressionLevel } from "../../../api/archiveCreate";
import type { ForensicPreset } from "./types";

export const FORENSIC_PRESETS: ForensicPreset[] = [
  {
    id: "forensic-standard",
    name: "Standard",
    description: "SHA-256 hashing, manifest, and verification",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: true,
  },
  {
    id: "forensic-court",
    name: "Court",
    description: "No compression, split for media, dual hashes",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 4096,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256+MD5",
    includeExaminerInfo: true,
  },
  {
    id: "forensic-transfer",
    name: "Transfer",
    description: "No compression, split for USB/cloud",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: false,
  },
  {
    id: "forensic-archive-long",
    name: "Long-term",
    description: "No compression, split for media, dual hashes",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256+MD5",
    includeExaminerInfo: true,
  },
  {
    id: "custom",
    name: "Custom",
    description: "Configure all settings manually",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: false,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: false,
  },
];

export const COMPRESSION_LEVELS: { value: number; label: string }[] = [
  { value: CompressionLevel.Store, label: "None (Store)" },
  { value: CompressionLevel.Fastest, label: "Fastest" },
  { value: CompressionLevel.Fast, label: "Fast" },
  { value: CompressionLevel.Normal, label: "Normal" },
  { value: CompressionLevel.Maximum, label: "Maximum" },
  { value: CompressionLevel.Ultra, label: "Ultra" },
];
