// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * File Type Utilities - Centralized File Type Detection
 * 
 * Provides file type detection and categorization for consistent behavior
 * across the application. Eliminates duplicate regex patterns and array checks.
 */

import { getExtension } from "./pathUtils";

// =============================================================================
// File Extension Categories
// =============================================================================

/** Image file extensions */
export const IMAGE_EXTENSIONS = [
  // Common formats
  "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico",
  // Professional formats
  "tiff", "tif", "heic", "heif",
  // RAW camera formats
  "raw", "cr2", "nef", "arw", "dng", "orf", "rw2",
] as const;

/** Video file extensions */
export const VIDEO_EXTENSIONS = [
  "mp4", "avi", "mov", "mkv", "wmv", "flv", "webm",
  "m4v", "mpeg", "mpg", "3gp", "ogv", "m2ts", "mts",
] as const;

/** Audio file extensions */
export const AUDIO_EXTENSIONS = [
  "mp3", "wav", "flac", "aac", "ogg", "wma", "m4a",
  "aiff", "aif", "opus", "ape", "alac",
] as const;

/** Document file extensions (office, text, PDF) */
export const DOCUMENT_EXTENSIONS = [
  // PDF
  "pdf",
  // Microsoft Office
  "doc", "docx", "xls", "xlsx", "ppt", "pptx",
  // OpenDocument
  "odt", "ods", "odp",
  // Text formats
  "rtf", "txt", "md", "markdown",
  // Spreadsheets/data
  "csv", "tsv",
] as const;

/** Spreadsheet file extensions */
export const SPREADSHEET_EXTENSIONS = [
  "xlsx", "xls", "ods", "csv", "tsv", "numbers",
] as const;

/** Text-based document extensions (for DocumentViewer) */
export const TEXT_DOCUMENT_EXTENSIONS = [
  "docx", "doc", "html", "htm", "md", "markdown", "txt", "rtf",
] as const;

/** Code/source file extensions */
export const CODE_EXTENSIONS = [
  // JavaScript/TypeScript
  "js", "ts", "jsx", "tsx", "mjs", "cjs",
  // Python
  "py", "pyw", "pyx",
  // Rust
  "rs",
  // Go
  "go",
  // Java/Kotlin
  "java", "kt", "kts",
  // C/C++
  "c", "cpp", "cc", "cxx", "h", "hpp", "hxx",
  // C#
  "cs", "csx",
  // Ruby
  "rb", "rake",
  // PHP
  "php", "phtml",
  // Web
  "html", "css", "scss", "sass", "less",
  // Data formats
  "json", "xml", "yaml", "yml", "toml",
  // SQL
  "sql",
  // Shell
  "sh", "bash", "zsh", "fish", "ps1", "psm1",
] as const;

/** Database file extensions */
export const DATABASE_EXTENSIONS = [
  "db", "sqlite", "sqlite3", "mdb", "accdb", "sql",
  "dbf", "sdf", "ndf", "mdf", "ldf",
] as const;

/** Archive file extensions */
export const ARCHIVE_EXTENSIONS = [
  "zip", "tar", "gz", "bz2", "7z", "rar", "xz", "tgz",
  "tar.gz", "tar.bz2", "tar.xz", "tbz2", "txz",
] as const;

// =============================================================================
// Type Helpers
// =============================================================================

/**
 * Type-safe includes check for readonly tuple arrays.
 * Avoids `as any` casts when checking if a string is in a const array.
 */
function includesExtension<T extends readonly string[]>(
  extensions: T,
  ext: string
): ext is T[number] {
  return (extensions as readonly string[]).includes(ext);
}

// =============================================================================
// Type Guards
// =============================================================================

/**
 * Check if file is an image.
 * 
 * @param filename - File name or path
 * @returns true if file is an image
 */
export function isImage(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(IMAGE_EXTENSIONS, ext);
}

/**
 * Check if file is a video.
 * 
 * @param filename - File name or path
 * @returns true if file is a video
 */
export function isVideo(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(VIDEO_EXTENSIONS, ext);
}

/**
 * Check if file is audio.
 * 
 * @param filename - File name or path
 * @returns true if file is audio
 */
export function isAudio(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(AUDIO_EXTENSIONS, ext);
}

/**
 * Check if file is a document (office, PDF, text).
 * 
 * @param filename - File name or path
 * @returns true if file is a document
 */
export function isDocument(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(DOCUMENT_EXTENSIONS, ext);
}

/**
 * Check if file is a spreadsheet.
 * 
 * @param filename - File name or path
 * @returns true if file is a spreadsheet
 */
export function isSpreadsheet(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(SPREADSHEET_EXTENSIONS, ext);
}

/**
 * Check if file is a text-based document (for DocumentViewer).
 * 
 * @param filename - File name or path
 * @returns true if file can be rendered by DocumentViewer
 */
export function isTextDocument(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(TEXT_DOCUMENT_EXTENSIONS, ext);
}

/**
 * Check if file is source code.
 * 
 * @param filename - File name or path
 * @returns true if file is source code
 */
export function isCode(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(CODE_EXTENSIONS, ext);
}

/**
 * Check if file is a database.
 * 
 * @param filename - File name or path
 * @returns true if file is a database
 */
export function isDatabase(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(DATABASE_EXTENSIONS, ext);
}

/**
 * Check if file is an archive.
 * 
 * @param filename - File name or path
 * @returns true if file is an archive
 */
export function isArchive(filename: string): boolean {
  const ext = getExtension(filename);
  return includesExtension(ARCHIVE_EXTENSIONS, ext);
}

/**
 * Check if file is a PDF.
 * 
 * @param filename - File name or path
 * @returns true if file is a PDF
 */
export function isPdf(filename: string): boolean {
  return getExtension(filename) === "pdf";
}

// =============================================================================
// File Type Detection
// =============================================================================

/**
 * File type categories for UI rendering and icon selection.
 */
export type FileTypeCategory = 
  | "image"
  | "video"
  | "audio"
  | "document"
  | "spreadsheet"
  | "code"
  | "database"
  | "archive"
  | "container"
  | "unknown";

/**
 * Detect file type category for UI rendering.
 * Checks forensic containers first, then by extension.
 * 
 * @param filename - File name or path
 * @returns File type category
 */
export function detectFileType(filename: string): FileTypeCategory {
  // Import container detection to check for forensic containers
  // (doing inline to avoid circular dependency)
  const ext = getExtension(filename);
  
  // Check forensic containers
  const containerExts = ["ad1", "e01", "ex01", "l01", "lx01", "ufd", "ufdr", "ufdx", "dd", "raw", "img"];
  if (containerExts.includes(ext)) return "container";
  
  // Check by category
  if (isImage(filename)) return "image";
  if (isVideo(filename)) return "video";
  if (isAudio(filename)) return "audio";
  if (isSpreadsheet(filename)) return "spreadsheet"; // Check before document (csv/xls)
  if (isDocument(filename)) return "document";
  if (isCode(filename)) return "code";
  if (isDatabase(filename)) return "database";
  if (isArchive(filename)) return "archive";
  
  return "unknown";
}

// Re-export path utilities for convenience
export { 
  getExtension, 
  hasExtension, 
  hasAnyExtension,
  getBasenameWithoutExt as getBaseName
} from "./pathUtils";
