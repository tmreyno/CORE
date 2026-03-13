// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { TourStep } from "./types";
import { APP_NAME } from "../../utils/edition";

export const DEFAULT_TOUR_STEPS: TourStep[] = [
  {
    id: "welcome",
    title: `Welcome to ${APP_NAME}!`,
    content: "Let's take a quick tour of the main features.",
    position: "center",
  },
  {
    id: "file-panel",
    title: "Evidence Files",
    content:
      "This panel shows all loaded evidence files. Click 'Add Files' to load forensic images like E01, AD1, or other supported formats.",
    target: ".evidence-panel",
    position: "right",
  },
  {
    id: "tree-view",
    title: "File Tree",
    content:
      "Browse the contents of your evidence files here. Click folders to expand them and see their contents.",
    target: ".tree-panel",
    position: "right",
  },
  {
    id: "hex-viewer",
    title: "Hex Viewer",
    content:
      "View the raw bytes of selected files in hexadecimal format. Great for analyzing file headers and binary data.",
    target: ".hex-viewer",
    position: "left",
  },
  {
    id: "hash-verification",
    title: "Hash Verification",
    content:
      "Verify the integrity of evidence files by computing and comparing cryptographic hashes.",
    target: ".hash-panel",
    position: "left",
  },
  {
    id: "keyboard-shortcuts",
    title: "Keyboard Shortcuts",
    content:
      "Press ? anytime to see all available keyboard shortcuts. Use ⌘K to open the command palette for quick actions.",
    position: "center",
  },
];
