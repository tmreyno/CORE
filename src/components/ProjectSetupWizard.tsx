// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Re-export from decomposed project-wizard/ directory.
 * Original 810-line file split into:
 *   - project-wizard/types.ts           - ProjectLocations, ProjectSetupWizardProps
 *   - project-wizard/useWizardState.ts  - All signals, discovery, browse, hash loading
 *   - project-wizard/WizardStepIndicator.tsx - Step progress indicator
 *   - project-wizard/WizardFooter.tsx   - Footer buttons per step
 *   - project-wizard/ProjectSetupWizard.tsx  - Slim main component
 */

export { ProjectSetupWizard } from "./project-wizard";
export type { ProjectLocations, ProjectSetupWizardProps } from "./project-wizard";
