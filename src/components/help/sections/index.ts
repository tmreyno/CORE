// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Re-exports all help section content components and the HELP_SECTIONS registry.
 */


import {
  HiOutlineQuestionMarkCircle,
  HiOutlineArchiveBox,
  HiOutlineFingerPrint,
  HiOutlineArrowUpTray,
  HiOutlineClipboardDocumentList,
  HiOutlineMagnifyingGlass,
  HiOutlineFolder,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineRectangleGroup,
  HiOutlineCommandLine,
  HiOutlineLockClosed,
  HiOutlineEye,
  HiOutlineCircleStack,
  HiOutlineBookmark,
  HiOutlineChartBar,
} from "../../icons";
import type { HelpSection } from "../types";
import { GettingStartedContent } from "./GettingStarted";
import { EvidenceContainersContent } from "./EvidenceContainers";
import { FileViewersContent } from "./FileViewers";
import { HashVerificationContent } from "./HashVerification";
import { SearchContent } from "./Search";
import { ExportContent } from "./Export";
import { ReportsContent } from "./Reports";
import { ChainOfCustodyContent } from "./ChainOfCustody";
import { EvidenceCollectionContent } from "./EvidenceCollection";
import { ProcessedDatabasesContent } from "./ProcessedDatabases";
import { ProjectManagementContent } from "./ProjectManagement";
import { FilesystemsContent } from "./Filesystems";
import { BookmarksNotesContent } from "./BookmarksNotes";
import { KeyboardShortcutsContent } from "./KeyboardShortcuts";
import { AboutContent } from "./About";

// Re-export individual section components
export { GettingStartedContent } from "./GettingStarted";
export { EvidenceContainersContent } from "./EvidenceContainers";
export { FileViewersContent, ViewerCard } from "./FileViewers";
export { HashVerificationContent } from "./HashVerification";
export { SearchContent } from "./Search";
export { ExportContent } from "./Export";
export { ReportsContent } from "./Reports";
export { ChainOfCustodyContent } from "./ChainOfCustody";
export { EvidenceCollectionContent } from "./EvidenceCollection";
export { ProcessedDatabasesContent } from "./ProcessedDatabases";
export { ProjectManagementContent } from "./ProjectManagement";
export { FilesystemsContent } from "./Filesystems";
export { BookmarksNotesContent } from "./BookmarksNotes";
export { KeyboardShortcutsContent, ShortcutGroup } from "./KeyboardShortcuts";
export { AboutContent } from "./About";

// =============================================================================
// Section Registry — ordered list of all help sections
// =============================================================================

export const HELP_SECTIONS: HelpSection[] = [
  { id: "getting-started", title: "Getting Started", icon: HiOutlineRectangleGroup, content: GettingStartedContent },
  { id: "evidence-containers", title: "Evidence Containers", icon: HiOutlineArchiveBox, content: EvidenceContainersContent },
  { id: "file-viewers", title: "File Viewers", icon: HiOutlineEye, content: FileViewersContent },
  { id: "hash-verification", title: "Hash Verification", icon: HiOutlineFingerPrint, content: HashVerificationContent },
  { id: "search", title: "Search & Deduplication", icon: HiOutlineMagnifyingGlass, content: SearchContent },
  { id: "export", title: "Export Formats", icon: HiOutlineArrowUpTray, content: ExportContent },
  { id: "reports", title: "Reports", icon: HiOutlineClipboardDocumentList, content: ReportsContent },
  { id: "chain-of-custody", title: "Chain of Custody", icon: HiOutlineLockClosed, content: ChainOfCustodyContent },
  { id: "evidence-collection", title: "Evidence Collection", icon: HiOutlineArchiveBoxArrowDown, content: EvidenceCollectionContent },
  { id: "processed-databases", title: "Processed Databases", icon: HiOutlineChartBar, content: ProcessedDatabasesContent },
  { id: "project-management", title: "Project Management", icon: HiOutlineFolder, content: ProjectManagementContent },
  { id: "filesystems", title: "Filesystem Drivers", icon: HiOutlineCircleStack, content: FilesystemsContent },
  { id: "bookmarks-notes", title: "Bookmarks & Notes", icon: HiOutlineBookmark, content: BookmarksNotesContent },
  { id: "keyboard-shortcuts", title: "Keyboard Shortcuts", icon: HiOutlineCommandLine, content: KeyboardShortcutsContent },
  { id: "about", title: "About CORE-FFX", icon: HiOutlineQuestionMarkCircle, content: AboutContent },
];
