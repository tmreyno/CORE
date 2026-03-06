// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import {
  HiOutlineBookmark,
  HiOutlineDocument,
  HiOutlineTag,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineMapPin,
} from "../icons";
import type { BookmarkTargetType } from "./types";

// =============================================================================
// Icon / Label helpers
// =============================================================================

export const getTargetTypeIcon = (type: BookmarkTargetType): Component<{ class?: string }> => {
  switch (type) {
    case "file":
      return HiOutlineDocument;
    case "artifact":
      return HiOutlineTag;
    case "search_result":
      return HiOutlineDocumentMagnifyingGlass;
    case "location":
      return HiOutlineMapPin;
    default:
      return HiOutlineBookmark;
  }
};

export const getTargetTypeLabel = (type: BookmarkTargetType): string => {
  switch (type) {
    case "file":
      return "Files";
    case "artifact":
      return "Artifacts";
    case "search_result":
      return "Search Results";
    case "location":
      return "Locations";
    default:
      return "Other";
  }
};

// =============================================================================
// Color helpers
// =============================================================================

export const getBookmarkColorClass = (color?: string): string => {
  if (!color) return "text-accent";
  const colorMap: Record<string, string> = {
    red: "text-error",
    yellow: "text-warning",
    green: "text-success",
    blue: "text-info",
    purple: "text-accent",
    orange: "text-type-ad1",
  };
  return colorMap[color.toLowerCase()] || "text-accent";
};

export const BOOKMARK_COLORS = [
  { value: "", label: "Default", class: "bg-accent" },
  { value: "red", label: "Red", class: "bg-error" },
  { value: "yellow", label: "Yellow", class: "bg-warning" },
  { value: "green", label: "Green", class: "bg-success" },
  { value: "blue", label: "Blue", class: "bg-info" },
  { value: "purple", label: "Purple", class: "bg-accent" },
  { value: "orange", label: "Orange", class: "bg-type-ad1" },
];
