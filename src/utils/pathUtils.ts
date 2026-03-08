// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Path Utilities
 * 
 * Centralized path manipulation utilities for frontend operations.
 * These utilities are designed for forensic file paths (container internal paths,
 * evidence file paths, etc.) and use forward slashes as separators.
 * 
 * Key Functions:
 * - Extension extraction: getExtension(), hasExtension(), hasAnyExtension()
 * - Path parsing: getBasename(), getBasenameWithoutExt(), getDirname()
 * - Path manipulation: joinPath(), normalizePath()
 * 
 * @module pathUtils
 */

// =============================================================================
// Extension Utilities
// =============================================================================

/**
 * Get the lowercase file extension from a filename or path.
 * 
 * Implementation uses lastIndexOf for optimal performance with long paths.
 * Handles edge cases: no extension, multiple dots, hidden files.
 * 
 * @param filename - File name or path (e.g., "file.PDF", "/path/to/doc.txt")
 * @returns Extension string without dot, lowercase (e.g., "pdf", "txt")
 * 
 * @example
 * getExtension("file.PDF") => "pdf"
 * getExtension("/path/to/document.TXT") => "txt"
 * getExtension("archive.tar.gz") => "gz"
 * getExtension("README") => ""
 * getExtension(".gitignore") => "" (dotfiles have no extension)
 * getExtension("/path/to/.hidden") => "" (dotfiles have no extension)
 * getExtension("file.") => ""
 */
export function getExtension(filename: string): string {
  const lower = filename.toLowerCase();
  const lastDot = lower.lastIndexOf('.');
  
  // No dot found or dot is last character
  if (lastDot <= 0 || lastDot === lower.length - 1) {
    return '';
  }
  
  return lower.slice(lastDot + 1);
}

/**
 * Check if file has a specific extension.
 * Case-insensitive comparison.
 * 
 * @param filename - File name or path
 * @param extension - Extension to check (without dot, e.g., "pdf", "txt")
 * @returns true if file has the specified extension
 * 
 * @example
 * hasExtension("file.PDF", "pdf") => true
 * hasExtension("doc.txt", "pdf") => false
 * hasExtension("README", "txt") => false
 */
export function hasExtension(filename: string, extension: string): boolean {
  return getExtension(filename) === extension.toLowerCase();
}

/**
 * Check if file has any of the provided extensions.
 * Case-insensitive comparison.
 * 
 * @param filename - File name or path
 * @param extensions - Array of extensions to check (without dots)
 * @returns true if file has any of the specified extensions
 * 
 * @example
 * hasAnyExtension("file.pdf", ["pdf", "doc"]) => true
 * hasAnyExtension("image.png", ["jpg", "jpeg"]) => false
 */
export function hasAnyExtension(filename: string, extensions: readonly string[]): boolean {
  const ext = getExtension(filename);
  return extensions.some(e => e.toLowerCase() === ext);
}

// =============================================================================
// Path Component Extraction
// =============================================================================

/**
 * Get basename (filename with extension) from a path.
 * Handles both forward and backward slashes.
 * 
 * @param path - File path (e.g., "/path/to/file.txt", "C:\\folder\\file.txt")
 * @returns Filename with extension (e.g., "file.txt")
 * 
 * @example
 * getBasename("/path/to/file.txt") => "file.txt"
 * getBasename("C:\\folder\\doc.pdf") => "doc.pdf"
 * getBasename("file.txt") => "file.txt"
 * getBasename("/path/to/") => ""
 * getBasename("") => ""
 */
export function getBasename(path: string): string {
  const lastForward = path.lastIndexOf('/');
  const lastBackward = path.lastIndexOf('\\');
  const lastSlash = Math.max(lastForward, lastBackward);
  
  if (lastSlash < 0) {
    // No slashes - entire string is the basename
    return path;
  }
  
  // Return everything after the last slash
  return path.slice(lastSlash + 1);
}

/**
 * Get basename without extension (filename only).
 * 
 * @param path - File path or filename
 * @returns Filename without extension
 * 
 * @example
 * getBasenameWithoutExt("/path/to/file.txt") => "file"
 * getBasenameWithoutExt("archive.tar.gz") => "archive.tar"
 * getBasenameWithoutExt("README") => "README"
 * getBasenameWithoutExt(".gitignore") => ""
 */
export function getBasenameWithoutExt(path: string): string {
  const basename = getBasename(path);
  const lastDot = basename.lastIndexOf('.');
  
  // No dot, or dot is first character (hidden file)
  if (lastDot <= 0) {
    return lastDot === 0 ? '' : basename;
  }
  
  return basename.slice(0, lastDot);
}

/**
 * Get directory name (parent path) from a path.
 * 
 * @param path - File path
 * @returns Directory path, or empty string if no parent
 * 
 * @example
 * getDirname("/path/to/file.txt") => "/path/to"
 * getDirname("/path/to/") => "/path/to"
 * getDirname("file.txt") => ""
 * getDirname("/file.txt") => ""
 */
export function getDirname(path: string): string {
  // Remove trailing slashes
  const trimmed = path.replace(/[\/\\]+$/, '');
  
  const lastForward = trimmed.lastIndexOf('/');
  const lastBackward = trimmed.lastIndexOf('\\');
  const lastSlash = Math.max(lastForward, lastBackward);
  
  if (lastSlash < 0) {
    // No parent directory
    return '';
  }
  
  return trimmed.slice(0, lastSlash);
}

// =============================================================================
// Path Manipulation
// =============================================================================

/**
 * Join path components with forward slashes.
 * Handles empty components and trailing slashes.
 * 
 * @param parts - Path components to join
 * @returns Joined path with forward slashes
 * 
 * @example
 * joinPath("/path", "to", "file.txt") => "/path/to/file.txt"  // preserves leading /
 * joinPath("/path/", "to/", "file.txt") => "/path/to/file.txt"
 * joinPath("", "file.txt") => "file.txt"
 * joinPath("/path", "", "file.txt") => "/path/file.txt"
 */
export function joinPath(...parts: string[]): string {
  // Preserve leading slash from the first non-empty part (absolute paths)
  const firstNonEmpty = parts.find(p => p.length > 0);
  const prefix = firstNonEmpty && firstNonEmpty.startsWith('/') ? '/' : '';
  const joined = parts
    .filter(part => part.length > 0)
    .map(part => part.replace(/^\/+|\/+$/g, ''))
    .filter(part => part.length > 0)
    .join('/');
  return prefix + joined;
}

/**
 * Normalize path to use forward slashes and remove redundant slashes.
 * Does NOT resolve .. or . components (use backend path_security for that).
 * 
 * @param path - Path to normalize
 * @returns Normalized path with forward slashes
 * 
 * @example
 * normalizePath("C:\\path\\to\\file.txt") => "C:/path/to/file.txt"
 * normalizePath("/path//to///file.txt") => "/path/to/file.txt"
 * normalizePath("path/to/file.txt") => "path/to/file.txt"
 */
export function normalizePath(path: string): string {
  // Convert backslashes to forward slashes
  let normalized = path.replace(/\\/g, '/');
  
  // Remove redundant slashes (but keep leading slash if present)
  normalized = normalized.replace(/\/+/g, '/');
  
  // Remove trailing slash (except for root)
  if (normalized.length > 1 && normalized.endsWith('/')) {
    normalized = normalized.slice(0, -1);
  }
  
  return normalized;
}

// =============================================================================
// Path Type Guards
// =============================================================================

/**
 * Check if path is absolute (starts with / or drive letter).
 * 
 * @param path - Path to check
 * @returns true if path is absolute
 * 
 * @example
 * isAbsolutePath("/path/to/file") => true
 * isAbsolutePath("C:\\path\\to\\file") => true
 * isAbsolutePath("relative/path") => false
 */
export function isAbsolutePath(path: string): boolean {
  if (path.startsWith('/')) return true;
  // Check for Windows drive letter (C:, D:, etc.)
  if (/^[A-Za-z]:/.test(path)) return true;
  return false;
}

/**
 * Check if path appears to be a hidden file (starts with dot).
 * Only checks the basename, not intermediate directories.
 * 
 * @param path - Path or filename to check
 * @returns true if basename starts with dot
 * 
 * @example
 * isHiddenFile(".gitignore") => true
 * isHiddenFile("/path/.config") => true
 * isHiddenFile("/path/to/file.txt") => false
 */
export function isHiddenFile(path: string): boolean {
  const basename = getBasename(path);
  return basename.startsWith('.');
}

/**
 * Split a path into its component segments (cross-platform).
 * Handles both forward and backward slashes.
 * 
 * @param path - Path to split
 * @returns Array of non-empty path segments
 * 
 * @example
 * splitPath("/path/to/file.txt") => ["path", "to", "file.txt"]
 * splitPath("C:\\Users\\docs") => ["C:", "Users", "docs"]
 * splitPath("/") => []
 * splitPath("") => []
 */
export function splitPath(path: string): string[] {
  return path.split(/[/\\]/).filter(Boolean);
}

// =============================================================================
// Compatibility Exports (removed deprecated getBaseName alias - use getBasenameWithoutExt)
// =============================================================================
