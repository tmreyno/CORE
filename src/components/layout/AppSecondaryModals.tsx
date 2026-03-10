// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AppSecondaryModals — Lazy-loaded modal/wizard overlays extracted from App.tsx.
 *
 * Contains:
 *  - UserConfirmModal   — Profile confirmation on project open/create
 *  - ReportWizard       — Multi-step report generation wizard
 *  - UpdateModal        — Application update checker
 *  - MergeProjectsWizard — Combine multiple .cffx projects
 *  - RecoveryModal      — Project recovery / auto-save restore
 *
 * Each modal is conditionally rendered with <Show> and lazy-loaded via
 * <Suspense> to minimise the initial bundle size.
 */

import { Show, lazy, Suspense, type Component, type Accessor, type Setter } from "solid-js";
import type { UserProfile, AppPreferences } from "../preferences";
import type { ReportType } from "../report/types";
import type { useFileManager } from "../../hooks/useFileManager";
import type { useHashManager } from "../../hooks/useHashManager";
import type { useProject } from "../../hooks/useProject";
import { getBasename } from "../../utils/pathUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("AppSecondaryModals");

// Lazy-loaded heavy components (code-split)
const ReportWizard = lazy(() => import("../report/wizard/ReportWizard").then(m => ({ default: m.ReportWizard })));
const UpdateModal = lazy(() => import("../UpdateModal"));
const UserConfirmModal = lazy(() => import("../project/UserConfirmModal").then(m => ({ default: m.UserConfirmModal })));

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface AppSecondaryModalsProps {
  /** Data sources — used to derive ReportWizard data bindings */
  fileManager: ReturnType<typeof useFileManager>;
  hashManager: ReturnType<typeof useHashManager>;
  projectManager: ReturnType<typeof useProject>;

  // User Confirm Modal
  showUserConfirmModal: Accessor<boolean>;
  setShowUserConfirmModal: Setter<boolean>;
  userConfirmAction: Accessor<"create" | "open">;
  userConfirmProjectName: Accessor<string>;
  userProfiles: UserProfile[];
  defaultUserProfileId: string;
  onUpdatePreference: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
  setShowSettingsPanel: Setter<boolean>;

  // Report Wizard
  showReportWizard: Accessor<boolean>;
  setShowReportWizard: Setter<boolean>;
  initialReportType: Accessor<ReportType | undefined>;
  setInitialReportType: Setter<ReportType | undefined>;

  // Update Modal
  showUpdateModal: Accessor<boolean>;
  setShowUpdateModal: Setter<boolean>;

  // Merge Projects Wizard
  showMergeWizard: Accessor<boolean>;
  setShowMergeWizard: Setter<boolean>;
  onLoadProject: (path: string) => void;

  // Recovery Modal
  showRecoveryModal: Accessor<boolean>;
  setShowRecoveryModal: Setter<boolean>;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const AppSecondaryModals: Component<AppSecondaryModalsProps> = (props) => {
  return (
    <>
      {/* User Confirm Modal — shown on project create/open */}
      <Suspense fallback={null}>
        <UserConfirmModal
          isOpen={props.showUserConfirmModal()}
          onClose={() => props.setShowUserConfirmModal(false)}
          profiles={props.userProfiles}
          defaultProfileId={props.defaultUserProfileId}
          onConfirm={() => {}}
          onUpdatePreference={props.onUpdatePreference}
          onOpenSettings={() => {
            props.setShowUserConfirmModal(false);
            props.setShowSettingsPanel(true);
          }}
          action={props.userConfirmAction()}
          projectName={props.userConfirmProjectName()}
        />
      </Suspense>

      {/* Report Wizard Modal */}
      <Show when={props.showReportWizard()}>
        <Suspense fallback={<div class="modal-overlay"><div class="flex items-center justify-center h-full text-txt-muted text-sm">Loading report wizard…</div></div>}>
          <ReportWizard
            files={props.fileManager.discoveredFiles()}
            fileInfoMap={props.fileManager.fileInfoMap()}
            fileHashMap={props.hashManager.fileHashMap()}
            activityLog={props.projectManager.project()?.activity_log}
            sessions={props.projectManager.project()?.sessions}
            projectName={props.projectManager.projectName() || undefined}
            projectDescription={props.projectManager.project()?.description}
            caseNumber={props.projectManager.caseNumber() || undefined}
            caseName={props.projectManager.caseName() || undefined}
            caseDocumentsCache={props.projectManager.project()?.case_documents_cache?.documents}
            bookmarks={props.projectManager.project()?.bookmarks}
            notes={props.projectManager.project()?.notes}
            initialReportType={props.initialReportType()}
            onClose={() => { props.setShowReportWizard(false); props.setInitialReportType(undefined); }}
            onGenerated={(path: string, format: string) => {
              log.info(`Report generated: ${path} (${format})`);
              props.fileManager.setOk(`Report saved to ${path}`);
              props.projectManager.logActivity(
                'export',
                'report',
                `Report generated: ${getBasename(path) || path} (${format})`,
                path,
                { format },
              );
            }}
          />
        </Suspense>
      </Show>

      {/* Update Modal */}
      <Show when={props.showUpdateModal()}>
        <Suspense>
          <UpdateModal
            show={props.showUpdateModal()}
            onClose={() => props.setShowUpdateModal(false)}
          />
        </Suspense>
      </Show>

      {/* Merge Projects Wizard */}
      <Show when={props.showMergeWizard()}>
        <Suspense>
          {(() => {
            const MergeProjectsWizard = lazy(() => import("../MergeProjectsWizard"));
            return (
              <MergeProjectsWizard
                currentProjectPath={props.projectManager.projectPath() || undefined}
                onClose={() => props.setShowMergeWizard(false)}
                onMergeComplete={(cffxPath: string) => {
                  props.setShowMergeWizard(false);
                  props.onLoadProject(cffxPath);
                }}
              />
            );
          })()}
        </Suspense>
      </Show>

      {/* Project Recovery Modal */}
      <Show when={props.showRecoveryModal()}>
        <Suspense>
          {(() => {
            const RecoveryModal = lazy(() => import("../project/RecoveryModal").then(m => ({ default: m.RecoveryModal })));
            return (
              <RecoveryModal
                isOpen={props.showRecoveryModal()}
                onClose={() => props.setShowRecoveryModal(false)}
                projectPath={props.projectManager.projectPath() || ""}
              />
            );
          })()}
        </Suspense>
      </Show>
    </>
  );
};
