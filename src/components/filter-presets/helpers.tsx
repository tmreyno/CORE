// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import {
  HiOutlineFunnel,
  HiOutlineDocument,
  HiOutlinePhoto,
  HiOutlineFilm,
  HiOutlineMusicalNote,
  HiOutlineArchiveBox,
  HiOutlineCodeBracket,
  HiOutlineCircleStack,
  HiOutlineEnvelope,
} from "../icons";

/** Maps an icon name string to its Heroicon outline component. */
export const getQuickFilterIcon = (iconName?: string): Component<{ class?: string }> => {
  switch (iconName) {
    case "document": return HiOutlineDocument;
    case "photo": return HiOutlinePhoto;
    case "film": return HiOutlineFilm;
    case "music": return HiOutlineMusicalNote;
    case "archive": return HiOutlineArchiveBox;
    case "code": return HiOutlineCodeBracket;
    case "database": return HiOutlineCircleStack;
    case "email": return HiOutlineEnvelope;
    default: return HiOutlineFunnel;
  }
};
