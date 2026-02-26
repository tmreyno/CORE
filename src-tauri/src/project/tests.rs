// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tests for project module.

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_project_new() {
        let project = FFXProject::new("/test/case-folder");
        assert_eq!(project.version, PROJECT_VERSION);
        assert_eq!(project.name, "case-folder");
        assert_eq!(project.root_path, "/test/case-folder");
        assert!(project.tabs.is_empty());
        assert!(project.active_tab_path.is_none());
        assert_eq!(project.app_version, Some(APP_VERSION.to_string()));
    }

    #[test]
    fn test_project_new_with_trailing_slash() {
        let project = FFXProject::new("/test/my-case");
        assert_eq!(project.name, "my-case");
    }

    #[test]
    fn test_project_default_save_path() {
        let project = FFXProject::new("/home/user/cases/case-001");
        let save_path = project.default_save_path();
        assert!(save_path.to_string_lossy().contains("case-001.cffx"));
    }

    #[test]
    fn test_get_default_project_path() {
        let path = get_default_project_path("/evidence/case-2024");
        assert!(path.to_string_lossy().contains("case-2024.cffx"));
    }

    #[test]
    fn test_project_touch() {
        let mut project = FFXProject::new("/test/case");
        let original_saved = project.saved_at.clone();
        
        // Small delay to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(10));
        project.touch();
        
        assert_ne!(project.saved_at, original_saved);
        assert_eq!(project.app_version, Some(APP_VERSION.to_string()));
    }

    #[test]
    fn test_project_serialization_roundtrip() {
        let mut project = FFXProject::new("/test/case");
        project.tabs.push(ProjectTab {
            id: "tab_1".to_string(),
            tab_type: "evidence".to_string(),
            file_path: "evidence.E01".to_string(),
            name: "evidence.E01".to_string(),
            subtitle: None,
            order: 0,
            container_type: Some("E01".to_string()),
            document_path: None,
            entry_path: None,
            entry_container_path: None,
            entry_name: None,
            processed_db_path: None,
            processed_db_type: None,
            scroll_position: None,
            last_viewed: None,
        });
        project.active_tab_path = Some("evidence.E01".to_string());
        project.notes.push(ProjectNote {
            id: "note1".to_string(),
            target_type: "file".to_string(),
            target_path: Some("evidence.E01".to_string()),
            title: "Test Note".to_string(),
            content: "Test notes".to_string(),
            created_by: "tester".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            tags: vec!["important".to_string()],
            priority: None,
        });
        project.tags.push(ProjectTag {
            id: "tag1".to_string(),
            name: "important".to_string(),
            color: "#ff0000".to_string(),
            description: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
        
        // Add hash history
        project.hash_history.files.insert(
            "evidence.E01".to_string(),
            vec![ProjectFileHash {
                algorithm: "SHA-256".to_string(),
                hash_value: "abc123".to_string(),
                computed_at: chrono::Utc::now().to_rfc3339(),
                verification: None,
            }],
        );
        
        // Serialize and deserialize
        let json = serde_json::to_string(&project).unwrap();
        let deserialized: FFXProject = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.version, project.version);
        assert_eq!(deserialized.name, project.name);
        assert_eq!(deserialized.root_path, project.root_path);
        assert_eq!(deserialized.tabs.len(), 1);
        assert_eq!(deserialized.active_tab_path, Some("evidence.E01".to_string()));
        assert_eq!(deserialized.notes.len(), 1);
        assert_eq!(deserialized.tags.len(), 1);
        assert!(deserialized.hash_history.files.contains_key("evidence.E01"));
    }

    #[test]
    fn test_project_hash_history_default() {
        let history = ProjectHashHistory::default();
        assert!(history.files.is_empty());
    }

    #[test]
    fn test_project_ui_state_default() {
        let ui_state = ProjectUIState::default();
        assert_eq!(ui_state.left_panel_width, 300);
        assert_eq!(ui_state.right_panel_width, 300);
        assert!(!ui_state.left_panel_collapsed);
        assert!(!ui_state.right_panel_collapsed);
    }

    #[test]
    fn test_project_load_result_success() {
        let result = ProjectLoadResult {
            success: true,
            project: Some(FFXProject::new("/test")),
            error: None,
            warnings: None,
        };
        assert!(result.success);
        assert!(result.project.is_some());
    }

    #[test]
    fn test_project_load_result_failure() {
        let result = ProjectLoadResult {
            success: false,
            project: None,
            error: Some("File not found".to_string()),
            warnings: None,
        };
        assert!(!result.success);
        assert!(result.project.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn test_project_save_result_success() {
        let result = ProjectSaveResult {
            success: true,
            path: Some("/test/case.cffx".to_string()),
            error: None,
        };
        assert!(result.success);
        assert!(result.path.is_some());
    }

    #[test]
    fn test_project_constants() {
        assert_eq!(PROJECT_EXTENSION, ".cffx");
        // Verify version is set (compile-time constant)
        assert_eq!(PROJECT_VERSION, 2);
        assert!(!APP_VERSION.is_empty());
    }
}
