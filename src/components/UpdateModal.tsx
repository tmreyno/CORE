// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UpdateModal — Thin wrapper around @core-suite/components UpdateModal.
 *
 * Provides CORE-FFX-specific config (app name, repo URL, auth token)
 * while preserving the `show`/`onClose`-only props interface expected
 * by AppSecondaryModals.tsx's lazy import.
 */

import { Component } from "solid-js";
import { UpdateModal as SharedUpdateModal, type UpdaterConfig } from "@core-suite/components";

interface UpdateModalProps {
  show: boolean;
  onClose: () => void;
}

const config: UpdaterConfig = {
  appName: "CORE-FFX",
  repoUrl: "https://github.com/tmreyno/CORE",
  authToken: typeof __GITHUB_UPDATE_TOKEN__ === "string" ? __GITHUB_UPDATE_TOKEN__ : "",
  currentVersion: typeof __APP_VERSION__ === "string" ? __APP_VERSION__ : undefined,
  defaultBranch: "main",
};

const UpdateModal: Component<UpdateModalProps> = (props) => {
  return (
    <SharedUpdateModal
      show={props.show}
      onClose={props.onClose}
      config={config}
    />
  );
};

export default UpdateModal;
