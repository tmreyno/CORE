// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UpdateModal — Thin wrapper around the shared configured updater modal.
 *
 * Provides CORE-FFX-specific config (app name, repo URL, auth token)
 * while preserving the `show`/`onClose`-only props interface expected
 * by AppSecondaryModals.tsx's lazy import.
 */

import {
  createConfiguredUpdateModal,
  type BasicUpdateModalProps,
} from "@core-suite/components/updater";

export type UpdateModalProps = BasicUpdateModalProps;

const UpdateModal = createConfiguredUpdateModal({
  appName: "CORE-FFX",
  repoUrl: "https://github.com/tmreyno/CORE",
  authToken: typeof __GITHUB_UPDATE_TOKEN__ === "string" ? __GITHUB_UPDATE_TOKEN__ : "",
  currentVersion: typeof __APP_VERSION__ === "string" ? __APP_VERSION__ : undefined,
  defaultBranch: "main",
});

export default UpdateModal;
