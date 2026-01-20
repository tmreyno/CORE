// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Transfer Panel Utilities
 * 
 * Helper functions for container detection, file type identification,
 * tree building, and formatting.
 */

import type { TransferFileInfo } from "../../transfer";
import type { ContainerType, FileType, FileTreeNode } from "./types";

// =============================================================================
// Container Detection
// =============================================================================

/** Detect container type from file path */
export function detectContainerType(path: string): ContainerType {
  const lower = path.toLowerCase();
  if (lower.endsWith(".e01") || lower.endsWith(".ex01") || lower.includes(".e0")) return "e01";
  if (lower.endsWith(".l01") || lower.endsWith(".lx01") || lower.includes(".l0")) return "l01";
  if (lower.endsWith(".ad1")) return "ad1";
  // UFED files have specific extensions - do NOT include generic .zip
  if (lower.endsWith(".ufd") || lower.endsWith(".ufdr") || lower.endsWith(".ufdx")) return "ufed";
  if (lower.endsWith(".dd") || lower.endsWith(".raw") || lower.endsWith(".img") || lower.endsWith(".bin")) return "raw";
  return "unknown";
}

/** Check if path is a forensic container that needs special hashing */
export function isForensicContainer(path: string): boolean {
  return detectContainerType(path) !== "unknown";
}

// =============================================================================
// File Type Detection
// =============================================================================

/** Detect file type for icon selection */
export function detectFileType(path: string): FileType {
  const lower = path.toLowerCase();
  // Forensic containers
  if (detectContainerType(path) !== "unknown") return "container";
  // Images
  if (/\.(jpg|jpeg|png|gif|bmp|webp|svg|ico|tiff?|heic|raw|cr2|nef|arw)$/.test(lower)) return "image";
  // Video
  if (/\.(mp4|avi|mov|mkv|wmv|flv|webm|m4v|mpeg|mpg|3gp)$/.test(lower)) return "video";
  // Audio
  if (/\.(mp3|wav|flac|aac|ogg|wma|m4a|aiff?)$/.test(lower)) return "audio";
  // Documents
  if (/\.(pdf|doc|docx|xls|xlsx|ppt|pptx|odt|ods|odp|rtf|txt|md|csv)$/.test(lower)) return "document";
  // Code
  if (/\.(js|ts|jsx|tsx|py|rs|go|java|c|cpp|h|hpp|cs|rb|php|html|css|scss|json|xml|yaml|yml|toml|sql|sh|bash|ps1)$/.test(lower)) return "code";
  // Database
  if (/\.(db|sqlite|sqlite3|mdb|accdb|sql)$/.test(lower)) return "database";
  // Archives
  if (/\.(zip|tar|gz|bz2|7z|rar|xz|tgz)$/.test(lower)) return "archive";
  return "unknown";
}

// =============================================================================
// Tree Building
// =============================================================================

/** Build tree structure from flat file list */
export function buildFileTree(files: TransferFileInfo[]): FileTreeNode[] {
  const root: FileTreeNode[] = [];
  const dirMap = new Map<string, FileTreeNode>();
  
  // Filter out files without relative_path and sort
  const validFiles = files.filter(f => f.relative_path);
  const sortedFiles = [...validFiles].sort((a, b) => 
    (a.relative_path || '').localeCompare(b.relative_path || '')
  );
  
  for (const file of sortedFiles) {
    if (!file.relative_path) continue;
    const parts = file.relative_path.split('/').filter(p => p);
    if (parts.length === 0) continue;
    
    // Ensure all parent directories exist
    let currentPath = '';
    let parentChildren = root;
    
    for (let i = 0; i < parts.length - 1; i++) {
      const part = parts[i];
      currentPath = currentPath ? `${currentPath}/${part}` : part;
      
      if (!dirMap.has(currentPath)) {
        const dirNode: FileTreeNode = {
          name: part,
          path: currentPath,
          size: 0,
          sizeFormatted: '',
          isDirectory: true,
          fileType: 'unknown',
          containerType: 'unknown',
          children: [],
          expanded: true,
        };
        dirMap.set(currentPath, dirNode);
        parentChildren.push(dirNode);
      }
      parentChildren = dirMap.get(currentPath)!.children;
    }
    
    // Add the file node
    const fileName = parts[parts.length - 1];
    const fileNode: FileTreeNode = {
      name: fileName,
      path: file.source,
      size: file.size,
      sizeFormatted: file.size_formatted,
      isDirectory: false,
      fileType: detectFileType(file.source),
      containerType: detectContainerType(file.source),
      children: [],
      expanded: false,
    };
    parentChildren.push(fileNode);
  }
  
  // Sort children: directories first, then files, alphabetically
  const sortChildren = (nodes: FileTreeNode[]) => {
    nodes.sort((a, b) => {
      if (a.isDirectory !== b.isDirectory) {
        return a.isDirectory ? -1 : 1;
      }
      return a.name.localeCompare(b.name);
    });
    for (const node of nodes) {
      if (node.children.length > 0) {
        sortChildren(node.children);
      }
    }
  };
  sortChildren(root);
  
  return root;
}

// =============================================================================
// Date Formatting
// =============================================================================

/** Format date as short date string (e.g., "Jan 16, 2026") */
export function formatShortDate(date: Date): string {
  return date.toLocaleDateString(undefined, { 
    month: "short", 
    day: "numeric", 
    year: "numeric" 
  });
}

/** Format full date/time for tooltip */
export function formatFullDateTime(date: Date): string {
  return date.toLocaleString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}
