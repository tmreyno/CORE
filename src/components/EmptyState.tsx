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
import { ShortcutHint, CommonShortcuts, Shortcut } from "./ui/Kbd";

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
  error: "text-error",
  upload: "text-success",
  folder: "text-warning",
  database: "text-type-ufed",
};

// Gradient backgrounds for visual appeal
const variantGradients: Record<EmptyStateVariant, string> = {
  default: "from-txt-secondary/5 to-transparent",
  search: "from-accent/10 to-transparent",
  error: "from-error/10 to-transparent",
  upload: "from-success/10 to-transparent",
  folder: "from-warning/10 to-transparent",
  database: "from-type-ufed/10 to-transparent",
};

export function EmptyState(props: EmptyStateProps) {
  const variant = () => props.variant ?? "default";
  const iconColor = () => variantColors[variant()];
  const gradient = () => variantGradients[variant()];

  return (
    <div class="flex flex-col items-center justify-center px-8 py-16 text-center animate-fade-in">
      {/* Decorative background */}
      <div class={`absolute inset-0 bg-gradient-radial ${gradient()} opacity-50 pointer-events-none`} />
      
      {/* Icon with subtle animation */}
      <div class={`mb-5 ${iconColor()} relative`}>
        <div class="absolute inset-0 blur-xl opacity-30 animate-pulse-slow">
          <Show when={props.icon} fallback={<VariantIcon variant={variant()} class="w-16 h-16" />}>
            {props.icon}
          </Show>
        </div>
        <Show when={props.icon} fallback={<VariantIcon variant={variant()} class="w-14 h-14 relative" />}>
          <div class="relative">{props.icon}</div>
        </Show>
      </div>

      {/* Title */}
      <h3 class="text-xl font-semibold text-txt mb-2">
        {props.title}
      </h3>

      {/* Description */}
      <Show when={props.description}>
        <p class="text-sm text-txt-secondary max-w-md mb-6 leading-relaxed">
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
              class="px-5 py-2.5 bg-accent hover:bg-accent-hover text-white text-sm font-medium rounded-lg transition-all duration-200 hover:shadow-lg hover:shadow-accent/20 focus:outline-none focus:ring-2 focus:ring-accent/50 focus:ring-offset-2 focus:ring-offset-bg"
              onClick={props.action!.onClick}
            >
              {props.action!.label}
            </button>
          </Show>
          <Show when={props.secondaryAction}>
            <button
              class="px-5 py-2.5 bg-bg-panel hover:bg-bg-hover border border-border text-txt-secondary hover:text-txt text-sm font-medium rounded-lg transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-border/50"
              onClick={props.secondaryAction!.onClick}
            >
              {props.secondaryAction!.label}
            </button>
          </Show>
        </div>
      </Show>
      
      {/* Keyboard hints */}
      <Show when={props.action}>
        <p class="mt-6 text-xs text-txt-muted flex items-center gap-2">
          Press <Shortcut {...CommonShortcuts.open} /> to browse files
        </p>
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
      description="Browse to a folder containing forensic container files to get started. Supported formats include E01, AD1, L01, and more."
      action={{
        label: "Browse Folder",
        onClick: props.onBrowse,
      }}
    >
      <div class="flex flex-wrap items-center justify-center gap-2 text-xs text-txt-muted mb-2">
        <span class="px-2.5 py-1 bg-type-e01/10 text-type-e01 rounded-full border border-type-e01/20">E01</span>
        <span class="px-2.5 py-1 bg-type-ad1/10 text-type-ad1 rounded-full border border-type-ad1/20">AD1</span>
        <span class="px-2.5 py-1 bg-type-l01/10 text-type-l01 rounded-full border border-type-l01/20">L01</span>
        <span class="px-2.5 py-1 bg-type-ufed/10 text-type-ufed rounded-full border border-type-ufed/20">UFDR</span>
        <span class="px-2.5 py-1 bg-type-archive/10 text-type-archive rounded-full border border-type-archive/20">7Z/ZIP</span>
      </div>
    </EmptyState>
  );
}

export function NoSearchResultsEmptyState(props: { query: string; onClear: () => void }) {
  return (
    <EmptyState
      variant="search"
      title="No Results Found"
      description={`No files match "${props.query}". Try a different search term, adjust filters, or clear the search.`}
      action={{
        label: "Clear Search",
        onClick: props.onClear,
      }}
    >
      <p class="text-xs text-txt-muted">
        Tip: Use wildcards like <code class="px-1 bg-bg-secondary rounded">*.pdf</code> or partial names
      </p>
    </EmptyState>
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
    >
      <p class="text-xs text-txt-muted">
        If the problem persists, check the console for details or contact support.
      </p>
    </EmptyState>
  );
}

export function NoDatabasesEmptyState(props: { onScan: () => void; onAdd: () => void }) {
  return (
    <EmptyState
      variant="database"
      title="No Processed Databases"
      description="Add processed databases from forensic tools like AXIOM, Cellebrite, or X-Ways to analyze extracted artifacts."
      action={{
        label: "Scan Folder",
        onClick: props.onScan,
      }}
      secondaryAction={{
        label: "Add Database",
        onClick: props.onAdd,
      }}
    >
      <div class="flex items-center gap-3 text-xs text-txt-muted">
        <span class="flex items-center gap-1">
          <HiOutlineCircleStack class="w-3.5 h-3.5" />
          SQLite
        </span>
        <span class="flex items-center gap-1">
          <HiOutlineCircleStack class="w-3.5 h-3.5" />
          Case Files
        </span>
      </div>
    </EmptyState>
  );
}

export function DropZoneEmptyState(props: { onBrowse: () => void }) {
  return (
    <EmptyState
      variant="upload"
      title="Drop Files Here"
      description="Drag and drop forensic container files, or click to browse your file system."
      action={{
        label: "Browse Files",
        onClick: props.onBrowse,
      }}
    >
      <div class="flex flex-wrap items-center justify-center gap-2 text-xs">
        <span class="px-2.5 py-1 bg-type-e01/10 text-type-e01 rounded-full border border-type-e01/20 hover:bg-type-e01/20 transition-colors">E01</span>
        <span class="px-2.5 py-1 bg-type-ad1/10 text-type-ad1 rounded-full border border-type-ad1/20 hover:bg-type-ad1/20 transition-colors">AD1</span>
        <span class="px-2.5 py-1 bg-type-l01/10 text-type-l01 rounded-full border border-type-l01/20 hover:bg-type-l01/20 transition-colors">L01</span>
        <span class="px-2.5 py-1 bg-type-raw/10 text-type-raw rounded-full border border-type-raw/20 hover:bg-type-raw/20 transition-colors">RAW</span>
        <span class="px-2.5 py-1 bg-type-archive/10 text-type-archive rounded-full border border-type-archive/20 hover:bg-type-archive/20 transition-colors">ZIP/7Z</span>
      </div>
    </EmptyState>
  );
}
