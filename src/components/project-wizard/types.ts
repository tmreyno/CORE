// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ProcessedDatabase } from "../../types/processed";
import type { StoredHash } from "../../types";

export interface ProjectLocations {
  /** Project name */
  projectName: string;
  /** Owner/Examiner name */
  ownerName?: string;
  /** Parent case number */
  caseNumber?: string;
  /** Parent case name/title */
  caseName?: string;
  /** Root project directory */
  projectRoot: string;
  /** Path to evidence files directory */
  evidencePath: string;
  /** Path to processed databases directory */
  processedDbPath: string;
  /** Path to case documents directory (COC, forms, etc.) */
  caseDocumentsPath: string;
  /** Auto-discovered evidence files */
  discoveredEvidence: string[];
  /** Auto-discovered processed databases */
  discoveredDatabases: ProcessedDatabase[];
  /** Whether to load stored hashes on project open */
  loadStoredHashes: boolean;
  /** Pre-loaded stored hashes map (path -> StoredHash[]) - fast hash-only extraction */
  loadedStoredHashes?: Map<string, StoredHash[]>;
}

export interface ProjectSetupWizardProps {
  /** The selected project root directory */
  projectRoot: string;
  /** Whether the wizard is visible */
  isOpen: boolean;
  /** Called when wizard is closed/cancelled */
  onClose: () => void;
  /** Called when setup is complete with locations */
  onComplete: (locations: ProjectLocations) => void;
}
