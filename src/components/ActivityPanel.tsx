// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ActivityPanel — Re-export from decomposed activity/ module.
 *
 * The full implementation lives in:
 *   - activity/ActivityPanel.tsx (main component)
 *   - activity/SessionItem.tsx, activity/ActivityItem.tsx (list items)
 *   - activity/helpers.tsx (icon/format utilities)
 *   - activity/activityExport.ts (CSV/JSON export)
 *   - activity/types.ts (shared interfaces)
 */

export { ActivityPanel } from "./activity";
export { activitiesToCsv, activitiesToJson } from "./activity";
