// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProjectSetupWizard — Slim main component.
 *
 * Composes the modal shell (focus trap, escape key), step indicator,
 * step content (delegated to existing wizard/ sub-components), and footer.
 * All state lives in useWizardState.
 */

import { Component, Show, onMount } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import { HiOutlineFolder, HiOutlineXMark } from "../icons";
import { useFocusTrap } from "../../hooks/useFocusTrap";
import { logger } from "../../utils/logger";
import { SelectFolderStep } from "../wizard/SelectFolderStep";
import { ScanningStep } from "../wizard/ScanningStep";
import { ConfigureLocationsStep } from "../wizard/ConfigureLocationsStep";
import { LoadHashesStep } from "../wizard/LoadHashesStep";
import type { ProjectSetupWizardProps } from "./types";
import { useWizardState } from "./useWizardState";
import { WizardStepIndicator } from "./WizardStepIndicator";
import { WizardFooter } from "./WizardFooter";

const log = logger.scope("Wizard");

/**
 * Project Setup Wizard — prompts the user to select evidence and processed
 * database locations after opening a project directory.
 */
export const ProjectSetupWizard: Component<ProjectSetupWizardProps> = (props) => {
  let modalRef: HTMLDivElement | undefined;
  useFocusTrap(
    () => modalRef,
    () => props.isOpen,
  );

  onMount(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && props.isOpen) {
        props.onClose();
      }
    };
    makeEventListener(document, "keydown", handleEscape);
  });

  const state = useWizardState(props);

  return (
    <Show when={props.isOpen}>
      <div class="wizard-overlay" onClick={(e) => e.target === e.currentTarget && props.onClose()}>
        <div
          ref={modalRef}
          class="wizard-modal"
          role="dialog"
          aria-modal="true"
          aria-labelledby="project-wizard-title"
        >
          {/* Header */}
          <div class="wizard-header">
            <h2 id="project-wizard-title" class="flex items-center gap-2">
              <HiOutlineFolder class="w-5 h-5" /> Project Setup
            </h2>
            <button
              class="wizard-close flex items-center justify-center"
              onClick={props.onClose}
              title="Close"
              aria-label="Close project setup"
            >
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </div>

          {/* Step Indicator */}
          <WizardStepIndicator step={state.step} showHashLoadingStep={state.showHashLoadingStep} />

          {/* Content */}
          <div class="wizard-content">
            <Show when={state.step() === -1}>
              <SelectFolderStep
                error={state.error}
                onBrowse={state.browseProjectRoot}
                onCreateFromTemplate={(name, examiner) =>
                  state.browseAndCreateTemplate(name, examiner)
                }
              />
            </Show>

            <Show when={state.step() === 0}>
              <ScanningStep scanMessage={state.scanMessage} error={state.error} />
            </Show>

            <Show when={state.step() === 1}>
              <ConfigureLocationsStep
                projectName={state.projectName}
                setProjectName={state.setProjectName}
                ownerName={state.ownerName}
                setOwnerName={state.setOwnerName}
                evidencePath={state.evidencePath}
                setEvidencePath={state.setEvidencePath}
                evidenceCount={state.evidenceCount}
                evidenceChips={state.evidenceChips}
                suggestedEvidence={state.suggestedEvidence}
                browseEvidence={state.browseEvidence}
                discoverEvidence={state.discoverEvidence}
                processedDbPath={state.processedDbPath}
                setProcessedDbPath={state.setProcessedDbPath}
                databaseCount={state.databaseCount}
                processedChips={state.processedChips}
                suggestedProcessed={state.suggestedProcessed}
                browseProcessed={state.browseProcessed}
                discoverDatabases={state.discoverDatabases}
                caseDocumentsPath={state.caseDocumentsPath}
                setCaseDocumentsPath={state.setCaseDocumentsPath}
                discoveredCaseDocCount={state.discoveredCaseDocCount}
                caseDocsChips={state.caseDocsChips}
                suggestedCaseDocs={state.suggestedCaseDocs}
                browseCaseDocs={state.browseCaseDocs}
                setDiscoveredCaseDocCount={state.setDiscoveredCaseDocCount}
                loadStoredHashes={state.loadStoredHashes}
                setLoadStoredHashes={state.setLoadStoredHashes}
                onProfileChange={(profileId) => {
                  log.debug(`Profile changed in wizard: ${profileId}`);
                }}
              />
            </Show>

            <Show when={state.step() === 2}>
              <LoadHashesStep
                hashLoadingProgress={state.hashLoadingProgress}
                hashProgressPercent={state.hashProgressPercent}
                loadedStoredHashes={state.loadedStoredHashes}
              />
            </Show>
          </div>

          {/* Footer */}
          <WizardFooter
            step={state.step}
            scanning={state.scanning}
            onClose={props.onClose}
            handleSkip={state.handleSkip}
            handleContinue={state.handleContinue}
            cancelHashLoading={state.cancelHashLoading}
          />
        </div>
      </div>
    </Show>
  );
};
