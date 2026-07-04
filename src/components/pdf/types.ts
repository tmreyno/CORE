// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { HashSourceInput } from "../../api/commands";

export interface PdfViewerProps {
  /** Path to the PDF file */
  path: string;
  /** Optional evidence source for container or nested PDF entries */
  source?: HashSourceInput | null;
  /** Optional class name */
  class?: string;
}
