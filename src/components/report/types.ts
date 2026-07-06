// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { CollectedItem as CoreCollectedItem } from "@core-suite/types/forensic-report";

export * from "@core-suite/types/forensic-report";

export interface CollectedItem extends CoreCollectedItem {
  source_id?: string;
  source_ref_json?: string;
  hash_algorithm?: string;
  hash_value?: string;
  hash_computed_at?: string;
}
