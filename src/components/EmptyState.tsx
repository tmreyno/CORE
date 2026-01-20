// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, JSX, Component } from "solid-js";
import {
  HiOutlineInbox,
  HiOutlineMagnifyingGlass,
  HiOutlineExclamationTriangle,
  HiOutlineArrowUpTray,
  HiOutlineFolder,
  HiOutlineCircleStack,
} from "./icons";

export type EmptyStateVariant = "default" | "search" | "error" | "upload" | "folder" | "database";

interface EmptyStateProps {
  variant?: EmptyStateVariant;
  title: string;
  description?: string;
  icon?: JSX.Element;
  action?: {
    label: string;
    onClick: () => void;
  };
  secondaryAction?: {
    label: string;
    onClick: () => void;
  };
  children?: JSX.Element;
}

// Icon components for each variant
const VariantIcon: Component<{ variant: EmptyStateVariant; class?: string }> = (props) => {
  const iconClass = () => props.class || "w-12 h-12";
  
  switch (props.variant) {
    case "search":
      return <HiOutlineMagnifyingGlass class={iconClass()} />;
    case "error":
      return <HiOutlineExclamationTriangle class={iconClass()} />;
    case "upload":
      return <HiOutlineArrowUpTray class={iconClass()} />;
    case "folder":
      return <HiOutlineFolder class={iconClass()} />;
    case "database":
      return <HiOutlineCircleStack class={iconClass()} />;
    default:
      return <HiOutlineInbox class={iconClass()} />;
  }
};

// Default colors for each variant
const variantColors: Record<EmptyStateVariant, string> = {
  default: "text-txt-secondary",
  search: "text-accent",
  error: "text-red-400",
  upload: "text-green-400",
  folder: "text-amber-400",
  database: "text-purple-400",
};

export function EmptyState(props: EmptyStateProps) {
  const variant = () => props.variant ?? "default";
  const iconColor = () => variantColors[variant()];

  return (
    <div class="flex flex-col items-center justify-center px-8 py-12 text-center">
      {/* Icon */}
      <div class={`mb-4 ${iconColor()} opacity-80`}>
        <Show when={props.icon} fallback={<VariantIcon variant={variant()} />}>
          {props.icon}
        </Show>
      </div>

      {/* Title */}
      <h3 class="text-lg font-semibold text-txt mb-2">
        {props.title}
      </h3>

      {/* Description */}
      <Show when={props.description}>
        <p class="text-sm text-txt-secondary max-w-sm mb-6 leading-relaxed">
          {props.description}
        </p>
      </Show>

      {/* Custom content */}
      <Show when={props.children}>
        <div class="mb-6">
          {props.children}
        </div>
      </Show>

      {/* Actions */}
      <Show when={props.action || props.secondaryAction}>
        <div class="flex items-center gap-3">
          <Show when={props.action}>
            <button
              class="px-4 py-2 bg-accent hover:bg-accent-hover text-white text-sm font-medium rounded-lg transition-colors"
              onClick={props.action!.onClick}
            >
              {props.action!.label}
            </button>
          </Show>
          <Show when={props.secondaryAction}>
            <button
              class="px-4 py-2 bg-bg-panel hover:bg-bg-hover border border-border text-txt-tertiary text-sm font-medium rounded-lg transition-colors"
              onClick={props.secondaryAction!.onClick}
            >
              {props.secondaryAction!.label}
            </button>
          </Show>
        </div>
      </Show>
    </div>
  );
}

// Pre-built empty states for common scenarios
export function NoFilesEmptyState(props: { onBrowse: () => void }) {
  return (
    <EmptyState
      variant="folder"
      title="No Evidence Files"
      description="Browse to a folder containing forensic container files (E01, AD1, L01, etc.) to get started."
      action={{
        label: "Browse Folder",
        onClick: props.onBrowse,
      }}
    />
  );
}

export function NoSearchResultsEmptyState(props: { query: string; onClear: () => void }) {
  return (
    <EmptyState
      variant="search"
      title="No Results Found"
      description={`No files match "${props.query}". Try a different search term or clear the filter.`}
      action={{
        label: "Clear Search",
        onClick: props.onClear,
      }}
    />
  );
}

export function ErrorEmptyState(props: { error: string; onRetry?: () => void }) {
  return (
    <EmptyState
      variant="error"
      title="Something Went Wrong"
      description={props.error}
      action={props.onRetry ? {
        label: "Try Again",
        onClick: props.onRetry,
      } : undefined}
    />
  );
}

export function NoDatabasesEmptyState(props: { onScan: () => void; onAdd: () => void }) {
  return (
    <EmptyState
      variant="database"
      title="No Processed Databases"
      description="Add processed databases from AXIOM, Cellebrite, or other forensic tools to analyze artifacts."
      action={{
        label: "Scan Folder",
        onClick: props.onScan,
      }}
      secondaryAction={{
        label: "Add Database",
        onClick: props.onAdd,
      }}
    />
  );
}

export function DropZoneEmptyState(props: { onBrowse: () => void }) {
  return (
    <EmptyState
      variant="upload"
      title="Drop Files Here"
      description="Drag and drop forensic container files, or click to browse."
      action={{
        label: "Browse Files",
        onClick: props.onBrowse,
      }}
    >
      <div class="flex items-center gap-2 text-xs text-txt-muted">
        <span class="px-2 py-1 bg-bg-panel rounded">E01</span>
        <span class="px-2 py-1 bg-bg-panel rounded">AD1</span>
        <span class="px-2 py-1 bg-bg-panel rounded">L01</span>
        <span class="px-2 py-1 bg-bg-panel rounded">ZIP</span>
        <span class="px-2 py-1 bg-bg-panel rounded">7Z</span>
      </div>
    </EmptyState>
  );
}
