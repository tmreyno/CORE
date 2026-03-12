// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ProjectNote } from "../../types/project";

// =============================================================================
// Props interfaces
// =============================================================================

export interface NotesPanelProps {
  /** Notes from the project */
  notes: ProjectNote[];
  /** Handler to navigate to a note's target */
  onNavigate?: (note: ProjectNote) => void;
  /** Handler to remove a note */
  onRemove?: (noteId: string) => void;
  /** Handler to update a note's properties */
  onUpdate?: (noteId: string, updates: Partial<Pick<ProjectNote, "title" | "content" | "tags" | "priority">>) => void;
  /** Handler to create a new note */
  onCreate?: (note: { target_type: ProjectNote["target_type"]; target_path?: string; title: string; content: string; priority?: ProjectNote["priority"] }) => void;
  /** Loading state */
  loading?: boolean;
}

export interface NoteItemProps {
  note: ProjectNote;
  onNavigate?: (note: ProjectNote) => void;
  onRemove?: (noteId: string) => void;
  onEdit?: (note: ProjectNote) => void;
}

export interface NoteEditDialogProps {
  /** Note to edit, or null for creating a new note */
  note: ProjectNote | null;
  onSave: (noteId: string | null, data: { title: string; content: string; tags?: string[]; priority?: ProjectNote["priority"] }) => void;
  onCancel: () => void;
}

export type NoteTargetType = ProjectNote["target_type"];

export const NOTE_PRIORITIES: { value: ProjectNote["priority"]; label: string; color: string }[] = [
  { value: "low", label: "Low", color: "text-txt-muted" },
  { value: "normal", label: "Normal", color: "text-txt" },
  { value: "high", label: "High", color: "text-warning" },
  { value: "critical", label: "Critical", color: "text-error" },
];
