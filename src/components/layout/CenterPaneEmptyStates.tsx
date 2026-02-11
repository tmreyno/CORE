// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CenterPaneEmptyStates Component
 * 
 * Empty states for the center pane:
 * - No project loaded state
 * - Project loaded but ready for evidence state
 */

import { Component, Show } from "solid-js";
import {
  HiOutlineFolder,
  HiOutlineDocumentPlus,
  HiOutlineArchiveBox,
} from "../icons";

interface CenterPaneEmptyStatesProps {
  hasProject: boolean;
  onOpenProject: () => void;
  onOpenEvidence: () => void;
}

export const CenterPaneEmptyStates: Component<CenterPaneEmptyStatesProps> = (props) => {
  return (
    <>
      {/* No Project Loaded */}
      <Show when={!props.hasProject}>
        <div class="flex flex-col items-center justify-center h-full p-8 text-center">
          <HiOutlineFolder class="w-24 h-24 text-txt-muted mb-4" />
          <h3 class="text-xl font-semibold text-txt mb-2">
            No Project Open
          </h3>
          <p class="text-txt-secondary mb-6 max-w-md">
            Create a new project or open an existing one to get started with forensic analysis.
          </p>
          <button
            onClick={props.onOpenProject}
            class="btn-primary"
          >
            <HiOutlineDocumentPlus class="w-icon-base h-icon-base" />
            Open Project
          </button>
        </div>
      </Show>

      {/* Project Loaded - Ready for Evidence */}
      <Show when={props.hasProject}>
        <div class="flex flex-col items-center justify-center h-full p-8 text-center">
          <HiOutlineArchiveBox class="w-24 h-24 text-txt-muted mb-4" />
          <h3 class="text-xl font-semibold text-txt mb-2">
            Ready to Load Evidence
          </h3>
          <p class="text-txt-secondary mb-6 max-w-md">
            Open an evidence container (AD1, E01, UFED, Archive) to begin your forensic examination.
          </p>
          <button
            onClick={props.onOpenEvidence}
            class="btn-primary"
          >
            <HiOutlineFolder class="w-icon-base h-icon-base" />
            Open Evidence
          </button>
        </div>
      </Show>
    </>
  );
};
