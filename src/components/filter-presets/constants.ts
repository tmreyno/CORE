// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { QuickFilter } from "./types";

export const DEFAULT_QUICK_FILTERS: QuickFilter[] = [
  {
    id: "documents",
    name: "Documents",
    extensions: [".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".txt", ".rtf", ".odt"],
    icon: "document",
  },
  {
    id: "images",
    name: "Images",
    extensions: [".jpg", ".jpeg", ".png", ".gif", ".bmp", ".tiff", ".webp", ".svg", ".heic", ".raw"],
    icon: "photo",
  },
  {
    id: "videos",
    name: "Videos",
    extensions: [".mp4", ".avi", ".mkv", ".mov", ".wmv", ".flv", ".webm", ".m4v"],
    icon: "film",
  },
  {
    id: "audio",
    name: "Audio",
    extensions: [".mp3", ".wav", ".aac", ".flac", ".ogg", ".wma", ".m4a"],
    icon: "music",
  },
  {
    id: "archives",
    name: "Archives",
    extensions: [".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".ad1", ".e01", ".l01"],
    icon: "archive",
  },
  {
    id: "code",
    name: "Code",
    extensions: [".js", ".ts", ".py", ".java", ".c", ".cpp", ".h", ".cs", ".go", ".rs", ".html", ".css"],
    icon: "code",
  },
  {
    id: "databases",
    name: "Databases",
    extensions: [".db", ".sqlite", ".sqlite3", ".mdb", ".sql"],
    icon: "database",
  },
  {
    id: "emails",
    name: "Emails",
    extensions: [".eml", ".msg", ".pst", ".ost", ".mbox"],
    icon: "email",
  },
];
