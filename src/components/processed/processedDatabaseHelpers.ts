// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Processed Database Helpers - Utility functions
 */

import type { ProcessedDatabase, AxiomCaseInfo } from '../../types/processed';
import { ellipsePath } from '../../utils/processed';

/**
 * Get display name for a database
 */
export const getDatabaseDisplayName = (
  db: ProcessedDatabase,
  caseInfo?: AxiomCaseInfo
): string => {
  return caseInfo?.case_name || db.case_name || db.name || ellipsePath(db.path, 30);
};

/**
 * Get total keywords count from case info
 */
export const getTotalKeywords = (caseInfo?: AxiomCaseInfo): number => {
  return caseInfo?.keyword_info?.keywords?.length || 0;
};

/**
 * Get keyword files from case info
 */
export const getKeywordFiles = (caseInfo?: AxiomCaseInfo) => {
  return caseInfo?.keyword_info?.keyword_files || [];
};
