// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/** Get CSS color class for a plist value type */
export function getTypeColor(type: string): string {
  if (type === "String") return "text-green-400";
  if (type === "Integer" || type === "Real") return "text-blue-400";
  if (type === "Boolean") return "text-yellow-400";
  if (type === "Date") return "text-purple-400";
  if (type === "Data") return "text-orange-400";
  if (type.startsWith("Array")) return "text-cyan-400";
  if (type.startsWith("Dictionary")) return "text-pink-400";
  return "text-txt-muted";
}

/** Get depth of a key path (number of path segments - 1) */
export function getDepth(keyPath: string): number {
  return keyPath.split("/").filter(Boolean).length - 1;
}

/** Get the leaf key name from a key path */
export function getKeyName(keyPath: string): string {
  const parts = keyPath.split("/").filter(Boolean);
  return parts[parts.length - 1] || keyPath;
}

/** Check if a plist type is a container (Array or Dictionary) */
export function isContainerType(type: string): boolean {
  return type.startsWith("Array") || type.startsWith("Dictionary");
}

/** Notable key prefixes for forensic analysis */
export const NOTABLE_KEY_PREFIXES = [
  "CFBundleIdentifier",
  "CFBundleName",
  "CFBundleVersion",
  "CFBundleShortVersionString",
  "DTSDKName",
  "MinimumOSVersion",
];
