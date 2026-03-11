// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { HiOutlineFolder, HiOutlineDocument, HiOutlineBookmark, HiOutlineDocumentText, HiOutlineClock } from "../icons";
import { formatBytes } from "../../utils";
import { getBasename } from "../../utils/pathUtils";
import type { SearchResult } from "./types";

interface SearchResultItemProps {
  result: SearchResult;
  isSelected: boolean;
  onSelect: () => void;
  onMouseEnter: () => void;
}

/** Map FTS source types to badge styling */
const SOURCE_BADGE: Record<string, { label: string; color: string }> = {
  bookmarks: { label: "Bookmark", color: "bg-accent/20 text-accent" },
  notes: { label: "Note", color: "bg-info/20 text-info" },
  activity: { label: "Activity", color: "bg-warning/20 text-warning" },
};

/** Pick icon based on result source */
const ResultIcon: Component<{ result: SearchResult }> = (props) => {
  if (props.result.id.startsWith("fts:")) {
    const source = props.result.matchType || "";
    if (source === "bookmarks") return <HiOutlineBookmark class="w-5 h-5 text-accent" />;
    if (source === "notes") return <HiOutlineDocumentText class="w-5 h-5 text-info" />;
    if (source === "activity") return <HiOutlineClock class="w-5 h-5 text-warning" />;
    return <HiOutlineDocument class="w-5 h-5 text-txt-secondary" />;
  }
  if (props.result.isDir) return <HiOutlineFolder class="w-5 h-5 text-amber-400" />;
  return <HiOutlineDocument class="w-5 h-5 text-txt-secondary" />;
};

export const SearchResultItem: Component<SearchResultItemProps> = (props) => {
  const isFts = () => props.result.id.startsWith("fts:");
  const sourceBadge = () => SOURCE_BADGE[props.result.matchType || ""] ?? null;

  return (
    <button
      class={`w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors ${
        props.isSelected ? "bg-accent/20" : "hover:bg-bg-panel"
      }`}
      onClick={props.onSelect}
      onMouseEnter={props.onMouseEnter}
    >
      <span class="shrink-0">
        <ResultIcon result={props.result} />
      </span>
      <div class="flex-1 min-w-0">
        {/* FTS cross-entity result */}
        <Show when={isFts()}>
          <div class="flex items-center gap-1.5 mb-0.5">
            <Show when={sourceBadge()}>
              {(badge) => (
                <span class={`px-1.5 py-0.5 rounded text-2xs font-medium uppercase ${badge().color}`}>
                  {badge().label}
                </span>
              )}
            </Show>
          </div>
          <div
            class="text-sm text-txt truncate [&>mark]:bg-accent/30 [&>mark]:text-accent [&>mark]:rounded-sm [&>mark]:px-0.5"
            innerHTML={props.result.matchContext || props.result.name}
          />
          <div class="text-xs text-txt-muted truncate mt-0.5">{props.result.path}</div>
        </Show>

        {/* Container entry result */}
        <Show when={!isFts() && props.result.containerPath}>
          <div class="text-sm text-txt truncate">{props.result.name}</div>
          <div class="text-xs text-accent truncate flex items-center gap-1">
            <span class="px-1 py-0.5 bg-accent/20 rounded text-2xs uppercase">
              {props.result.containerType || "container"}
            </span>
            <span class="truncate">
              {getBasename(props.result.containerPath || "")}
            </span>
            <span class="text-txt-muted">→</span>
            <span class="truncate text-txt-secondary">{props.result.path}</span>
          </div>
        </Show>

        {/* Regular file result */}
        <Show when={!isFts() && !props.result.containerPath}>
          <div class="text-sm text-txt truncate">{props.result.name}</div>
          <div class="text-xs text-txt-muted truncate">{props.result.path}</div>
        </Show>

        <Show when={!isFts() && props.result.matchContext}>
          <div class="text-xs text-txt-secondary mt-0.5 truncate">
            ...{props.result.matchContext}...
          </div>
        </Show>
      </div>
      <Show when={!isFts()}>
        <span class="file-size">{formatBytes(props.result.size)}</span>
      </Show>
    </button>
  );
};
