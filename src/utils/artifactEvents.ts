// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

export const ARTIFACTS_UPDATED_EVENT = "core-ffx:artifacts-updated";

export interface ArtifactsUpdatedDetail {
  evidenceFileId: string;
  category?: string;
}

export function notifyArtifactsUpdated(detail: ArtifactsUpdatedDetail): void {
  if (!detail.evidenceFileId || typeof window === "undefined") return;
  window.dispatchEvent(new CustomEvent<ArtifactsUpdatedDetail>(ARTIFACTS_UPDATED_EVENT, {
    detail,
  }));
}

export function listenForArtifactsUpdated(
  callback: (detail: ArtifactsUpdatedDetail) => void,
): () => void {
  if (typeof window === "undefined") return () => undefined;
  const listener = (event: Event) => {
    callback((event as CustomEvent<ArtifactsUpdatedDetail>).detail);
  };
  window.addEventListener(ARTIFACTS_UPDATED_EVENT, listener);
  return () => window.removeEventListener(ARTIFACTS_UPDATED_EVENT, listener);
}
