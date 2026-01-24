// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";

/**
 * Project template category
 */
export type TemplateCategory =
  | "investigation"
  | "malware_analysis"
  | "incident_response"
  | "mobile_forensics"
  | "data_breach"
  | "custom";

/**
 * Template summary for listing
 */
export interface TemplateSummary {
  id: string;
  name: string;
  category: TemplateCategory;
  description: string;
  bookmark_count: number;
  note_count: number;
  is_builtin: boolean;
  created_at: string;
}

/**
 * Full template with content
 */
export interface ProjectTemplate {
  id: string;
  name: string;
  description: string;
  category: TemplateCategory;
  created_at: string;
  created_by: string;
  version: string;
  tags: string[];
  bookmarks: BookmarkTemplate[];
  notes: NoteTemplate[];
  workflows: WorkflowTemplate[];
  is_builtin: boolean;
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

export interface WorkflowTemplate {
  name: string;
  steps: string[];
  description: string;
}

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
  const listTemplates = async () => {
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
  const getTemplate = async (templateId: string) => {
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
   * Get builtin templates only
   */
  const getBuiltinTemplates = () => {
    return templates().filter((t) => t.is_builtin);
  };

  /**
   * Get custom templates only
   */
  const getCustomTemplates = () => {
    return templates().filter((t) => !t.is_builtin);
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
    getBuiltinTemplates,
    getCustomTemplates,
  };
}
