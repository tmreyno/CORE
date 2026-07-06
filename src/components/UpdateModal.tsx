// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UpdateModal — CORE-FFX wrapper around the shared configured updater modal.
 *
 * Provides CORE-FFX-specific config and bundled release notes while preserving
 * the `show`/`onClose`-only props interface expected by AppSecondaryModals.tsx's
 * lazy import.
 */

import { createEffect, on, type Component } from "solid-js";
import {
  UpdateModal as SharedUpdateModal,
  useUpdater,
  type BasicUpdateModalProps,
  type UpdaterConfig,
  type UpdaterState,
} from "@core-suite/components/updater";
import { mergeUpdateReleaseNotes } from "./updateReleaseNotes";

export type UpdateModalProps = BasicUpdateModalProps;

const updaterConfig: UpdaterConfig = {
  appName: "CORE-FFX",
  repoUrl: "https://github.com/tmreyno/CORE",
  authToken:
    typeof __GITHUB_UPDATE_TOKEN__ === "string" ? __GITHUB_UPDATE_TOKEN__ : "",
  currentVersion:
    typeof __APP_VERSION__ === "string" ? __APP_VERSION__ : undefined,
  defaultBranch: "main",
};

const UpdateModal: Component<UpdateModalProps> = (props) => {
  const updater = useUpdater({ ...updaterConfig, autoCheck: false });

  createEffect(
    on(
      () => props.show,
      (show) => {
        if (show) {
          void updater.checkForUpdates();
        }
      }
    )
  );

  const updateBody = () =>
    mergeUpdateReleaseNotes(
      updater.updateBody(),
      updater.updateVersion(),
      updater.currentVersion()
    );

  const updaterWithBundledNotes: UpdaterState = {
    ...updater,
    updateBody,
  };

  return (
    <SharedUpdateModal
      show={props.show}
      onClose={props.onClose}
      config={updaterConfig}
      updater={updaterWithBundledNotes}
    />
  );
};

export default UpdateModal;
