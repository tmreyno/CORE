// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Extended project commands for workspace profiles, templates, and timeline.

use crate::workspace_profiles::{ProfileManager, WorkspaceProfile, ProfileSummary};
use crate::project_templates::{TemplateManager, ProjectTemplate, TemplateSummary, TemplateCategory};
use crate::activity_timeline::{TimelineVisualization, TimelineExport, compute_timeline_visualization, export_timeline};
use crate::project_comparison::{ProjectComparison, MergeResult, MergeStrategy, compare_projects, merge_projects};
use crate::project::FFXProject;

// =============================================================================
// WORKSPACE PROFILE COMMANDS
// =============================================================================

#[tauri::command]
pub async fn profile_list() -> Result<Vec<ProfileSummary>, String> {
    let manager = ProfileManager::new();
    Ok(manager.list_profiles())
}

#[tauri::command]
pub async fn profile_get(id: String) -> Result<WorkspaceProfile, String> {
    let manager = ProfileManager::new();
    manager
        .get_profile(&id)
        .cloned()
        .ok_or_else(|| "Profile not found".to_string())
}

#[tauri::command]
pub async fn profile_get_active() -> Result<WorkspaceProfile, String> {
    let manager = ProfileManager::new();
    manager
        .get_active_profile()
        .cloned()
        .ok_or_else(|| "No active profile".to_string())
}

#[tauri::command]
pub async fn profile_set_active(id: String) -> Result<(), String> {
    let mut manager = ProfileManager::new();
    manager.set_active_profile(&id)
}

#[tauri::command]
pub async fn profile_add(profile: WorkspaceProfile) -> Result<(), String> {
    let mut manager = ProfileManager::new();
    manager.add_profile(profile);
    Ok(())
}

#[tauri::command]
pub async fn profile_update(profile: WorkspaceProfile) -> Result<(), String> {
    let mut manager = ProfileManager::new();
    manager.update_profile(profile)
}

#[tauri::command]
pub async fn profile_delete(id: String) -> Result<(), String> {
    let mut manager = ProfileManager::new();
    manager.delete_profile(&id)
}

#[tauri::command]
pub async fn profile_clone(source_id: String, new_name: String) -> Result<String, String> {
    let mut manager = ProfileManager::new();
    manager.clone_profile(&source_id, &new_name)
}

#[tauri::command]
pub async fn profile_export(id: String) -> Result<String, String> {
    let manager = ProfileManager::new();
    manager.export_profile(&id)
}

#[tauri::command]
pub async fn profile_import(json: String) -> Result<String, String> {
    let mut manager = ProfileManager::new();
    manager.import_profile(&json)
}

// =============================================================================
// TEMPLATE COMMANDS
// =============================================================================

#[tauri::command]
pub async fn template_list() -> Result<Vec<TemplateSummary>, String> {
    let manager = TemplateManager::new();
    Ok(manager.list_templates())
}

#[tauri::command]
pub async fn template_list_by_category(category: String) -> Result<Vec<TemplateSummary>, String> {
    let manager = TemplateManager::new();
    
    let cat = match category.as_str() {
        "Mobile" => TemplateCategory::Mobile,
        "Computer" => TemplateCategory::Computer,
        "Network" => TemplateCategory::Network,
        "Cloud" => TemplateCategory::Cloud,
        "IncidentResponse" => TemplateCategory::IncidentResponse,
        "Memory" => TemplateCategory::Memory,
        "Malware" => TemplateCategory::Malware,
        "EDiscovery" => TemplateCategory::EDiscovery,
        "General" => TemplateCategory::General,
        _ => TemplateCategory::Custom,
    };

    let templates = manager.get_templates_by_category(cat);
    Ok(templates.iter().map(|t| TemplateSummary {
        id: t.id.clone(),
        name: t.name.clone(),
        category: t.category,
        description: t.description.clone(),
        tags: t.tags.clone(),
        usage_count: t.usage_count,
    }).collect())
}

#[tauri::command]
pub async fn template_get(template_id: String) -> Result<ProjectTemplate, String> {
    let manager = TemplateManager::new();
    manager
        .get_template(&template_id)
        .cloned()
        .ok_or_else(|| "Template not found".to_string())
}

#[tauri::command]
pub async fn template_apply(template_id: String, project: FFXProject) -> Result<FFXProject, String> {
    let manager = TemplateManager::new();
    let mut modified_project = project;
    manager.apply_template(&template_id, &mut modified_project)?;
    Ok(modified_project)
}

#[tauri::command]
pub async fn template_create_from_project(
    project: FFXProject,
    name: String,
    category: String,
    description: String,
) -> Result<String, String> {
    let mut manager = TemplateManager::new();
    
    let cat = match category.as_str() {
        "Mobile" => TemplateCategory::Mobile,
        "Computer" => TemplateCategory::Computer,
        "Network" => TemplateCategory::Network,
        "Cloud" => TemplateCategory::Cloud,
        "IncidentResponse" => TemplateCategory::IncidentResponse,
        "Memory" => TemplateCategory::Memory,
        "Malware" => TemplateCategory::Malware,
        "EDiscovery" => TemplateCategory::EDiscovery,
        "General" => TemplateCategory::General,
        _ => TemplateCategory::Custom,
    };

    manager.create_from_project(&project, name, cat, description)
}

#[tauri::command]
pub async fn template_export(template_id: String) -> Result<String, String> {
    let manager = TemplateManager::new();
    manager.export_template(&template_id)
}

#[tauri::command]
pub async fn template_import(json: String) -> Result<String, String> {
    let mut manager = TemplateManager::new();
    manager.import_template(&json)
}

#[tauri::command]
pub async fn template_delete(template_id: String) -> Result<(), String> {
    let mut manager = TemplateManager::new();
    manager.delete_template(&template_id)
}

// =============================================================================
// TIMELINE COMMANDS
// =============================================================================

#[tauri::command]
pub async fn timeline_compute_visualization(project: FFXProject) -> Result<TimelineVisualization, String> {
    Ok(compute_timeline_visualization(&project))
}

#[tauri::command]
pub async fn timeline_export(project: FFXProject, exported_by: String) -> Result<TimelineExport, String> {
    Ok(export_timeline(&project, exported_by))
}

#[tauri::command]
pub async fn timeline_export_json(project: FFXProject, exported_by: String) -> Result<String, String> {
    let export = export_timeline(&project, exported_by);
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

// =============================================================================
// PROJECT COMPARISON COMMANDS
// =============================================================================

#[tauri::command]
pub async fn project_compare(project_a: FFXProject, project_b: FFXProject) -> Result<ProjectComparison, String> {
    Ok(compare_projects(&project_a, &project_b))
}

#[tauri::command]
pub async fn project_merge(
    project_a: FFXProject,
    project_b: FFXProject,
    strategy: String,
) -> Result<MergeResult, String> {
    let merge_strategy = match strategy.as_str() {
        "PreferA" => MergeStrategy::PreferA,
        "PreferB" => MergeStrategy::PreferB,
        "KeepBoth" => MergeStrategy::KeepBoth,
        "Skip" => MergeStrategy::Skip,
        _ => MergeStrategy::Manual,
    };
    
    merge_projects(&project_a, &project_b, merge_strategy)
}

#[tauri::command]
pub async fn project_sync_bookmarks(
    target: FFXProject,
    source: FFXProject,
    overwrite: bool,
) -> Result<FFXProject, String> {
    let mut synced = target.clone();
    
    for bookmark in &source.bookmarks {
        if overwrite {
            // Remove existing with same name
            synced.bookmarks.retain(|b| b.name != bookmark.name);
            synced.bookmarks.push(bookmark.clone());
        } else {
            // Only add if doesn't exist
            if !synced.bookmarks.iter().any(|b| b.name == bookmark.name) {
                synced.bookmarks.push(bookmark.clone());
            }
        }
    }
    
    Ok(synced)
}

#[tauri::command]
pub async fn project_sync_notes(
    target: FFXProject,
    source: FFXProject,
    overwrite: bool,
) -> Result<FFXProject, String> {
    let mut synced = target.clone();
    
    for note in &source.notes {
        if overwrite {
            // Remove existing with same title
            synced.notes.retain(|n| n.title != note.title);
            synced.notes.push(note.clone());
        } else {
            // Only add if doesn't exist
            if !synced.notes.iter().any(|n| n.title == note.title) {
                synced.notes.push(note.clone());
            }
        }
    }
    
    Ok(synced)
}
