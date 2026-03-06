// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { getPreference } from "../preferences";

export const INITIAL_LOAD_SIZE = 100000; // 100KB initial text load
export const LOAD_MORE_SIZE = 50000; // 50KB per additional load
export const SCROLL_THRESHOLD = 300; // pixels from bottom to trigger load

/** Get max loaded chars from preferences (convert MB to chars, 1 char ≈ 1 byte for ASCII) */
export const getMaxLoadedChars = () => getPreference("maxPreviewSizeMb") * 1024 * 1024;

/** Language extension → language name mapping for syntax detection. */
export const LANGUAGE_MAP: Record<string, string> = {
  js: "javascript",
  ts: "typescript",
  jsx: "javascript",
  tsx: "typescript",
  py: "python",
  rs: "rust",
  go: "go",
  java: "java",
  c: "c",
  cpp: "cpp",
  h: "c",
  hpp: "cpp",
  cs: "csharp",
  rb: "ruby",
  php: "php",
  html: "html",
  htm: "html",
  css: "css",
  scss: "scss",
  sass: "sass",
  less: "less",
  json: "json",
  xml: "xml",
  yaml: "yaml",
  yml: "yaml",
  toml: "toml",
  md: "markdown",
  sql: "sql",
  sh: "bash",
  bash: "bash",
  zsh: "bash",
  ps1: "powershell",
  bat: "batch",
  cmd: "batch",
};
