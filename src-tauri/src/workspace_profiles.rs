// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Workspace profiles for different investigation scenarios.
//!
//! Split into three files for maintainability:
//! - `workspace_profile_types.rs` — Type definitions (ProfileType, WorkspaceProfile, etc.)
//! - `workspace_profile_defaults.rs` — Default profile builders (Investigation, Analysis, etc.)
//! - `workspace_profiles.rs` (this file) — ProfileManager CRUD methods + tests
//!
//! Re-exports all types so consumers can still import from `crate::workspace_profiles::*`.

// Re-export all types for backward compatibility
pub use super::workspace_profile_types::*;

impl ProfileManager {
    /// Get profile by ID
    pub fn get_profile(&self, id: &str) -> Option<&WorkspaceProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    /// Get active profile
    pub fn get_active_profile(&self) -> Option<&WorkspaceProfile> {
        self.active_profile_id
            .as_ref()
            .and_then(|id| self.get_profile(id))
    }

    /// Set active profile
    pub fn set_active_profile(&mut self, id: &str) -> Result<(), String> {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_profile_id = Some(id.to_string());

            // Update usage stats
            if let Some(profile) = self.profiles.iter_mut().find(|p| p.id == id) {
                profile.usage_count += 1;
                profile.last_used = chrono::Utc::now().to_rfc3339();
            }

            Ok(())
        } else {
            Err(format!("Profile not found: {}", id))
        }
    }

    /// Add custom profile
    pub fn add_profile(&mut self, profile: WorkspaceProfile) {
        self.profiles.push(profile);
    }

    /// Update profile
    pub fn update_profile(&mut self, profile: WorkspaceProfile) -> Result<(), String> {
        if let Some(existing) = self.profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile;
            Ok(())
        } else {
            Err("Profile not found".to_string())
        }
    }

    /// Delete profile
    pub fn delete_profile(&mut self, id: &str) -> Result<(), String> {
        let index = self
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or("Profile not found")?;

        // Don't delete if it's the active or default profile
        if self.active_profile_id.as_deref() == Some(id) {
            return Err("Cannot delete active profile".to_string());
        }
        if self.default_profile_id.as_deref() == Some(id) {
            return Err("Cannot delete default profile".to_string());
        }

        self.profiles.remove(index);
        Ok(())
    }

    /// Clone profile to create new custom profile
    pub fn clone_profile(&mut self, source_id: &str, new_name: &str) -> Result<String, String> {
        let source = self
            .get_profile(source_id)
            .ok_or("Source profile not found")?
            .clone();

        let new_id = format!("custom_{}", uuid::Uuid::new_v4());
        let mut new_profile = source;
        new_profile.id = new_id.clone();
        new_profile.name = new_name.to_string();
        new_profile.profile_type = ProfileType::Custom;
        new_profile.created_at = chrono::Utc::now().to_rfc3339();
        new_profile.last_used = chrono::Utc::now().to_rfc3339();
        new_profile.usage_count = 0;

        self.profiles.push(new_profile);
        Ok(new_id)
    }

    /// Export profile to JSON
    pub fn export_profile(&self, id: &str) -> Result<String, String> {
        let profile = self.get_profile(id).ok_or("Profile not found")?;
        serde_json::to_string_pretty(profile).map_err(|e| e.to_string())
    }

    /// Import profile from JSON
    pub fn import_profile(&mut self, json: &str) -> Result<String, String> {
        let profile: WorkspaceProfile = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let id = profile.id.clone();
        self.profiles.push(profile);
        Ok(id)
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Vec<ProfileSummary> {
        self.profiles
            .iter()
            .map(|p| ProfileSummary {
                id: p.id.clone(),
                name: p.name.clone(),
                profile_type: p.profile_type,
                description: p.description.clone(),
                last_used: p.last_used.clone(),
                usage_count: p.usage_count,
                is_active: self.active_profile_id.as_deref() == Some(&p.id),
                is_default: self.default_profile_id.as_deref() == Some(&p.id),
            })
            .collect()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // =========================================================================
    // ProfileType
    // =========================================================================

    #[test]
    fn test_profile_type_as_str_all_variants() {
        assert_eq!(ProfileType::Investigation.as_str(), "Investigation");
        assert_eq!(ProfileType::Analysis.as_str(), "Analysis");
        assert_eq!(ProfileType::Review.as_str(), "Review");
        assert_eq!(ProfileType::Mobile.as_str(), "Mobile");
        assert_eq!(ProfileType::Computer.as_str(), "Computer");
        assert_eq!(ProfileType::Network.as_str(), "Network");
        assert_eq!(ProfileType::IncidentResponse.as_str(), "Incident Response");
        assert_eq!(ProfileType::Custom.as_str(), "Custom");
    }

    #[test]
    fn test_profile_type_description_all_variants() {
        assert_eq!(
            ProfileType::Investigation.description(),
            "General purpose investigation workspace"
        );
        assert_eq!(
            ProfileType::Analysis.description(),
            "Advanced analysis with all tools enabled"
        );
        assert_eq!(
            ProfileType::Review.description(),
            "Case review and documentation focused"
        );
        assert_eq!(
            ProfileType::Mobile.description(),
            "Optimized for mobile device forensics"
        );
        assert_eq!(
            ProfileType::Computer.description(),
            "Optimized for computer forensics"
        );
        assert_eq!(
            ProfileType::Network.description(),
            "Optimized for network forensics"
        );
        assert_eq!(
            ProfileType::IncidentResponse.description(),
            "Rapid incident response workflow"
        );
        assert_eq!(
            ProfileType::Custom.description(),
            "Custom user-defined workspace"
        );
    }

    #[test]
    fn test_profile_type_equality() {
        assert_eq!(ProfileType::Investigation, ProfileType::Investigation);
        assert_ne!(ProfileType::Investigation, ProfileType::Analysis);
    }

    #[test]
    fn test_profile_type_serialization_roundtrip() {
        for pt in [
            ProfileType::Investigation,
            ProfileType::Analysis,
            ProfileType::Review,
            ProfileType::Mobile,
            ProfileType::Computer,
            ProfileType::Network,
            ProfileType::IncidentResponse,
            ProfileType::Custom,
        ] {
            let json = serde_json::to_string(&pt).unwrap();
            let back: ProfileType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, pt);
        }
    }

    // =========================================================================
    // CenterLayout
    // =========================================================================

    #[test]
    fn test_center_layout_equality() {
        assert_eq!(CenterLayout::Single, CenterLayout::Single);
        assert_ne!(CenterLayout::Single, CenterLayout::SplitVertical);
        assert_ne!(CenterLayout::SplitHorizontal, CenterLayout::Grid);
    }

    // =========================================================================
    // ProfileManager creation
    // =========================================================================

    #[test]
    fn test_profile_manager_creation() {
        let manager = ProfileManager::new();
        assert_eq!(manager.profiles.len(), 5);
        assert!(manager.active_profile_id.is_some());
        assert!(manager.default_profile_id.is_some());
    }

    #[test]
    fn test_profile_manager_default() {
        let manager = ProfileManager::default();
        assert_eq!(manager.profiles.len(), 5);
    }

    #[test]
    fn test_default_profile_ids() {
        let manager = ProfileManager::new();
        let ids: Vec<&str> = manager.profiles.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"investigation"));
        assert!(ids.contains(&"analysis"));
        assert!(ids.contains(&"review"));
        assert!(ids.contains(&"mobile"));
        assert!(ids.contains(&"computer"));
    }

    #[test]
    fn test_default_profile_types() {
        let manager = ProfileManager::new();
        let types: Vec<ProfileType> = manager.profiles.iter().map(|p| p.profile_type).collect();
        assert!(types.contains(&ProfileType::Investigation));
        assert!(types.contains(&ProfileType::Analysis));
        assert!(types.contains(&ProfileType::Review));
        assert!(types.contains(&ProfileType::Mobile));
        assert!(types.contains(&ProfileType::Computer));
    }

    #[test]
    fn test_default_active_is_first_profile() {
        let manager = ProfileManager::new();
        assert_eq!(
            manager.active_profile_id.as_deref(),
            Some(manager.profiles[0].id.as_str())
        );
    }

    // =========================================================================
    // get_profile
    // =========================================================================

    #[test]
    fn test_get_profile_existing() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("investigation");
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().profile_type, ProfileType::Investigation);
    }

    #[test]
    fn test_get_profile_not_found() {
        let manager = ProfileManager::new();
        assert!(manager.get_profile("nonexistent").is_none());
    }

    // =========================================================================
    // get_active_profile
    // =========================================================================

    #[test]
    fn test_get_active_profile() {
        let manager = ProfileManager::new();
        let active = manager.get_active_profile();
        assert!(active.is_some());
        assert_eq!(
            active.unwrap().id,
            manager.active_profile_id.clone().unwrap()
        );
    }

    // =========================================================================
    // set_active_profile
    // =========================================================================

    #[test]
    fn test_set_active_profile() {
        let mut manager = ProfileManager::new();
        assert!(manager.set_active_profile("analysis").is_ok());
        assert_eq!(manager.active_profile_id.as_deref(), Some("analysis"));
    }

    #[test]
    fn test_set_active_profile_increments_usage() {
        let mut manager = ProfileManager::new();
        let initial_count = manager.get_profile("analysis").unwrap().usage_count;
        manager.set_active_profile("analysis").unwrap();
        let updated_count = manager.get_profile("analysis").unwrap().usage_count;
        assert_eq!(updated_count, initial_count + 1);
    }

    #[test]
    fn test_set_active_profile_not_found() {
        let mut manager = ProfileManager::new();
        let result = manager.set_active_profile("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // =========================================================================
    // delete_profile
    // =========================================================================

    #[test]
    fn test_delete_profile_success() {
        let mut manager = ProfileManager::new();
        // Make a different profile active and default so we can delete investigation
        manager.active_profile_id = Some("analysis".to_string());
        manager.default_profile_id = Some("analysis".to_string());

        let initial_count = manager.profiles.len();
        let result = manager.delete_profile("review");
        assert!(result.is_ok());
        assert_eq!(manager.profiles.len(), initial_count - 1);
    }

    #[test]
    fn test_delete_profile_cannot_delete_active() {
        let mut manager = ProfileManager::new();
        manager.active_profile_id = Some("investigation".to_string());
        manager.default_profile_id = Some("analysis".to_string());

        let result = manager.delete_profile("investigation");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("active"));
    }

    #[test]
    fn test_delete_profile_cannot_delete_default() {
        let mut manager = ProfileManager::new();
        manager.active_profile_id = Some("analysis".to_string());
        manager.default_profile_id = Some("investigation".to_string());

        let result = manager.delete_profile("investigation");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("default"));
    }

    #[test]
    fn test_delete_profile_not_found() {
        let mut manager = ProfileManager::new();
        let result = manager.delete_profile("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // =========================================================================
    // clone_profile
    // =========================================================================

    #[test]
    fn test_clone_profile() {
        let mut manager = ProfileManager::new();
        let result = manager.clone_profile("investigation", "My Custom");
        assert!(result.is_ok());
        let new_id = result.unwrap();
        assert_eq!(manager.profiles.len(), 6);

        let cloned = manager.get_profile(&new_id).unwrap();
        assert_eq!(cloned.name, "My Custom");
        assert_eq!(cloned.profile_type, ProfileType::Custom);
        assert_eq!(cloned.usage_count, 0);
    }

    #[test]
    fn test_clone_profile_not_found() {
        let mut manager = ProfileManager::new();
        let result = manager.clone_profile("nonexistent", "Test");
        assert!(result.is_err());
    }

    // =========================================================================
    // add_profile
    // =========================================================================

    #[test]
    fn test_add_profile() {
        let mut manager = ProfileManager::new();
        let initial_count = manager.profiles.len();

        manager.add_profile(WorkspaceProfile {
            id: "custom_test".to_string(),
            name: "Test Profile".to_string(),
            profile_type: ProfileType::Custom,
            description: "A test profile".to_string(),
            created_at: String::new(),
            last_used: String::new(),
            usage_count: 0,
            layout: LayoutConfig {
                left_panel_width: 300,
                right_panel_width: 300,
                bottom_panel_height: 200,
                left_panel_collapsed: false,
                right_panel_collapsed: false,
                bottom_panel_collapsed: false,
                left_panel_tab: "evidence".into(),
                right_panel_tab: "details".into(),
                bottom_panel_tab: "activity".into(),
                center_layout: CenterLayout::Single,
            },
            tools: ToolConfig {
                enabled_tools: vec![],
                tool_settings: HashMap::new(),
                default_hash_algorithms: vec![],
                auto_hash: false,
                auto_verify: false,
                default_export_format: "json".into(),
                show_hex_viewer: false,
                show_metadata: false,
            },
            filters: vec![],
            view_settings: ViewSettings {
                theme: "dark".into(),
                font_size: 14,
                show_hidden_files: false,
                show_file_extensions: true,
                tree_indent: 16,
                icon_size: 20,
                detail_view_mode: "list".into(),
                thumbnail_size: 128,
            },
            quick_actions: vec![],
            shortcuts: HashMap::new(),
            metadata: HashMap::new(),
        });

        assert_eq!(manager.profiles.len(), initial_count + 1);
        assert!(manager.get_profile("custom_test").is_some());
    }

    // =========================================================================
    // update_profile
    // =========================================================================

    #[test]
    fn test_update_profile() {
        let mut manager = ProfileManager::new();
        let mut profile = manager.get_profile("investigation").unwrap().clone();
        profile.description = "Updated description".to_string();

        let result = manager.update_profile(profile);
        assert!(result.is_ok());
        assert_eq!(
            manager.get_profile("investigation").unwrap().description,
            "Updated description"
        );
    }

    #[test]
    fn test_update_profile_not_found() {
        let mut manager = ProfileManager::new();
        let profile = WorkspaceProfile {
            id: "nonexistent".to_string(),
            name: "X".to_string(),
            profile_type: ProfileType::Custom,
            description: String::new(),
            created_at: String::new(),
            last_used: String::new(),
            usage_count: 0,
            layout: LayoutConfig {
                left_panel_width: 0,
                right_panel_width: 0,
                bottom_panel_height: 0,
                left_panel_collapsed: false,
                right_panel_collapsed: false,
                bottom_panel_collapsed: false,
                left_panel_tab: String::new(),
                right_panel_tab: String::new(),
                bottom_panel_tab: String::new(),
                center_layout: CenterLayout::Single,
            },
            tools: ToolConfig {
                enabled_tools: vec![],
                tool_settings: HashMap::new(),
                default_hash_algorithms: vec![],
                auto_hash: false,
                auto_verify: false,
                default_export_format: String::new(),
                show_hex_viewer: false,
                show_metadata: false,
            },
            filters: vec![],
            view_settings: ViewSettings {
                theme: String::new(),
                font_size: 14,
                show_hidden_files: false,
                show_file_extensions: true,
                tree_indent: 16,
                icon_size: 20,
                detail_view_mode: String::new(),
                thumbnail_size: 128,
            },
            quick_actions: vec![],
            shortcuts: HashMap::new(),
            metadata: HashMap::new(),
        };
        assert!(manager.update_profile(profile).is_err());
    }

    // =========================================================================
    // export/import profile
    // =========================================================================

    #[test]
    fn test_export_profile() {
        let manager = ProfileManager::new();
        let json = manager.export_profile("investigation");
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("investigation"));
        assert!(json_str.contains("Investigation"));
    }

    #[test]
    fn test_export_profile_not_found() {
        let manager = ProfileManager::new();
        assert!(manager.export_profile("nonexistent").is_err());
    }

    #[test]
    fn test_import_export_roundtrip() {
        let manager = ProfileManager::new();
        let json = manager.export_profile("investigation").unwrap();

        let mut manager2 = ProfileManager {
            profiles: vec![],
            active_profile_id: None,
            default_profile_id: None,
        };
        let result = manager2.import_profile(&json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "investigation");
        assert_eq!(manager2.profiles.len(), 1);
    }

    #[test]
    fn test_import_profile_invalid_json() {
        let mut manager = ProfileManager::new();
        let result = manager.import_profile("not valid json");
        assert!(result.is_err());
    }

    // =========================================================================
    // list_profiles
    // =========================================================================

    #[test]
    fn test_list_profiles() {
        let manager = ProfileManager::new();
        let summaries = manager.list_profiles();
        assert_eq!(summaries.len(), 5);
        for summary in &summaries {
            assert!(!summary.id.is_empty());
            assert!(!summary.name.is_empty());
        }
    }

    #[test]
    fn test_list_profiles_active_flag() {
        let manager = ProfileManager::new();
        let summaries = manager.list_profiles();
        let active_count = summaries.iter().filter(|s| s.is_active).count();
        assert_eq!(active_count, 1);
    }

    #[test]
    fn test_list_profiles_default_flag() {
        let manager = ProfileManager::new();
        let summaries = manager.list_profiles();
        let default_count = summaries.iter().filter(|s| s.is_default).count();
        assert_eq!(default_count, 1);
    }

    // =========================================================================
    // Profile content validation
    // =========================================================================

    #[test]
    fn test_investigation_profile_layout() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("investigation").unwrap();
        assert_eq!(profile.layout.left_panel_width, 300);
        assert!(!profile.layout.left_panel_collapsed);
        assert_eq!(profile.layout.center_layout, CenterLayout::Single);
    }

    #[test]
    fn test_analysis_profile_all_panels_visible() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("analysis").unwrap();
        assert!(!profile.layout.left_panel_collapsed);
        assert!(!profile.layout.right_panel_collapsed);
        assert!(!profile.layout.bottom_panel_collapsed);
        assert_eq!(profile.layout.center_layout, CenterLayout::SplitVertical);
    }

    #[test]
    fn test_analysis_profile_auto_hash() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("analysis").unwrap();
        assert!(profile.tools.auto_hash);
        assert!(profile.tools.default_hash_algorithms.len() >= 3);
    }

    #[test]
    fn test_review_profile_right_panel_wider() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("review").unwrap();
        assert_eq!(profile.layout.right_panel_width, 450);
        assert_eq!(profile.layout.right_panel_tab, "notes");
    }

    #[test]
    fn test_mobile_profile_has_plist_viewer() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("mobile").unwrap();
        assert!(profile
            .tools
            .enabled_tools
            .contains(&"plist_viewer".to_string()));
        assert!(profile
            .tools
            .enabled_tools
            .contains(&"sqlite_viewer".to_string()));
    }

    #[test]
    fn test_computer_profile_has_registry_viewer() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("computer").unwrap();
        assert!(profile
            .tools
            .enabled_tools
            .contains(&"registry_viewer".to_string()));
        assert!(profile
            .tools
            .enabled_tools
            .contains(&"event_log_viewer".to_string()));
    }

    #[test]
    fn test_mobile_profile_has_database_filters() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("mobile").unwrap();
        let filter_ids: Vec<&str> = profile.filters.iter().map(|f| f.id.as_str()).collect();
        assert!(filter_ids.contains(&"databases"));
        assert!(filter_ids.contains(&"plists"));
    }

    #[test]
    fn test_investigation_profile_has_shortcuts() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("investigation").unwrap();
        assert!(profile.shortcuts.contains_key("save"));
        assert!(profile.shortcuts.contains_key("search"));
    }

    #[test]
    fn test_investigation_profile_has_quick_actions() {
        let manager = ProfileManager::new();
        let profile = manager.get_profile("investigation").unwrap();
        assert!(!profile.quick_actions.is_empty());
        let action_ids: Vec<&str> = profile
            .quick_actions
            .iter()
            .map(|a| a.id.as_str())
            .collect();
        assert!(action_ids.contains(&"hash_file"));
        assert!(action_ids.contains(&"bookmark"));
    }

    // =========================================================================
    // create_filter_preset helper
    // =========================================================================

    #[test]
    fn test_create_filter_preset() {
        let preset =
            ProfileManager::create_filter_preset("test_filter", "Test Files", vec!["TXT", "CSV"]);
        assert_eq!(preset.id, "test_filter");
        assert_eq!(preset.name, "Test Files");
        assert_eq!(preset.extensions, vec!["TXT", "CSV"]);
        assert!(!preset.include_hidden);
        assert!(!preset.include_system);
        assert!(preset.size_range.is_none());
    }
}
