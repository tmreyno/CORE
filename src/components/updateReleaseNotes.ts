// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import bundledUpdateReview from "../../docs/releases/core-ffx-0.1.112-0.1.114.md?raw";

const releaseOrder = [
  "0.1.112",
  "0.1.113",
  "0.1.114",
  "0.1.115",
  "0.1.116",
] as const;
type BundledReleaseVersion = (typeof releaseOrder)[number];

const releaseSectionPattern =
  /<!-- release-note:start ([\d.]+) -->([\s\S]*?)<!-- release-note:end \1 -->/g;

function normalizeVersion(version: string | null | undefined): string {
  return (version ?? "").trim().replace(/^v/i, "");
}

function parseVersion(version: string): number[] {
  return normalizeVersion(version)
    .split(".")
    .map((part) => Number.parseInt(part, 10))
    .map((part) => (Number.isFinite(part) ? part : 0));
}

function compareVersions(left: string, right: string): number {
  const leftParts = parseVersion(left);
  const rightParts = parseVersion(right);
  const max = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < max; index += 1) {
    const diff = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (diff !== 0) return diff;
  }

  return 0;
}

function getReleaseSections(): Map<string, string> {
  const sections = new Map<string, string>();
  for (const match of bundledUpdateReview.matchAll(releaseSectionPattern)) {
    sections.set(match[1], match[2].trim());
  }
  return sections;
}

function isBundledReleaseVersion(
  version: string
): version is BundledReleaseVersion {
  return releaseOrder.includes(version as BundledReleaseVersion);
}

function selectBundledVersions(
  updateVersion: string,
  currentVersion: string
): BundledReleaseVersion[] {
  const normalizedUpdateVersion = normalizeVersion(updateVersion);
  const normalizedCurrentVersion = normalizeVersion(currentVersion);

  if (!isBundledReleaseVersion(normalizedUpdateVersion)) {
    return [];
  }

  if (!normalizedCurrentVersion) {
    return releaseOrder.filter(
      (version) => compareVersions(version, normalizedUpdateVersion) <= 0
    );
  }

  return releaseOrder.filter(
    (version) =>
      compareVersions(version, normalizedCurrentVersion) > 0 &&
      compareVersions(version, normalizedUpdateVersion) <= 0
  );
}

function hasBundledNotes(
  remoteBody: string,
  updateVersion: string | null | undefined,
  currentVersion: string | null | undefined
): boolean {
  const selectedVersions = selectBundledVersions(
    normalizeVersion(updateVersion),
    normalizeVersion(currentVersion)
  );

  return (
    selectedVersions.length > 0 &&
    selectedVersions.every((version) =>
      remoteBody.includes(`CORE-FFX ${version}`)
    )
  );
}

export function getBundledUpdateReleaseNotes(
  updateVersion: string | null | undefined,
  currentVersion: string | null | undefined
): string {
  const selectedVersions = selectBundledVersions(
    normalizeVersion(updateVersion),
    normalizeVersion(currentVersion)
  );

  if (selectedVersions.length === 0) {
    return "";
  }

  const sections = getReleaseSections();
  const notes = selectedVersions
    .slice()
    .reverse()
    .map((version) => sections.get(version))
    .filter((section): section is string => Boolean(section));

  if (notes.length === 0) {
    return "";
  }

  return [
    "# CORE-FFX Update Review",
    "These notes are bundled with the app so you can review the available update without opening a website.",
    ...notes,
  ].join("\n\n");
}

export function mergeUpdateReleaseNotes(
  remoteBody: string | null | undefined,
  updateVersion: string | null | undefined,
  currentVersion: string | null | undefined
): string {
  const trimmedRemoteBody = (remoteBody ?? "").trim();
  const bundledNotes = getBundledUpdateReleaseNotes(
    updateVersion,
    currentVersion
  );

  if (
    !bundledNotes ||
    hasBundledNotes(trimmedRemoteBody, updateVersion, currentVersion)
  ) {
    return trimmedRemoteBody;
  }

  if (!trimmedRemoteBody) {
    return bundledNotes;
  }

  return [
    bundledNotes,
    "---",
    "### Release Manifest Notes",
    trimmedRemoteBody,
  ].join("\n\n");
}
