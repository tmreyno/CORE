// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Lazy-loaded viewer components — only loaded when the user previews that type.
 */

import { lazy } from "solid-js";

export const DocumentViewer = lazy(() =>
  import("../DocumentViewer").then((m) => ({ default: m.DocumentViewer })),
);
export const PdfViewer = lazy(() =>
  import("../PdfViewer").then((m) => ({ default: m.PdfViewer })),
);
export const SpreadsheetViewer = lazy(() =>
  import("../SpreadsheetViewer").then((m) => ({ default: m.SpreadsheetViewer })),
);
export const ImageViewer = lazy(() =>
  import("../ImageViewer").then((m) => ({ default: m.ImageViewer })),
);
export const EmailViewer = lazy(() =>
  import("../EmailViewer").then((m) => ({ default: m.EmailViewer })),
);
export const PstViewer = lazy(() =>
  import("../PstViewer").then((m) => ({ default: m.PstViewer })),
);
export const PlistViewer = lazy(() =>
  import("../PlistViewer").then((m) => ({ default: m.PlistViewer })),
);
export const ExifPanel = lazy(() =>
  import("../ExifPanel").then((m) => ({ default: m.ExifPanel })),
);
export const BinaryViewer = lazy(() =>
  import("../BinaryViewer").then((m) => ({ default: m.BinaryViewer })),
);
export const RegistryViewer = lazy(() =>
  import("../RegistryViewer").then((m) => ({ default: m.RegistryViewer })),
);
export const DatabaseViewer = lazy(() =>
  import("../DatabaseViewer").then((m) => ({ default: m.DatabaseViewer })),
);
export const OfficeViewer = lazy(() =>
  import("../OfficeViewer").then((m) => ({ default: m.OfficeViewer })),
);
