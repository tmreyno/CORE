// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Simple hash verification indicator
 * Used in places where just the status icon/text is needed
 */
export function HashVerificationIndicator(props: {
  verified: boolean | null | undefined;
  class?: string;
}) {
  const colorClass = () =>
    props.verified === true
      ? "text-success"
      : props.verified === false
        ? "text-error"
        : "text-txt/60";

  const indicator = () =>
    props.verified === true ? " ✓" : props.verified === false ? " ✗" : "";

  return (
    <span class={`${colorClass()} ${props.class ?? ""}`}>{indicator()}</span>
  );
}
