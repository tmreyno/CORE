// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";

// =============================================================================
// Type Definitions - Aligned with backend project_templates.rs
// =============================================================================

/**
 * Project template category - matches backend TemplateCategory
 */
export type TemplateCategory =
  | "Mobile"
  | "Computer"
  | "Network"
  | "Cloud"
  | "IncidentResponse"
  | "Memory"
  | "Malware"
  | "EDiscovery"
  | "General"
  | "Custom";

/**
 * Template summary for listing - matches backend TemplateSummary
 */
export interface TemplateSummary {
  id: string;
  name: string;
  category: TemplateCategory;
  description: string;
  tags: string[];
  usage_count: number;
}

/**
 * Full template with content - matches backend ProjectTemplate
 */
export interface ProjectTemplate {
  id: string;
  name: string;
  category: TemplateCategory;
  description: string;
  author: string;
  version: string;
  created_at: string;
  updated_at: string;
  usage_count: number;
  tags: string[];
  bookmarks: BookmarkTemplate[];
  notes: NoteTemplate[];
  tabs: TabTemplate[];
  hash_algorithms: string[];
  recommended_tools: string[];
  checklist: ChecklistItem[];
  metadata_fields: MetadataField[];
  workspace_profile: string | null;
}

export interface BookmarkTemplate {
  name: string;
  description: string;
  category: string;
  tags: string[];
}

export interface NoteTemplate {
  title: string;
  content: string;
  category: string;
  tags: string[];
}

export interface TabTemplate {
  name: string;
  tab_type: string;
  config: Record<string, unknown>;
}

export interface ChecklistItem {
  id: string;
  text: string;
  category: string;
  required: boolean;
  completed: boolean;
  help: string | null;
}

export interface MetadataField {
  name: string;
  field_type: string;
  default_value: string | null;
  required: boolean;
  help: string | null;
}

// =============================================================================
// Hook Implementation
// =============================================================================

/**
 * Hook for project template management
 */
export function useProjectTemplates() {
  const [templates, setTemplates] = createSignal<TemplateSummary[]>([]);
  const [currentTemplate, setCurrentTemplate] = createSignal<ProjectTemplate | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * List all available templates
   */
  const listTemplates = async (): Promise<TemplateSummary[]> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<TemplateSummary[]>("template_list");
      setTemplates(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to list templates:", err);
      return [];
    } finally {
      setLoading(false);
    }
  };

  /**
   * Get full template details
   */
  const getTemplate = async (templateId: string): Promise<ProjectTemplate | null> => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ProjectTemplate>("template_get", {
        templateId,
      });
      setCurrentTemplate(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to get template:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Apply template to current project
   */
  const applyTemplate = async (
    projectPath: string,
    templateId: string
  ): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("template_apply", {
        projectPath,
        templateId,
      });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to apply template:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Create template from current project
   */
  const createFromProject = async (
    projectPath: string,
    name: string,
    category: TemplateCategory,
    description: string = ""
  ): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const templateId = await invoke<string>("template_create_from_project", {
        projectPath,
        name,
        category,
        description,
      });
      // Refresh template list
      await listTemplates();
      return templateId;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to create template from project:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Delete a custom template
   */
  const deleteTemplate = async (templateId: string): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("template_delete", { templateId });
      // Refresh template list
      await listTemplates();
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to delete template:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Export template to file
   */
  const exportTemplate = async (
    templateId: string,
    outputPath: string
  ): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("template_export", {
        templateId,
        outputPath,
      });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to export template:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Import template from file
   */
  const importTemplate = async (filePath: string): Promise<string | null> => {
    try {
      setLoading(true);
      setError(null);
      const templateId = await invoke<string>("template_import", { filePath });
      // Refresh template list
      await listTemplates();
      return templateId;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to import template:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Get templates by category
   */
  const getByCategory = (category: TemplateCategory) => {
    return templates().filter((t) => t.category === category);
  };

  /**
   * Check if a template is built-in (by checking known IDs)
   */
  const isBuiltinTemplate = (templateId: string): boolean => {
    const builtinIds = [
      "mobile_forensics",
      "computer_forensics",
      "incident_response",
      "malware_analysis",
      "ediscovery",
    ];
    return builtinIds.includes(templateId);
  };

  /**
   * Get builtin templates only
   */
  const getBuiltinTemplates = () => {
    return templates().filter((t) => isBuiltinTemplate(t.id));
  };

  /**
   * Get custom templates only
   */
  const getCustomTemplates = () => {
    return templates().filter((t) => !isBuiltinTemplate(t.id));
  };

  return {
    // State
    templates,
    currentTemplate,
    loading,
    error,
    // Actions
    listTemplates,
    getTemplate,
    applyTemplate,
    createFromProject,
    deleteTemplate,
    exportTemplate,
    importTemplate,
    // Utilities
    getByCategory,
    isBuiltinTemplate,
    getBuiltinTemplates,
    getCustomTemplates,
  };
}
